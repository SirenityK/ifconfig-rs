use crate::{Headers, Routes, gen_response, helpers::CONFIG, json};
use actix_http::header::{ACCEPT_LANGUAGE, HOST};
use actix_web::HttpRequest;
use buildid::build_id;
use maud::{DOCTYPE, Markup, html};
use serde_json::to_string_pretty;

static HEADER_BREAK: &[char] = &[',', ';'];

enum Icon {
    Terminal,
    Server,
}

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

fn icon(name: Icon) -> Result<Markup, &'static str> {
    let markup: Markup;
    match name {
        Icon::Terminal => {
            markup = html! {
                svg class="lucide lucide-icon lucide-terminal" fill="none" height="24" stroke="currentColor" stroke-linecap="round" stroke-linejoin="round" stroke-width="2" viewBox="0 0 24 24" width="24" xmlns="http://www.w3.org/2000/svg" {
                    polyline points="4 17 10 11 4 5" {}
                    line x1="12" x2="20" y1="19" y2="19" {}
                }
            };
        }
        Icon::Server => {
            markup = html! {
                svg class="lucide lucide-icon lucide-server" fill="none" height="24" stroke="currentColor" stroke-linecap="round" stroke-linejoin="round" stroke-width="2" viewBox="0 0 24 24" width="24" xmlns="http://www.w3.org/2000/svg" {
                    rect height="8" rx="2" ry="2" width="20" x="2" y="2" {}
                    rect height="8" rx="2" ry="2" width="20" x="2" y="14" {}
                    line x1="6" x2="6.01" y1="6" y2="6" {}
                    line x1="6" x2="6.01" y1="18" y2="18" {}
                }
            }
        }
    }
    Ok(markup)
}

fn layout(title: &str, lang: &str, body: Markup) -> Markup {
    html! {
        (DOCTYPE)
        html lang=(lang) {
            head {
                meta charser="UTF-8";
                meta name="viewport" content="width=device-width";
                title { (title) }
                link rel="stylesheet" href=(format!("/{}", CONFIG.css_file));
            }
            body { (body) }
        }
    }
}

fn footer(build_id: &str) -> Markup {
    html! {
        footer class="mx-2 space-y-4 py-4" {
            h1 {
                "About the server"
            }
            div {
                p {
                    "This server is running on actix-web, a high perfomance web framework for Rust. It is a simple web server that returns the IP address of the client."
                }
                p {
                    "This ensures the fastest possible response time, all thanks to Rust!"
                }
                p {
                    "The source code is available on "
                    a class="link link-primary" href="https://github.com/sirenityk/ifconfig-rs" target="_blank" rel="noopener noreferrer" {
                        "GitHub"
                    }
                    "."
                }
                p {
                    "build ID: "
                    b { (build_id) }
                }
            }
        }
    }
}

pub fn render(req: HttpRequest) -> Markup {
    let headers = req.headers();
    let response_headers = gen_response(req.clone());
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

    let id = match build_id() {
        Some(id) => hex::encode(id),
        None => "unknown".to_string(),
    };
    layout(
        "Your IP address",
        lang,
        html! {
            main class="mx-auto max-w-4xl space-y-4" {
                header class="text-center" {
                    h1 {
                        "Your public IP address"
                    }
                    p {
                        "Fast, simple IP address and HTTP request information"
                    }
                }
                div class="card-ip" {
                    h3 class="inline-flex items-center gap-2" { "Your IP address" }
                    p class="text-description text-sm" { "Your current public IP address" }
                    h4 class="my-2" { (ip) }
                }
                div class="tabs tabs-lift" {
                    input class="grow tab" aria-label="Connection Info" name="connection_info" type="radio" checked {}
                    div class="bg-base-100 border-base-300 tab-content" {
                        div class="m-6" {
                            h3 class="gap-2 inline-flex items-center" {
                                (icon(Icon::Server).unwrap())
                                "Your connection"
                            }
                            p class="text-description" {
                                "Additional details about your current HTTP request"
                            }
                            div class="divider m-0" {
                            }
                            @for header in &response_headers {
                                div class="flex *:whitespace-nowrap *:p-2 max-md:flex-col" {
                                    p class="text-description grow" {
                                        (header.0.to_string().as_pretty())
                                    }
                                    p class="overflow-x-auto" { (header.1) }
                                }
                            }
                        }
                    }
                    input class="grow tab" aria-label="API Usage" name="connection_info" type="radio";
                    div class="bg-base-100 border-base-300 tab-content" {
                        div class="m-6" {
                            h3 class="gap-2 inline-flex items-center" {
                                (icon(Icon::Terminal).unwrap())
                                "Your connection"
                            }
                            p class="text-description" {
                                "How to access your IP information programmatically"
                            }
                            div class="divider m-0" {
                            }
                            @for command in vec!["",Routes::ALL,Routes::ALL_JSON] {
                                div class="dark:bg-base-200 mockup-code my-4" {
                                    pre data-prefix="$" {
                                        code {
                                            span class="text-orange-400" {
                                                "curl "
                                            }
                                            span class="text-green-700" {
                                                (host) (command)
                                            }
                                        }
                                    }
                                    @match command {
                                        "" => {
                                            pre data-prefix=">" { (ip) }
                                        },
                                        Routes::ALL => {
                                            @for response in &response_list {
                                                pre data-prefix=">" { (response) }
                                            }
                                        },
                                        Routes::ALL_JSON => {
                                            pre data-prefix=">" {
                                                (to_string_pretty(&json(rh.clone())).unwrap())
                                            }
                                        },
                                        _ => { "Unknown command" }
                                    }
                                }
                            }
                        }
                    }
                }
                (footer(&id))
            }
        },
    )
}
