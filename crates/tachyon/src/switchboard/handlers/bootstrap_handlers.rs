use crate::matrix::extensions::direct::DirectRoom;
use crate::matrix::extensions::msn_user_resolver::{FindRoomFromEmail, ToMsnUser};
use crate::shared::identifiers::MatrixIdCompatible;
use crate::switchboard::extensions::CustomStyles;
use crate::switchboard::models::connection_phase::ConnectionPhase;
use crate::switchboard::models::local_switchboard_data::LocalSwitchboardData;
use crate::switchboard::models::switchboard_handle::{SwitchboardHandle, SwitchboardState};
use crate::switchboard::models::switchboard_token::SwitchboardToken;
use crate::tachyon::client_store::ClientStoreFacade;
use crate::tachyon::tachyon_client::TachyonClient;
use matrix_sdk::{Room, RoomMemberships};
use msnp::msnp::switchboard::command::cal::{CalServer, CalServerFunction};
use msnp::msnp::switchboard::command::command::{SwitchboardClientCommand, SwitchboardServerCommand};
use msnp::msnp::switchboard::command::iro::IroServer;
use msnp::msnp::switchboard::command::joi::JoiServer;
use msnp::msnp::switchboard::command::msg::{MsgPayload, MsgServer};
use msnp::msnp::switchboard::models::session_id::SessionId;
use msnp::shared::models::email_address::EmailAddress;
use msnp::shared::models::endpoint_id::EndpointId;
use msnp::shared::models::msn_user::MsnUser;
use msnp::shared::models::ticket_token::TicketToken;
use msnp::shared::payload::msg::text_plain_msg::TextPlainMessagePayload;
use std::str::FromStr;
use tokio::sync::mpsc::Sender;
use msnp::shared::models::display_name::DisplayName;
use msnp::shared::payload::msg::datacast_msg::{Datacast, DatacastMessagePayload};

const ROOM_USER_PORTAL_MODE: bool = true;

