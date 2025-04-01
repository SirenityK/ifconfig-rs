mod helpers;
mod ssr;

use actix_files::NamedFile;
use actix_web::{
    App, HttpRequest, HttpResponse, HttpServer,
    http::header::{ContentType, USER_AGENT},
    middleware, web,
};
use helpers::{CONFIG, gen_response};
use serde_json::{Value, json};
use ssr::{Beautifier, render};
use std::{io::Result, path::PathBuf};

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

static IP: &str = if cfg!(debug_assertions) {
    "0.0.0.0"
} else {
    "127.0.0.1" // host with a reverse proxy
};

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
        let mut resp = req
            .connection_info()
            .realip_remote_addr()
            .unwrap()
            .to_owned();
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

async fn serve() -> Result<NamedFile> {
    let mut path = PathBuf::from(CONFIG.serve_path.to_owned());
    let fpath: PathBuf = CONFIG.css_file.to_owned().into();
    path.push(fpath);

    Ok(NamedFile::open(path)?)
}

#[actix_web::main]
async fn main() -> Result<()> {
    let mut app = HttpServer::new(|| {
        App::new()
            .wrap(middleware::DefaultHeaders::new().add(("Server", "actix-web")))
            .route(Routes::ROOT, web::get().to(index))
            .route(Routes::ALL, web::get().to(index_all))
            .route(Routes::ALL_JSON, web::get().to(index_all_json))
            .route(&format!("/{}", CONFIG.css_file), web::get().to(serve))
    });

    if !CONFIG.bind_ip.is_empty() {
        app = app.bind((CONFIG.bind_ip.to_owned(), CONFIG.port))?;
    }
    if !CONFIG.bind_ip6.is_empty() {
        app = app.bind(format!("{}:{}", CONFIG.bind_ip6, CONFIG.port6))?;
    }

    app.run().await
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
            .insert_header((HOST, CONFIG.bind_ip.to_owned()))
            .to_request()
    }

    fn browser_request() -> Request {
        test::TestRequest::default()
            .insert_header((USER_AGENT, "Mozilla/5.0"))
            .insert_header((ACCEPT, "text/html"))
            .insert_header((HOST, CONFIG.bind_ip.to_owned()))
            .to_request()
    }

    #[actix_web::test]
    async fn curl_ip() {
        let app = test::init_service(App::new().route(Routes::ROOT, web::get().to(index))).await;
        let resp = test::call_service(&app, curl_request()).await;

        assert_eq!(
            test::read_body(resp).await,
            format!("{}\n", CONFIG.bind_ip).as_bytes()
        );
    }

    #[actix_web::test]
    async fn html() {
        let app = test::init_service(App::new().route(Routes::ROOT, web::get().to(index))).await;
        let resp = test::call_service(&app, browser_request()).await;

        assert!(test::read_body(resp).await.len() > 0);
    }
}
