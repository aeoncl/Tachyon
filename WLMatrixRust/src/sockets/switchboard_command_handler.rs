use crate::models::msg_payload::MsgPayload;
use crate::models::msn_user::MSNUser;
use crate::models::notification::error::{MsnpError, MsnpErrorCode};
use crate::models::owned_user_id_traits::{FromMsnAddr, ToMsnAddr};
use crate::models::p2p::events::p2p_event::P2PEvent;
use crate::models::p2p::p2p_session::P2PSession;
use crate::models::p2p::p2p_transport_packet::P2PTransportPacket;
use crate::models::p2p::pending_packet::PendingPacket;
use async_trait::async_trait;
use base64::{engine::general_purpose, Engine};
use log::info;
use matrix_sdk::ruma::{OwnedUserId, UserId};
use matrix_sdk::Client;
use std::str::FromStr;
use std::sync::Arc;
use substring::Substring;
use tokio::sync::broadcast::{self, Receiver, Sender};
use tokio::sync::oneshot;

use super::command_handler::CommandHandler;
use super::msnp_command::MSNPCommand;
use crate::models::msg_payload::factories::MsgPayloadFactory;
use crate::models::switchboard::events::switchboard_event::SwitchboardEvent;
use crate::models::switchboard::switchboard::Switchboard;
use crate::models::uuid::UUID;
use crate::utils::identifiers::matrix_room_id_to_annoying_matrix_room_id;
use crate::{MATRIX_CLIENT_LOCATOR, MSN_CLIENT_LOCATOR, P2P_REPO};

pub struct SwitchboardCommandHandler {
    protocol_version: Arc<i16>,
    msn_addr: String,
    endpoint_guid: String,
    matrix_token: String,
    target_room_id: String,
    target_matrix_id: Option<OwnedUserId>,
    target_msn_addr: String,
    matrix_client: Option<Client>,
    sender: Sender<String>,
    switchboard: Option<Switchboard>,
    sb_bridge: Option<P2PSession>,
    stop_sender: Option<oneshot::Sender<()>>,
}

impl Drop for SwitchboardCommandHandler {
    fn drop(&mut self) {
        if let Some(client_data) = MSN_CLIENT_LOCATOR.get() {
            client_data.get_switchboards().remove(&self.target_room_id);
        }
        if let Some(stop_sender) = self.stop_sender.take() {
            stop_sender.send(());
        }
    }
}

impl SwitchboardCommandHandler {
    pub fn new(sender: Sender<String>) -> SwitchboardCommandHandler {
        return SwitchboardCommandHandler {
            protocol_version: Arc::new(-1),
            msn_addr: String::new(),
            endpoint_guid: String::new(),
            matrix_token: String::new(),
            target_room_id: String::new(),
            matrix_client: None,
            target_matrix_id: None,
            target_msn_addr: String::new(),
            sender: sender,
            switchboard: None,
            sb_bridge: None,
            stop_sender: None,
        };
    }

