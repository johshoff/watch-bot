Simple program to watch for changes on URLs, render their JSON-contents using
[handlebars](https://handlebarsjs.com/guide/) and post it to slack.

You need to create a `config.json` for it to work. For example:

    {
        "slack_url": "https://hooks.slack.com/services/xxxxxxx/xxxxxx/xxxxxx",
        "checks": [
            {
                "url": "https://example.com/status_page.json",
                "template": "status.hbs"
            }
        ]
    }

`slack_url` can be skipped, in which case the contents will just be written to
stdout.
