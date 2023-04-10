use std::{sync::{Arc, Mutex, atomic::{AtomicBool, Ordering}}, collections::HashSet, str::FromStr, io::Cursor, mem};
use log::info;
use matrix_sdk::{ruma::{OwnedRoomId, OwnedUserId, OwnedEventId, events::room::message::RoomMessageEventContent}, Client, attachment::AttachmentConfig};
use tokio::sync::{broadcast::{Sender, Receiver, error::SendError, self}, oneshot};

use crate::{utils::emoji::smiley_to_emoji, models::{p2p::{file::File, p2p_session::P2PSession, events::p2p_event::{P2PEvent, self}, p2p_transport_packet::P2PTransportPacket, pending_packet::PendingPacket}, msg_payload::{MsgPayload, factories::MsgPayloadFactory}, msn_user::MSNUser}, P2P_REPO};

use super::{events::{switchboard_event::SwitchboardEvent, content::message_event_content::MessageEventContent}, switchboard_error::SwitchboardError};


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
    p2p_session: Mutex<P2PSession>,
    sb_event_sender: Sender<SwitchboardEvent>,
    sb_event_queued_listener: Mutex<Option<Receiver<SwitchboardEvent>>>,
    p2p_stop_sender: Mutex<Option<oneshot::Sender<()>>>
}

impl Drop for SwitchboardInner {
    fn drop(&mut self) {
        info!("DROPPING SWITCHBOARDInner");
        if let Ok(mut sender) = self.p2p_stop_sender.lock() {
            if let Some(sender) = sender.take() {
                sender.send(());
           }
        }
    }
}

impl SwitchboardInner {
    pub fn new() {

    }
}


impl Switchboard {
    pub fn new(matrix_client: Client, target_room_id: OwnedRoomId, creator_id: OwnedUserId) -> Self {
        let (sb_event_sender, sb_event_queued_listener) = broadcast::channel::<SwitchboardEvent>(30);
        let (p2p_sender, p2p_listener) = broadcast::channel::<P2PEvent>(30);
        let (p2p_stop_sender, mut p2p_stop_receiver) = oneshot::channel::<()>();

        let inner = Arc::new(SwitchboardInner {
            events_sent: Mutex::new(HashSet::new()),
            target_room_id,
            matrix_client,
            creator_id,
            sb_event_sender,
            sb_event_queued_listener: Mutex::new(Some(sb_event_queued_listener)),
            p2p_session: Mutex::new(P2PSession::new(p2p_sender)),
            p2p_stop_sender: Mutex::new(Some(p2p_stop_sender)),
        });

        let out = Switchboard {
            inner
        };


        Self::start_listening_for_p2p(out.clone(), p2p_listener, p2p_stop_receiver);
        return out;
    }

    pub fn start_listening_for_p2p(switchboard: Switchboard, mut p2p_listener: Receiver<P2PEvent>, mut p2p_stop_listener: oneshot::Receiver<()>) {
        tokio::spawn(async move {
            loop {
                info!("START LISTENING FOR P2P");
                tokio::select! {
                    p2p_event = p2p_listener.recv() => {
                        if let Ok(p2p_event) = p2p_event {
                            match p2p_event {
                                P2PEvent::Message(content) => {
                                    //Todo change this
                                    P2P_REPO.set_seq_number(content.packet.get_next_sequence_number());
                                    info!("DEBUG P2PEVENT::Message");
                                    let msg = MsgPayloadFactory::get_p2p(&content.sender, &content.receiver, &content.packet);
                                    switchboard.on_message_received(msg, content.sender.clone(), None).unwrap();
                                },
                                P2PEvent::FileReceived(content) => {
                                    switchboard.send_file(content.file).await;
                                }
                                _ => {

                                }
                            }
                        }
                    },
                    p2p_stop = &mut p2p_stop_listener => {
                        info!("STOP LISTENING FOR P2P");
                        break;
                    }
                }
            }
        });
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
            "application/x-msnmsgrp2p" => {
                //P2P Message received
                log::info!("P2P Message received ! ");
                if let Ok(mut p2p_packet) = P2PTransportPacket::from_str(&payload.body){
                    let source = MSNUser::from_mpop_addr_string(payload.get_header(&String::from("P2P-Src")).unwrap().to_owned()).unwrap();
                    let dest = MSNUser::from_mpop_addr_string(payload.get_header(&String::from("P2P-Dest")).unwrap().to_owned()).unwrap(); 
                    if let Ok(p2p_session) = self.inner.p2p_session.lock().as_deref_mut() {
                    p2p_session.on_message_received(PendingPacket::new(p2p_packet, source, dest));
                    }
                }
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

    pub fn on_file_received(&self) {
        self.dispatch_event(SwitchboardEvent::FileUploadEvent);
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