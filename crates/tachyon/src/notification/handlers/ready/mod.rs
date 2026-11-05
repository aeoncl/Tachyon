use log::warn;
use matrix_sdk::Client;
use tokio::sync::mpsc::Sender;
use msnp::msnp::notification::command::command::{NotificationClientCommand, NotificationServerCommand};
use adl_handler::handle_adl;
use chg_handler::handle_chg;
use fqy_handler::decide_fqy;
use png_handler::decide_png;
use prp_handler::handle_prp;
use put_handler::decide_put;
use rml_handler::handle_rml;
use url_handler::decide_url;
use usr_handler::handle_usr;
use uum_handler::handle_uum;
use uux_handler::handle_uux;
use xfr_handler::handle_xfr;
use crate::notification::handlers::ready::uun_handler::decide_uun;
use crate::notification::models::client_out::ClientOut;
use crate::notification::models::session_state::SessionState;
use crate::tachyon::client::tachyon_client::TachyonClient;
use crate::tachyon::config::tachyon_config::TachyonConfig;

mod usr_handler;
mod png_handler;
mod adl_handler;
mod rml_handler;
mod uux_handler;
mod uum_handler;
mod chg_handler;
mod put_handler;
mod xfr_handler;
mod prp_handler;
mod fqy_handler;
mod url_handler;
mod uun_handler;

pub(super) async fn handle_ready(raw_command: NotificationClientCommand, command_sender: Sender<NotificationServerCommand>, tachyon_client: TachyonClient, matrix_client: Client, local_store: &mut SessionState, config: &TachyonConfig) -> Result<(), anyhow::Error> {
    match raw_command {
        NotificationClientCommand::USR(command) => handle_usr(command, local_store.email_addr.clone(), command_sender).await,
        NotificationClientCommand::PNG => send_client_outs([decide_png()], &command_sender).await,
        NotificationClientCommand::ADL(command) => handle_adl(command, tachyon_client, matrix_client, command_sender).await,
        NotificationClientCommand::RML(command) => send_client_outs([handle_rml(command, tachyon_client)?], &command_sender).await,
        NotificationClientCommand::UUX(command) => send_client_outs([handle_uux(command, local_store)], &command_sender).await,
        NotificationClientCommand::UUM(command) => handle_uum(command, tachyon_client, matrix_client, command_sender).await,
        NotificationClientCommand::XFR(command) => handle_xfr(command, local_store, command_sender, config).await,
        NotificationClientCommand::BLP(command) => {
            command_sender.send(NotificationServerCommand::BLP(command)).await?;
            Ok(())
        }
        NotificationClientCommand::CHG(command) => handle_chg(command, local_store, tachyon_client, matrix_client, command_sender).await,
        NotificationClientCommand::PRP(command) => send_client_outs([handle_prp(command, tachyon_client)], &command_sender).await,
        NotificationClientCommand::UUN(command) => send_client_outs(decide_uun(command), &command_sender).await,
        NotificationClientCommand::RAW(command) => {
            warn!("Received RAW command: {:?}", command);
            Ok(())
        }
        NotificationClientCommand::PUT(command) => send_client_outs(decide_put(command), &command_sender).await,
        NotificationClientCommand::OUT => {Ok(())}
        NotificationClientCommand::VER(_) => {Ok(())}
        NotificationClientCommand::CVR(_) => {Ok(())}
        NotificationClientCommand::FQY(command) => {send_client_outs([decide_fqy(command)], &command_sender).await}
        NotificationClientCommand::SDG(_) => {Ok(())}
        NotificationClientCommand::URL(command) => {
            send_client_outs([decide_url(command)], &command_sender).await
        }
    }
}

/// Dispatch-level execute step (§3.1): pattern A verbs return `ClientOut`s
/// without holding a `Sender`; the dispatcher sends them through its own
/// channel, preserving each verb's pre-D2 send policy (`?` on every send).
/// Every pattern A verb emits `Command(..)`; `ContactPresence` is produced
/// only by chg, which is a pattern B shell that renders and sends its own
/// fan-out — so this arm would be a defect, logged loudly, never rendered.
async fn send_client_outs(outs: impl IntoIterator<Item = ClientOut>, command_sender: &Sender<NotificationServerCommand>) -> Result<(), anyhow::Error> {
    for out in outs {
        match out {
            ClientOut::Command(command) => {
                command_sender.send(command).await?;
            }
            ClientOut::ContactPresence { .. } => {
                warn!("MSNP|NS: a dispatch-level verb produced ContactPresence; only chg renders presence — dropped");
            }
        }
    }
    Ok(())
}
