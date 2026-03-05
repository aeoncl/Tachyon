use std::str::FromStr;
use anyhow::{anyhow, Error};
use matrix_sdk::Room;
use crate::notification::client_store::{ClientData, ClientStoreFacade};
use crate::switchboard::models::connection_phase::ConnectionPhase;
use crate::switchboard::models::local_switchboard_data::LocalSwitchboardData;
use msnp::msnp::switchboard::command::command::{SwitchboardClientCommand, SwitchboardServerCommand};
use tokio::sync::mpsc::Sender;
use msnp::msnp::switchboard::command::usr::UsrServer;
use msnp::shared::models::ticket_token::TicketToken;
use crate::matrix::extensions::msn_user_resolver::FindRoomFromEmail;

pub(crate) async fn handle_command(command: SwitchboardClientCommand, command_sender: Sender<SwitchboardServerCommand>, client_store: &ClientStoreFacade, local_switchboard_data: &mut LocalSwitchboardData) -> Result<(), anyhow::Error> {

    match local_switchboard_data.phase {
        ConnectionPhase::Authenticating => {
            handle_auth(command, command_sender, client_store, local_switchboard_data).await?
        }
        ConnectionPhase::Ready => {
            let room = local_switchboard_data.room.as_ref().ok_or(anyhow!("Room should be here by now"))?.clone();
            let client_data = local_switchboard_data.client_data.as_ref().ok_or(anyhow!("Client Data should be here by now"))?.clone();
        }
    }


    Ok(())
}

pub(crate) async fn handle_auth(command: SwitchboardClientCommand, command_sender: Sender<SwitchboardServerCommand>, client_store: &ClientStoreFacade, local_switchboard_data: &mut LocalSwitchboardData) -> Result<(), anyhow::Error> {

    match command {
        SwitchboardClientCommand::ANS(ans_command) => {



        }
        SwitchboardClientCommand::USR(usr_command) => {
            local_switchboard_data.email_addr = usr_command.endpoint_id.email_addr.clone();

            let token = TicketToken::from_str(&usr_command.token).unwrap();

            match client_store.get_client_data(token.as_str()) {
                None => {
                    //TODO AUTH error
                }
                Some(client_data) => {

                    match client_data.get_matrix_client()
                        .find_room_from_email(&local_switchboard_data.email_addr) {
                        Ok(Some(room)) => {
                            local_switchboard_data.room = Some(room);
                        }
                        Ok(None) => {
                            //TODO Error room does not exist.
                        }
                        Err(err) => {
                            //TODO Error finding room
                        }
                    }
                    local_switchboard_data.token = token;
                    local_switchboard_data.client_data = Some(client_data);

                    //Start load initial roster task
                    local_switchboard_data.phase = ConnectionPhase::Ready;
                    let email = &local_switchboard_data.email_addr;
                    let _ = command_sender.send(SwitchboardServerCommand::USR(usr_command.get_ok_response_for(email.to_string()))).await;
                }
            }

        }
        _ => {
            //TODO: send error when receiving another command
        }
    }

    Ok(())

}
