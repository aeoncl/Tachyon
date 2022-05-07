use std::{sync::{Mutex, RwLock, Arc}, collections::HashSet, mem};

use chashmap::CHashMap;
use matrix_sdk::{ruma::{events::{room::message::{RoomMessageEventContent}, OriginalSyncMessageLikeEvent}, OwnedEventId, OwnedRoomId, RoomId}, Client};
use tokio::sync::broadcast::{Sender, self, Receiver, error::SendError};

use super::{msg_payload::{factories::MsgPayloadFactory, MsgPayload}, msn_user::MSNUser, uuid::UUID};

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

    pub async fn send_message_to_server(&mut self, payload: MsgPayload) {
            let room_id = &self.target_room_id;
            if let Some(room) = self.matrix_client.get_joined_room(room_id) {
                match payload.content_type.as_str() {
                    "text/plain" => {
                        let content = RoomMessageEventContent::text_plain(payload.body);
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