    fn start_receiving(
        &mut self,
        mut sb_receiver: Receiver<SwitchboardEvent>,
        mut p2p_receiver: Receiver<P2PEvent>,
        mut p2p_stop_listener: oneshot::Receiver<()>,
    ) {
        let sender = self.sender.clone();
        let switchboard = self
            .switchboard
            .as_ref()
            .expect("Start receiving with switchboard not present")
            .clone();

        let mut sb_bridge = self
            .sb_bridge
            .as_ref()
            .expect("Start receiving with switchboard not present")
            .clone();

        tokio::spawn(async move {
            let sender = sender;
            loop {
                tokio::select! {
                    sb_event = sb_receiver.recv() => {
                        if let Ok(msg) = sb_event {
                            match msg {
                                SwitchboardEvent::MessageEvent(content) => {
                                    let payload = content.msg.serialize();
                                    let _result = sender.send(format!("MSG {msn_addr} {display_name} {payload_size}\r\n{payload}", msn_addr = &content.sender.get_msn_addr(), display_name = &content.sender.get_display_name(), payload_size = payload.len(), payload = &payload));
                                },
                                SwitchboardEvent::FileUploadEvent(content) => {
                                    let client_data = MSN_CLIENT_LOCATOR.get().unwrap();
                                    sb_bridge.transfer_file(content.sender, client_data.get_user());
                                },
                                _ => {

                                }
                            }
                        }
                    },
                    p2p_event = p2p_receiver.recv() => {
                        if let Ok(msg) = p2p_event {
                            match msg {
                                P2PEvent::Message(content) => {
                                    //Todo change this
                                    P2P_REPO.set_seq_number(content.packet.get_next_sequence_number());
                                    info!("DEBUG P2PEVENT::Message");
                                    let msg = MsgPayloadFactory::get_p2p(&content.sender, &content.receiver, &content.packet);
                                    switchboard.on_message_received(msg, content.sender.clone(), None).unwrap();
                                },
                                P2PEvent::FileReceived(content) => {
                                    switchboard.send_file(content.file).await;
                                },
                                _ => {

                                }
                            }
                        }
                    },
                    _p2p_stop = &mut p2p_stop_listener => {
                        info!("STOP LISTENING FOR SB");
                        break;
                    }
                }
            }
        });
    }

    pub async fn send_initial_roster(&mut self, tr_id: &str) {
        let room_id = matrix_room_id_to_annoying_matrix_room_id(&self.target_room_id);
        if let Some(room) = &self
            .matrix_client
            .as_ref()
            .unwrap()
            .get_joined_room(&room_id)
        {
            let members = room.joined_members().await.unwrap();
            let mut index = 1;
            //let count = (members.len() - 1)*2;
            let count = members.len() - 1;

            for member in members {
                let msn_user = MSNUser::from_matrix_id(member.user_id().to_owned());
                if msn_user.get_msn_addr() != self.msn_addr {
                    self.send_initial_roster_member(tr_id, index, count as i32, &msn_user);
                    index += 2;
                }
            }
        }
    }

    fn send_initial_roster_member(&self, tr_id: &str, index: i32, count: i32, msn_user: &MSNUser) {
        self.sender.send(format!(
            "IRO {tr_id} {index} {roster_count} {passport} {friendly_name} {capabilities}\r\n",
            tr_id = &tr_id,
            index = &index,
            roster_count = &count,
            passport = &msn_user.get_msn_addr(),
            friendly_name = &msn_user.get_msn_addr(),
            capabilities = &msn_user.get_capabilities().to_string()
        ));

        let endpoint_guid = UUID::from_string(&msn_user.get_msn_addr())
            .to_string()
            .to_uppercase();

        self.sender.send(format!("IRO {tr_id} {index} {roster_count} {passport};{{{endpoint_guid}}} {friendly_name} {capabilities}\r\n",
            tr_id = &tr_id,
            index = &index+1,
            roster_count = &count,
            passport = &msn_user.get_msn_addr(),
            friendly_name = &msn_user.get_msn_addr(),
            endpoint_guid = &endpoint_guid,
            capabilities = &msn_user.get_capabilities().to_string()));
    }

    pub fn send_me_joined(&self) {
        let mut me = MSNUser::new(self.msn_addr.clone());
        me.set_endpoint_guid(self.endpoint_guid.clone());
        self.send_contact_joined(&me);
    }

    pub fn send_contact_joined(&self, user: &MSNUser) {
        self.sender.send(format!(
            "JOI {passport} {friendly_name} {capabilities}\r\n",
            passport = &user.get_msn_addr(),
            friendly_name = &user.get_msn_addr(),
            capabilities = &user.get_capabilities().to_string()
        ));
        self.sender.send(format!(
            "JOI {passport};{{{endpoint_guid}}} {friendly_name} {capabilities}\r\n",
            passport = &user.get_msn_addr(),
            endpoint_guid = &user.get_endpoint_guid(),
            friendly_name = &user.get_msn_addr(),
            capabilities = &user.get_capabilities().to_string()
        ));
    }

