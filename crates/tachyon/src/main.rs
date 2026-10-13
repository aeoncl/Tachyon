use chrono::Local;
use env_logger::Builder;
use log::{error, info, LevelFilter};
use std::fs;
use std::fs::File;
use tokio::{join, signal, sync::broadcast::{self, Sender}};

use tachyon::tachyon::config::paths;
use tachyon::tachyon::config::paths::create_dirs;
use tachyon::tachyon::config::tachyon_config::TachyonConfig;
use anyhow::anyhow;
use std::str::FromStr;
use std::io::Write;
use std::path::PathBuf;

#[tokio::main]
async fn main() {
    //console_subscriber::init();

    let tachyon_path = paths::get_tachyon_path();
    create_dirs(&tachyon_path);

    let config = setup_config(tachyon_path.config_dir().to_path_buf());
    setup_logs(tachyon_path.data_dir().to_path_buf(), &config);

    let (global_shutdown_signal_snd, global_shutdown_signal_rcv) = broadcast::channel::<()>(1);

    let _result = join!(tachyon::run(config, global_shutdown_signal_rcv), listen_for_stop_signal(global_shutdown_signal_snd));

    info!("Byebye, world!");
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


async fn listen_for_stop_signal(global_shutdown_signal_snd: Sender<()>) {
    match signal::ctrl_c().await {
        Ok(()) => {},
        Err(err) => {
            error!("Unable to listen for shutdown signal: {}", err);
            // we also shut down in case of error
        },
    }
    info!("Sending global shutdown signal");
    let _result = global_shutdown_signal_snd.send(());

}
