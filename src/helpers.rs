use std::sync::LazyLock;

use crate::{Headers, IP};
use actix_http::header::SERVER;
use actix_web::HttpRequest;
use actix_web::http::header::{
    ACCEPT, ACCEPT_CHARSET, ACCEPT_ENCODING, ACCEPT_LANGUAGE, ACCEPT_RANGES, HOST, USER_AGENT,
};
use clap::Parser;

pub static CONFIG: LazyLock<Config> = LazyLock::new(|| config());

#[derive(Parser, Debug)]
#[command(about = "An ifconfig clone, made in Rust!")]
pub struct Config {
    #[arg(
        long = "host",
        help = "Bind to all interfaces",
        default_value_t = false
    )]
    pub host: bool,

    #[arg(
        long = "serve-path",
        help = "Path to serve css file from, a web server like nginx is recommended to serve",
        default_value = "/srv"
    )]
    pub serve_path: String,

    #[arg(
        long = "css-file",
        help = "CSS file to serve",
        default_value = "styles.min.css"
    )]
    pub css_file: String,

    #[arg(
        short = '4',
        help = "IPv4 address range to bind to",
        default_value = IP
    )]
    pub bind_ip: String,

    #[arg(
        short = '6',
        help = "IPv6 address range to bind to",
        default_value = "[::1]"
    )]
    pub bind_ip6: String,

    #[arg(short = 'p', help = "Port to bind to", default_value_t = 8080)]
    pub port: u16,

    #[arg(long = "p6", help = "IPv6 port to bind to", default_value_t = 8081)]
    pub port6: u16,
}

pub fn config() -> Config {
    let mut config = Config::parse();

    if config.host {
        config.bind_ip = "0.0.0.0".to_string();
        config.bind_ip6 = "[::]".to_string();
    }

    println!(
        "Listening on {}:{} and {}:{}",
        config.bind_ip, config.port, config.bind_ip6, config.port6
    );
    println!(
        "Serving {} file from {}",
        config.css_file, config.serve_path
    );

    config
}

pub fn gen_response(req: HttpRequest) -> Vec<(String, String)> {
    let headers = req.headers();
    let connection = req.connection_info();
    let ip = connection.realip_remote_addr().unwrap();

    let header_list = vec![
        ACCEPT,
        ACCEPT_ENCODING,
        ACCEPT_LANGUAGE,
        ACCEPT_CHARSET,
        ACCEPT_RANGES,
        USER_AGENT,
        HOST,
        SERVER,
    ];

    let mut response_headers = vec![];

    response_headers.push((Headers::IP_ADDRESS.into(), ip.into()));
    for header in header_list {
        if let Some(value) = headers.get(&header) {
            response_headers.push((header.as_str().into(), value.to_str().unwrap().into()));
        }
    }
    response_headers.push((Headers::METHOD.into(), req.method().as_str().into()));
    response_headers.push(("version".into(), format!("{:?}", req.version())));

    response_headers
}
