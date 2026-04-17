use chrono::Local;
use env_logger::Builder;
use log::{error, info, warn, LevelFilter};
use std::fs;
use std::fs::File;
use tokio::{join, signal, sync::broadcast::{self, Sender}};

use crate::notification::notification_server::NotificationServer;
use crate::switchboard::switchboard_server::SwitchboardServer;
use crate::tachyon::global_state::GlobalState;
use self::tachyon::config::secret_encryptor::SecretEncryptor;
use crate::web::web_server::WebServer;
use anyhow::anyhow;
use directories::ProjectDirs;
use rand::{random, Rng};
use std::io::Write;
use std::path::PathBuf;
use std::str::FromStr;
use self::tachyon::config::paths;
use self::tachyon::config::paths::create_dirs;
use self::tachyon::config::tachyon_config::TachyonConfig;

mod notification;
mod web;
mod switchboard;
mod matrix;
mod tachyon;

#[tokio::main]
async fn main() {

    let tachyon_path = paths::get_tachyon_path().expect("Tachyon Path to be availlable");
    create_dirs(&tachyon_path);

    let config = setup_config(tachyon_path.config_dir().to_path_buf());
    setup_logs(tachyon_path.data_dir().to_path_buf(), &config);
    let secret = setup_key(tachyon_path.data_local_dir().to_path_buf()).expect("secret key is mandatory");

    let (master_kill_signal,  kill_recv) = broadcast::channel::<()>(1);

    let global_state = GlobalState::new(config.clone(), SecretEncryptor::new(&secret).expect("secret key to be valid"));

    let notification_server = NotificationServer::listen("127.0.0.1", config.notification_port, kill_recv.resubscribe(), global_state.clone());
    let switchboard_server = SwitchboardServer::listen("127.0.0.1", config.switchboard_port, kill_recv.resubscribe(), global_state.clone());
    let web_server = WebServer::listen("127.0.0.1", config.http_port, kill_recv, global_state);

    let _result = join!(notification_server, switchboard_server, web_server, listen_for_stop_signal(master_kill_signal));

    info!("Byebye, world!");
}

fn setup_key(config_folder_path: PathBuf) -> Result<Vec<u8>, anyhow::Error> {

    let key_path = config_folder_path.join("local.key");
    match fs::read(&key_path) {
        Ok(existing_key) => {
            Ok(existing_key)
        }
        Err(_) => {
            let secret_bytes: [u8; 32] = random();
            fs::write(&key_path, secret_bytes)?;
            Ok(secret_bytes.to_vec())
        }
    }
}

fn setup_config(config_dir: PathBuf) -> TachyonConfig {

    let settings_path = config_dir.join("config.ini");

    let result : Result<TachyonConfig, anyhow::Error> = match fs::read_to_string(&settings_path) {
        Ok(content) => {
            TachyonConfig::from_str(&content).map_err(|e| anyhow!(e))
        }
        Err(err) => {
            println!("Couldn't load Tachyon config: {:?}", err);
            let default = TachyonConfig::default();
            let default_str = default.to_string();
            if let Err(e) = fs::write(&settings_path, default_str) {
                println!("Couldn't write default config to disk: {:?}", e);
            }
            Ok(default)
        }
    };

    result.expect("default config to be here")
}

fn setup_logs(path: PathBuf, config: &TachyonConfig) {
    if !config.logs_enabled {
        return;
    }

    println!("Logs enabled");

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
        .filter(Some("matrix-sdk"), LevelFilter::Off)
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
    let _result = master_kill_signal.send(());

}

