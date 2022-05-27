mod sockets;
mod generated;
mod repositories;
mod models;
mod web;
mod utils;

use std::sync::Arc;

use actix_web::App;
use actix_web::HttpServer;
use actix_web::middleware::Logger;
use web::webserver::*;
use web::sharing_service::*;
use web::storage_service::*;
use web::ab_service::*;

use tokio::join;
use lazy_static::lazy_static;
#[macro_use] extern crate lazy_static_include;
#[macro_use] extern crate serde_derive;

use crate::repositories::ab_data_repository::AbDataRepository;
use crate::repositories::client_data_repository::ClientDataRepository;
use crate::repositories::matrix_client_repository::MatrixClientRepository;
use crate::repositories::p2p_repository::P2PRepository;
use crate::repositories::repository::Repository;
use crate::sockets::echoserver::EchoServer;
use crate::sockets::p2p_server::P2PServer;
use crate::sockets::notification_server::NotificationServer;
use crate::sockets::switchboard_server::SwitchboardServer;
use crate::sockets::tcpserver::TCPServer;

lazy_static! {
    static ref MATRIX_CLIENT_REPO: Arc<MatrixClientRepository> = Arc::new(MatrixClientRepository::new());
    static ref CLIENT_DATA_REPO : Arc<ClientDataRepository> = Arc::new(ClientDataRepository::new());
    static ref AB_DATA_REPO : Arc<AbDataRepository> = Arc::new(AbDataRepository::new());
    static ref P2P_REPO : Arc<P2PRepository> = Arc::new(P2PRepository::new());

}

#[tokio::main]
async fn main() {

    env_logger::init_from_env(env_logger::Env::new().default_filter_or("info"));

    let notif_server = NotificationServer::new("127.0.0.1".to_string(), 1863);
    let switchboard_server = SwitchboardServer::new("127.0.0.1".to_string(), 1864);
    let bad_server = P2PServer::new("127.0.0.1".to_string(), 1865);
    let echo_server = EchoServer::new("127.0.0.1".to_string(), 7001);
    
    let notif_server_future = notif_server.listen();

    let switchboard_server_future = switchboard_server.listen();

    let bad_server_future = bad_server.listen();

    let echo_server_future = echo_server.listen();

    let http_server = HttpServer::new(|| App::new().wrap(Logger::new(r#"%a "%r" %{SOAPAction}i %s %b "%{Referer}i" "%{User-Agent}i" %T"#))
    .service(greet).service(rst2)
    .service(get_msgr_config)
    .service(soap_adress_book_service)
    .service(soap_sharing_service)
    .service(soap_storage_service)
    .service(sha1auth)
    .service(get_profile_pic))
    .bind(("127.0.0.1", 8080)).unwrap()
    .run();

    let _test = join!(notif_server_future, switchboard_server_future, bad_server_future, http_server, echo_server_future);
    println!("See you next time 👀!");
}
