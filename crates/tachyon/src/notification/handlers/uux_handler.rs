use log::debug;
use tokio::sync::mpsc::Sender;
use msnp::msnp::notification::command::adl::RmlClient;
use msnp::msnp::notification::command::command::NotificationServerCommand;
use msnp::msnp::notification::command::uux::{UuxClient, UuxPayload};
use crate::notification::client_store::ClientData;
use crate::notification::models::local_client_data::LocalClientData;

pub async fn handle_uux(command: UuxClient, local_store: &mut LocalClientData, command_sender: Sender<NotificationServerCommand>) -> Result<(), anyhow::Error>  {
    let ok_resp = command.get_ok_response();

    match command.payload {
        None => {}
        Some(payload) => {
            match payload {
                UuxPayload::PrivateEndpointData(private_endpoint_data) => {
                    local_store.private_endpoint_data = private_endpoint_data;
                    //TODO
                }
                UuxPayload::Unknown(_) => {}
            }
        }
    }

    command_sender.send(NotificationServerCommand::UUX(ok_resp)).await?;
    Ok(())
}