    fn bootstrap_loops(&mut self, mut switchboard: Switchboard) {
        self.switchboard = Some(switchboard.clone());
        let sb_receiver = switchboard.get_receiver();

        let (p2p_sender, p2p_receiver) = broadcast::channel::<P2PEvent>(30);
        self.sb_bridge = Some(P2PSession::new(p2p_sender));

        let (stop_sender, mut stop_receiver) = oneshot::channel::<()>();
        self.stop_sender = Some(stop_sender);

        self.start_receiving(sb_receiver, p2p_receiver, stop_receiver);
    }
}

#[async_trait]
impl CommandHandler for SwitchboardCommandHandler {
    async fn handle_command(&mut self, command: &MSNPCommand) -> Result<String, MsnpError> {
        let split = command.split();
        match command.operand.as_str() {
            "ANS" => {
                // >>> ANS 3 aeontest@shl.local;{F52973B6-C926-4BAD-9BA8-7C1E840E4AB0} base64token 4060759068338340280
                // <<<
                let token =
                    String::from_utf8(general_purpose::STANDARD.decode(split[3]).unwrap()).unwrap();
                let split_token: Vec<&str> = token.split(";").collect();
                let tr_id = split[1];
                let endpoint = split[2];
                let endpoint_parts: Vec<&str> = endpoint.split(";").collect();

                self.msn_addr = endpoint_parts.get(0).unwrap().to_string();

                let endpoint_guid = endpoint_parts.get(1).unwrap().to_string();
                self.endpoint_guid = endpoint_guid
                    .substring(1, endpoint_guid.len() - 1)
                    .to_string();
                self.target_room_id = split_token.get(0).unwrap().to_string();
                self.matrix_token = split_token.get(1).unwrap().to_string();
                self.target_matrix_id = Some(
                    UserId::parse(split_token.get(2).unwrap())
                        .or(Err(MsnpError::internal_server_error(&tr_id)))?,
                );
                self.target_msn_addr = self.target_matrix_id.as_ref().unwrap().to_msn_addr();

                self.matrix_client = MATRIX_CLIENT_LOCATOR.get().clone();

                let client_data = MSN_CLIENT_LOCATOR.get().unwrap();

                {
                    if let Some(mut sb) = client_data.get_switchboards().find(&self.target_room_id)
                    {
                        self.bootstrap_loops(sb);
                    }
                };

                let _result = self.send_initial_roster(&tr_id).await;
                self.sender
                    .send(format!("ANS {tr_id} OK\r\n", tr_id = &tr_id));
                self.send_me_joined();

                return Ok(String::new());
            }
            "USR" => {
                // >>> USR 55 aeontest@shl.local;{F52973B6-C926-4BAD-9BA8-7C1E840E4AB0} matrix_token
                // <<< USR 55 aeontest@shl.local aeontest@shl.local OK

                let tr_id = split[1];
                let endpoint_str = split[2].to_string();
                self.matrix_token = split[3].to_string();
                let endpoint_str_split: Vec<&str> = endpoint_str.split(";").collect();
                if let Some(msn_addr) = endpoint_str_split.get(0) {
                    self.msn_addr = msn_addr.to_string();
                    let endpoint_guid = endpoint_str_split.get(1).unwrap().to_string();
                    self.endpoint_guid = endpoint_guid
                        .substring(1, endpoint_guid.len() - 1)
                        .to_string();

                    if let Some(client_data) = MSN_CLIENT_LOCATOR.get() {
                        if let Some(client) = MATRIX_CLIENT_LOCATOR.get() {
                            self.matrix_client = Some(client.clone());
                            self.protocol_version = Arc::new(client_data.get_msnp_version());
                            return Ok(format!(
                                "USR {tr_id} {msn_addr} {msn_addr} OK\r\n",
                                tr_id = tr_id,
                                msn_addr = msn_addr
                            ));
                        }
                    }
                }
                return Ok(format!(
                    "{error_code} {tr_id}\r\n",
                    error_code = MsnpErrorCode::AuthFail as i32,
                    tr_id = tr_id
                ));
            }
            "CAL" => {
                //Calls all the members to join the SB
                // >>> CAL 58 aeontest@shl.local
                // <<< CAL 58 RINGING 4324234

                let tr_id = split[1];
                let msn_addr_to_add = split[2].to_string();
                let session_id = UUID::new().to_decimal_cid_string();

                self.sender.send(format!(
                    "CAL {tr_id} RINGING {session_id}\r\n",
                    tr_id = tr_id,
                    session_id = session_id
                ));

                if msn_addr_to_add == self.msn_addr {
                    self.send_me_joined();

                    //that's me !
                } else {
                    let user_to_add = OwnedUserId::from_msn_addr(&msn_addr_to_add);

                    let client = self.matrix_client.as_ref().unwrap().clone();

                    let target_room = client.find_or_create_dm_room(&user_to_add).await.unwrap(); //TODO handle this
                    self.target_room_id = target_room.room_id().to_string();
                    let client_data = MSN_CLIENT_LOCATOR.get().unwrap();

                    //Move this elsewhere TODO (What if we have more than two people in one SB)
                    let switchboard = Switchboard::new(
                        client.clone(),
                        target_room.room_id().to_owned(),
                        client.user_id().unwrap().to_owned(),
                    );
                    self.bootstrap_loops(switchboard.clone());
                    client_data
                        .get_switchboards()
                        .add(target_room.room_id().to_string(), switchboard);

                    let user_to_add = MSNUser::new(msn_addr_to_add.clone());
                    self.send_contact_joined(&user_to_add);
                }
                return Ok(String::new());
            }
            "MSG" => {
                //      0   1  2 3
                // >>> MSG 231 U 91
                // <<< ACK 231           on success
                // <<< NAK 231          on failure
                // The 2nd parameter is the type of ack the clients wants.
                // N: ack only when the message was not received
                // A + D: always send an ack
                // U: never ack
                let client_data = MSN_CLIENT_LOCATOR.get().unwrap();
                let sb = client_data
                    .get_switchboards()
                    .find(&self.target_room_id)
                    .unwrap();

                if let Ok(payload) = MsgPayload::from_str(command.payload.as_str()) {
                    if "application/x-msnmsgrp2p" == payload.content_type.as_str() {
                        //P2P, send to SBBridge
                        if let Some(sb_bridge) = self.sb_bridge.as_mut() {
                            if let Ok(mut p2p_packet) = P2PTransportPacket::from_str(&payload.body)
                            {
                                let source = MSNUser::from_mpop_addr_string(
                                    payload
                                        .get_header(&String::from("P2P-Src"))
                                        .unwrap()
                                        .to_owned(),
                                )
                                .unwrap();
                                let dest = MSNUser::from_mpop_addr_string(
                                    payload
                                        .get_header(&String::from("P2P-Dest"))
                                        .unwrap()
                                        .to_owned(),
                                )
                                .unwrap();

                                sb_bridge.on_message_received(PendingPacket::new(
                                    p2p_packet, source, dest,
                                ));
                            }
                        }
                    } else {
                        // Send to SB
                        if let Some(sb_handle) = self.switchboard.as_ref() {
                            sb.send_message(payload).await;
                        }
                    }
                }
                let tr_id = split[1];
                let type_of_ack = split[2];

                if type_of_ack == "A" || type_of_ack == "D" {
                    return Ok(format!("ACK {tr_id}\r\n", tr_id = &tr_id));
                }

                return Ok(String::new());
            }
            _ => {
                return Ok(String::new());
            }
        }
    }

    fn get_matrix_token(&self) -> String {
        return String::new();
    }
}
