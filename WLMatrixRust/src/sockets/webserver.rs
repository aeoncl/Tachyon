use std::io::Error;

use actix_web::{get, web, App, HttpServer, Responder, dev::Server, HttpResponse, post, HttpRequest};
use yaserde::de::from_str;
use yaserde::ser::to_string;
use http::StatusCode;

use crate::generated::ppcrl_webservice::*;

#[get("/hello/{name}")]
async fn greet(name: web::Path<String>) -> impl Responder {
    format!("Hello {name}!")
}

#[post("/RST2.srf")]
async fn rst2(body: web::Bytes, request: HttpRequest) -> HttpResponse {
    let test = std::str::from_utf8(&body).unwrap();

    let request_parsed : RST2RequestMessageSoapEnvelope = from_str(test).unwrap();

    println!("username: {}", request_parsed.header.security.username_token.unwrap().username);

    return HttpResponse::new(StatusCode::OK);
}

