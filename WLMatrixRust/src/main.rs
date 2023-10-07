#[macro_use] extern crate lazy_static_include;
#[macro_use] extern crate num_derive;
#[macro_use] extern crate serde_derive;

use std::env;
use std::fs::File;
use std::io::Write;
use std::sync::Arc;

use actix_web::App;
use actix_web::HttpServer;
use actix_web::middleware::Logger;
use chrono::Local;
use env_logger::{Builder, Target};
use lazy_static::lazy_static;
use log::info;
use log::LevelFilter;
use tokio::join;

use web::ab_service::*;
use web::sharing_service::*;
use web::storage_service::*;
use web::webserver::*;

use crate::repositories::ab_locator::ABLocator;
use crate::repositories::matrix_client_locator::MatrixClientLocator;
use crate::repositories::msn_client_locator::MSNClientLocator;
use crate::repositories::p2p_repository::P2PRepository;
use crate::repositories::settings_locator::SettingsLocator;

use crate::repositories::repository::Repository;
use crate::sockets::notification_server::NotificationServer;
use crate::sockets::switchboard_server::SwitchboardServer;
use crate::sockets::tcpserver::TCPServer;

mod sockets;
mod generated;
mod repositories;
mod models;
mod web;
mod utils;

lazy_static! {
    static ref MSN_CLIENT_LOCATOR: Arc<MSNClientLocator> = Arc::new(MSNClientLocator::new());
    static ref MATRIX_CLIENT_LOCATOR: Arc<MatrixClientLocator> = Arc::new(MatrixClientLocator::new());
    static ref AB_LOCATOR : Arc<ABLocator> = Arc::new(ABLocator::new());
    static ref P2P_REPO : Arc<P2PRepository> = Arc::new(P2PRepository::new());
    static ref SETTINGS_LOCATOR: Arc<SettingsLocator> = Arc::new(SettingsLocator::from(env::args()));
}

#[tokio::main]
async fn main() {
    setup_logs();

    let notif_server = NotificationServer::new("127.0.0.1".to_string(), 1863);
    let switchboard_server = SwitchboardServer::new("127.0.0.1".to_string(), 1864);
    
    //let direct_p2p_server = P2PServer::new("127.0.0.1".to_string(), 1865);
    //let direct_p2p_server_future = direct_p2p_server.listen();

    //let echo_server = EchoServer::new("127.0.0.1".to_string(), 7001);
    //let echo_server_future = echo_server.listen();

    let notif_server_future = notif_server.listen();

    let switchboard_server_future = switchboard_server.listen();

    let http_server = HttpServer::new(|| App::new().wrap(Logger::new(r#"%a "%r" %{SOAPAction}i %s %b "%{Referer}i" "%{User-Agent}i" %T"#))
    .service(firewall_test).service(rst2)
    .service(get_msgr_config)
    .service(soap_adress_book_service)
    .service(soap_sharing_service)
    .service(soap_storage_service)
    .service(sha1auth)
    .service(get_profile_pic)
    .service(get_text_ad)
    .service(get_banner))
    .bind(("127.0.0.1", 8080)).unwrap()
    .run();

    let _test = join!(notif_server_future, switchboard_server_future, http_server);
    println!("See you next time 👀!");
}

fn setup_logs() {
    let target = Box::new(File::create("C:\\temp\\log.txt").expect("Can't create file"));
    log_print_panics::init();
    Builder::new()
        .format(|buf, record| {
            writeln!(
                buf,
                "{}:{} {} [{}] - {}",
                record.file().unwrap_or("unknown"),
                record.line().unwrap_or(0),
                Local::now().format("%Y-%m-%dT%H:%M:%S%.3f"),
                record.level(),
                record.args()
            )
        })
        .target(env_logger::Target::Pipe(target))
        .target(Target::Stdout)
        .filter(Some("wlmatrix_rust") , LevelFilter::Info)
        .filter(Some("matrix-sdk"), LevelFilter::Debug)
        .init();

    //Some("wlmatrix_rust")    
    info!("=========NEW LOG SESSION (✿◡‿◡)  - {}=========", Local::now().format("%Y-%m-%dT%H:%M:%S%.3f"));
}