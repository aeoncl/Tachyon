use crate::tachyon::client_store::ClientStoreFacade;
use crate::switchboard::models::connection_phase::ConnectionPhase;
use crate::switchboard::models::local_switchboard_data::LocalSwitchboardData;
use msnp::msnp::switchboard::command::command::{SwitchboardClientCommand, SwitchboardServerCommand};
use msnp::shared::models::ticket_token::TicketToken;
use std::str::FromStr;
use anyhow::anyhow;
use rand::Rng;
use tokio::sync::mpsc::Sender;
use msnp::msnp::switchboard::command::cal::{CalServer, CalServerFunction};
use msnp::msnp::switchboard::command::joi::JoiServer;
use msnp::msnp::switchboard::models::session_id::SessionId;
use msnp::shared::models::endpoint_id::EndpointId;
use crate::matrix::extensions::msn_user_resolver::{FindRoomFromEmail, ToMsnUser};
use crate::tachyon::tachyon_client::TachyonClient;
use crate::shared::identifiers::MatrixIdCompatible;

pub(crate) async fn handle_command(command: SwitchboardClientCommand, command_sender: Sender<SwitchboardServerCommand>, client_store: &ClientStoreFacade, local_switchboard_data: &mut LocalSwitchboardData) -> Result<(), anyhow::Error> {

    match local_switchboard_data.phase {
        ConnectionPhase::Authenticating => {
            handle_auth(command, command_sender, client_store, local_switchboard_data).await?
        }
        ConnectionPhase::Initializing => {
            //Wait for CAL of room
            //Start load initial roster task
            let client_data = local_switchboard_data.client_data.as_ref().ok_or(anyhow!("Client Data should be here by now"))?.clone();
            handle_init(command, command_sender, client_data, local_switchboard_data).await?

        }
        ConnectionPhase::Ready => {
            //let room = local_switchboard_data.room.as_ref().ok_or(anyhow!("Room should be here by now"))?.clone();
            //let client_data = local_switchboard_data.client_data.as_ref().ok_or(anyhow!("Client Data should be here by now"))?.clone();
        }
    }


    Ok(())
}

pub(crate) async fn handle_auth(command: SwitchboardClientCommand, command_sender: Sender<SwitchboardServerCommand>, client_store: &ClientStoreFacade, local_switchboard_data: &mut LocalSwitchboardData) -> Result<(), anyhow::Error> {

    match command {
        SwitchboardClientCommand::ANS(ans_command) => {



        }
        SwitchboardClientCommand::USR(usr_command) => {

            let token = TicketToken::from_str(&usr_command.token).unwrap();

            match client_store.get_client_data(token.as_str()) {
                None => {
                    //TODO AUTH error
                }
                Some(client_data) => {

                   /* match client_data.get_matrix_client()
                        .find_room_from_email(&local_switchboard_data.email_addr) {
                        Ok(Some(room)) => {
                            local_switchboard_data.room = Some(room);
                        }
                        Ok(None) => {
                            //TODO Error room does not exist.
                        }
                        Err(err) => {
                            //TODO Error finding room
                            println!("{}", &local_switchboard_data.email_addr);
                            println!("{:?}", err)
                        }
                    }*/
                    local_switchboard_data.email_addr = usr_command.endpoint_id.email_addr.clone();
                    local_switchboard_data.endpoint_guid = usr_command.endpoint_id.endpoint_guid.clone();
                    local_switchboard_data.token = token;
                    local_switchboard_data.client_data = Some(client_data);
                    local_switchboard_data.session_id = SessionId::random();
                    local_switchboard_data.phase = ConnectionPhase::Initializing;
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

pub(crate) async fn handle_init(command: SwitchboardClientCommand, command_sender: Sender<SwitchboardServerCommand>, client_data: TachyonClient, local_switchboard_data: &mut LocalSwitchboardData) -> Result<(), anyhow::Error> {
    match command {
        SwitchboardClientCommand::ANS(_) => {}
        SwitchboardClientCommand::USR(_) => {}
        SwitchboardClientCommand::CAL(cal_client) => {
            let email = cal_client.email_addr;
            let user_id = email.to_owned_user_id();


            let _ = command_sender.send(SwitchboardServerCommand::CAL(CalServer {
                tr_id: cal_client.tr_id,
                function: CalServerFunction::RINGING,
                session_id: local_switchboard_data.session_id.clone()
            })).await;


            if email == local_switchboard_data.email_addr {
                // It's me !
                let me = client_data.own_user().unwrap();

                let _ = command_sender.send(SwitchboardServerCommand::JOI(JoiServer {
                    display_name: me.compute_display_name().to_string(),
                    endpoint_id: EndpointId {
                        email_addr: me.get_email_address().clone(),
                        endpoint_guid: None,
                    },
                    capabilities: me.capabilities.clone(),
                })).await;

                if  me.endpoint_id.endpoint_guid.is_some() {
                    let _ = command_sender.send(SwitchboardServerCommand::JOI(JoiServer {
                        display_name: me.compute_display_name().to_string(),
                        endpoint_id: me.endpoint_id.clone(),
                        capabilities: me.capabilities.clone(),
                    })).await;
                }

            } else {
                let matrix_client = client_data.matrix_client();
                let maybe_found = matrix_client.find_room_from_email(&email).unwrap();
                if let Some(room) = maybe_found {
                    let target_room_user = room.to_msn_user_lazy().await.unwrap();


                    local_switchboard_data.room = Some(room);
                    local_switchboard_data.phase = ConnectionPhase::Ready;

                    let _ = command_sender.send(SwitchboardServerCommand::JOI(JoiServer {
                        display_name: target_room_user.compute_display_name().to_string(),
                        endpoint_id: EndpointId {
                            email_addr: target_room_user.get_email_address().clone(),
                            endpoint_guid: None,
                        },
                        capabilities: target_room_user.capabilities.clone(),
                    })).await;

                    if  target_room_user.endpoint_id.endpoint_guid.is_some() {
                        let _ = command_sender.send(SwitchboardServerCommand::JOI(JoiServer {
                            display_name: target_room_user.compute_display_name().to_string(),
                            endpoint_id: target_room_user.endpoint_id.clone(),
                            capabilities: target_room_user.capabilities.clone(),
                        })).await;
                    }
                }

                //TODO: handle room not found

            }



        }
        SwitchboardClientCommand::MSG(_) => {}
        SwitchboardClientCommand::OUT => {}
        SwitchboardClientCommand::RAW(_) => {}
    };
    Ok(())
}


pub(crate) async fn handle_ready(command: SwitchboardClientCommand, command_sender: Sender<SwitchboardServerCommand>, client_store: &ClientStoreFacade, local_switchboard_data: &mut LocalSwitchboardData) -> Result<(), anyhow::Error> {
    Ok(())
}