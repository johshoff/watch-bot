use std::{
    error::Error,
    fs::{self, File},
    io::BufReader,
};

use handlebars::Handlebars;
use serde::Deserialize;
use serde_json::Value;

#[derive(Deserialize)]
struct Check {
    url: String,
    template: String,
}

#[derive(Deserialize)]
struct Config {
    checks: Vec<Check>,
    slack_url: Option<String>,
}

fn read_config(path: &str) -> Result<Config, Box<dyn Error>> {
    let file = File::open(path)?;
    let reader = BufReader::new(file);
    Ok(serde_json::from_reader(reader)?)
}

fn perform_check(check: &Check) -> Result<String, Box<dyn Error>> {
    let body: String = ureq::get(&check.url).call()?.into_string()?;

    let data: Value = serde_json::from_str(&body)?;
    let template_string = fs::read_to_string(&check.template)?;

    Ok(Handlebars::new().render_template(&template_string, &data)?)
}

/// Filename for previously written content for a URL
fn content_filename(url: &str) -> String {
    format!(".last-{:x}", md5::compute(url))
}

/// Check if the content has changed since last update (or if it's never been posted)
fn content_has_changed(url: &str, new_content: &str) -> bool {
    let filename = content_filename(url);
    if let Ok(last_content) = fs::read_to_string(filename) {
        return new_content != last_content;
    }
    return true;
}

/// Update contents file
fn update_content(url: &str, new_content: &str) -> Result<(), std::io::Error> {
    let filename = content_filename(url);
    fs::write(filename, new_content)
}

fn main() -> Result<(), Box<dyn Error>> {
    let config = read_config("config.json")?;

    for check in config.checks {
        let rendered = perform_check(&check)
            .unwrap_or_else(|err| format!("*Unhandled error:*\n{}", err.to_string()));

        if !content_has_changed(&check.url, &rendered) {
            continue;
        }

        if let Some(slack_url) = &config.slack_url {
            ureq::post(slack_url).send_json(ureq::json!({ "text": rendered }))?;
        } else {
            println!("{}", rendered);
        }

        update_content(&check.url, &rendered)?;
    }

    Ok(())
}
