use log::error;
use matrix_sdk::reqwest;
use maud::{html, Markup};
use yaserde::de::from_str;
use yaserde_derive::YaDeserialize;

#[derive(Debug, Default, YaDeserialize)]
#[yaserde(
    namespace = "atom: http://www.w3.org/2005/Atom",
    prefix="atom"
    default_namespace = "atom"
)]
struct Feed {
    #[yaserde(rename = "entry", prefix="atom")]
    entries: Vec<Entry>,
}

#[derive(Debug, Default,  YaDeserialize)]
#[yaserde(
    namespace = "atom: http://www.w3.org/2005/Atom",
    prefix="atom"
    default_namespace = "atom"
)]
struct Entry {
    #[yaserde(prefix = "atom")]
    title: String,
    #[yaserde(prefix = "atom")]
    published: String,
    #[yaserde(prefix = "atom")]
    author: Author,
    #[yaserde(prefix = "atom")]
    link: Link,
    #[yaserde(prefix = "atom")]
    content: String,
}

#[derive(Debug, Default,  YaDeserialize)]
#[yaserde(
    namespace = "atom: http://www.w3.org/2005/Atom",
    prefix="atom"
    default_namespace = "atom"
)]
struct Author {
    #[yaserde(prefix = "atom")]
    name: String,
}

#[derive(Debug, Default,  YaDeserialize)]
#[yaserde(
    namespace = "atom: http://www.w3.org/2005/Atom",
    prefix="atom"
    default_namespace = "atom"
)]
struct Link {
    #[yaserde(rename = "href", prefix = "atom", attribute = true)]
    href: String,
}

pub async fn get_msn_today() -> Markup {
    let feed_url = "https://matrix.org/atom.xml";

    let xml_text = match reqwest::get(feed_url).await {
        Ok(resp) => match resp.text().await {
            Ok(text) => text,
            Err(_) => return error_page("Failed to read feed response"),
        },
        Err(_) => return error_page("Failed to fetch feed"),
    };


    let mut feed: Feed = match from_str(&xml_text) {
        Ok(f) => f,
        Err(e) => {
            error!("Failed to parse feed XML: {}", e);
            return error_page("Failed to parse feed XML")
        },
    };

    for entry in feed.entries.iter_mut() {
        entry.content = entry.content.replace("href", "target=\"_blank\" href");
    }

    html! {
        html {
            head {
                title { "Matrix Today" }
                meta http-equiv="Content-Type" content="text/html; charset=utf-8" {}
                style type="text/css" {
                    "body { font-family: Arial, sans-serif; font-size: 13px; background-color: #ffffff; color: #000000; margin: 0; padding: 0; }"
                    "h1 { background-color: #003399; color: #ffffff; padding: 8px 12px; margin: 0; font-size: 18px; }"
                    ".entry { border-bottom: 1px solid #cccccc; padding: 12px; margin: 0; }"
                    ".entry-title { font-size: 15px; font-weight: bold; margin-bottom: 4px; }"
                    ".entry-title a { color: #003399; text-decoration: none; }"
                    ".entry-title a:hover { text-decoration: underline; }"
                    ".entry-meta { font-size: 11px; color: #666666; margin-bottom: 6px; }"
                    ".entry-content { font-size: 13px; }"
                    ".entry-content img { width: 400px; border: 1px solid #cccccc; }"
                    ".footer { font-size: 11px; color: #999999; padding: 8px 12px; border-top: 1px solid #cccccc; }"
                }
            }
            body {
                h1 { "Matrix Today - Blog & News" }
                @for entry_index in 0..feed.entries.len() {
                    @let entry = &feed.entries[entry_index];
                    div class="entry" {
                        div class="entry-title" {
                            a href=(entry.link.href) target="_blank" { (entry.title) }
                        }
                        div class="entry-meta" {
                            "By " (entry.author.name) " | " (entry.published.get(..10).unwrap_or(&entry.published))
                        }

                    @if entry_index < 1 {
                        div class="entry-content" {
                            (maud::PreEscaped(&entry.content))
                        }
                    }

                    }
                }
                div class="footer" {
                    "Matrix.org Blog | "
                    a href="https://matrix.org/blog/" target="_blank" { "View Posts" }
                }
            }
        }
    }
}

fn error_page(message: &str) -> Markup {
    html! {
        html {
            head {
                title { "MSN Today - Error" }
                meta http-equiv="Content-Type" content="text/html; charset=utf-8" {}
            }
            body {
                h1 { "MSN Today" }
                p { (message) }
            }
        }
    }
}