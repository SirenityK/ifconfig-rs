use crate::{HEADER_BREAK, Headers, gen_headers, json};
use actix_http::header::{ACCEPT_LANGUAGE, HOST};
use actix_web::HttpRequest;
use maud::{DOCTYPE, Markup, html};
use serde_json::to_string_pretty;

static CSS: &str = include_str!("styles.min.css");

pub trait Beautifier {
    fn breakline(&mut self);
    fn as_pretty(&self) -> String;
}

impl Beautifier for String {
    fn breakline(&mut self) {
        self.push('\n');
    }

    fn as_pretty(&self) -> String {
        let mut result = String::with_capacity(self.len());
        let mut capitalize = true;
        let mut chars = self.chars().peekable();

        while let Some(c) = chars.next() {
            match c {
                '-' | '_' => {
                    result.push(' ');
                    capitalize = true;
                }
                'i' if capitalize && chars.peek().map_or(false, |&next| next == 'p') => {
                    result.push_str("IP");
                    chars.next(); // Skip the 'p'
                    capitalize = false;
                }
                c if capitalize && c.is_ascii_alphabetic() => {
                    result.push(c.to_ascii_uppercase());
                    capitalize = false;
                }
                _ => result.push(c),
            }
        }

        result
    }
}

pub fn layout(title: &str, lang: &str, body: Markup) -> Markup {
    html! {
        (DOCTYPE)
        html lang=(lang) color-mode="user" {
            head {
                meta charser="UTF-8";
                meta name="viewport" content="width=device-width, initial-scale=1.0";
                link rel="stylesheet" href="https://unpkg.com/mvp.css";
                link rel="stylesheet" href="https://cdnjs.cloudflare.com/ajax/libs/prism/1.29.0/themes/prism-tomorrow.min.css" integrity="sha512-vswe+cgvic/XBoF1OcM/TeJ2FW0OofqAVdCZiEYkd6dwGXthvkSFWOoGGJgS2CW70VK5dQM5Oh+7ne47s74VTg==" crossorigin="anonymous" referrerpolicy="no-referrer";
                // custom styles at the end to override prism
                style { (CSS) }
                title { (title) }
                // core
                script src="https://cdnjs.cloudflare.com/ajax/libs/prism/1.29.0/components/prism-core.min.js" integrity="sha512-9khQRAUBYEJDCDVP2yw3LRUQvjJ0Pjx0EShmaQjcHa6AXiOv6qHQu9lCAIR8O+/D8FtaCoJ2c0Tf9Xo7hYH01Q==" crossorigin="anonymous" referrerpolicy="no-referrer" {}
                // bash
                script src="https://cdnjs.cloudflare.com/ajax/libs/prism/1.29.0/components/prism-bash.min.js" integrity="sha512-whYhDwtTmlC/NpZlCr6PSsAaLOrfjVg/iXAnC4H/dtiHawpShhT2SlIMbpIhT/IL/NrpdMm+Hq2C13+VKpHTYw==" crossorigin="anonymous" referrerpolicy="no-referrer" {}
                // json
                script src="https://cdnjs.cloudflare.com/ajax/libs/prism/1.29.0/components/prism-json.min.js" integrity="sha512-QXFMVAusM85vUYDaNgcYeU3rzSlc+bTV4JvkfJhjxSHlQEo+ig53BtnGkvFTiNJh8D+wv6uWAQ2vJaVmxe8d3w==" crossorigin="anonymous" referrerpolicy="no-referrer" {}
            }
            body { (body) }
        }
    }
}

pub fn render(req: HttpRequest) -> Markup {
    let headers = req.headers();
    let response_headers = gen_headers(req.clone());
    let ip = &response_headers
        .iter()
        .find(|(key, _)| *key == Headers::IP_ADDRESS)
        .unwrap()
        .1;
    let lang = match headers.get(ACCEPT_LANGUAGE) {
        Some(lang) => lang.to_str().unwrap().split(HEADER_BREAK).nth(1).unwrap(),
        None => "en",
    };

    let host = &response_headers
        .iter()
        .find(|(key, _)| *key == HOST.as_str())
        .unwrap()
        .1;
    let rh = response_headers.clone();
    let response_list = response_headers
        .iter()
        .map(|(key, value)| format!("{}: {}", key, value))
        .collect::<Vec<String>>();
    layout(
        "Your IP address",
        lang,
        html! {
            h1 { "Your connection "}
            table {
                @for header in &response_headers {
                    tr {
                        @if header.0 == Headers::IP_ADDRESS {
                            td { h4 { b {(header.0.to_string().as_pretty())} } }
                            td { h4 { b {(header.1)} } }
                        } @else {
                            td { (header.0.to_string().as_pretty()) }
                            td { (header.1) }
                        }
                    }
                }
            }
            h1 { "API" }
            table {
                thead {
                    th { "Description" }
                    th { "Command" }
                    th { "Response" }
                }
                tr {
                    td { "Plain IP address" }
                    td {
                        pre {
                            code class="language-bash" { "curl " (host) }
                        }
                    }
                    td {
                        p.response.token.string { (ip) }
                    }
                }
                tr {
                    td { "IP and headers" }
                    td {
                        pre {
                            code class="language-bash" { "curl " (host) "/all" }
                        }
                    }
                    td {
                        @for response in response_list {
                            p.response.token.string { (response) }
                        }
                    }
                }
                tr {
                    td { "IP and headers as json" }
                    td {
                        pre {
                            code class="language-bash" { "curl " (host) "/all.json" }
                        }
                    }
                    td {
                        pre {
                            code class="language-json" { ({ to_string_pretty(&json(rh)).unwrap() })}
                        }
                    }
                }
            }
            h1 { "About the server"}
            div {
                p { "This server is running on actix-web, a high-performance web framework for Rust. It is a simple server that returns the IP address of the client." }
                p { "This ensures the fastest possible response time, all thanks to Rust!" }
                p { "The source code is available on " a href="https://github.com/sirenityk/ifconfig-rs" target="_blank" rel="noopener noreferrer" { "GitHub" } "." }
            }
        },
    )
}
