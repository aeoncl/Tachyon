use std::collections::HashMap;
use std::str::FromStr;
use std::sync::Arc;
use std::time::Duration;

use async_trait::async_trait;
use base64::{Engine, engine::general_purpose};
use log::{error, info};
use log::debug;
use matrix_sdk::{Client, Room, RoomMemberships};
use matrix_sdk::config::RequestConfig;
use matrix_sdk::media::{MediaFormat, MediaRequest};
use matrix_sdk::ruma::{MxcUri, OwnedUserId, UserId};
use matrix_sdk::ruma::events::room::MediaSource;
use substring::Substring;
use tokio::sync::{mpsc, oneshot};
use tokio::sync::mpsc::{UnboundedReceiver, UnboundedSender};

use crate::{MATRIX_CLIENT_LOCATOR, MSN_CLIENT_LOCATOR};
use crate::models::conversion::audio_conversion;
use crate::models::msg_payload::factories::MsgPayloadFactory;
use crate::models::msg_payload::MsgPayload;
use crate::models::msn_object::{MSNObject, MSNObjectType};
use crate::models::msn_user::MSNUser;
use crate::models::notification::error::{MSNPErrorCode, MSNPServerError};
use crate::models::owned_user_id_traits::{FromMsnAddr, ToMsnAddr};
use crate::models::p2p::events::p2p_event::P2PEvent;
use crate::models::p2p::p2p_client::P2PClient;
use crate::models::p2p::p2p_transport_packet::P2PTransportPacket;
use crate::models::p2p::pending_packet::PendingPacket;
use crate::models::p2p::session::file_transfer_session_content::FileTransferSessionContent;
use crate::models::p2p::session::p2p_session_type::P2PSessionType;
use crate::models::switchboard::events::switchboard_event::SwitchboardEvent;
use crate::models::switchboard::switchboard::Switchboard;
use crate::models::uuid::UUID;
use crate::repositories::msn_user_repository::MSNUserRepository;
use crate::utils::identifiers::{self, matrix_room_id_to_annoying_matrix_room_id};

use super::command_handler::CommandHandler;
use super::events::socket_event::SocketEvent;
use super::msnp_command::MSNPCommand;

