mod ssr;

use actix_web::{
    App, HttpRequest, HttpResponse, HttpServer,
    dev::ConnectionInfo,
    get,
    http::header::{
        ACCEPT, ACCEPT_CHARSET, ACCEPT_ENCODING, ACCEPT_LANGUAGE, ACCEPT_RANGES, ContentType, HOST,
        USER_AGENT,
    },
    middleware,
};
use serde_json::{Value, json};
use ssr::{Beautifier, render};

static PORT: u16 = 8080;
static HEADER_BREAK: &[char] = &[',', ';'];
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
            None => Some(IP),
        }
    }
}

struct Headers {}
impl Headers {
    const IP_ADDRESS: &'static str = "ip_address";
    const METHOD: &'static str = "method";
}

fn headers_checks(header: &str) -> bool {
    !(header.starts_with("dnt") || header.starts_with("sec") || header.starts_with("x-"))
}

fn gen_headers(req: HttpRequest) -> Vec<(String, String)> {
    let accepted_headers = vec![
        ACCEPT,
        ACCEPT_ENCODING,
        ACCEPT_LANGUAGE,
        USER_AGENT,
        ACCEPT_CHARSET,
        ACCEPT_RANGES,
        HOST,
    ];
    let mut header_pairs: Vec<(String, String)> = Vec::new();
    let headers = req.headers();
    let connection = req.connection_info();
    let ip = connection.ip().unwrap();

    for (key, value) in headers.iter() {
        let key_str = key.as_str();
        if accepted_headers.contains(key) && headers_checks(key_str) && value.len() > 0 {
            header_pairs.push((key_str.to_string(), value.to_str().unwrap().to_string()));
        }
    }

    // sort fheaders by accepted_headers order
    header_pairs.sort_by(|a, b| {
        let a = accepted_headers
            .iter()
            .position(|x| x == a.0.as_str())
            .unwrap();
        let b = accepted_headers
            .iter()
            .position(|x| x == b.0.as_str())
            .unwrap();
        a.cmp(&b)
    });

    header_pairs.insert(0, (Headers::IP_ADDRESS.to_string(), ip.to_string()));
    header_pairs.push((Headers::METHOD.to_string(), req.method().to_string()));

    header_pairs
}

fn all(req: HttpRequest) -> String {
    let response_headers = gen_headers(req);

    response_headers
        .iter()
        .map(|(key, value)| format!("{}: {}", key, value))
        .collect()
}

fn json(response_headers: Vec<(String, String)>) -> Value {
    let mut body = json!({});

    for (key, value) in response_headers {
        body[key] = value.into();
    }

    body
}

#[get("/")]
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
        render(req).into_string()
    };
    HttpResponse::Ok().content_type(content_type).body(body)
}

#[get("/all")]
async fn index_all(req: HttpRequest) -> HttpResponse {
    let body = all(req);

    HttpResponse::Ok()
        .content_type(ContentType::plaintext())
        .body(body)
}

#[get("/all.json")]
async fn index_all_json(req: HttpRequest) -> HttpResponse {
    let response_headers = gen_headers(req);
    let body = json(response_headers);

    HttpResponse::Ok()
        .content_type(ContentType::json())
        .body(body.to_string().breakline())
}

#[actix_web::main]
async fn main() -> std::io::Result<()> {
    println!("Server running at http://{}:{}", IP, PORT);
    println!("Server running at http://[::1]:{}", PORT);
    HttpServer::new(|| {
        App::new()
            .wrap(middleware::DefaultHeaders::new().add(("Server", "boringcalculator")))
            .service(index)
            .service(index_all_json)
            .service(index_all)
    })
    .bind((IP, PORT))?
    .bind("[::1]:8081")?
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
            .insert_header((HOST, "127.0.0.1"))
            .to_request()
    }

    fn browser_request() -> Request {
        test::TestRequest::default()
            .insert_header((USER_AGENT, "Mozilla/5.0"))
            .insert_header((ACCEPT, "text/html"))
            .insert_header((HOST, "127.0.0.1"))
            .to_request()
    }

    #[actix_web::test]
    async fn curl_ip() {
        let app = test::init_service(App::new().service(index)).await;
        let resp = test::call_service(&app, curl_request()).await;

        assert_eq!(test::read_body(resp).await, IP.as_bytes());
    }

    #[actix_web::test]
    async fn html() {
        let app = test::init_service(App::new().service(index)).await;
        let resp = test::call_service(&app, browser_request()).await;

        assert!(test::read_body(resp).await.len() > 0);
    }
}
