use std::sync::Arc;
use tokio::{join, sync::broadcast::Receiver};

use crate::notification::notification_server::NotificationServer;
use crate::switchboard::switchboard_server::SwitchboardServer;
use crate::tachyon::global_state::GlobalState;
use crate::web::web_server::WebServer;
use crate::matrix::services::login::MatrixLoginServiceImpl;
use crate::tachyon::config::paths;
use crate::tachyon::config::paths::create_dirs;
use crate::tachyon::config::tachyon_config::TachyonConfig;
use tachyon_sqlite_store::SqliteCredentialsStore;

pub mod notification;
pub mod web;
pub mod switchboard;
pub mod matrix;
pub mod tachyon;
pub mod p2p;
pub mod audio;

pub async fn run(config: TachyonConfig, global_shutdown_signal_rcv: Receiver<()>) -> Result<(), anyhow::Error> {
    let tachyon_path = paths::get_tachyon_path();
    create_dirs(&tachyon_path);

    let credentials = Arc::new(SqliteCredentialsStore::open(&paths::credentials_db_path())?);

    let global_state = GlobalState::new(config.clone(), Box::new(MatrixLoginServiceImpl::new()), credentials);

    let notification_server = NotificationServer::listen("127.0.0.1", config.notification_port, global_shutdown_signal_rcv.resubscribe(), global_state.clone());
    let switchboard_server = SwitchboardServer::listen("127.0.0.1", config.switchboard_port, global_shutdown_signal_rcv.resubscribe(), global_state.clone());
    let web_server = WebServer::listen("127.0.0.1", config.http_port, global_shutdown_signal_rcv, global_state);

    let _result = join!(notification_server, switchboard_server, web_server);

    Ok(())
}