pub struct SwitchboardCommandHandler {
    protocol_version: Arc<i16>,
    msn_addr: String,
    endpoint_guid: String,
    matrix_token: String,
    target_room_id: String,
    target_matrix_id: Option<OwnedUserId>,
    target_msn_addr: String,
    matrix_client: Option<Client>,
    sender: UnboundedSender<SocketEvent>,
    switchboard: Option<Switchboard>,
    sb_bridge: Option<P2PClient>,
    stop_sender: Option<oneshot::Sender<()>>,
    session_id: String
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
    pub fn new(sender: UnboundedSender<SocketEvent>) -> SwitchboardCommandHandler {
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
            session_id: identifiers::get_sb_session_id(),
        };
    }

    fn start_receiving(
        &mut self,
        mut sb_receiver: UnboundedReceiver<SwitchboardEvent>,
        mut p2p_receiver: UnboundedReceiver<P2PEvent>,
        mut stop_listener: oneshot::Receiver<()>,
    ) {
        let sender = self.sender.clone();
        let matrix_client = self.matrix_client.clone().unwrap();
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
                        let msg = sb_event.expect("SB Pipe to not lag");
                        match msg {
                            SwitchboardEvent::MessageEvent(content) => {
                                let payload = content.msg.serialize();
                                let _result = sender.send(format!("MSG {msn_addr} {display_name} {payload_size}\r\n{payload}", msn_addr = &content.sender.get_msn_addr(), display_name = &content.sender.get_display_name(), payload_size = payload.len(), payload = &payload).into());
                            },
                            SwitchboardEvent::FileUploadEvent(content) => {
                                let session_type = P2PSessionType::FileTransfer(FileTransferSessionContent { filename: content.filename, filesize: content.filesize, source: Some(content.source) });
                                sb_bridge.initiate_session(content.sender, content.receiver, session_type);
                            },
                            _ => {
                            }
                        }
                    },
                    p2p_event = p2p_receiver.recv() => {
                        let msg = p2p_event.expect("P2P Pipe to not lag");
                        match msg {
                            P2PEvent::Message(content) => {
                                let msgs : Vec<String> = content.packets.iter().map(|packet| {
                                    debug!("SBBridge OUT: {:?}", packet);
                                    let payload = MsgPayloadFactory::get_p2p(&content.sender, &content.receiver, &packet).serialize();
                                    format!("MSG {msn_addr} {display_name} {payload_size}\r\n{payload}", msn_addr = &content.sender.get_msn_addr(), display_name = &content.sender.get_display_name(), payload_size = payload.len(), payload = &payload)
                                }).collect();

                                let _result = sender.send(SocketEvent::Multiple(msgs));
                            },
                            P2PEvent::FileReceived(content) => {
                               if let Err(error_resp) = switchboard.send_file(content.file).await {
                                    error!("An error occured while sending file: {:?}", &error_resp);
                               }
                            },
                            P2PEvent::FileTransferAccepted(content) => {
                                
                                let media_request = MediaRequest{source: content.source.clone(), format: MediaFormat::File };
                                let file_content = matrix_client.media().get_media_content(&media_request, true).await;
                                
                                 if let Ok(file_downloaded) = file_content {
                                    sb_bridge.send_file(content.session_id, file_downloaded);
                                 } else {
                                     //Todo return error
                                     error!("There was an error downloading the file to send: {:?} {}", &media_request, file_content.unwrap_err());
                                 }
                            },
                            P2PEvent::MSNObjectReceived(content) => {
                                match &content.msn_object.obj_type {
                                    MSNObjectType::VoiceClip => {
                                        switchboard.send_voice_message(content.file_content).await;
                                    },
                                    _ =>{
                                        info!("Received a yet unsupported MSNObject type: {:?}", &content.msn_object);
                                    }
                                }
                            },
                            P2PEvent::MSNObjectRequested(content) => {
                            info!("RECEIVED MSNObjectRequestedEvent: {:?}", &content);
                            let msn_obj = &content.msn_object;
                            //TODO ERROR HANDLING


                            let base64decoded_id  = if msn_obj.location == "0" && msn_obj.friendly.is_some(){
                            general_purpose::STANDARD.decode(msn_obj.friendly.as_ref().unwrap()).expect("MSNObj friendly to be base64encoded")
                            } else {
                            let base64encoded_id =  msn_obj.location.trim_end_matches(".tmp");
                            general_purpose::STANDARD.decode(base64encoded_id).expect("MSNObj location to be base64encoded")
                            };

                            match msn_obj.obj_type {
                            MSNObjectType::Avatar | MSNObjectType::DisplayPicture => {
                            let avatar_url = String::from_utf8(base64decoded_id).expect("MSNObj location to be UTF-8");
                            let msn_user_repo = MSNUserRepository::new(matrix_client.clone());
                            if let Ok(avatar_bytes) = msn_user_repo.get_avatar_from_string(avatar_url.clone()).await {
                            sb_bridge.send_msn_object(content.session_id, content.call_id.clone(), avatar_bytes, content.invitee, content.inviter);
                            } else {
                            error!("Could not download avatar for: {}", &avatar_url);
                            //TODO sent SLP error
                            }
                                    },
                            MSNObjectType::SharedFile => {},
                            MSNObjectType::Background => {},
                            MSNObjectType::History => {},
                            MSNObjectType::DynamicDisplayPicture => {},
                            MSNObjectType::Wink => {},
                            MSNObjectType::MapFile => {},
                            MSNObjectType::DynamicBackground => {},
                            MSNObjectType::VoiceClip => {
                            let uri = String::from_utf8(base64decoded_id).expect("MSNObj location to be UTF-8");
                            let owned_mxc_uri = <&MxcUri>::try_from(uri.as_str()).unwrap().to_owned();


                                        
                            let media_request = MediaRequest{ source: MediaSource::Plain(uri.into()), format: MediaFormat::File };
                            let media_client = &matrix_client.media();
                            let media = media_client.get_media_content(&media_request, true).await.unwrap(); //TODO exception handling

                            let converted_media = audio_conversion::convert_incoming_audio_message(media).await.unwrap(); //TODO change this double conversion shit


                            sb_bridge.send_msn_object(content.session_id, content.call_id.clone(), converted_media, content.invitee, content.inviter);
                            //TODO sent SLP error
                            },
                            MSNObjectType::PluginState => {},
                            MSNObjectType::RoamingObject => {},
                            MSNObjectType::SignatureSound => {},
                            MSNObjectType::UnknownYet => {},
                            MSNObjectType::Scene => {},
                            MSNObjectType::WebcamDynamicDisplayPicture => {},
                            MSNObjectType::CustomEmoticon => {}
                            }
                             }
                            _ => {
                            }
                        }
                    },
                    _stop = &mut stop_listener => {
                        info!("STOP LISTENING FOR SB & SB Bridge");
                        break;
                    }
                }
            }
        });
    }

    pub async fn send_initial_roster(&mut self, tr_id: &str) {
        let room_id = matrix_room_id_to_annoying_matrix_room_id(&self.target_room_id);
        let mut room_members = Vec::new();


        if let Some(room) = self
            .matrix_client
            .as_ref()
            .unwrap()
            .get_room(&room_id)
        {
            room_members = room.members(RoomMemberships::JOIN).await.expect("Room to have members").into_iter().map(|member| {
                member.user_id().to_owned()
            }).collect();
        } else {
            //We couldn't fetch the room yet :(
            let room_event_request = matrix_sdk::ruma::api::client::membership::joined_members::v3::Request::new(room_id);

            let config = RequestConfig::new().timeout(Duration::from_secs(30)).retry_limit(2);

            let response = self.matrix_client.as_ref().unwrap().send(room_event_request, Some(config)).await;
            if let Ok(response) = response {
                room_members = response.joined.into_iter().map(|(owned_user_id, room_member)| {
                    owned_user_id
                }).collect();
            }
        }

        let mut index = 1;
        let count = if room_members.len() > 0 { room_members.len() - 1 } else { 0 };

        for member in room_members {
            let msn_user = MSNUser::from_matrix_id(member);
            if msn_user.get_msn_addr() != self.msn_addr {
                self.send_initial_roster_member(tr_id, index, count as i32, &msn_user);
                index += 2;
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
        ).into());

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
            capabilities = &msn_user.get_capabilities().to_string()).into());
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
        ).into());
        self.sender.send(format!(
            "JOI {passport};{{{endpoint_guid}}} {friendly_name} {capabilities}\r\n",
            passport = &user.get_msn_addr(),
            endpoint_guid = &user.get_endpoint_guid(),
            friendly_name = &user.get_msn_addr(),
            capabilities = &user.get_capabilities().to_string()
        ).into());
    }

    fn bootstrap_loops(&mut self, mut switchboard: Switchboard) {
        self.switchboard = Some(switchboard.clone());
        let sb_receiver = switchboard.get_receiver();

        let (p2p_sender, p2p_receiver) = mpsc::unbounded_channel::<P2PEvent>();
        self.sb_bridge = Some(P2PClient::new(p2p_sender));

        let (stop_sender, mut stop_receiver) = oneshot::channel::<()>();
        self.stop_sender = Some(stop_sender);

        self.start_receiving(sb_receiver, p2p_receiver, stop_receiver);
    }

    async fn find_or_create_dm_room(client: &Client, user_id: &UserId) -> matrix_sdk::Result<Room> {
        if let Some(found) = client.get_dm_room(user_id) {
            return Ok(found);
        }
        return client.create_dm(user_id).await;
    }
}

