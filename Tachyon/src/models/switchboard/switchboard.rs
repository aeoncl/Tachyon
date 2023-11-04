use std::{collections::HashSet, mem, str::FromStr, sync::{Arc, Mutex}};

use log::info;
use matrix_sdk::{attachment::AttachmentConfig, Client, ruma::{events::room::{MediaSource, message::RoomMessageEventContent}, OwnedEventId, OwnedRoomId, OwnedUserId}};
use tokio::sync::mpsc;
use tokio::sync::mpsc::{UnboundedReceiver, UnboundedSender};
use tokio::sync::mpsc::error::SendError;

use crate::{models::{msg_payload::MsgPayload, msn_user::MSNUser, p2p::file::File, msn_object::MSNObject}, MSN_CLIENT_LOCATOR, utils::emoji::smiley_to_emoji};

use super::{events::{content::{file_upload_event_content::FileUploadEventContent, message_event_content::MessageEventContent}, switchboard_event::SwitchboardEvent}, switchboard_error::SwitchboardError};

#[derive(Clone, Debug)]
pub struct Switchboard {
    pub(crate) inner: Arc<SwitchboardInner>,
}

#[derive(Debug)]
pub(crate) struct SwitchboardInner {
    //Matrix Events ID That were sent from here (for dedup)
    events_sent : Mutex<HashSet<String>>,
    target_room_id: OwnedRoomId,
    matrix_client: Client,
    creator_id: OwnedUserId,
    sb_event_sender: UnboundedSender<SwitchboardEvent>,
    sb_event_queued_listener: Mutex<Option<UnboundedReceiver<SwitchboardEvent>>>,
}

impl Drop for SwitchboardInner {
    fn drop(&mut self) {
        info!("DROPPING SWITCHBOARDInner");
    }
}

impl Switchboard {
    pub fn new(matrix_client: Client, target_room_id: OwnedRoomId, creator_id: OwnedUserId) -> Self {
        let (sb_event_sender, sb_event_queued_listener) = mpsc::unbounded_channel::<SwitchboardEvent>();

        let inner = Arc::new(SwitchboardInner {
            events_sent: Mutex::new(HashSet::new()),
            target_room_id,
            matrix_client,
            creator_id,
            sb_event_sender,
            sb_event_queued_listener: Mutex::new(Some(sb_event_queued_listener)),
        });

        let out = Switchboard {
            inner
        };

        return out;
    }

    /**
     * Sends a file to the other participants of the Switchboard
     */
    pub async fn send_file(&self, file: File) -> Result<OwnedEventId, SwitchboardError>{

        let mime = mime::Mime::from_str(file.get_mime().as_str())?;
        let room = self.inner.matrix_client.get_room(&self.inner.target_room_id)
        .ok_or(SwitchboardError::MatrixRoomNotFound)?;

        let config = AttachmentConfig::new().generate_thumbnail(None);
        let response = room.send_attachment(file.filename.as_str(), &mime, file.bytes, config).await?;

        info!("Sent file to server: {:?}", &response);

        self.add_to_events_sent(response.event_id.to_string());
        Ok(response.event_id)
    }

    /* Sends a Message to the other participants of the Switchboard */
    pub async fn send_message(&self, payload: MsgPayload) -> Result<(), SwitchboardError> {
        let room_id = &self.inner.target_room_id;
        let room = self.inner.matrix_client.get_room(&self.inner.target_room_id)
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
    pub fn on_message_received(&self, msg: MsgPayload, sender: MSNUser, event_id: Option<String>) -> Result<(), SendError<SwitchboardEvent>> {
        if let Some(event_id) = event_id {
            if self.is_ignored_event(&event_id) {
                return Ok(());
            }
        }
        let content = MessageEventContent{ msg, sender };
        return self.dispatch_event(SwitchboardEvent::MessageEvent(content));
    }

    pub fn on_file_received(&self, sender: MSNUser, filename: String, source: MediaSource, filesize: usize, event_id: String) -> Result<(), SendError<SwitchboardEvent>> {
        if self.is_ignored_event(&event_id) {
            return Ok(());
        }
        let client_data = MSN_CLIENT_LOCATOR.get().unwrap();

        return self.dispatch_event(FileUploadEventContent::new(sender,client_data.get_user(), filename, source, filesize).into());
    }

    pub fn get_receiver(&mut self) -> UnboundedReceiver<SwitchboardEvent> {
        let mut lock = self.inner.sb_event_queued_listener.lock().expect("a switchboard receiver lock to not be poisoned");

        if lock.is_none() {
            panic!("we expect a Switchboard to have a tokio receiver available")
        }

        let receiver = mem::replace(&mut *lock, None).expect("taking the receiver of a switchboard should work once");
        return receiver;
    }

    fn dispatch_event(&self, event: SwitchboardEvent) -> Result<(), tokio::sync::mpsc::error::SendError<SwitchboardEvent>> {
        return self.inner.sb_event_sender.send(event);
    }

    fn is_ignored_event(&self, event_id: &String) -> bool {
        return self.inner.events_sent.lock().unwrap().remove(event_id);
    }

    fn add_to_events_sent(&self, event_id: String) {
        self.inner.events_sent.lock().unwrap().insert(event_id);
    }

    
}