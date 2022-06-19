use std::{sync::{Mutex, RwLock, Arc}, collections::HashSet, mem, io::Cursor, str::FromStr};
use chashmap::CHashMap;
use matrix_sdk::{ruma::{events::{room::message::{RoomMessageEventContent}, OriginalSyncMessageLikeEvent}, OwnedEventId, OwnedRoomId, RoomId}, Client, attachment::AttachmentConfig};
use mime::Mime;
use tokio::sync::broadcast::{Sender, self, Receiver, error::SendError};

use crate::utils::emoji::{smiley_to_emoji};

use super::{p2p::{factories::{SlpPayloadFactory, P2PPayloadFactory, TLVFactory}, p2p_transport_packet::P2PTransportPacket, File::File}, msn_user::MSNUser, msg_payload::{factories::MsgPayloadFactory, MsgPayload}};


#[derive(Clone)]
pub struct SwitchboardHandle {
    sender: Sender<String>,
    receiver: Arc<Mutex<Option<Receiver<String>>>>,
    events_sent : Arc<Mutex<HashSet<String>>>,
    matrix_client: Client,
    target_room_id: OwnedRoomId,
    client_msn_addr: String
}

impl SwitchboardHandle {

    pub fn new(matrix_client: Client, target_room_id: OwnedRoomId, client_msn_addr: String) -> SwitchboardHandle {
        let (sender, receiver) = broadcast::channel::<String>(30);
        return SwitchboardHandle{ sender: sender, 
            receiver: Arc::new(Mutex::new(Some(receiver))), 
            events_sent: Arc::new(Mutex::new(HashSet::new())), 
            matrix_client, 
            target_room_id, 
            client_msn_addr: client_msn_addr
            };
    }

    pub fn send_message_to_client(&self, msg: MsgPayload, sender_msn_addr: &String, matrix_event_id: Option<&String>) -> Result<usize, SendError<String>> {
        if let Some(event_id) = matrix_event_id {
            if self.is_ignored_event(event_id) {
                return Ok(0);
            }
        } 
        
        let serialized = msg.serialize();
        return self.sender.send(format!("MSG {sender} {sender} {payload_size}\r\n{payload}",sender=sender_msn_addr, payload_size=serialized.len(), payload=&serialized));
    }

    pub fn send_typing_notification_to_client(&self, typing_user_msn_addr: &String) -> Result<usize, SendError<String>> {
        let typing_user = MsgPayloadFactory::get_typing_user(typing_user_msn_addr.clone());
        return self.send_message_to_client(typing_user, typing_user_msn_addr, None);
    }

    pub async fn send_file_to_server(&self, file: File) {

            let mut cursor = Cursor::new(&file.bytes);
            if let Ok(mime) = mime::Mime::from_str(file.get_mime().as_str()) {
                if let Some (room) = self.matrix_client.get_joined_room(&self.target_room_id) {

                    let config = AttachmentConfig::new().generate_thumbnail(None);
                    room.send_attachment(file.filename.as_str(), &mime, &mut cursor, config).await;
                }
                
   
            }
            
    }

    pub async fn send_message_to_server(&mut self, payload: MsgPayload) {
            let room_id = &self.target_room_id;
            if let Some(room) = self.matrix_client.get_joined_room(room_id) {
                match payload.content_type.as_str() {
                    "text/plain" => {

                        //remove this
                        if payload.body.as_str() == "patate" {
                            let slp_payload = SlpPayloadFactory::get_direct_connect_invite();
                            let mut p2p_payload = P2PPayloadFactory::get_sip_text_message();
                            p2p_payload.set_payload(slp_payload.to_string().as_bytes().to_owned());
                            let mut p2p_transport = P2PTransportPacket::new(100000, Some(p2p_payload));
                            p2p_transport.set_syn(TLVFactory::get_client_peer_info());

                            let source = MSNUser::from_mpop_addr_string(String::from("aeontest3@shl.local;{77c46a8f-33a3-5282-9a5d-905ecd3eb069}")).unwrap();
                            let dest = MSNUser::from_mpop_addr_string(String::from("aeontest@shl.local;{f52973b6-c926-4bad-9ba8-7c1e840e4ab0}")).unwrap();

                            let msg_to_send = MsgPayloadFactory::get_p2p(&source, &dest,  &p2p_transport);
                            let serialized_response = msg_to_send.serialize();
                            let _result = self.sender.send(format!("MSG {msn_addr} {msn_addr} {payload_size}\r\n{payload}", msn_addr = &source.msn_addr, payload_size = serialized_response.len(), payload = &serialized_response));
                        
                        }

                        let content = RoomMessageEventContent::text_plain(smiley_to_emoji(&payload.body));
                        if let Ok(response) = room.send(content, None).await {
                             self.add_to_events_sent(response.event_id.to_string());
                        }
                    },
                    "text/x-msmsgscontrol" => {
                        //typing user
                        room.typing_notice(true).await;
                    }
                    _=> {

                    }
                }
        }
    }

    pub fn add_to_events_sent(&mut self, event_id: String) {
        self.events_sent.lock().unwrap().insert(event_id);
    }

    pub fn is_ignored_event(&self, event_id: &String) -> bool {
        return self.events_sent.lock().unwrap().remove(event_id);
    }

    pub fn get_sender(&self) -> Sender<String> {
        return self.sender.clone();
    }

    pub fn stop(&self) {
        let _result = self.sender.send(String::from("STOP"));
    }

    pub fn take_receiver(&mut self) -> Option<Receiver<String>> {
        let mut lock = self.receiver.lock().unwrap();
        let receiver = mem::replace(&mut *lock, None);
        return receiver;
    }
}