pub(crate) async fn handle_auth(command: SwitchboardClientCommand, command_sender: Sender<SwitchboardServerCommand>, client_store: &ClientStoreFacade, local_switchboard_data: &mut LocalSwitchboardData) -> Result<(), anyhow::Error> {

    match command {
        SwitchboardClientCommand::ANS(ans_command) => {
            let token = SwitchboardToken::try_from(ans_command.token.clone())?;

            match client_store.get_client(&token.matrix_token) {
                None => {}
                Some(tachyon_client) => {
                    let matrix_client = tachyon_client.matrix_client();
                    match matrix_client.get_room(token.room_id.as_ref()) {
                        None => {}
                        Some(room) => {
                            let room_msn_user = room.to_msn_user_lazy().await?;
                            local_switchboard_data.token = TicketToken(token.matrix_token);
                            local_switchboard_data.session_id = SessionId::random();
                            local_switchboard_data.email_addr = room_msn_user.get_email_address().clone();
                            local_switchboard_data.endpoint_guid = room_msn_user.endpoint_id.endpoint_guid.clone();
                            local_switchboard_data.tachyon_client = Some(tachyon_client.clone());
                            local_switchboard_data.room_id = Some(room.room_id().to_owned());
                            local_switchboard_data.room = Some(room.clone());
                            local_switchboard_data.phase = ConnectionPhase::Initializing;


                            //Send Initial roster = everyone but me
                            let mut initial_roster = if ROOM_USER_PORTAL_MODE {
                                vec![room_msn_user.clone()]
                            } else {
                                get_initial_roster_with_room_user(&room, room_msn_user.clone()).await?
                            };

                            let count = initial_roster.len() as u32;
                            let mut index = 1;
                            for member in initial_roster.drain(..) {
                                send_initial_roster_member(ans_command.tr_id, index, count, member, &command_sender).await?;
                                index += 1;
                            }

                            command_sender.send(SwitchboardServerCommand::OK(ans_command.get_ok_response())).await?;

                            //Send me joined
                            let me = tachyon_client.own_user()?;
                            command_sender.send(SwitchboardServerCommand::JOI(JoiServer {
                                display_name: me.compute_display_name().to_string(),
                                endpoint_id: me.endpoint_id.clone(),
                                capabilities: me.capabilities.clone(),
                            })).await?;

                            if ROOM_USER_PORTAL_MODE {
                                send_active_members_notice(&room, &command_sender).await?;
                            }

                            local_switchboard_data.phase = ConnectionPhase::Ready;

                            //TODO: send an error if this expect is not here.
                            let mut switchboard_handle = tachyon_client.switchboards().get(token.room_id.as_ref()).expect("To be here");
                            //set handle as ready
                            switchboard_handle.set_state(SwitchboardState::Ready {
                                msnp_sender: command_sender.clone()
                            }).await?;

                        }
                    };



                }
            }


            //TODO send errors

        }
        SwitchboardClientCommand::USR(usr_command) => {

            let token = TicketToken::from_str(&usr_command.token).unwrap();

            match client_store.get_client(token.as_str()) {
                None => {
                    //TODO AUTH error
                }
                Some(client_data) => {
                    local_switchboard_data.email_addr = usr_command.endpoint_id.email_addr.clone();
                    local_switchboard_data.endpoint_guid = usr_command.endpoint_id.endpoint_guid.clone();
                    local_switchboard_data.token = token;
                    local_switchboard_data.tachyon_client = Some(client_data);
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


pub(crate) async fn handle_init(command: SwitchboardClientCommand, command_sender: Sender<SwitchboardServerCommand>, tachyon_client: TachyonClient, local_switchboard_data: &mut LocalSwitchboardData) -> Result<(), anyhow::Error> {
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
                let me = tachyon_client.own_user().unwrap();
                send_initial_joined_member(me, &command_sender).await?;
            } else {
                let matrix_client = tachyon_client.matrix_client();
                let maybe_found = matrix_client.find_room_from_email(&email).unwrap();
                if let Some(room) = maybe_found {
                    let target_room_user = room.to_msn_user_lazy().await?;

                    local_switchboard_data.room_id = Some(room.room_id().to_owned());
                    local_switchboard_data.room = Some(room.clone());
                    local_switchboard_data.phase = ConnectionPhase::Ready;

                    send_initial_joined_member(target_room_user, &command_sender).await?;

                    let switchboard_handle = SwitchboardHandle::new_ready(local_switchboard_data.session_id.clone(), room.room_id().to_owned(), command_sender.clone());
                    tachyon_client.switchboards().insert(switchboard_handle)?;

                    let command_sender_clone = command_sender.clone();

                    if ROOM_USER_PORTAL_MODE {
                        send_active_members_notice(&room, &command_sender_clone).await?;
                    } else {
                        tokio::spawn(async move {
                            let mut result = get_initial_roster(&room).await;
                            match result {
                                Ok(mut initial_roster) => {


                                    for member in initial_roster.drain(..) {
                                        if let Err(_) = send_initial_joined_member(member, &command_sender_clone.clone()).await {
                                            //TODO we should retry here or disconnect

                                        }
                                    }
                                }
                                Err(err) => {
                                    //Todo disconnect if we cannot fetch members.
                                }
                            }});
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

async fn send_active_members_notice(room: &Room, sender: &Sender<SwitchboardServerCommand>) -> Result<(), anyhow::Error> {
    let active = room.active_members_count();
    if !room.is_valid_one_to_one_direct() || active > 2 {
        let heroes_string = room.heroes().drain(..).map(|hero| hero.display_name.unwrap_or(hero.user_id.to_string())).reduce(|acc, hero| format!("{}, {}", acc, hero));
        sender.send(SwitchboardServerCommand::MSG(create_notice_message(format!("This room is a portal room.\r\nYou are sharing it with {} other member(s).\r\n", active-1).as_str()))).await?;
        if let Some(heroes) = heroes_string {
            sender.send(SwitchboardServerCommand::MSG(create_notice_message(format!("Room heroes: {}\r\n", heroes).as_str()))).await?;
        }
    }
    Ok(())
}

fn create_notice_message(text: &str) -> MsgServer {
    MsgServer{
        sender: EmailAddress::from_str("tachyon@tachyon.internal").unwrap(),
        display_name: DisplayName::new_from_ref("System"),
        payload: MsgPayload::TextPlain(
            TextPlainMessagePayload::new_with_notice_style(text)
        ),
    }
}

async fn get_initial_roster(room: &Room) -> Result<Vec<MsnUser>, anyhow::Error> {
    let mut out = Vec::new();

    let direct_target = room.get_single_direct_target();
    let members = room.members(RoomMemberships::JOIN).await?;

    for member in members {
        if let Some(direct_target_user_id) = direct_target.as_ref() {
            if member.user_id() == direct_target_user_id || member.user_id() == room.own_user_id() {
                continue;
            }
        }

        out.push(member.to_msn_user_lazy().await?);
    }

    Ok(out)
}

async fn get_initial_roster_with_room_user(room: &Room, room_msn_user: MsnUser) -> Result<Vec<MsnUser>, anyhow::Error> {
    let mut out = get_initial_roster(room).await?;
    out.push(room_msn_user);
    Ok(out)
}

async fn send_initial_roster_member(tr_id: u128, index: u32, count: u32, member: MsnUser, command_sender: &Sender<SwitchboardServerCommand>) -> Result<(), anyhow::Error>{


    command_sender.send(SwitchboardServerCommand::IRO(IroServer::new(
        tr_id,
        index,
        count,
        member.compute_display_name().to_string(),
        member.endpoint_id.strip_endpoint_guid(),
        member.capabilities.clone()
    ))).await?;


    if member.endpoint_id.endpoint_guid.is_some() {
        command_sender.send(SwitchboardServerCommand::IRO(IroServer::new(
            tr_id,
            index,
            count,
            member.compute_display_name().to_string(),
            member.endpoint_id,
            member.capabilities
        ))).await?;
    }
    Ok(())
}

async fn send_initial_joined_member(member: MsnUser, command_sender:  &Sender<SwitchboardServerCommand>) -> Result<(), anyhow::Error>{
    command_sender.send(SwitchboardServerCommand::JOI(JoiServer {
        display_name: member.compute_display_name().to_string(),
        endpoint_id: EndpointId {
            email_addr: member.get_email_address().clone(),
            endpoint_guid: None,
        },
        capabilities: member.capabilities.clone(),
    })).await?;

    if  member.endpoint_id.endpoint_guid.is_some() {
        let _ = command_sender.send(SwitchboardServerCommand::JOI(JoiServer {
            display_name: member.compute_display_name().to_string(),
            endpoint_id: member.endpoint_id.clone(),
            capabilities: member.capabilities.clone(),
        })).await?;
    }

    Ok(())
}