#[async_trait]
impl CommandHandler for SwitchboardCommandHandler {
    async fn handle_command(&mut self, command: &MSNPCommand) -> Result<String, MSNPServerError> {
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
                let session_id = split[4];
                self.session_id = session_id.to_string();

                let endpoint_parts: Vec<&str> = endpoint.split(";").collect();

                self.msn_addr = endpoint_parts.get(0).unwrap().to_string();

                let endpoint_guid = endpoint_parts.get(1).unwrap().to_string();
                self.endpoint_guid = endpoint_guid
                    .substring(1, endpoint_guid.len() - 1)
                    .to_string();
                self.target_room_id = split_token.get(0).unwrap().to_string();
                self.matrix_token = split_token.get(1).unwrap().to_string();
               self.target_matrix_id = Some(UserId::parse(split_token.get(2).unwrap()).unwrap());

                self.target_msn_addr = self.target_matrix_id.as_ref().unwrap().to_msn_addr();

                self.matrix_client = MATRIX_CLIENT_LOCATOR.get().clone();

                let client_data = MSN_CLIENT_LOCATOR.get().unwrap();


                let _result = self.send_initial_roster(&tr_id).await;
                self.sender
                    .send(format!("ANS {tr_id} OK\r\n", tr_id = &tr_id).into());
                self.send_me_joined();

                {
                    if let Some(mut sb) = client_data.get_switchboards().find(&self.target_room_id)
                    {
                        self.bootstrap_loops(sb);
                    }
                };

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
                    error_code = MSNPErrorCode::AuthFail as i32,
                    tr_id = tr_id
                ));
            }
            "CAL" => {
                //Calls all the members to join the SB
                // >>> CAL 58 aeontest@shl.local
                // <<< CAL 58 RINGING 4324234

                let tr_id = split[1];
                let msn_addr_to_add = split[2].to_string();
        
                self.sender.send(format!(
                    "CAL {tr_id} RINGING {session_id}\r\n",
                    tr_id = tr_id,
                    session_id = &self.session_id
                ).into());

                if msn_addr_to_add == self.msn_addr {
                    self.send_me_joined();

                    //that's me !
                } else {
                    let user_to_add = OwnedUserId::from_msn_addr(&msn_addr_to_add);

                    if self.target_msn_addr.is_empty() {
                        //Todo fetch the dm room target from matrix events and stuff
                        self.target_msn_addr = msn_addr_to_add.clone();
                    }

                    let client = self.matrix_client.as_ref().unwrap().clone();

                    let target_room = Self::find_or_create_dm_room(&client, &user_to_add).await.unwrap(); //TODO handle this

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

                    match payload.content_type.as_str() {
                        "application/x-msnmsgrp2p" => {
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
                        },
                        "text/x-msnmsgr-datacast" => {
                            let split: Vec<&str> = payload.body.split_terminator("\r\n").collect();
                            let body_map : HashMap<String, String> = split.iter().map_while(|e| {
                                if e.is_empty() {
                                    None
                                } else {
                                   let split = e.split_once(":").expect("datacast MSG body field to be key value");
                                   Some((split.0.trim().to_owned(), split.1.trim().to_owned()))
                                }
                            } ).collect();

                            let id = body_map.get("ID").unwrap();
                            match id.as_str() {
                                "3" => {
                                    let data = body_map.get("Data").unwrap();
                                    //Datacast is MSNObj type
                                    // Data is MSNObj as string
                                    if let Some(sb_bridge) = self.sb_bridge.as_mut() {

                                        let obj = MSNObject::from_str(data.as_str()).unwrap();

                                        let target = MSN_CLIENT_LOCATOR.get().expect("MSN Client to be present").get_user();
                                        let inviter = MSNUser::new(self.target_msn_addr.clone());
                                        sb_bridge.initiate_session(inviter, target, P2PSessionType::MSNObject(obj));
                                    }

                                    }
                                _ => {

                                }
                            }

                        },
                        _ => {
                            if let Some(sb_handle) = self.switchboard.as_ref() {
                                sb.send_message(payload).await;
                            }
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
