mod ssr;

use std::env;

use actix_http::header::SERVER;
use actix_web::{
    App, HttpRequest, HttpResponse, HttpServer,
    dev::ConnectionInfo,
    http::header::{
        ACCEPT, ACCEPT_CHARSET, ACCEPT_ENCODING, ACCEPT_LANGUAGE, ACCEPT_RANGES, ContentType, HOST,
        USER_AGENT,
    },
    middleware, web,
};
use serde_json::{Value, json};
use ssr::{Beautifier, render};

static PORT: u16 = 8080;
static PORT6: u16 = 8081; // for IPv6, different port to avoid conflicts
static IP: &str = if cfg!(debug_assertions) {
    "0.0.0.0"
} else {
    "127.0.0.1" // host with a reverse proxy
};

trait Client {
    fn ip(&self) -> Option<&str>;
}
impl Client for ConnectionInfo {
    fn ip(&self) -> Option<&str> {
        match self.realip_remote_addr() {
            Some(ip) => Some(ip),
            // only matches in tests
            None => Some(IP),
        }
    }
}

struct Routes {}
impl Routes {
    const ROOT: &'static str = "/";
    const ALL: &'static str = "/all";
    const ALL_JSON: &'static str = "/all.json";
}

struct Headers {}
impl Headers {
    const IP_ADDRESS: &'static str = "ip_address";
    const METHOD: &'static str = "method";
}

fn gen_response(req: HttpRequest) -> Vec<(String, String)> {
    let headers = req.headers();
    let connection = req.connection_info();
    let ip = connection.ip().unwrap();

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

fn json(response_headers: Vec<(String, String)>) -> Value {
    let mut body = json!({});

    for (key, value) in response_headers {
        body[key] = value.into();
    }

    body
}

async fn index(req: HttpRequest) -> HttpResponse {
    let headers = req.headers();
    let user_agent = headers.get(USER_AGENT).unwrap().to_str().unwrap();
    let is_curl = user_agent.contains("curl");
    let content_type = if is_curl {
        ContentType::plaintext()
    } else {
        ContentType::html()
    };

    let body = if is_curl {
        let mut resp = req.connection_info().ip().unwrap().to_string();
        resp.breakline();
        resp
    } else {
        render(req).into()
    };
    HttpResponse::Ok().content_type(content_type).body(body)
}

async fn index_all(req: HttpRequest) -> HttpResponse {
    let response_headers = gen_response(req);
    let mut body = String::new();
    response_headers.iter().for_each(|(key, value)| {
        body.push_str(&format!("{}: {}\n", key, value));
    });

    HttpResponse::Ok()
        .content_type(ContentType::plaintext())
        .body(body)
}

async fn index_all_json(req: HttpRequest) -> HttpResponse {
    let response_headers = gen_response(req);
    let body = json(response_headers);

    HttpResponse::Ok()
        .content_type(ContentType::json())
        .body(body.to_string())
}

#[actix_web::main]
async fn main() -> std::io::Result<()> {
    let mut host = false;
    for arg in env::args() {
        if arg == "--host" {
            host = true;
        }
    }

    let (bind_ip, bind_ip6) = if host {
        ("0.0.0.0", "[::]")
    } else {
        (IP, "[::1]")
    };

    println!("Server running at http://{bind_ip}:{PORT}");
    println!("Server running at http://{bind_ip6}:{PORT6}");
    HttpServer::new(|| {
        App::new()
            .wrap(middleware::DefaultHeaders::new().add(("Server", "actix-web")))
            .route(Routes::ROOT, web::get().to(index))
            .route(Routes::ALL, web::get().to(index_all))
            .route(Routes::ALL_JSON, web::get().to(index_all_json))
    })
    .bind((bind_ip, PORT))?
    .bind(format!("{bind_ip6}:{PORT6}"))?
    .run()
    .await
}

#[cfg(test)]
mod tests {
    use super::*;
    use actix_http::{
        Request,
        header::{ACCEPT, HOST, USER_AGENT},
    };
    use actix_web::test;

    fn curl_request() -> Request {
        test::TestRequest::default()
            .insert_header((USER_AGENT, "curl"))
            .insert_header((ACCEPT, "*/*"))
            .insert_header((HOST, IP))
            .to_request()
    }

    fn browser_request() -> Request {
        test::TestRequest::default()
            .insert_header((USER_AGENT, "Mozilla/5.0"))
            .insert_header((ACCEPT, "text/html"))
            .insert_header((HOST, IP))
            .to_request()
    }

    #[actix_web::test]
    async fn curl_ip() {
        let app = test::init_service(App::new().route(Routes::ROOT, web::get().to(index))).await;
        let resp = test::call_service(&app, curl_request()).await;

        assert_eq!(test::read_body(resp).await, format!("{}\n", IP).as_bytes());
    }

    #[actix_web::test]
    async fn html() {
        let app = test::init_service(App::new().route(Routes::ROOT, web::get().to(index))).await;
        let resp = test::call_service(&app, browser_request()).await;

        assert!(test::read_body(resp).await.len() > 0);
    }
}
