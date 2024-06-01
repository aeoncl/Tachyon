use std::fs;
use std::fs::File;
use chrono::Local;
use env_logger::{Builder, Target};
use log::{error, info, LevelFilter, warn};
use tokio::{join, select, signal, sync::broadcast::{self, Sender}};

use crate::notification::notification_server::NotificationServer;
use crate::switchboard::switchboard_server::SwitchboardServer;
use crate::web::web_server::WebServer;
use std::io::Write;
use std::path::{Path, PathBuf};
use std::str::FromStr;
use anyhow::anyhow;
use directories::ProjectDirs;
use env_logger::Target::Stdout;
use crate::notification::client_store::{ClientStoreFacade};
use crate::shared::paths;
use crate::shared::paths::create_dirs;
use crate::shared::tachyon_config::TachyonConfig;

mod notification;
mod web;
mod switchboard;
mod shared;
mod matrix;

#[tokio::main]
async fn main() {
    let (tachyon_path, config) = setup_config();
    setup_logs(tachyon_path.data_dir().to_path_buf(), &config);

    let (master_kill_signal,  kill_recv) = broadcast::channel::<()>(1);

    let client_store_facade = ClientStoreFacade::default();

    let notification_server = NotificationServer::listen("127.0.0.1", 1863, kill_recv.resubscribe(), client_store_facade.clone());
    let switchboard_server = SwitchboardServer::listen("127.0.0.1", 1864, kill_recv.resubscribe(), client_store_facade.clone());
    let web_server = WebServer::listen("127.0.0.1", 8080, kill_recv, client_store_facade);

    join!(notification_server, switchboard_server, web_server, listen_for_stop_signal(master_kill_signal));

    info!("Byebye, world!");
}

fn setup_config() -> (ProjectDirs, TachyonConfig) {
    let paths = paths::get_tachyon_path().expect("Tachyon Path to be availlable");
    create_dirs(&paths);

    let settings_path = paths.config_dir().join("config.json");

    let result : Result<TachyonConfig, anyhow::Error> = match fs::read_to_string(&settings_path) {
        Ok(content) => {
            TachyonConfig::from_str(&content).map_err(|e| anyhow!(e))
        }
        Err(err) => {
            println!("Couldn't load Tachyon config: {:?}", err);
            let config = TachyonConfig::default();
            if let Err(e) = fs::write(settings_path, config.to_string().into_bytes()) {
                println!("Couldn't write Tachyon config file {:?}", e);
            }

            Ok(config)
        }
    };

    (paths, result.expect("default config to be here"))
}

fn setup_logs(path: PathBuf, config: &TachyonConfig) {
    if !config.enable_logging {
        return;
    }

    let log_path = path.join("logs.txt");
    let target = Box::new(File::create(log_path).expect("Can't create file"));
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
        .target(env_logger::Target::Pipe(target))
        .filter(Some("v2") , LevelFilter::Debug)
        .filter(Some("tachyon") , LevelFilter::Debug)
        .filter(Some("msnp") , LevelFilter::Debug)
        .filter(Some("matrix-sdk"), LevelFilter::Warn)
        .filter(Some("yaserde"), LevelFilter::Warn)
        .filter(None, LevelFilter::Warn)
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

