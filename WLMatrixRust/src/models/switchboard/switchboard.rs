use std::{sync::{Arc, Mutex}, collections::HashSet, str::FromStr, io::Cursor, mem};
use matrix_sdk::{ruma::{OwnedRoomId, OwnedUserId, OwnedEventId, events::room::message::RoomMessageEventContent}, Client, attachment::AttachmentConfig};
use tokio::sync::{broadcast::{Sender, Receiver, error::SendError, self}};

use crate::{utils::emoji::smiley_to_emoji, models::{p2p::file::File, msg_payload::MsgPayload, msn_user::MSNUser}};

use super::{events::{switchboard_event::SwitchboardEvent, content::message_event_content::MessageEventContent}, switchboard_error::SwitchboardError};


#[derive(Clone)]
pub struct Switchboard {
    pub(crate) inner: Arc<SwitchboardInner>,
}

pub(crate) struct SwitchboardInner {
    //Matrix Events ID That were sent from here (for dedup)
    events_sent : Mutex<HashSet<String>>,
    target_room_id: OwnedRoomId,
    matrix_client: Client,
    creator_id: OwnedUserId,
    sb_event_sender: Sender<SwitchboardEvent>,
    sb_event_queued_listener: Mutex<Option<Receiver<SwitchboardEvent>>>
}

impl Switchboard {
    pub fn new(matrix_client: Client, target_room_id: OwnedRoomId, creator_id: OwnedUserId) -> Self {
        let (sb_event_sender, sb_event_queued_listener) = broadcast::channel::<SwitchboardEvent>(30);

        let inner = Arc::new(SwitchboardInner {
            events_sent: Mutex::new(HashSet::new()),
            target_room_id,
            matrix_client,
            creator_id,
            sb_event_sender,
            sb_event_queued_listener: Mutex::new(Some(sb_event_queued_listener)),
        });

        return Switchboard {
            inner
        };
    }

    /**
     * Sends a file to the other participants of the Switchboard
     */
    pub async fn send_file(&self, file: File) -> Result<OwnedEventId, SwitchboardError>{

        let mime = mime::Mime::from_str(file.get_mime().as_str())?;
        let room = self.inner.matrix_client.get_joined_room(&self.inner.target_room_id)
        .ok_or(SwitchboardError::MatrixRoomNotFound)?;

        let config = AttachmentConfig::new().generate_thumbnail(None);
        let response = room.send_attachment(file.filename.as_str(), &mime, file.bytes, config).await?;

        self.add_to_events_sent(response.event_id.to_string());
        Ok(response.event_id)
    }

    /* Sends a Message to the other participants of the Switchboard */
    pub async fn send_message(&self, payload: MsgPayload) -> Result<(), SwitchboardError> {
        let room_id = &self.inner.target_room_id;
        let room = self.inner.matrix_client.get_joined_room(&self.inner.target_room_id)
        .ok_or(SwitchboardError::MatrixRoomNotFound)?;
        
        match payload.content_type.as_str() {
            "text/plain" => {
                let content = RoomMessageEventContent::text_plain(smiley_to_emoji(&payload.body));
                let response = room.send(content, None).await?;
                self.add_to_events_sent(response.event_id.to_string());
                Ok(())
            },
            "text/x-msmsgscontrol" => {
                //typing user
                room.typing_notice(true).await;
                Ok(())
            },
            _=> {
                Ok(())
            }
        }
    }

    /** Sends a received message to the Client */
    pub fn on_message_received(&self, msg: MsgPayload, sender: MSNUser, event_id: Option<String>) -> Result<usize, SendError<SwitchboardEvent>> {
        if let Some(event_id) = event_id {
            if self.is_ignored_event(&event_id) {
                return Ok(0);
            }
        }
        let content = MessageEventContent{ msg, sender };
        return self.dispatch_event(SwitchboardEvent::MessageEvent(content));
    }

    pub fn get_receiver(&mut self) -> Receiver<SwitchboardEvent> {
        let mut lock = self.inner.sb_event_queued_listener.lock().unwrap();
        if lock.is_none() {
            return self.inner.sb_event_sender.subscribe();
        } else {
            let receiver = mem::replace(&mut *lock, None).unwrap();
            return receiver;
        }
    }

    fn dispatch_event(&self, event: SwitchboardEvent)-> Result<usize, SendError<SwitchboardEvent>> {
        return self.inner.sb_event_sender.send(event);
    }

    fn is_ignored_event(&self, event_id: &String) -> bool {
        return self.inner.events_sent.lock().unwrap().remove(event_id);
    }

    fn add_to_events_sent(&self, event_id: String) {
        self.inner.events_sent.lock().unwrap().insert(event_id);
    }

    
}