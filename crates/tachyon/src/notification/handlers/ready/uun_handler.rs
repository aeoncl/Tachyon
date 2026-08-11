use log::debug;
use crate::tachyon::client::tachyon_client::TachyonClient;
use matrix_sdk::{Client, Room};
use msnp::msnp::notification::command::command::NotificationServerCommand;
use msnp::msnp::notification::command::uun::{UunClient, UunPayload};
use tokio::sync::mpsc::Sender;
use msnp::msnp::error::CommandError;
use msnp::msnp::notification::command::ubn::{UbnPayload, UbnServer};
use msnp::p2p::v2::slp::raw_slp_payload::SlpPayloadFactory;
use msnp::shared::command::err::{ErrCommand, MsnpError};
use crate::matrix::extensions::msn_user_resolver::FindRoomFromEmail;

pub async fn handle_uun(command: UunClient, client_data: TachyonClient, matrix_client: Client, command_sender: Sender<NotificationServerCommand>) -> Result<(), anyhow::Error>  {
    let ok_response = command.get_ok_response();

    match command.payload {
        UunPayload::DisconnectClient => {
            command_sender.send(NotificationServerCommand::OK(ok_response)).await?;

        }
        UunPayload::DisconnectAllClients => {
            command_sender.send(NotificationServerCommand::OK(ok_response)).await?;

        }
        UunPayload::ConversationWindowClosed { .. } => {
            command_sender.send(NotificationServerCommand::OK(ok_response)).await?;

        }
        UunPayload::DismissUserInvite { .. } => {
            command_sender.send(NotificationServerCommand::OK(ok_response)).await?;

        }
        UunPayload::Resynchronize(_) => {
            command_sender.send(NotificationServerCommand::OK(ok_response)).await?;

        }
        UunPayload::P2PData(transport_req) => {

            let slp_transport_req_error_response = SlpPayloadFactory::get_500_error_direct_connect(
                &transport_req,
                String::from("TCPv1"),
            ).unwrap();


            command_sender.send(NotificationServerCommand::OK(ok_response)).await?;
            let ubn = NotificationServerCommand::UBN(UbnServer::new(command.destination, UbnPayload::P2PData(slp_transport_req_error_response)));
            command_sender.send(ubn).await?;
        }
        UunPayload::Unknown(content) => {

            debug!("UUN: {}", String::from_utf8(content).unwrap());


        }
    };

    Ok(())

}