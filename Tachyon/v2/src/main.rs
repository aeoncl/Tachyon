use std::fs::File;
use chrono::Local;
use env_logger::{Builder, Target};
use log::{error, info, LevelFilter};
use tokio::{join, select, signal, sync::broadcast::{self, Sender}};

use crate::notification::notification_server::NotificationServer;
use crate::switchboard::switchboard_server::SwitchboardServer;
use crate::web::web_server::WebServer;
use std::io::Write;
use crate::notification::client_store::{ClientStoreFacade, start_client_store_task};

mod notification;
mod web;
mod switchboard;
mod shared;

#[tokio::main]
async fn main() {
    setup_logs();
    let (master_kill_signal,  kill_recv) = broadcast::channel::<()>(1);

    let client_store_facade = ClientStoreFacade::new(start_client_store_task(kill_recv.resubscribe()));

    let notification_server = NotificationServer::listen("127.0.0.1", 1863, kill_recv.resubscribe(), client_store_facade.clone());
    let switchboard_server = SwitchboardServer::listen("127.0.0.1", 1864, kill_recv.resubscribe(), client_store_facade.clone());
    let web_server = WebServer::listen("127.0.0.1", 8080, kill_recv, client_store_facade);

    join!(notification_server, switchboard_server, web_server, listen_for_stop_signal(master_kill_signal));
    info!("Byebye, world!");
}


fn setup_logs() {
    let target = Box::new(File::create("C:\\temp\\log.txt").expect("Can't create file"));
    log_print_panics::init();
    Builder::new()
        .format(|buf, record| {
            writeln!(
                buf,
                "{} [{}] - {} @ {}:{}",
                Local::now().format("%d-%m-%YT%H:%M:%S%.3f"),
                record.level(),
                record.args(),
                record.file().unwrap_or("unknown"),
                record.line().unwrap_or(0),
            )
        })
        //.target(env_logger::Target::Pipe(target))
        .target(Target::Stdout)
        .filter(Some("v2") , LevelFilter::Debug)
        .filter(Some("matrix-sdk"), LevelFilter::Trace)
        .filter(None, LevelFilter::Info)
        .init();

    //Some("wlmatrix_rust")
    info!("=========NEW LOG SESSION (✿◡‿◡)  - {}=========", Local::now().format("%d-%m-%YT%H:%M:%S%.3f"));
}


async fn listen_for_stop_signal(master_kill_signal: Sender<()>) {

    match signal::ctrl_c().await {
        Ok(()) => {},
        Err(err) => {
            error!("Unable to listen for shutdown signal: {}", err);
            // we also shut down in case of error
        },
    }
    info!("Sending kill signals");
    master_kill_signal.send(());

}

