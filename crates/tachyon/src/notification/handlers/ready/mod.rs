use log::warn;
use matrix_sdk::Client;
use tokio::sync::mpsc::Sender;
use msnp::msnp::notification::command::command::{NotificationClientCommand, NotificationServerCommand};
use adl_handler::handle_adl;
use chg_handler::handle_chg;
use fqy_handler::handle_fqy;
use png_handler::handle_png;
use prp_handler::handle_prp;
use put_handler::handle_put;
use rml_handler::handle_rml;
use url_handler::handle_url;
use usr_handler::handle_usr;
use uum_handler::handle_uum;
use uux_handler::handle_uux;
use xfr_handler::handle_xfr;
use crate::notification::models::local_client_data::LocalClientData;
use crate::tachyon::client::tachyon_client::TachyonClient;

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

pub(super) async fn handle_ready(raw_command: NotificationClientCommand, command_sender: Sender<NotificationServerCommand>, tachyon_client: TachyonClient, matrix_client: Client, local_store: &mut LocalClientData) -> Result<(), anyhow::Error> {
    match raw_command {
        NotificationClientCommand::USR(command) => handle_usr(command, local_store.email_addr.clone(), command_sender).await,
        NotificationClientCommand::PNG => handle_png(command_sender).await,
        NotificationClientCommand::ADL(command) => handle_adl(command, tachyon_client, matrix_client, command_sender).await,
        NotificationClientCommand::RML(command) => handle_rml(command, tachyon_client, command_sender).await,
        NotificationClientCommand::UUX(command) => handle_uux(command, local_store, command_sender).await,
        NotificationClientCommand::UUM(command) => handle_uum(command, tachyon_client, matrix_client, command_sender).await,
        NotificationClientCommand::XFR(command) => handle_xfr(command, local_store, command_sender).await,
        NotificationClientCommand::BLP(command) => {
            command_sender.send(NotificationServerCommand::BLP(command)).await?;
            Ok(())
        }
        NotificationClientCommand::CHG(command) => handle_chg(command, local_store, tachyon_client, matrix_client, command_sender).await,
        NotificationClientCommand::PRP(command) => handle_prp(command, local_store, tachyon_client, command_sender).await,
        NotificationClientCommand::UUN(_command) => {Ok(())},
        NotificationClientCommand::RAW(command) => {
            warn!("Received RAW command: {:?}", command);
            Ok(())
        },
        NotificationClientCommand::PUT(command) => handle_put(command, local_store, tachyon_client, command_sender).await,
        NotificationClientCommand::OUT => {Ok(())}
        NotificationClientCommand::VER(_) => {Ok(())}
        NotificationClientCommand::CVR(_) => {Ok(())}
        NotificationClientCommand::FQY(command) => {handle_fqy(command, tachyon_client, command_sender).await}
        NotificationClientCommand::SDG(_) => {Ok(())}
        NotificationClientCommand::URL(command) => {
            handle_url(command, local_store, tachyon_client, command_sender).await
        }
    }
}
