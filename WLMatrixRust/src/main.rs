mod sockets;
mod generated;
mod repositories;
mod models;
mod web;
mod utils;

use std::collections::HashMap;
use std::sync::Arc;
use std::sync::Mutex;

use actix_web::App;
use actix_web::HttpServer;
use actix_web::middleware::Logger;
use matrix_sdk::Client;
use sockets::tcpserver::*;
use web::webserver::*;
use tokio::join;
use lazy_static::lazy_static;
#[macro_use] extern crate lazy_static_include;
#[macro_use] extern crate serde_derive;

use crate::repositories::client_data_repository::ClientDataRepository;
use crate::repositories::matrix_client_repository::MatrixClientRepository;
use crate::repositories::repository::Repository;

lazy_static! {
    static ref MATRIX_CLIENT_REPO: Arc<MatrixClientRepository> = Arc::new(MatrixClientRepository::new());
    static ref CLIENT_DATA_REPO : Arc<ClientDataRepository> = Arc::new(ClientDataRepository::new());
}

#[tokio::main]
async fn main() {

    env_logger::init_from_env(env_logger::Env::new().default_filter_or("info"));

    let notif_server = TCPServer::new("127.0.0.1".to_string(), 1863, ServerType::Notification);
    let switchboard_server = TCPServer::new("127.0.0.1".to_string(), 1864, ServerType::Switchboard);

    let notif_server_future = notif_server.listen();

    let switchboard_server_future = switchboard_server.listen();

    let http_server = HttpServer::new(|| App::new().wrap(Logger::new(r#"%a "%r" %{SOAPAction}i %s %b "%{Referer}i" "%{User-Agent}i" %T"#))
    .service(greet).service(rst2)
    .service(get_msgr_config)
    .service(soap_adress_book_service)
    .service(soap_sharing_service)
    .service(soap_storage_service)
    .service(sha1auth)
    .service(get_profile_pic))
    .workers(2)
    .bind(("127.0.0.1", 8080)).unwrap()
    .run();

    let _test = join!(notif_server_future, switchboard_server_future, http_server);
    println!("See you next time 👀!");
}
