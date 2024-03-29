use std::{
    error::Error,
    fs::{self, File},
    io::BufReader,
};

use clap::{App, Arg};
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
    content_cache_prefix: Option<String>,
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

/// Check if the content has changed since last update (or if it's never been posted)
fn content_has_changed(filename: &str, new_content: &str) -> bool {
    if let Ok(last_content) = fs::read_to_string(filename) {
        return new_content != last_content;
    }
    return true;
}

/// Update contents file
fn update_content(filename: &str, new_content: &str) -> Result<(), std::io::Error> {
    fs::write(filename, new_content)
}

fn main() -> Result<(), Box<dyn Error>> {
    let matches = App::new("watch-bot")
        .arg(
            Arg::with_name("config")
                .short("c")
                .long("config")
                .value_name("FILE")
                .takes_value(true)
                .help("Specify config file"),
        )
        .get_matches();

    let config_file = matches.value_of("config").unwrap_or("config.json");
    let config = read_config(config_file)?;

    for check in config.checks {
        let rendered = perform_check(&check)
            .unwrap_or_else(|err| format!("*Unhandled error:*\n{}", err.to_string()));

        // Filename for previously written content for a URL
        let prefix = config.content_cache_prefix.as_deref().unwrap_or(".last-");
        let content_filename = format!("{}{:x}", prefix, md5::compute(check.url));

        if !content_has_changed(&content_filename, &rendered) {
            continue;
        }

        if let Some(slack_url) = &config.slack_url {
            ureq::post(slack_url).send_json(ureq::json!({ "text": rendered }))?;
        } else {
            println!("{}", rendered);
        }

        update_content(&content_filename, &rendered)?;
    }

    Ok(())
}
