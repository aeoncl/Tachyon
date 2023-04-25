use std::{
    collections::HashMap,
    f32::consts::E,
    mem,
    sync::{
        atomic::{AtomicBool, AtomicI32, Ordering},
        Arc, Mutex,
    },
    time::Duration,
};

use log::{debug, error, info, logger, warn};
use matrix_sdk::{
    media::{MediaEventContent, MediaFormat, MediaRequest},
    Client,
};
use rand::Rng;
use tokio::sync::broadcast::Sender;
use tokio::time::{self};

use crate::models::{
    errors::Errors,
    msn_user::MSNUser,
    p2p::{
        app_id::AppID,
        events::content::{
            file_received_event_content::FileReceivedEventContent,
            file_transfer_accepted_event_content::FileTransferAcceptedEventContent, msn_object_requested_event_content::MSNObjectRequestedEventContent,
        },
        slp_payload::EufGUID,
    },
    switchboard::{
        events::content::file_upload_event_content::FileUploadEventContent,
        switchboard::Switchboard,
    }, msn_object::MSNObject,
};

use super::{
    events::{content::message_event_content::MessageEventContent, p2p_event::P2PEvent},
    factories::{P2PPayloadFactory, P2PTransportPacketFactory, SlpPayloadFactory, TLVFactory},
    file::File,
    p2p_payload::P2PPayload,
    p2p_transport_packet::P2PTransportPacket,
    pending_packet::PendingPacket,
    session::{
        p2p_session::P2PSession, p2p_session_type::P2PSessionType, p2p_status::P2PSessionStatus,
    },
    slp_context::PreviewData,
    slp_payload::SlpPayload,
};

#[derive(Debug)]
pub struct InnerP2PClient {
    sender: Sender<P2PEvent>,

    //p2ppayload package number & P2PTransport packet
    inbound_chunked_packets: Mutex<HashMap<u16, PendingPacket>>,

    /* packets received before the handshake was done */
    inbound_pending_packets: Mutex<Vec<PendingPacket>>,

    /* Session has been initialized */
    initialized: AtomicBool,

    transport_session_status: AtomicI32,

    /* The current sequence number */
    sequence_number: Mutex<u32>,

    package_number: Mutex<u16>,

    /** a map of session_ids / files */
    pending_files: Mutex<HashMap<u32, File>>,

    /** a map of session_ids / FileUploadEventContent */
    pending_outbound_sessions: Mutex<HashMap<u32, P2PSession>>,
}

#[derive(Clone, Debug)]
pub struct P2PClient {
    inner: Arc<InnerP2PClient>,
}

impl P2PClient {
    pub fn new(sender: Sender<P2PEvent>) -> Self {
        let mut rng = rand::thread_rng();
        let seq_number = rng.gen::<u32>();

        return P2PClient {
            inner: Arc::new(InnerP2PClient {
                sender,
                inbound_chunked_packets: Mutex::new(HashMap::new()),
                inbound_pending_packets: Mutex::new(Vec::new()),
                initialized: AtomicBool::new(false),
                sequence_number: Mutex::new(seq_number),
                pending_files: Mutex::new(HashMap::new()),
                package_number: Mutex::new(150),
                pending_outbound_sessions: Mutex::new(HashMap::new()),
                transport_session_status: AtomicI32::new(P2PSessionStatus::WAITING as i32),
            }),
        };
    }

    fn get_transport_session_status(&self) -> P2PSessionStatus {
        let status = self.inner.transport_session_status.load(Ordering::Relaxed);
        return num::FromPrimitive::from_i32(status).unwrap();
    }

    fn set_transport_session_status(&mut self, status: P2PSessionStatus) {
        self.inner
            .transport_session_status
            .store(status as i32, Ordering::Relaxed);
    }

    pub fn set_seq_number(&mut self, seq_number: u32) {
        let mut seq_num = self
            .inner
            .sequence_number
            .lock()
            .expect("seq_number to be unlocked");

        let old_value = mem::replace(&mut *seq_num, seq_number);
        debug!(
            "Replacing seq_number: {} ({:01x}) with {} ({:01x}) (diff +{})",
            old_value,
            old_value,
            seq_number,
            seq_number,
            seq_number - old_value
        );
    }

    fn get_seq_number(&self) -> u32 {
        let mut seq_num = self
            .inner
            .sequence_number
            .lock()
            .expect("seq_number to be unlocked");
        return seq_num.clone();
    }

    pub fn set_initialized(&mut self, initialized: bool) {
        self.inner.initialized.store(initialized, Ordering::Relaxed);
    }

    fn is_in_chunks(&self, msg: &PendingPacket) -> bool {
        let mut found: bool = false;
        if let Some(package_num) = msg.packet.get_payload_package_number() {
            found = self
                .inner
                .inbound_chunked_packets
                .lock()
                .expect("chunkedpackets to be unlocked")
                .contains_key(&package_num);
        }
        return found;
    }

    fn pop_from_chunks(&mut self, package_number: u16) -> Option<PendingPacket> {
        return self
            .inner
            .inbound_chunked_packets
            .lock()
            .expect("chunkedpackets to be unlocked")
            .remove(&package_number);
    }

    fn add_or_append_to_chunks(&mut self, msg: &PendingPacket) -> u16 {
        if let Some(package_num) = msg.packet.get_payload_package_number() {
            let mut chunked_packets = self
                .inner
                .inbound_chunked_packets
                .lock()
                .expect("chunkedpackets to be unlocked");

            if let Some(found) = chunked_packets.get_mut(&package_num) {
                found.add_chunk(msg.packet.to_owned());
            } else {
                chunked_packets.insert(package_num, msg.to_owned());
            }
            return package_num;
        }
        return 0;
    }

    pub fn on_message_received(&mut self, msg: PendingPacket) {
        info!("OnMsgReceived: {:?}", &msg);

        let is_chunk = self.handle_chunks(&msg);
        info!("is_chunk: {}", &is_chunk);
        if is_chunk {
            info!("payload was chunked");
            if msg.packet.is_rak() && !msg.packet.is_slp_msg() {
                self.reply_ack(&msg);
            }
            return;
        }

        let is_initialized = self.handle_handshake(&msg);
        info!("is_initialized: {}", &is_initialized);
        if !is_initialized {
            // save the packet while we wait for handshake
            self.inner
                .inbound_pending_packets
                .lock()
                .expect("received_pending_packets to be unlocked")
                .push(msg);
            return;
        }

        if !self
            .inner
            .inbound_pending_packets
            .lock()
            .expect("received_pending_packets to be unlocked")
            .is_empty()
        {
            self.handle_pending_packets();
        }

        //main section
        let packet = msg.get_packet().expect("Packet was not complete");
        if !packet.is_syn() && packet.is_rak() {
            info!("REPLY_ACK: {:?}", &msg);
            self.reply_ack(&msg);
        }

        if let Some(payload) = packet.get_payload() {
            info!("Payload was P2PPayload");

            if let Ok(slp_request) = payload.get_payload_as_slp() {
                info!("Payload contained SLPRequest");

                if let Ok(Some(slp_response)) = self.handle_slp_payload(&slp_request, &msg.sender, &msg.receiver) {
                    self.reply_slp(&msg.receiver, &msg.sender, slp_response);
                } else {
                    //TODO reply error slp
                }
            } else if payload.is_file_transfer() {
                info!(
                    "file transfer packet received!, retrieveing pending file: {}",
                    &payload.session_id
                );
                let file = self
                    .inner
                    .pending_files
                    .lock()
                    .expect("pending_files to be unlocked")
                    .remove(&payload.session_id);
                if let Some(mut file) = file {
                    file.bytes = payload.get_payload_bytes().clone();
                    self.inner
                        .sender
                        .send(P2PEvent::FileReceived(FileReceivedEventContent {
                            file: file,
                        }));
                }
                self.reply_ack(&msg);
            }
        }
    }

    fn handle_pending_packets(&mut self) {
        let mut packets = Vec::new();
        packets.append(
            &mut self
                .inner
                .inbound_pending_packets
                .lock()
                .expect("received_pending_packets to be unlocked"),
        );

        for packet in packets {
            self.on_message_received(packet);
        }
    }

    fn handle_handshake(&mut self, msg: &PendingPacket) -> bool {
        info!("HANDLE HANDSHAKE: {:?}", msg);
        if !self.inner.initialized.load(Ordering::Relaxed) {
            let packet = msg.get_packet().expect("Packet was not complete");
            if packet.is_syn() {
                if packet.is_rak() {
                    //We need to send a syn + ack + rak and wait for handshake
                    self.reply_handshake(&msg);
                } else {
                    //Bypassed handshake
                    self.set_initialized(true);
                }
            } else if packet.is_ack() {
                //ack received for our rak (maybe check with number later)
                self.set_initialized(true);
            }
        }

        return self.inner.initialized.load(Ordering::Relaxed);
    }

    fn handle_chunks(&mut self, msg: &PendingPacket) -> bool {
        let is_in_chunks = self.is_in_chunks(&msg);

        if is_in_chunks || !msg.is_complete() {
            let package_number = self.add_or_append_to_chunks(&msg);

            if msg.is_complete() {
                //this is the last chunk
                info!("chunked payload is now complete!");
                self.on_message_complete(package_number);
            }
        }

        return is_in_chunks || !msg.is_complete();
    }

    pub fn setup_handshake(&mut self, sender: &MSNUser, receiver: &MSNUser) {
        let init_slp_msg = SlpPayloadFactory::get_transport_request(sender, receiver);
        self.reply_slp(sender, receiver, init_slp_msg);
    }

    fn on_message_complete(&mut self, package_number: u16) {
        info!("message complete! {}", &package_number);
        let packet = self
            .pop_from_chunks(package_number)
            .expect("on_message_complete did not in fact contain a message");
        self.on_message_received(packet);
    }

    /**
    * EUF GUID
    *     MSN_OBJECT = "{A4268EEC-FEC5-49E5-95C3-F126696BDBF6}"
       FILE_TRANSFER = "{5D3E02AB-6190-11D3-BBBB-00C04F795683}"
       MEDIA_RECEIVE_ONLY = "{1C9AA97E-9C05-4583-A3BD-908A196F1E92}"
       MEDIA_SESSION = "{4BD96FC0-AB17-4425-A14A-439185962DC8}"
       SHARE_PHOTO = "{41D3E74E-04A2-4B37-96F8-08ACDB610874}"
       ACTIVITY = "{6A13AF9C-5308-4F35-923A-67E8DDA40C2F}"
    */

    /**
    * APP IDS
        FILE_TRANSFER = 2
      CUSTOM_EMOTICON_TRANSFER = 11
      DISPLAY_PICTURE_TRANSFER = 12
      WEBCAM = 4
    */

    fn reply_slp(&mut self, sender: &MSNUser, receiver: &MSNUser, slp_response: SlpPayload) {
        info!("reply SLP! {}", slp_response.to_string());
        let mut p2p_payload_response = P2PPayloadFactory::get_sip_text_message();
        p2p_payload_response.set_payload(slp_response.to_string().as_bytes().to_owned());

        let slp_transport_resp = P2PTransportPacket::new(0, Some(p2p_payload_response));
        self.reply(sender, receiver, slp_transport_resp);
    }

    fn reply_ack(&mut self, request: &PendingPacket) {
        self.reply(
            &request.receiver,
            &request.sender,
            P2PTransportPacketFactory::get_ack(request.get_last_chunk_next_seq_number()),
        );
    }

    fn reply_handshake(&mut self, request: &PendingPacket) {
        self.reply(
            &request.receiver,
            &request.sender,
            P2PTransportPacketFactory::get_syn_ack(request.get_last_chunk_next_seq_number()),
        );
    }

    fn reply(&mut self, sender: &MSNUser, receiver: &MSNUser, msg_to_send: P2PTransportPacket) {
        let mut packet_to_send = msg_to_send.clone();

        if let Some(payload) = packet_to_send.get_payload_as_mut() {
            payload.package_number = self.get_package_number();
            //setting next package number
            self.set_package_number(self.get_package_number() + 1);
        }

        if packet_to_send.get_payload().is_some()
            && packet_to_send
                .get_payload()
                .unwrap()
                .get_payload_bytes()
                .len()
                > 1222
        {
            //We need to split this joker, he's too big
            let split = self.split(packet_to_send);
            info!("OnChunkedMsgReply");
            self.inner
                .sender
                .send(P2PEvent::Message(MessageEventContent {
                    packets: split,
                    sender: sender.clone(),
                    receiver: receiver.clone(),
                }));
        } else {
            info!("OnSingleMsgReply: {:?}", &packet_to_send);
            //setting next sequence number
            packet_to_send.sequence_number = self.get_seq_number();
            self.set_seq_number(self.get_seq_number() + packet_to_send.get_payload_length());

            let mut out = Vec::new();
            out.push(packet_to_send);
            self.inner
                .sender
                .send(P2PEvent::Message(MessageEventContent {
                    packets: out,
                    sender: sender.clone(),
                    receiver: receiver.clone(),
                }));
        }
    }

    fn split(&mut self, to_split: P2PTransportPacket) -> Vec<P2PTransportPacket> {
        let payload = to_split
            .get_payload()
            .expect("A payload should be present when we split");
        let payload_bytes = payload.get_payload_bytes();

        let chunks: Vec<&[u8]> = payload_bytes.chunks(1222).collect();

        let mut out: Vec<P2PTransportPacket> = Vec::new();
        let mut remaining_bytes = payload_bytes.len();

        for i in 0..chunks.len() {
            let chunk = chunks[i];
            remaining_bytes -= chunk.len();

            let mut payloadToAdd = P2PPayload::new(payload.tf_combination, payload.session_id);
            payloadToAdd.package_number = payload.get_package_number();
            payloadToAdd.payload = chunk.to_vec();
            payloadToAdd.session_id = payload.session_id;

            let mut toAdd: P2PTransportPacket = P2PTransportPacket::new(0, None);

            if i < chunks.len() - 1 {
                //We need to add the remaining bytes TLV
                payloadToAdd.add_tlv(TLVFactory::get_untransfered_data_size(
                    remaining_bytes.try_into().unwrap(),
                ));
                info!("remainingBytes: {}", remaining_bytes);
            }

            if i == 0 {
                toAdd.op_code = to_split.op_code;
                toAdd.tlvs = to_split.tlvs.clone();
                toAdd.set_payload(Some(payloadToAdd));
                //We are creating the first packet
            } else {
                payloadToAdd.tf_combination = payloadToAdd.tf_combination - 1;
                toAdd.set_payload(Some(payloadToAdd));
                //We are creating other packets
            }

            toAdd.sequence_number = self.get_seq_number();
            self.set_seq_number(self.get_seq_number() + toAdd.get_payload_length());

            out.push(toAdd);
        }

        return out;
    }

    pub fn initiate_session(
        &mut self,
        inviter: MSNUser,
        invitee: MSNUser,
        session_type: P2PSessionType,
    ) {
        //Todo setup handshake


        let mut slp_request: Option<SlpPayload> = None;

        let mut rng = rand::thread_rng();
        let session_id: u32 = rng.gen();

        match session_type {
            P2PSessionType::FileTransfer(ref content) => {
                let context = PreviewData::new(content.filesize.clone(), content.filename.clone());
                slp_request = Some(
                    SlpPayloadFactory::get_file_transfer_request(
                        &inviter, &invitee, &context, session_id,
                    )
                    .unwrap(),
                );
            }
        }

        let mut p2p_payload = P2PPayloadFactory::get_sip_text_message();
        p2p_payload.set_payload(
            slp_request
                .expect("An SLP Payload to have")
                .to_string()
                .as_bytes()
                .to_owned(),
        );
        p2p_payload.tf_combination = 0x01;
        p2p_payload.session_id = 0;

        let mut slp_transport_req = P2PTransportPacket::new(0, Some(p2p_payload));



        let session: P2PSession =
            P2PSession::new(session_type, session_id, inviter.clone(), invitee.clone());
        self.inner
            .pending_outbound_sessions
            .lock()
            .expect("pending_files_to_send to be unlocked")
            .insert(session_id, session);

            if !self.inner.initialized.load(Ordering::Relaxed) {
                slp_transport_req.op_code = 0x03;
                //Added this so we don't answer their syn by another syn
                self.inner.initialized.store(true, Ordering::Relaxed);
            } 
        self.reply(&inviter, &invitee, slp_transport_req);
    }

    fn handle_slp_payload(
        &mut self,
        slp_payload: &SlpPayload,
        sender: &MSNUser,
        receiver: &MSNUser
    ) -> Result<Option<SlpPayload>, Errors> {
        let error = String::from("error");
        let content_type = slp_payload.get_content_type().unwrap_or(&error);
        match content_type.as_str() {
            "application/x-msnmsgr-transreqbody" => {
                //  let slp_payload_response = SlpPayloadFactory::get_200_ok_direct_connect_bad_port(&slp_payload)?;
                //let mut slp_payload_response = SlpPayloadFactory::get_500_error_direct_connect(slp_payload, String::from("TCPv1"))?; //todo unwrap_or error slp message
                // if self.test > 0 {
                let slp_payload_response = SlpPayloadFactory::get_500_error_direct_connect(
                    slp_payload,
                    String::from("TCPv1"),
                )
                .unwrap(); //todo unwrap_or error slp message
                           //  }

                // self.test += 1;

                // let mut p2p_payload_response = P2PPayloadFactory::get_sip_text_message();
                // p2p_payload_response.set_payload(slp_payload_response.to_string().as_bytes().to_owned());
                return Ok(Some(slp_payload_response));
                // return Err(Errors::PayloadNotComplete);
            }
            "application/x-msnmsgr-sessionreqbody" => {
                //if it's a file transfer request. TODO change this and put it inside slp_payload via an enum
                info!("GOT SESS REQ_BODY");
                return self.handle_sessionreqbody(slp_payload, sender, receiver);
            }
            "application/x-msnmsgr-transrespbody" => {
                let bridge = slp_payload
                    .get_body_property(&String::from("Bridge"))
                    .unwrap();
                let slp_payload_response = SlpPayloadFactory::get_500_error_direct_connect(
                    slp_payload,
                    bridge.to_owned(),
                )?;
                return Ok(Some(slp_payload_response));
            }
            "application/x-msnmsgr-sessionclosebody" => {
                return Err(Errors::PayloadNotComplete);
            }
            _ => {
                info!("not handled slp payload: {:?}", slp_payload);
                return Err(Errors::PayloadNotComplete);
            }
        }
    }

    fn handle_sessionreqbody(
        &mut self,
        slp_payload: &SlpPayload,
        sender: &MSNUser,
        receiver: &MSNUser
    ) -> Result<Option<SlpPayload>, Errors> {
        debug!(
            "handle_sessionreqbody: is_invite: {}, is_200_ok: {} - {:?}",
            &slp_payload.is_invite(),
            &slp_payload.is_200_ok(),
            &slp_payload
        );

        if slp_payload.is_invite() {
            let euf_guid = slp_payload
                .get_euf_guid()
                .expect("EUF-GUID to be valid")
                .expect("EUF-GUID to be here");
            let app_id = slp_payload
                .get_app_id()
                .expect("AppID to be valid")
                .expect("AppID to be present");

            let session_id = slp_payload
            .get_body_property(&String::from("SessionID"))
            .ok_or(Errors::PayloadDeserializeError)?
            .parse::<u32>()?;


            match euf_guid {
                EufGUID::FileTransfer => {
                    if app_id == AppID::FILE_TRANSFER {
                        let context = slp_payload.get_context_as_preview_data().expect("Preview Data to be present here");

                            self.inner
                                .pending_files
                                .lock()
                                .expect("pending_files to be unlocked")
                                .insert(
                                    session_id,
                                    File::new(context.get_size(), context.get_filename()),
                                );
                        
                    }
                },
                EufGUID::MSNObject => {
                    let context = *slp_payload.get_context_as_msnobj().expect("MSNObject to be present here");
                    if app_id == AppID::DISPLAY_PICTURE_TRANSFER {
                        self.inner.sender.send(P2PEvent::MSNObjectRequested(MSNObjectRequestedEventContent{
                            msn_object: context,
                            session_id: session_id,
                            inviter: sender.clone(),
                            invitee: receiver.clone()
                        }));
                    }
                }
                _ => {
                    warn!(
                        "Received unsupported invite EufGUID: {} - payload: {}",
                        euf_guid, slp_payload
                    )
                }
            }

            return Ok(Some(SlpPayloadFactory::get_200_ok_session(slp_payload)?));

        } else if slp_payload.is_200_ok() {
            //transfer stuff
            if let Some(session_id) = slp_payload.get_body_property(&String::from("SessionID")) {
                let session_id = session_id.parse::<u32>().unwrap();

                let pending_sessions_lock = self
                    .inner
                    .pending_outbound_sessions
                    .lock()
                    .expect("pending_files_to_send to be unlocked while sending");

                let maybe_session = pending_sessions_lock.get(&session_id);
                let session =
                    maybe_session.expect(format!("session {} to be present", session_id).as_str());

                match session.get_type() {
                    &P2PSessionType::FileTransfer(ref content) => {
                        self.inner.sender.send(P2PEvent::FileTransferAccepted(
                            FileTransferAcceptedEventContent {
                                source: content
                                    .source
                                    .as_ref()
                                    .expect("media source to be present")
                                    .clone(),
                                session_id,
                            },
                        ));
                    }
                    _ => {
                        warn!("Session Type not handled yet");
                    }
                }
            }
        }
        return Ok(None);
    }

    pub fn send_file(&mut self, session_id: u32, file: Vec<u8>) {
        let maybe_session = self
            .inner
            .pending_outbound_sessions
            .lock()
            .expect("pending_files_to_send to be unlocked while sending")
            .remove(&session_id);
        if let Some(session) = maybe_session {
            let mut payload = P2PPayloadFactory::get_file_transfer(session_id);
            payload.set_payload(file);
            let packet = P2PTransportPacket::new(0, Some(payload));
            self.reply(&session.get_inviter(), &session.get_invitee(), packet);
        }
    }

    pub fn send_msn_object(&mut self, session_id: u32, file: Vec<u8>, sender: MSNUser, receiver: MSNUser) {
        let data_preparation_message = P2PPayloadFactory::get_data_preparation_message(session_id);
        let data_preparation_packet = P2PTransportPacket::new(0, Some(data_preparation_message));
        self.reply(&sender, &receiver, data_preparation_packet);

        let mut msn_obj_message = P2PPayloadFactory::get_msn_obj(session_id);
        msn_obj_message.set_payload(file);
        let msn_obj_packet = P2PTransportPacket::new(0, Some(msn_obj_message));
        self.reply(&sender, &receiver, msn_obj_packet);
    }

    fn set_package_number(&mut self, package_number: u16) {
        let mut package_num = self
            .inner
            .package_number
            .lock()
            .expect("Package number to be unlocked");
        *package_num = package_number;
    }

    fn get_package_number(&self) -> u16 {
        return self
            .inner
            .package_number
            .lock()
            .expect("Package number to be unlocked")
            .clone();
    }
}

#[cfg(test)]
mod tests {
    use std::str::from_utf8_unchecked;

    use log::info;
    use tokio::sync::broadcast;

    use crate::{
        models::{
            msn_user::MSNUser,
            p2p::{
                events::p2p_event::P2PEvent,
                factories::{P2PTransportPacketFactory, TLVFactory},
                p2p_payload::P2PPayload,
                p2p_transport_packet::P2PTransportPacket,
                pending_packet::PendingPacket,
            },
        },
        sockets::{
            command_handler::CommandHandler, events::socket_event::SocketEvent,
            msnp_command::MSNPCommandParser,
            switchboard_command_handler::SwitchboardCommandHandler,
        },
    };

    use super::P2PClient;

    #[actix_rt::test]
    async fn test_chunked_payload() {
        let part1_msg: [u8; 1833] = [
            24, 3, 4, 202, 138, 185, 205, 99, 1, 12, 0, 2, 0, 0, 0, 14, 48, 48, 15, 1, 0, 0, 0, 0,
            20, 1, 0, 0, 0, 0, 0, 0, 1, 8, 0, 0, 0, 0, 0, 0, 0, 131, 0, 0, 73, 78, 86, 73, 84, 69,
            32, 77, 83, 78, 77, 83, 71, 82, 58, 97, 101, 111, 110, 116, 101, 115, 116, 51, 64, 115,
            104, 108, 46, 108, 111, 99, 97, 108, 59, 123, 55, 55, 99, 52, 54, 97, 56, 102, 45, 51,
            51, 97, 51, 45, 53, 50, 56, 50, 45, 57, 97, 53, 100, 45, 57, 48, 53, 101, 99, 100, 51,
            101, 98, 48, 54, 57, 125, 32, 77, 83, 78, 83, 76, 80, 47, 49, 46, 48, 13, 10, 84, 111,
            58, 32, 60, 109, 115, 110, 109, 115, 103, 114, 58, 97, 101, 111, 110, 116, 101, 115,
            116, 51, 64, 115, 104, 108, 46, 108, 111, 99, 97, 108, 59, 123, 55, 55, 99, 52, 54, 97,
            56, 102, 45, 51, 51, 97, 51, 45, 53, 50, 56, 50, 45, 57, 97, 53, 100, 45, 57, 48, 53,
            101, 99, 100, 51, 101, 98, 48, 54, 57, 125, 62, 13, 10, 70, 114, 111, 109, 58, 32, 60,
            109, 115, 110, 109, 115, 103, 114, 58, 97, 101, 111, 110, 116, 101, 115, 116, 64, 115,
            104, 108, 46, 108, 111, 99, 97, 108, 59, 123, 102, 53, 50, 57, 55, 51, 98, 54, 45, 99,
            57, 50, 54, 45, 52, 98, 97, 100, 45, 57, 98, 97, 56, 45, 55, 99, 49, 101, 56, 52, 48,
            101, 52, 97, 98, 48, 125, 62, 13, 10, 86, 105, 97, 58, 32, 77, 83, 78, 83, 76, 80, 47,
            49, 46, 48, 47, 84, 76, 80, 32, 59, 98, 114, 97, 110, 99, 104, 61, 123, 55, 65, 49, 54,
            65, 50, 67, 69, 45, 70, 68, 66, 70, 45, 52, 49, 69, 56, 45, 57, 55, 57, 56, 45, 54, 55,
            56, 53, 52, 67, 69, 51, 68, 53, 55, 57, 125, 13, 10, 67, 83, 101, 113, 58, 32, 48, 32,
            13, 10, 67, 97, 108, 108, 45, 73, 68, 58, 32, 123, 56, 50, 57, 66, 70, 50, 52, 50, 45,
            68, 57, 68, 67, 45, 52, 66, 66, 48, 45, 57, 70, 52, 48, 45, 52, 68, 48, 48, 65, 52, 49,
            52, 49, 68, 57, 48, 125, 13, 10, 77, 97, 120, 45, 70, 111, 114, 119, 97, 114, 100, 115,
            58, 32, 48, 13, 10, 67, 111, 110, 116, 101, 110, 116, 45, 84, 121, 112, 101, 58, 32,
            97, 112, 112, 108, 105, 99, 97, 116, 105, 111, 110, 47, 120, 45, 109, 115, 110, 109,
            115, 103, 114, 45, 115, 101, 115, 115, 105, 111, 110, 114, 101, 113, 98, 111, 100, 121,
            13, 10, 67, 111, 110, 116, 101, 110, 116, 45, 76, 101, 110, 103, 116, 104, 58, 32, 56,
            56, 51, 13, 10, 13, 10, 69, 85, 70, 45, 71, 85, 73, 68, 58, 32, 123, 53, 68, 51, 69,
            48, 50, 65, 66, 45, 54, 49, 57, 48, 45, 49, 49, 68, 51, 45, 66, 66, 66, 66, 45, 48, 48,
            67, 48, 52, 70, 55, 57, 53, 54, 56, 51, 125, 13, 10, 83, 101, 115, 115, 105, 111, 110,
            73, 68, 58, 32, 49, 57, 57, 57, 51, 52, 50, 50, 52, 54, 13, 10, 65, 112, 112, 73, 68,
            58, 32, 50, 13, 10, 82, 101, 113, 117, 101, 115, 116, 70, 108, 97, 103, 115, 58, 32,
            49, 54, 13, 10, 67, 111, 110, 116, 101, 120, 116, 58, 32, 80, 103, 73, 65, 65, 65, 73,
            65, 65, 65, 68, 109, 106, 81, 65, 65, 65, 65, 65, 65, 65, 65, 69, 65, 65, 65, 66, 104,
            65, 71, 85, 65, 98, 119, 66, 117, 65, 67, 65, 65, 99, 65, 66, 112, 65, 72, 103, 65, 90,
            81, 66, 115, 65, 67, 52, 65, 99, 65, 66, 122, 65, 71, 81, 65, 65, 65, 65, 65, 65, 65,
            65, 65, 65, 65, 65, 65, 65, 65, 65, 65, 65, 65, 65, 65, 65, 65, 65, 65, 65, 65, 65, 65,
            65, 65, 65, 65, 65, 65, 65, 65, 65, 65, 65, 65, 65, 65, 65, 65, 65, 65, 65, 65, 65, 65,
            65, 65, 65, 65, 65, 65, 65, 65, 65, 65, 65, 65, 65, 65, 65, 65, 65, 65, 65, 65, 65, 65,
            65, 65, 65, 65, 65, 65, 65, 65, 65, 65, 65, 65, 65, 65, 65, 65, 65, 65, 65, 65, 65, 65,
            65, 65, 65, 65, 65, 65, 65, 65, 65, 65, 65, 65, 65, 65, 65, 65, 65, 65, 65, 65, 65, 65,
            65, 65, 65, 65, 65, 65, 65, 65, 65, 65, 65, 65, 65, 65, 65, 65, 65, 65, 65, 65, 65, 65,
            65, 65, 65, 65, 65, 65, 65, 65, 65, 65, 65, 65, 65, 65, 65, 65, 65, 65, 65, 65, 65, 65,
            65, 65, 65, 65, 65, 65, 65, 65, 65, 65, 65, 65, 65, 65, 65, 65, 65, 65, 65, 65, 65, 65,
            65, 65, 65, 65, 65, 65, 65, 65, 65, 65, 65, 65, 65, 65, 65, 65, 65, 65, 65, 65, 65, 65,
            65, 65, 65, 65, 65, 65, 65, 65, 65, 65, 65, 65, 65, 65, 65, 65, 65, 65, 65, 65, 65, 65,
            65, 65, 65, 65, 65, 65, 65, 65, 65, 65, 65, 65, 65, 65, 65, 65, 65, 65, 65, 65, 65, 65,
            65, 65, 65, 65, 65, 65, 65, 65, 65, 65, 65, 65, 65, 65, 65, 65, 65, 65, 65, 65, 65, 65,
            65, 65, 65, 65, 65, 65, 65, 65, 65, 65, 65, 65, 65, 65, 65, 65, 65, 65, 65, 65, 65, 65,
            65, 65, 65, 65, 65, 65, 65, 65, 65, 65, 65, 65, 65, 65, 65, 65, 65, 65, 65, 65, 65, 65,
            65, 65, 65, 65, 65, 65, 65, 65, 65, 65, 65, 65, 65, 65, 65, 65, 65, 65, 65, 65, 65, 65,
            65, 65, 65, 65, 65, 65, 65, 65, 65, 65, 65, 65, 65, 65, 65, 65, 65, 65, 65, 65, 65, 65,
            65, 65, 65, 65, 65, 65, 65, 65, 65, 65, 65, 65, 65, 65, 65, 65, 65, 65, 65, 65, 65, 65,
            65, 65, 65, 65, 65, 65, 65, 65, 65, 65, 65, 65, 65, 65, 65, 65, 65, 65, 65, 65, 65, 65,
            65, 65, 65, 65, 65, 65, 65, 65, 65, 65, 65, 65, 65, 65, 65, 65, 65, 65, 65, 65, 65, 65,
            65, 65, 65, 65, 65, 65, 65, 65, 65, 65, 65, 65, 65, 65, 65, 65, 65, 65, 65, 65, 65, 65,
            65, 65, 65, 65, 65, 65, 65, 65, 65, 65, 65, 65, 65, 65, 65, 65, 65, 65, 65, 65, 65, 65,
            65, 65, 65, 65, 65, 65, 65, 65, 65, 65, 65, 65, 65, 65, 65, 65, 65, 65, 65, 65, 65, 65,
            65, 65, 65, 65, 65, 65, 65, 65, 65, 65, 65, 65, 65, 65, 65, 65, 65, 65, 65, 65, 65, 65,
            65, 65, 65, 65, 65, 65, 65, 65, 65, 65, 65, 65, 65, 65, 65, 65, 65, 65, 65, 65, 65, 65,
            65, 65, 65, 65, 65, 65, 65, 65, 65, 65, 65, 65, 65, 65, 65, 65, 65, 65, 65, 65, 65, 65,
            65, 65, 65, 65, 65, 65, 65, 65, 65, 65, 65, 65, 65, 65, 65, 65, 65, 65, 65, 65, 65, 65,
            0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0,
            0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0,
            0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0,
            0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0,
            0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0,
            0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0,
            0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0,
            0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0,
            0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0,
            0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0,
            0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0,
            0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0,
            0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0,
            0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0,
            0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0,
            0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0,
            0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0,
            0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0,
            0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0,
            0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0,
            0, 0, 0,
        ];
        let part2_msg: [u8; 151] = [
            8, 0, 0, 139, 138, 185, 210, 45, 8, 0, 0, 0, 0, 0, 0, 0, 65, 65, 65, 65, 65, 65, 65,
            65, 65, 65, 65, 65, 65, 65, 65, 65, 65, 65, 65, 65, 65, 65, 65, 65, 65, 65, 65, 65, 65,
            65, 65, 65, 65, 65, 65, 65, 65, 65, 65, 65, 65, 65, 65, 65, 65, 65, 65, 65, 65, 65, 65,
            65, 65, 65, 65, 65, 65, 65, 65, 65, 65, 65, 65, 65, 65, 65, 65, 65, 65, 65, 65, 65, 65,
            65, 65, 65, 65, 65, 65, 65, 65, 65, 65, 65, 65, 65, 65, 65, 65, 65, 65, 65, 65, 65, 65,
            65, 65, 65, 65, 65, 65, 65, 65, 65, 65, 65, 65, 65, 65, 65, 65, 65, 65, 65, 65, 65, 65,
            65, 65, 65, 65, 65, 65, 65, 61, 61, 13, 10, 13, 10, 0, 0, 0, 0, 0,
        ];

        // let p2p_transport_packet1 = P2PTransportPacket::try_from(part1_msg.as_ref()).unwrap();
        // let p2p_transport_packet2 = P2PTransportPacket::try_from(part2_msg.as_ref()).unwrap();

        // let (p2p_sender, mut p2p_receiver) = broadcast::channel::<P2PEvent>(10);

        // let mut p2p_session = P2PSession::new(p2p_sender);

        // p2p_session.on_message_received(PendingPacket::new(
        //     p2p_transport_packet1,
        //     MSNUser::default(),
        //     MSNUser::default(),
        // ));
        // p2p_session.on_message_received(PendingPacket::new(
        //     p2p_transport_packet2,
        //     MSNUser::default(),
        //     MSNUser::default(),
        // ));

        // let syn_ack = p2p_receiver.recv().await.unwrap();
        // assert!(syn_ack.is_complete());

        // let syn_ack_packet = syn_ack.get_packet().unwrap();
        // assert!(syn_ack_packet.is_syn());
        // assert!(syn_ack_packet.is_rak());
        // assert!(syn_ack_packet.is_ack());

        // let mut ack = P2PTransportPacketFactory::get_ack(syn_ack_packet.get_next_sequence_number());
        // let ack_packet = PendingPacket::new(ack, MSNUser::default(), MSNUser::default());

        // p2p_session.on_message_received(ack_packet.clone());

        // let invite_response = p2p_receiver.recv().await.unwrap();
        // assert!(invite_response.is_complete());
        // let invite_p2p_transport = invite_response.get_packet().unwrap();
        // let invite_slp_payload = invite_p2p_transport
        //     .get_payload()
        //     .unwrap()
        //     .get_payload_as_slp()
        //     .unwrap();
        // assert!(invite_slp_payload.first_line.contains("200 OK"));

        // println!("{}", invite_slp_payload.to_string());
    }

    #[actix_rt::test]
    async fn sb_session_test() {
        let mut parser = MSNPCommandParser::new();

        let first_msg: [u8; 1466] = [
            77, 83, 71, 32, 49, 55, 57, 32, 68, 32, 49, 52, 53, 51, 13, 10, 77, 73, 77, 69, 45, 86,
            101, 114, 115, 105, 111, 110, 58, 32, 49, 46, 48, 13, 10, 67, 111, 110, 116, 101, 110,
            116, 45, 84, 121, 112, 101, 58, 32, 97, 112, 112, 108, 105, 99, 97, 116, 105, 111, 110,
            47, 120, 45, 109, 115, 110, 109, 115, 103, 114, 112, 50, 112, 13, 10, 80, 50, 80, 45,
            68, 101, 115, 116, 58, 32, 97, 101, 111, 110, 116, 101, 115, 116, 51, 64, 115, 104,
            108, 46, 108, 111, 99, 97, 108, 59, 123, 55, 55, 99, 52, 54, 97, 56, 102, 45, 51, 51,
            97, 51, 45, 53, 50, 56, 50, 45, 57, 97, 53, 100, 45, 57, 48, 53, 101, 99, 100, 51, 101,
            98, 48, 54, 57, 125, 13, 10, 80, 50, 80, 45, 83, 114, 99, 58, 32, 97, 101, 111, 110,
            116, 101, 115, 116, 64, 115, 104, 108, 46, 108, 111, 99, 97, 108, 59, 123, 102, 53, 50,
            57, 55, 51, 98, 54, 45, 99, 57, 50, 54, 45, 52, 98, 97, 100, 45, 57, 98, 97, 56, 45,
            55, 99, 49, 101, 56, 52, 48, 101, 52, 97, 98, 48, 125, 13, 10, 13, 10, 8, 0, 4, 218,
            247, 181, 42, 13, 20, 7, 0, 0, 188, 72, 245, 112, 1, 8, 0, 0, 0, 0, 1, 66, 219, 204, 0,
            0, 56, 66, 80, 83, 0, 1, 0, 0, 0, 0, 0, 0, 0, 3, 0, 0, 4, 56, 0, 0, 7, 128, 0, 8, 0, 3,
            0, 0, 0, 0, 0, 0, 92, 130, 56, 66, 73, 77, 4, 4, 0, 0, 0, 0, 0, 7, 28, 2, 0, 0, 2, 0,
            0, 0, 56, 66, 73, 77, 4, 37, 0, 0, 0, 0, 0, 16, 232, 241, 92, 243, 47, 193, 24, 161,
            162, 123, 103, 173, 197, 100, 213, 186, 56, 66, 73, 77, 4, 36, 0, 0, 0, 0, 61, 8, 60,
            63, 120, 112, 97, 99, 107, 101, 116, 32, 98, 101, 103, 105, 110, 61, 34, 239, 187, 191,
            34, 32, 105, 100, 61, 34, 87, 53, 77, 48, 77, 112, 67, 101, 104, 105, 72, 122, 114,
            101, 83, 122, 78, 84, 99, 122, 107, 99, 57, 100, 34, 63, 62, 10, 60, 120, 58, 120, 109,
            112, 109, 101, 116, 97, 32, 120, 109, 108, 110, 115, 58, 120, 61, 34, 97, 100, 111, 98,
            101, 58, 110, 115, 58, 109, 101, 116, 97, 47, 34, 32, 120, 58, 120, 109, 112, 116, 107,
            61, 34, 65, 100, 111, 98, 101, 32, 88, 77, 80, 32, 67, 111, 114, 101, 32, 53, 46, 54,
            45, 99, 49, 52, 50, 32, 55, 57, 46, 49, 54, 48, 57, 50, 52, 44, 32, 50, 48, 49, 55, 47,
            48, 55, 47, 49, 51, 45, 48, 49, 58, 48, 54, 58, 51, 57, 32, 32, 32, 32, 32, 32, 32, 32,
            34, 62, 10, 32, 32, 32, 60, 114, 100, 102, 58, 82, 68, 70, 32, 120, 109, 108, 110, 115,
            58, 114, 100, 102, 61, 34, 104, 116, 116, 112, 58, 47, 47, 119, 119, 119, 46, 119, 51,
            46, 111, 114, 103, 47, 49, 57, 57, 57, 47, 48, 50, 47, 50, 50, 45, 114, 100, 102, 45,
            115, 121, 110, 116, 97, 120, 45, 110, 115, 35, 34, 62, 10, 32, 32, 32, 32, 32, 32, 60,
            114, 100, 102, 58, 68, 101, 115, 99, 114, 105, 112, 116, 105, 111, 110, 32, 114, 100,
            102, 58, 97, 98, 111, 117, 116, 61, 34, 34, 10, 32, 32, 32, 32, 32, 32, 32, 32, 32, 32,
            32, 32, 120, 109, 108, 110, 115, 58, 120, 109, 112, 77, 77, 61, 34, 104, 116, 116, 112,
            58, 47, 47, 110, 115, 46, 97, 100, 111, 98, 101, 46, 99, 111, 109, 47, 120, 97, 112,
            47, 49, 46, 48, 47, 109, 109, 47, 34, 10, 32, 32, 32, 32, 32, 32, 32, 32, 32, 32, 32,
            32, 120, 109, 108, 110, 115, 58, 115, 116, 82, 101, 102, 61, 34, 104, 116, 116, 112,
            58, 47, 47, 110, 115, 46, 97, 100, 111, 98, 101, 46, 99, 111, 109, 47, 120, 97, 112,
            47, 49, 46, 48, 47, 115, 84, 121, 112, 101, 47, 82, 101, 115, 111, 117, 114, 99, 101,
            82, 101, 102, 35, 34, 10, 32, 32, 32, 32, 32, 32, 32, 32, 32, 32, 32, 32, 120, 109,
            108, 110, 115, 58, 115, 116, 69, 118, 116, 61, 34, 104, 116, 116, 112, 58, 47, 47, 110,
            115, 46, 97, 100, 111, 98, 101, 46, 99, 111, 109, 47, 120, 97, 112, 47, 49, 46, 48, 47,
            115, 84, 121, 112, 101, 47, 82, 101, 115, 111, 117, 114, 99, 101, 69, 118, 101, 110,
            116, 35, 34, 10, 32, 32, 32, 32, 32, 32, 32, 32, 32, 32, 32, 32, 120, 109, 108, 110,
            115, 58, 120, 109, 112, 61, 34, 104, 116, 116, 112, 58, 47, 47, 110, 115, 46, 97, 100,
            111, 98, 101, 46, 99, 111, 109, 47, 120, 97, 112, 47, 49, 46, 48, 47, 34, 10, 32, 32,
            32, 32, 32, 32, 32, 32, 32, 32, 32, 32, 120, 109, 108, 110, 115, 58, 100, 99, 61, 34,
            104, 116, 116, 112, 58, 47, 47, 112, 117, 114, 108, 46, 111, 114, 103, 47, 100, 99, 47,
            101, 108, 101, 109, 101, 110, 116, 115, 47, 49, 46, 49, 47, 34, 10, 32, 32, 32, 32, 32,
            32, 32, 32, 32, 32, 32, 32, 120, 109, 108, 110, 115, 58, 112, 104, 111, 116, 111, 115,
            104, 111, 112, 61, 34, 104, 116, 116, 112, 58, 47, 47, 110, 115, 46, 97, 100, 111, 98,
            101, 46, 99, 111, 109, 47, 112, 104, 111, 116, 111, 115, 104, 111, 112, 47, 49, 46, 48,
            47, 34, 62, 10, 32, 32, 32, 32, 32, 32, 32, 32, 32, 60, 120, 109, 112, 77, 77, 58, 79,
            114, 105, 103, 105, 110, 97, 108, 68, 111, 99, 117, 109, 101, 110, 116, 73, 68, 62,
            120, 109, 112, 46, 100, 105, 100, 58, 53, 52, 51, 51, 55, 101, 49, 54, 45, 54, 49, 100,
            54, 45, 52, 51, 101, 51, 45, 56, 55, 102, 102, 45, 48, 99, 48, 51, 101, 56, 101, 98,
            99, 49, 102, 51, 60, 47, 120, 109, 112, 77, 77, 58, 79, 114, 105, 103, 105, 110, 97,
            108, 68, 111, 99, 117, 109, 101, 110, 116, 73, 68, 62, 10, 32, 32, 32, 32, 32, 32, 32,
            32, 32, 60, 120, 109, 112, 77, 77, 58, 68, 111, 99, 117, 109, 101, 110, 116, 73, 68,
            62, 97, 100, 111, 98, 101, 58, 100, 111, 99, 105, 100, 58, 112, 104, 111, 116, 111,
            115, 104, 111, 112, 58, 55, 102, 50, 53, 57, 57, 100, 99, 45, 99, 55, 102, 55, 45, 56,
            49, 52, 55, 45, 56, 49, 51, 102, 45, 102, 98, 99, 56, 52, 52, 100, 52, 101, 48, 102,
            51, 60, 47, 120, 109, 112, 77, 77, 58, 68, 111, 99, 117, 109, 101, 110, 116, 73, 68,
            62, 10, 32, 32, 32, 32, 32, 32, 32, 32, 32, 60, 120, 109, 112, 77, 77, 58, 73, 110,
            115, 116, 97, 110, 99, 101, 73, 68, 62, 120, 109, 112, 46, 105, 105, 100, 58, 98, 99,
            97, 56, 99, 49, 48, 55, 45, 48, 101, 102, 97, 45, 48, 97, 52, 53, 45, 56, 52, 50, 52,
            45, 56, 97, 100, 49, 51, 97, 55, 98, 49, 51, 53, 102, 60, 47, 120, 109, 112, 77, 77,
            58, 73, 110, 115, 116, 97, 110, 99, 101, 73, 68, 62, 10, 32, 32, 32, 32, 32, 32, 32,
            32, 32, 60, 120, 109, 112, 77, 77, 58, 68, 101, 114, 105, 118, 101, 100, 70, 114, 111,
            109, 32, 114, 100, 102, 58, 112, 97, 114, 115, 101, 84, 121, 112, 101, 61, 34, 82, 101,
            115, 111, 117, 114, 99, 101, 34, 62, 10, 32, 32, 32, 32, 32, 32, 32, 32, 32, 32, 32,
            32, 60, 115, 116, 82, 101, 102, 58, 105, 110, 115, 116, 97, 110, 99, 101, 73, 68, 62,
            120, 109, 112, 46, 105, 105, 100, 58, 48, 55, 50, 54, 48, 48, 101, 54, 45, 53, 55, 102,
            48, 45, 51, 52, 52, 101, 45, 98, 101, 57, 48, 45, 57, 102, 57, 53, 52, 97, 54, 48, 57,
            99, 55, 52, 60, 47, 115, 116, 82, 101, 102, 58, 105, 110, 115, 116, 97, 110, 99, 101,
            73, 68, 62, 10, 32, 32, 32, 32, 32, 32, 32, 32, 32, 32, 32, 32, 60, 115, 116, 82, 101,
            102, 58, 100, 111, 99, 117, 109, 101, 110, 116, 0,
        ];
        let first_msg_str = unsafe { from_utf8_unchecked(&first_msg) };

        let second_msg: [u8; 2048] = [
            77, 83, 71, 32, 49, 56, 48, 32, 68, 32, 49, 52, 53, 51, 13, 10, 77, 73, 77, 69, 45, 86,
            101, 114, 115, 105, 111, 110, 58, 32, 49, 46, 48, 13, 10, 67, 111, 110, 116, 101, 110,
            116, 45, 84, 121, 112, 101, 58, 32, 97, 112, 112, 108, 105, 99, 97, 116, 105, 111, 110,
            47, 120, 45, 109, 115, 110, 109, 115, 103, 114, 112, 50, 112, 13, 10, 80, 50, 80, 45,
            68, 101, 115, 116, 58, 32, 97, 101, 111, 110, 116, 101, 115, 116, 51, 64, 115, 104,
            108, 46, 108, 111, 99, 97, 108, 59, 123, 55, 55, 99, 52, 54, 97, 56, 102, 45, 51, 51,
            97, 51, 45, 53, 50, 56, 50, 45, 57, 97, 53, 100, 45, 57, 48, 53, 101, 99, 100, 51, 101,
            98, 48, 54, 57, 125, 13, 10, 80, 50, 80, 45, 83, 114, 99, 58, 32, 97, 101, 111, 110,
            116, 101, 115, 116, 64, 115, 104, 108, 46, 108, 111, 99, 97, 108, 59, 123, 102, 53, 50,
            57, 55, 51, 98, 54, 45, 99, 57, 50, 54, 45, 52, 98, 97, 100, 45, 57, 98, 97, 56, 45,
            55, 99, 49, 101, 56, 52, 48, 101, 52, 97, 98, 48, 125, 13, 10, 13, 10, 8, 0, 4, 218,
            247, 181, 46, 231, 20, 6, 0, 0, 188, 72, 245, 112, 1, 8, 0, 0, 0, 0, 1, 66, 215, 6, 0,
            0, 73, 68, 62, 120, 109, 112, 46, 100, 105, 100, 58, 54, 56, 57, 68, 66, 48, 67, 48,
            68, 56, 50, 68, 49, 49, 69, 55, 57, 50, 48, 68, 69, 70, 54, 48, 66, 67, 51, 49, 70, 65,
            51, 56, 60, 47, 115, 116, 82, 101, 102, 58, 100, 111, 99, 117, 109, 101, 110, 116, 73,
            68, 62, 10, 32, 32, 32, 32, 32, 32, 32, 32, 32, 32, 32, 32, 60, 115, 116, 82, 101, 102,
            58, 111, 114, 105, 103, 105, 110, 97, 108, 68, 111, 99, 117, 109, 101, 110, 116, 73,
            68, 62, 120, 109, 112, 46, 100, 105, 100, 58, 53, 52, 51, 51, 55, 101, 49, 54, 45, 54,
            49, 100, 54, 45, 52, 51, 101, 51, 45, 56, 55, 102, 102, 45, 48, 99, 48, 51, 101, 56,
            101, 98, 99, 49, 102, 51, 60, 47, 115, 116, 82, 101, 102, 58, 111, 114, 105, 103, 105,
            110, 97, 108, 68, 111, 99, 117, 109, 101, 110, 116, 73, 68, 62, 10, 32, 32, 32, 32, 32,
            32, 32, 32, 32, 60, 47, 120, 109, 112, 77, 77, 58, 68, 101, 114, 105, 118, 101, 100,
            70, 114, 111, 109, 62, 10, 32, 32, 32, 32, 32, 32, 32, 32, 32, 60, 120, 109, 112, 77,
            77, 58, 72, 105, 115, 116, 111, 114, 121, 62, 10, 32, 32, 32, 32, 32, 32, 32, 32, 32,
            32, 32, 32, 60, 114, 100, 102, 58, 83, 101, 113, 62, 10, 32, 32, 32, 32, 32, 32, 32,
            32, 32, 32, 32, 32, 32, 32, 32, 60, 114, 100, 102, 58, 108, 105, 32, 114, 100, 102, 58,
            112, 97, 114, 115, 101, 84, 121, 112, 101, 61, 34, 82, 101, 115, 111, 117, 114, 99,
            101, 34, 62, 10, 32, 32, 32, 32, 32, 32, 32, 32, 32, 32, 32, 32, 32, 32, 32, 32, 32,
            32, 60, 115, 116, 69, 118, 116, 58, 97, 99, 116, 105, 111, 110, 62, 115, 97, 118, 101,
            100, 60, 47, 115, 116, 69, 118, 116, 58, 97, 99, 116, 105, 111, 110, 62, 10, 32, 32,
            32, 32, 32, 32, 32, 32, 32, 32, 32, 32, 32, 32, 32, 32, 32, 32, 60, 115, 116, 69, 118,
            116, 58, 105, 110, 115, 116, 97, 110, 99, 101, 73, 68, 62, 120, 109, 112, 46, 105, 105,
            100, 58, 48, 55, 50, 54, 48, 48, 101, 54, 45, 53, 55, 102, 48, 45, 51, 52, 52, 101, 45,
            98, 101, 57, 48, 45, 57, 102, 57, 53, 52, 97, 54, 48, 57, 99, 55, 52, 60, 47, 115, 116,
            69, 118, 116, 58, 105, 110, 115, 116, 97, 110, 99, 101, 73, 68, 62, 10, 32, 32, 32, 32,
            32, 32, 32, 32, 32, 32, 32, 32, 32, 32, 32, 32, 32, 32, 60, 115, 116, 69, 118, 116, 58,
            119, 104, 101, 110, 62, 50, 48, 49, 56, 45, 48, 53, 45, 48, 53, 84, 49, 48, 58, 53, 55,
            58, 49, 57, 43, 48, 50, 58, 48, 48, 60, 47, 115, 116, 69, 118, 116, 58, 119, 104, 101,
            110, 62, 10, 32, 32, 32, 32, 32, 32, 32, 32, 32, 32, 32, 32, 32, 32, 32, 32, 32, 32,
            60, 115, 116, 69, 118, 116, 58, 115, 111, 102, 116, 119, 97, 114, 101, 65, 103, 101,
            110, 116, 62, 65, 100, 111, 98, 101, 32, 80, 104, 111, 116, 111, 115, 104, 111, 112,
            32, 67, 67, 32, 50, 48, 49, 56, 32, 40, 87, 105, 110, 100, 111, 119, 115, 41, 60, 47,
            115, 116, 69, 118, 116, 58, 115, 111, 102, 116, 119, 97, 114, 101, 65, 103, 101, 110,
            116, 62, 10, 32, 32, 32, 32, 32, 32, 32, 32, 32, 32, 32, 32, 32, 32, 32, 32, 32, 32,
            60, 115, 116, 69, 118, 116, 58, 99, 104, 97, 110, 103, 101, 100, 62, 47, 60, 47, 115,
            116, 69, 118, 116, 58, 99, 104, 97, 110, 103, 101, 100, 62, 10, 32, 32, 32, 32, 32, 32,
            32, 32, 32, 32, 32, 32, 32, 32, 32, 60, 47, 114, 100, 102, 58, 108, 105, 62, 10, 32,
            32, 32, 32, 32, 32, 32, 32, 32, 32, 32, 32, 32, 32, 32, 60, 114, 100, 102, 58, 108,
            105, 32, 114, 100, 102, 58, 112, 97, 114, 115, 101, 84, 121, 112, 101, 61, 34, 82, 101,
            115, 111, 117, 114, 99, 101, 34, 62, 10, 32, 32, 32, 32, 32, 32, 32, 32, 32, 32, 32,
            32, 32, 32, 32, 32, 32, 32, 60, 115, 116, 69, 118, 116, 58, 97, 99, 116, 105, 111, 110,
            62, 99, 111, 110, 118, 101, 114, 116, 101, 100, 60, 47, 115, 116, 69, 118, 116, 58, 97,
            99, 116, 105, 111, 110, 62, 10, 32, 32, 32, 32, 32, 32, 32, 32, 32, 32, 32, 32, 32, 32,
            32, 32, 32, 32, 60, 115, 116, 69, 118, 116, 58, 112, 97, 114, 97, 109, 101, 116, 101,
            114, 115, 62, 102, 114, 111, 109, 32, 105, 109, 97, 103, 101, 47, 106, 112, 101, 103,
            32, 116, 111, 32, 97, 112, 112, 108, 105, 99, 97, 116, 105, 111, 110, 47, 118, 110,
            100, 46, 97, 100, 111, 98, 101, 46, 112, 104, 111, 116, 111, 115, 104, 111, 112, 60,
            47, 115, 116, 69, 118, 116, 58, 112, 97, 114, 97, 109, 101, 116, 101, 114, 115, 62, 10,
            32, 32, 32, 32, 32, 32, 32, 32, 32, 32, 32, 32, 32, 32, 32, 60, 47, 114, 100, 102, 58,
            108, 105, 62, 10, 32, 32, 32, 32, 32, 32, 32, 32, 32, 32, 32, 32, 32, 32, 32, 60, 114,
            100, 102, 58, 108, 105, 32, 114, 100, 102, 58, 112, 97, 114, 115, 101, 84, 121, 112,
            101, 61, 34, 82, 101, 115, 111, 117, 114, 99, 101, 34, 62, 10, 32, 32, 32, 32, 32, 32,
            32, 32, 32, 32, 32, 32, 32, 32, 32, 32, 32, 32, 60, 115, 116, 69, 118, 116, 58, 97, 99,
            116, 105, 111, 110, 62, 100, 101, 114, 105, 118, 101, 100, 60, 47, 115, 116, 69, 118,
            116, 58, 97, 99, 116, 105, 111, 110, 62, 10, 32, 32, 32, 32, 32, 32, 32, 32, 32, 32,
            32, 32, 32, 32, 32, 32, 32, 32, 60, 115, 116, 69, 118, 116, 58, 112, 97, 114, 97, 109,
            101, 116, 101, 114, 115, 62, 99, 111, 110, 118, 101, 114, 116, 101, 100, 32, 102, 114,
            111, 109, 32, 105, 109, 97, 103, 101, 47, 106, 112, 101, 103, 32, 116, 111, 32, 97,
            112, 112, 108, 105, 99, 97, 116, 105, 111, 110, 47, 118, 110, 100, 46, 97, 100, 111,
            98, 101, 46, 112, 104, 111, 116, 111, 115, 104, 111, 112, 60, 47, 115, 116, 69, 118,
            116, 58, 112, 97, 114, 97, 109, 101, 116, 101, 114, 115, 62, 10, 32, 32, 32, 32, 32,
            32, 32, 32, 32, 32, 32, 32, 32, 32, 32, 60, 47, 114, 100, 102, 58, 108, 105, 62, 10,
            32, 32, 32, 32, 32, 32, 32, 32, 32, 32, 32, 32, 32, 32, 32, 60, 114, 100, 102, 58, 108,
            105, 32, 114, 100, 102, 58, 112, 97, 114, 115, 101, 84, 121, 112, 101, 61, 34, 82, 101,
            115, 111, 117, 114, 99, 101, 34, 62, 0, 0, 0, 0, 77, 83, 71, 32, 49, 56, 49, 32, 68,
            32, 49, 52, 53, 51, 13, 10, 77, 73, 77, 69, 45, 86, 101, 114, 115, 105, 111, 110, 58,
            32, 49, 46, 48, 13, 10, 67, 111, 110, 116, 101, 110, 116, 45, 84, 121, 112, 101, 58,
            32, 97, 112, 112, 108, 105, 99, 97, 116, 105, 111, 110, 47, 120, 45, 109, 115, 110,
            109, 115, 103, 114, 112, 50, 112, 13, 10, 80, 50, 80, 45, 68, 101, 115, 116, 58, 32,
            97, 101, 111, 110, 116, 101, 115, 116, 51, 64, 115, 104, 108, 46, 108, 111, 99, 97,
            108, 59, 123, 55, 55, 99, 52, 54, 97, 56, 102, 45, 51, 51, 97, 51, 45, 53, 50, 56, 50,
            45, 57, 97, 53, 100, 45, 57, 48, 53, 101, 99, 100, 51, 101, 98, 48, 54, 57, 125, 13,
            10, 80, 50, 80, 45, 83, 114, 99, 58, 32, 97, 101, 111, 110, 116, 101, 115, 116, 64,
            115, 104, 108, 46, 108, 111, 99, 97, 108, 59, 123, 102, 53, 50, 57, 55, 51, 98, 54, 45,
            99, 57, 50, 54, 45, 52, 98, 97, 100, 45, 57, 98, 97, 56, 45, 55, 99, 49, 101, 56, 52,
            48, 101, 52, 97, 98, 48, 125, 13, 10, 13, 10, 8, 0, 4, 218, 247, 181, 51, 193, 20, 6,
            0, 0, 188, 72, 245, 112, 1, 8, 0, 0, 0, 0, 1, 66, 210, 64, 0, 0, 10, 32, 32, 32, 32,
            32, 32, 32, 32, 32, 32, 32, 32, 32, 32, 32, 32, 32, 32, 60, 115, 116, 69, 118, 116, 58,
            97, 99, 116, 105, 111, 110, 62, 115, 97, 118, 101, 100, 60, 47, 115, 116, 69, 118, 116,
            58, 97, 99, 116, 105, 111, 110, 62, 10, 32, 32, 32, 32, 32, 32, 32, 32, 32, 32, 32, 32,
            32, 32, 32, 32, 32, 32, 60, 115, 116, 69, 118, 116, 58, 105, 110, 115, 116, 97, 110,
            99, 101, 73, 68, 62, 120, 109, 112, 46, 105, 105, 100, 58, 98, 99, 97, 56, 99, 49, 48,
            55, 45, 48, 101, 102, 97, 45, 48, 97, 52, 53, 45, 56, 52, 50, 52, 45, 56, 97, 100, 49,
            51, 97, 55, 98, 49, 51, 53, 102, 60, 47, 115, 116, 69, 118, 116, 58, 105, 110, 115,
            116, 97, 110, 99, 101, 73, 68, 62, 10, 32, 32, 32, 32, 32, 32, 32, 32, 32, 32, 32, 32,
            32, 32, 32, 32, 32, 32, 60, 115, 116, 69, 118, 116, 58, 119, 104, 101, 110, 62, 50, 48,
            49, 56, 45, 48, 53, 45, 48, 53, 84, 49, 48, 58, 53, 55, 58, 49, 57, 43, 48, 50, 58, 48,
            48, 60, 47, 115, 116, 69, 118, 116, 58, 119, 104, 101, 110, 62, 10, 32, 32, 32, 32, 32,
            32, 32, 32, 32, 32, 32, 32, 32, 32, 32, 32, 32, 32, 60, 115, 116, 69, 118, 116, 58,
            115, 111, 102, 116, 119, 97, 114, 101, 65, 103, 101, 110, 116, 62, 65, 100, 111, 98,
            101, 32, 80, 104, 111, 116, 111, 115, 104, 111, 112, 32, 67, 67, 32, 50, 48, 49, 56,
            32, 40, 87, 105, 110, 100, 111, 119, 115, 41, 60, 47, 115, 116, 69, 118, 116, 58, 115,
            111, 102, 116, 119, 97, 114, 101, 65, 103, 101, 110, 116, 62, 10, 32, 32, 32, 32, 32,
            32, 32, 32, 32, 32, 32, 32, 32, 32, 32, 32, 32, 32,
        ];
        let second_msg_str = unsafe { from_utf8_unchecked(&second_msg) };

        let third_msg: [u8; 2048] = [
            60, 115, 116, 69, 118, 116, 58, 99, 104, 97, 110, 103, 101, 100, 62, 47, 60, 47, 115,
            116, 69, 118, 116, 58, 99, 104, 97, 110, 103, 101, 100, 62, 10, 32, 32, 32, 32, 32, 32,
            32, 32, 32, 32, 32, 32, 32, 32, 32, 60, 47, 114, 100, 102, 58, 108, 105, 62, 10, 32,
            32, 32, 32, 32, 32, 32, 32, 32, 32, 32, 32, 60, 47, 114, 100, 102, 58, 83, 101, 113,
            62, 10, 32, 32, 32, 32, 32, 32, 32, 32, 32, 60, 47, 120, 109, 112, 77, 77, 58, 72, 105,
            115, 116, 111, 114, 121, 62, 10, 32, 32, 32, 32, 32, 32, 32, 32, 32, 60, 120, 109, 112,
            58, 67, 114, 101, 97, 116, 111, 114, 84, 111, 111, 108, 62, 65, 100, 111, 98, 101, 32,
            80, 104, 111, 116, 111, 115, 104, 111, 112, 32, 67, 67, 32, 40, 77, 97, 99, 105, 110,
            116, 111, 115, 104, 41, 60, 47, 120, 109, 112, 58, 67, 114, 101, 97, 116, 111, 114, 84,
            111, 111, 108, 62, 10, 32, 32, 32, 32, 32, 32, 32, 32, 32, 60, 120, 109, 112, 58, 67,
            114, 101, 97, 116, 101, 68, 97, 116, 101, 62, 50, 48, 49, 56, 45, 48, 53, 45, 48, 52,
            84, 49, 57, 58, 51, 48, 58, 49, 57, 43, 48, 50, 58, 48, 48, 60, 47, 120, 109, 112, 58,
            67, 114, 101, 97, 116, 101, 68, 97, 116, 101, 62, 10, 32, 32, 32, 32, 32, 32, 32, 32,
            32, 60, 120, 109, 112, 58, 77, 111, 100, 105, 102, 121, 68, 97, 116, 101, 62, 50, 48,
            49, 56, 45, 48, 53, 45, 48, 53, 84, 49, 48, 58, 53, 55, 58, 49, 57, 43, 48, 50, 58, 48,
            48, 60, 47, 120, 109, 112, 58, 77, 111, 100, 105, 102, 121, 68, 97, 116, 101, 62, 10,
            32, 32, 32, 32, 32, 32, 32, 32, 32, 60, 120, 109, 112, 58, 77, 101, 116, 97, 100, 97,
            116, 97, 68, 97, 116, 101, 62, 50, 48, 49, 56, 45, 48, 53, 45, 48, 53, 84, 49, 48, 58,
            53, 55, 58, 49, 57, 43, 48, 50, 58, 48, 48, 60, 47, 120, 109, 112, 58, 77, 101, 116,
            97, 100, 97, 116, 97, 68, 97, 116, 101, 62, 10, 32, 32, 32, 32, 32, 32, 32, 32, 32, 60,
            100, 99, 58, 102, 111, 114, 109, 97, 116, 62, 97, 112, 112, 108, 105, 99, 97, 116, 105,
            111, 110, 47, 118, 110, 100, 46, 97, 100, 111, 98, 101, 46, 112, 104, 111, 116, 111,
            115, 104, 111, 112, 60, 47, 100, 99, 58, 102, 111, 114, 109, 97, 116, 62, 10, 32, 32,
            32, 32, 32, 32, 32, 32, 32, 60, 112, 104, 111, 116, 111, 115, 104, 111, 112, 58, 67,
            111, 108, 111, 114, 77, 111, 100, 101, 62, 51, 60, 47, 112, 104, 111, 116, 111, 115,
            104, 111, 112, 58, 67, 111, 108, 111, 114, 77, 111, 100, 101, 62, 10, 32, 32, 32, 32,
            32, 32, 32, 32, 32, 60, 112, 104, 111, 116, 111, 115, 104, 111, 112, 58, 73, 67, 67,
            80, 114, 111, 102, 105, 108, 101, 47, 62, 10, 32, 32, 32, 32, 32, 32, 32, 32, 32, 60,
            112, 104, 111, 116, 111, 115, 104, 111, 112, 58, 68, 111, 99, 117, 109, 101, 110, 116,
            65, 110, 99, 101, 115, 116, 111, 114, 115, 62, 10, 32, 32, 32, 32, 32, 32, 32, 32, 32,
            32, 32, 32, 60, 114, 100, 102, 58, 66, 97, 103, 62, 10, 32, 32, 32, 32, 32, 32, 32, 32,
            32, 32, 32, 32, 32, 32, 32, 60, 114, 100, 102, 58, 108, 105, 62, 120, 109, 112, 46,
            100, 105, 100, 58, 52, 97, 50, 48, 57, 55, 99, 52, 45, 54, 57, 99, 49, 45, 99, 101, 52,
            53, 45, 97, 98, 50, 50, 45, 102, 98, 56, 51, 52, 99, 55, 53, 49, 52, 99, 50, 60, 47,
            114, 100, 102, 58, 108, 105, 62, 10, 32, 32, 32, 32, 32, 32, 32, 32, 32, 32, 32, 32,
            32, 32, 32, 60, 114, 100, 102, 58, 108, 105, 62, 120, 109, 112, 46, 100, 105, 100, 58,
            54, 49, 97, 52, 49, 56, 102, 99, 45, 55, 54, 97, 55, 45, 53, 101, 52, 52, 45, 98, 51,
            51, 48, 45, 50, 101, 56, 54, 53, 97, 54, 54, 56, 101, 102, 52, 60, 47, 114, 100, 102,
            58, 108, 105, 62, 10, 32, 32, 32, 32, 32, 32, 32, 32, 32, 32, 32, 32, 60, 47, 114, 100,
            102, 58, 66, 97, 103, 62, 10, 32, 32, 32, 32, 32, 32, 32, 32, 32, 60, 47, 112, 104,
            111, 116, 111, 115, 104, 111, 112, 58, 68, 111, 99, 117, 109, 101, 110, 116, 65, 110,
            99, 101, 115, 116, 111, 114, 115, 62, 10, 32, 32, 32, 32, 32, 32, 60, 47, 114, 100,
            102, 58, 68, 101, 115, 99, 114, 105, 112, 116, 105, 111, 110, 62, 10, 32, 32, 32, 60,
            47, 114, 100, 102, 58, 82, 68, 70, 62, 10, 60, 47, 120, 58, 120, 109, 112, 109, 101,
            116, 97, 62, 10, 32, 32, 32, 32, 32, 32, 32, 32, 32, 32, 32, 32, 32, 32, 32, 0, 0, 0,
            0, 77, 83, 71, 32, 49, 56, 50, 32, 68, 32, 49, 52, 53, 51, 13, 10, 77, 73, 77, 69, 45,
            86, 101, 114, 115, 105, 111, 110, 58, 32, 49, 46, 48, 13, 10, 67, 111, 110, 116, 101,
            110, 116, 45, 84, 121, 112, 101, 58, 32, 97, 112, 112, 108, 105, 99, 97, 116, 105, 111,
            110, 47, 120, 45, 109, 115, 110, 109, 115, 103, 114, 112, 50, 112, 13, 10, 80, 50, 80,
            45, 68, 101, 115, 116, 58, 32, 97, 101, 111, 110, 116, 101, 115, 116, 51, 64, 115, 104,
            108, 46, 108, 111, 99, 97, 108, 59, 123, 55, 55, 99, 52, 54, 97, 56, 102, 45, 51, 51,
            97, 51, 45, 53, 50, 56, 50, 45, 57, 97, 53, 100, 45, 57, 48, 53, 101, 99, 100, 51, 101,
            98, 48, 54, 57, 125, 13, 10, 80, 50, 80, 45, 83, 114, 99, 58, 32, 97, 101, 111, 110,
            116, 101, 115, 116, 64, 115, 104, 108, 46, 108, 111, 99, 97, 108, 59, 123, 102, 53, 50,
            57, 55, 51, 98, 54, 45, 99, 57, 50, 54, 45, 52, 98, 97, 100, 45, 57, 98, 97, 56, 45,
            55, 99, 49, 101, 56, 52, 48, 101, 52, 97, 98, 48, 125, 13, 10, 13, 10, 8, 0, 4, 218,
            247, 181, 56, 155, 20, 6, 0, 0, 188, 72, 245, 112, 1, 8, 0, 0, 0, 0, 1, 66, 205, 122,
            0, 0, 32, 32, 32, 32, 32, 32, 32, 32, 32, 32, 32, 32, 32, 32, 32, 32, 32, 32, 32, 32,
            32, 32, 32, 32, 32, 32, 32, 32, 32, 32, 32, 32, 32, 32, 32, 32, 32, 32, 32, 32, 32, 32,
            32, 32, 32, 32, 32, 32, 32, 32, 32, 32, 32, 32, 32, 32, 32, 32, 32, 32, 32, 32, 32, 32,
            32, 32, 32, 32, 32, 32, 32, 32, 32, 32, 32, 32, 32, 32, 32, 32, 32, 32, 32, 32, 32, 10,
            32, 32, 32, 32, 32, 32, 32, 32, 32, 32, 32, 32, 32, 32, 32, 32, 32, 32, 32, 32, 32, 32,
            32, 32, 32, 32, 32, 32, 32, 32, 32, 32, 32, 32, 32, 32, 32, 32, 32, 32, 32, 32, 32, 32,
            32, 32, 32, 32, 32, 32, 32, 32, 32, 32, 32, 32, 32, 32, 32, 32, 32, 32, 32, 32, 32, 32,
            32, 32, 32, 32, 32, 32, 32, 32, 32, 32, 32, 32, 32, 32, 32, 32, 32, 32, 32, 32, 32, 32,
            32, 32, 32, 32, 32, 32, 32, 32, 32, 32, 32, 32, 10, 32, 32, 32, 32, 32, 32, 32, 32, 32,
            32, 32, 32, 32, 32, 32, 32, 32, 32, 32, 32, 32, 32, 32, 32, 32, 32, 32, 32, 32, 32, 32,
            32, 32, 32, 32, 32, 32, 32, 32, 32, 32, 32, 32, 32, 32, 32, 32, 32, 32, 32, 32, 32, 32,
            32, 32, 32, 32, 32, 32, 32, 32, 32, 32, 32, 32, 32, 32, 32, 32, 32, 32, 32, 32, 32, 32,
            32, 32, 32, 32, 32, 32, 32, 32, 32, 32, 32, 32, 32, 32, 32, 32, 32, 32, 32, 32, 32, 32,
            32, 32, 32, 10, 32, 32, 32, 32, 32, 32, 32, 32, 32, 32, 32, 32, 32, 32, 32, 32, 32, 32,
            32, 32, 32, 32, 32, 32, 32, 32, 32, 32, 32, 32, 32, 32, 32, 32, 32, 32, 32, 32, 32, 32,
            32, 32, 32, 32, 32, 32, 32, 32, 32, 32, 32, 32, 32, 32, 32, 32, 32, 32, 32, 32, 32, 32,
            32, 32, 32, 32, 32, 32, 32, 32, 32, 32, 32, 32, 32, 32, 32, 32, 32, 32, 32, 32, 32, 32,
            32, 32, 32, 32, 32, 32, 32, 32, 32, 32, 32, 32, 32, 32, 32, 32, 10, 32, 32, 32, 32, 32,
            32, 32, 32, 32, 32, 32, 32, 32, 32, 32, 32, 32, 32, 32, 32, 32, 32, 32, 32, 32, 32, 32,
            32, 32, 32, 32, 32, 32, 32, 32, 32, 32, 32, 32, 32, 32, 32, 32, 32, 32, 32, 32, 32, 32,
            32, 32, 32, 32, 32, 32, 32, 32, 32, 32, 32, 32, 32, 32, 32, 32, 32, 32, 32, 32, 32, 32,
            32, 32, 32, 32, 32, 32, 32, 32, 32, 32, 32, 32, 32, 32, 32, 32, 32, 32, 32, 32, 32, 32,
            32, 32, 32, 32, 32, 32, 32, 10, 32, 32, 32, 32, 32, 32, 32, 32, 32, 32, 32, 32, 32, 32,
            32, 32, 32, 32, 32, 32, 32, 32, 32, 32, 32, 32, 32, 32, 32, 32, 32, 32, 32, 32, 32, 32,
            32, 32, 32, 32, 32, 32, 32, 32, 32, 32, 32, 32, 32, 32, 32, 32, 32, 32, 32, 32, 32, 32,
            32, 32, 32, 32, 32, 32, 32, 32, 32, 32, 32, 32, 32, 32, 32, 32, 32, 32, 32, 32, 32, 32,
            32, 32, 32, 32, 32, 32, 32, 32, 32, 32, 32, 32, 32, 32, 32, 32, 32, 32, 32, 32, 10, 32,
            32, 32, 32, 32, 32, 32, 32, 32, 32, 32, 32, 32, 32, 32, 32, 32, 32, 32, 32, 32, 32, 32,
            32, 32, 32, 32, 32, 32, 32, 32, 32, 32, 32, 32, 32, 32, 32, 32, 32, 32, 32, 32, 32, 32,
            32, 32, 32, 32, 32, 32, 32, 32, 32, 32, 32, 32, 32, 32, 32, 32, 32, 32, 32, 32, 32, 32,
            32, 32, 32, 32, 32, 32, 32, 32, 32, 32, 32, 32, 32, 32, 32, 32, 32, 32, 32, 32, 32, 32,
            32, 32, 32, 32, 32, 32, 32, 32, 32, 32, 32, 10, 32, 32, 32, 32, 32, 32, 32, 32, 32, 32,
            32, 32, 32, 32, 32, 32, 32, 32, 32, 32, 32, 32, 32, 32, 32, 32, 32, 32, 32, 32, 32, 32,
            32, 32, 32, 32, 32, 32, 32, 32, 32, 32, 32, 32, 32, 32, 32, 32, 32, 32, 32, 32, 32, 32,
            32, 32, 32, 32, 32, 32, 32, 32, 32, 32, 32, 32, 32, 32, 32, 32, 32, 32, 32, 32, 32, 32,
            32, 32, 32, 32, 32, 32, 32, 32, 32, 32, 32, 32, 32, 32, 32, 32, 32, 32, 32, 32, 32, 32,
            32, 32, 10, 32, 32, 32, 32, 32, 32, 32, 32, 32, 32, 32, 32, 32, 32, 32, 32, 32, 32, 32,
            32, 32, 32, 32, 32, 32, 32, 32, 32, 32, 32, 32, 32, 32, 32, 32, 32, 32, 32, 32, 32, 32,
            32, 32, 32, 32, 32, 32, 32, 32, 32, 32, 32, 32, 32, 32, 32, 32, 32, 32, 32, 32, 32, 32,
            32, 32, 32, 32, 32, 32, 32, 32, 32, 32, 32, 32, 32, 32, 32, 32, 32, 32, 32, 32, 32, 32,
            32, 32, 32, 32, 32, 32, 32, 32, 32, 32, 32, 32, 32, 32, 32, 10, 32, 32, 32, 32, 32, 32,
            32, 32, 32, 32, 32, 32, 32, 32, 32, 32, 32, 32, 32, 32, 32,
        ];
        let third_msg_str = unsafe { from_utf8_unchecked(&third_msg) };

        let fourth_msg: [u8; 1777] = [
            32, 32, 32, 32, 32, 32, 32, 32, 32, 32, 32, 32, 32, 32, 32, 32, 32, 32, 32, 32, 32, 32,
            32, 32, 32, 32, 32, 32, 32, 32, 32, 32, 32, 32, 32, 32, 32, 32, 32, 32, 32, 32, 32, 32,
            32, 32, 32, 32, 32, 32, 32, 32, 32, 32, 32, 32, 32, 32, 32, 32, 32, 32, 32, 32, 32, 32,
            32, 32, 32, 32, 32, 32, 32, 32, 32, 32, 32, 32, 32, 10, 32, 32, 32, 32, 32, 32, 32, 32,
            32, 32, 32, 32, 32, 32, 32, 32, 32, 32, 32, 32, 32, 32, 32, 32, 32, 32, 32, 32, 32, 32,
            32, 32, 32, 32, 32, 32, 32, 32, 32, 32, 32, 32, 32, 32, 32, 32, 32, 32, 32, 32, 32, 32,
            32, 32, 32, 32, 32, 32, 32, 32, 32, 32, 32, 32, 32, 32, 32, 32, 32, 32, 32, 32, 32, 32,
            32, 32, 32, 32, 32, 32, 32, 32, 32, 32, 32, 32, 32, 32, 32, 32, 32, 32, 32, 32, 32, 32,
            32, 32, 32, 32, 10, 32, 32, 32, 32, 32, 32, 32, 32, 32, 32, 32, 32, 32, 32, 32, 32, 32,
            32, 32, 32, 32, 32, 32, 32, 32, 32, 32, 32, 32, 32, 32, 32, 32, 32, 32, 32, 32, 32, 32,
            32, 32, 32, 32, 32, 32, 32, 32, 32, 32, 32, 32, 32, 32, 32, 32, 32, 32, 32, 32, 32, 32,
            32, 32, 32, 32, 32, 32, 32, 32, 32, 32, 32, 32, 32, 32, 32, 32, 32, 32, 32, 32, 32, 32,
            32, 32, 32, 32, 32, 32, 32, 32, 32, 32, 32, 32, 32, 32, 32, 32, 32, 10, 32, 32, 32, 32,
            32, 32, 32, 32, 32, 32, 32, 32, 32, 32, 32, 32, 32, 32, 32, 32, 32, 32, 32, 32, 32, 0,
            0, 0, 0, 77, 83, 71, 32, 49, 56, 51, 32, 68, 32, 49, 52, 53, 51, 13, 10, 77, 73, 77,
            69, 45, 86, 101, 114, 115, 105, 111, 110, 58, 32, 49, 46, 48, 13, 10, 67, 111, 110,
            116, 101, 110, 116, 45, 84, 121, 112, 101, 58, 32, 97, 112, 112, 108, 105, 99, 97, 116,
            105, 111, 110, 47, 120, 45, 109, 115, 110, 109, 115, 103, 114, 112, 50, 112, 13, 10,
            80, 50, 80, 45, 68, 101, 115, 116, 58, 32, 97, 101, 111, 110, 116, 101, 115, 116, 51,
            64, 115, 104, 108, 46, 108, 111, 99, 97, 108, 59, 123, 55, 55, 99, 52, 54, 97, 56, 102,
            45, 51, 51, 97, 51, 45, 53, 50, 56, 50, 45, 57, 97, 53, 100, 45, 57, 48, 53, 101, 99,
            100, 51, 101, 98, 48, 54, 57, 125, 13, 10, 80, 50, 80, 45, 83, 114, 99, 58, 32, 97,
            101, 111, 110, 116, 101, 115, 116, 64, 115, 104, 108, 46, 108, 111, 99, 97, 108, 59,
            123, 102, 53, 50, 57, 55, 51, 98, 54, 45, 99, 57, 50, 54, 45, 52, 98, 97, 100, 45, 57,
            98, 97, 56, 45, 55, 99, 49, 101, 56, 52, 48, 101, 52, 97, 98, 48, 125, 13, 10, 13, 10,
            8, 0, 4, 218, 247, 181, 61, 117, 20, 6, 0, 0, 188, 72, 245, 112, 1, 8, 0, 0, 0, 0, 1,
            66, 200, 180, 0, 0, 32, 32, 32, 32, 32, 32, 32, 32, 32, 32, 32, 32, 32, 32, 32, 32, 32,
            32, 32, 32, 32, 32, 32, 32, 32, 32, 32, 32, 32, 32, 32, 32, 32, 32, 32, 32, 32, 32, 32,
            32, 32, 32, 32, 32, 32, 32, 32, 32, 32, 32, 32, 32, 32, 32, 32, 32, 32, 32, 32, 32, 32,
            32, 32, 32, 32, 32, 32, 32, 32, 32, 32, 32, 32, 32, 32, 10, 32, 32, 32, 32, 32, 32, 32,
            32, 32, 32, 32, 32, 32, 32, 32, 32, 32, 32, 32, 32, 32, 32, 32, 32, 32, 32, 32, 32, 32,
            32, 32, 32, 32, 32, 32, 32, 32, 32, 32, 32, 32, 32, 32, 32, 32, 32, 32, 32, 32, 32, 32,
            32, 32, 32, 32, 32, 32, 32, 32, 32, 32, 32, 32, 32, 32, 32, 32, 32, 32, 32, 32, 32, 32,
            32, 32, 32, 32, 32, 32, 32, 32, 32, 32, 32, 32, 32, 32, 32, 32, 32, 32, 32, 32, 32, 32,
            32, 32, 32, 32, 32, 10, 32, 32, 32, 32, 32, 32, 32, 32, 32, 32, 32, 32, 32, 32, 32, 32,
            32, 32, 32, 32, 32, 32, 32, 32, 32, 32, 32, 32, 32, 32, 32, 32, 32, 32, 32, 32, 32, 32,
            32, 32, 32, 32, 32, 32, 32, 32, 32, 32, 32, 32, 32, 32, 32, 32, 32, 32, 32, 32, 32, 32,
            32, 32, 32, 32, 32, 32, 32, 32, 32, 32, 32, 32, 32, 32, 32, 32, 32, 32, 32, 32, 32, 32,
            32, 32, 32, 32, 32, 32, 32, 32, 32, 32, 32, 32, 32, 32, 32, 32, 32, 32, 10, 32, 32, 32,
            32, 32, 32, 32, 32, 32, 32, 32, 32, 32, 32, 32, 32, 32, 32, 32, 32, 32, 32, 32, 32, 32,
            32, 32, 32, 32, 32, 32, 32, 32, 32, 32, 32, 32, 32, 32, 32, 32, 32, 32, 32, 32, 32, 32,
            32, 32, 32, 32, 32, 32, 32, 32, 32, 32, 32, 32, 32, 32, 32, 32, 32, 32, 32, 32, 32, 32,
            32, 32, 32, 32, 32, 32, 32, 32, 32, 32, 32, 32, 32, 32, 32, 32, 32, 32, 32, 32, 32, 32,
            32, 32, 32, 32, 32, 32, 32, 32, 32, 10, 32, 32, 32, 32, 32, 32, 32, 32, 32, 32, 32, 32,
            32, 32, 32, 32, 32, 32, 32, 32, 32, 32, 32, 32, 32, 32, 32, 32, 32, 32, 32, 32, 32, 32,
            32, 32, 32, 32, 32, 32, 32, 32, 32, 32, 32, 32, 32, 32, 32, 32, 32, 32, 32, 32, 32, 32,
            32, 32, 32, 32, 32, 32, 32, 32, 32, 32, 32, 32, 32, 32, 32, 32, 32, 32, 32, 32, 32, 32,
            32, 32, 32, 32, 32, 32, 32, 32, 32, 32, 32, 32, 32, 32, 32, 32, 32, 32, 32, 32, 32, 32,
            10, 32, 32, 32, 32, 32, 32, 32, 32, 32, 32, 32, 32, 32, 32, 32, 32, 32, 32, 32, 32, 32,
            32, 32, 32, 32, 32, 32, 32, 32, 32, 32, 32, 32, 32, 32, 32, 32, 32, 32, 32, 32, 32, 32,
            32, 32, 32, 32, 32, 32, 32, 32, 32, 32, 32, 32, 32, 32, 32, 32, 32, 32, 32, 32, 32, 32,
            32, 32, 32, 32, 32, 32, 32, 32, 32, 32, 32, 32, 32, 32, 32, 32, 32, 32, 32, 32, 32, 32,
            32, 32, 32, 32, 32, 32, 32, 32, 32, 32, 32, 32, 32, 10, 32, 32, 32, 32, 32, 32, 32, 32,
            32, 32, 32, 32, 32, 32, 32, 32, 32, 32, 32, 32, 32, 32, 32, 32, 32, 32, 32, 32, 32, 32,
            32, 32, 32, 32, 32, 32, 32, 32, 32, 32, 32, 32, 32, 32, 32, 32, 32, 32, 32, 32, 32, 32,
            32, 32, 32, 32, 32, 32, 32, 32, 32, 32, 32, 32, 32, 32, 32, 32, 32, 32, 32, 32, 32, 32,
            32, 32, 32, 32, 32, 32, 32, 32, 32, 32, 32, 32, 32, 32, 32, 32, 32, 32, 32, 32, 32, 32,
            32, 32, 32, 32, 10, 32, 32, 32, 32, 32, 32, 32, 32, 32, 32, 32, 32, 32, 32, 32, 32, 32,
            32, 32, 32, 32, 32, 32, 32, 32, 32, 32, 32, 32, 32, 32, 32, 32, 32, 32, 32, 32, 32, 32,
            32, 32, 32, 32, 32, 32, 32, 32, 32, 32, 32, 32, 32, 32, 32, 32, 32, 32, 32, 32, 32, 32,
            32, 32, 32, 32, 32, 32, 32, 32, 32, 32, 32, 32, 32, 32, 32, 32, 32, 32, 32, 32, 32, 32,
            32, 32, 32, 32, 32, 32, 32, 32, 32, 32, 32, 32, 32, 32, 32, 32, 32, 10, 32, 32, 32, 32,
            32, 32, 32, 32, 32, 32, 32, 32, 32, 32, 32, 32, 32, 32, 32, 32, 32, 32, 32, 32, 32, 32,
            32, 32, 32, 32, 32, 32, 32, 32, 32, 32, 32, 32, 32, 32, 32, 32, 32, 32, 32, 32, 32, 32,
            32, 32, 32, 32, 32, 32, 32, 32, 32, 32, 32, 32, 32, 32, 32, 32, 32, 32, 32, 32, 32, 32,
            32, 32, 32, 32, 32, 32, 32, 32, 32, 32, 32, 32, 32, 32, 32, 32, 32, 32, 32, 32, 32, 32,
            32, 32, 32, 32, 32, 32, 32, 32, 10, 32, 32, 32, 32, 32, 32, 32, 32, 32, 32, 32, 32, 32,
            32, 32, 32, 32, 32, 32, 32, 32, 32, 32, 32, 32, 32, 32, 32, 32, 32, 32, 32, 32, 32, 32,
            32, 32, 32, 32, 32, 32, 32, 32, 32, 32, 32, 32, 32, 32, 32, 32, 32, 32, 32, 32, 32, 32,
            32, 32, 32, 32, 32, 32, 32, 32, 32, 32, 32, 32, 32, 32, 32, 32, 32, 32, 32, 32, 32, 32,
            32, 32, 32, 32, 32, 32, 32, 32, 32, 32, 32, 32, 32, 32, 32, 32, 32, 32, 32, 32, 32, 10,
            32, 32, 32, 32, 32, 32, 32, 32, 32, 32, 32, 32, 32, 32, 32, 32, 32, 32, 32, 32, 32, 32,
            32, 32, 32, 32, 32, 32, 32, 32, 32, 32, 32, 32, 32, 32, 32, 32, 32, 32, 32, 32, 32, 32,
            32, 32, 32, 32, 32, 32, 32, 32, 32, 32, 32, 32, 32, 32, 32, 32, 32, 32, 32, 32, 32, 32,
            32, 32, 32, 32, 32, 32, 32, 32, 32, 32, 32, 32, 32, 32, 32, 32, 32, 32, 32, 32, 32, 32,
            32, 32, 32, 32, 32, 32, 32, 32, 32, 32, 32, 32, 10, 32, 32, 32, 32, 32, 32, 32, 32, 32,
            32, 32, 32, 32, 32, 32, 32, 32, 32, 32, 32, 32, 32, 32, 32, 32, 32, 32, 32, 32, 32, 32,
            32, 32, 32, 32, 32, 32, 32, 32, 32, 32, 32, 32, 32, 32, 32, 32, 32, 32, 32, 32, 32, 32,
            32, 32, 32, 32, 32, 32, 32, 32, 32, 32, 32, 32, 32, 32, 32, 32, 32, 32, 32, 32, 32, 32,
            32, 32, 32, 32, 32, 32, 32, 32, 32, 32, 32, 32, 32, 32, 32, 32, 32, 32, 32, 32, 32, 32,
            32, 32, 32, 10, 32, 32, 32, 32, 32, 32, 32, 32, 32, 32, 32, 32, 32, 32, 32, 32, 32, 32,
            32, 32, 32, 32, 32, 32, 32, 32, 32, 32, 32, 32, 32, 32, 32, 32, 32, 0,
        ];
        let fourth_msg_str = unsafe { from_utf8_unchecked(&fourth_msg) };

        let mut test1 = parser.parse_message(first_msg_str);

        let mut test2 = parser.parse_message(second_msg_str);

        let mut test3 = parser.parse_message(third_msg_str);

        let mut test4 = parser.parse_message(fourth_msg_str);

        test1.append(&mut test2);
        test1.append(&mut test3);
        test1.append(&mut test4);

        let (sb_sender, mut sb_receiver) = broadcast::channel::<SocketEvent>(10);

        let mut handler = SwitchboardCommandHandler::new(sb_sender);

        for command in test1 {
            handler.handle_command(&command).await;
        }

        let test = sb_receiver.recv().await.unwrap();

        let bp = 0;
    }

    #[actix_rt::test]
    async fn test_test() {

        // env_logger::init_from_env(env_logger::Env::new().default_filter_or("debug"));

        // let first_invite_msg_payload = [73, 78, 86, 73, 84, 69, 32, 77, 83, 78, 77, 83, 71, 82, 58, 97, 101, 111, 110, 116, 101, 115, 116, 49, 64, 115, 104, 108, 97, 115, 111, 117, 102, 46, 108, 111, 99, 97, 108, 59, 123, 102, 48, 54, 50, 56, 50, 53, 100, 45, 50, 54, 100, 57, 45, 53, 50, 97, 98, 45, 57, 52, 52, 55, 45, 98, 52, 98, 48, 97, 51, 49, 53, 100, 98, 101, 53, 125, 32, 77, 83, 78, 83, 76, 80, 47, 49, 46, 48, 13, 10, 84, 111, 58, 32, 60, 109, 115, 110, 109, 115, 103, 114, 58, 97, 101, 111, 110, 116, 101, 115, 116, 49, 64, 115, 104, 108, 97, 115, 111, 117, 102, 46, 108, 111, 99, 97, 108, 59, 123, 102, 48, 54, 50, 56, 50, 53, 100, 45, 50, 54, 100, 57, 45, 53, 50, 97, 98, 45, 57, 52, 52, 55, 45, 98, 52, 98, 48, 97, 51, 49, 53, 100, 98, 101, 53, 125, 62, 13, 10, 70, 114, 111, 109, 58, 32, 60, 109, 115, 110, 109, 115, 103, 114, 58, 97, 101, 111, 110, 99, 108, 49, 64, 115, 104, 108, 97, 115, 111, 117, 102, 46, 108, 111, 99, 97, 108, 59, 123, 102, 53, 50, 57, 55, 51, 98, 54, 45, 99, 57, 50, 54, 45, 52, 98, 97, 100, 45, 57, 98, 97, 56, 45, 55, 99, 49, 101, 56, 52, 48, 101, 52, 97, 98, 48, 125, 62, 13, 10, 86, 105, 97, 58, 32, 77, 83, 78, 83, 76, 80, 47, 49, 46, 48, 47, 84, 76, 80, 32, 59, 98, 114, 97, 110, 99, 104, 61, 123, 69, 68, 56, 56, 57, 68, 65, 48, 45, 52, 50, 56, 51, 45, 52, 49, 53, 49, 45, 56, 57, 48, 49, 45, 70, 55, 52, 50, 51, 70, 52, 66, 67, 48, 55, 56, 125, 13, 10, 67, 83, 101, 113, 58, 32, 48, 32, 13, 10, 67, 97, 108, 108, 45, 73, 68, 58, 32, 123, 66, 66, 56, 56, 54, 51, 51, 53, 45, 67, 69, 55, 57, 45, 52, 48, 48, 56, 45, 65, 65, 56, 56, 45, 65, 55, 56, 57, 54, 54, 50, 56, 52, 57, 67, 67, 125, 13, 10, 77, 97, 120, 45, 70, 111, 114, 119, 97, 114, 100, 115, 58, 32, 48, 13, 10, 67, 111, 110, 116, 101, 110, 116, 45, 84, 121, 112, 101, 58, 32, 97, 112, 112, 108, 105, 99, 97, 116, 105, 111, 110, 47, 120, 45, 109, 115, 110, 109, 115, 103, 114, 45, 115, 101, 115, 115, 105, 111, 110, 114, 101, 113, 98, 111, 100, 121, 13, 10, 67, 111, 110, 116, 101, 110, 116, 45, 76, 101, 110, 103, 116, 104, 58, 32, 56, 56, 50, 13, 10, 13, 10, 69, 85, 70, 45, 71, 85, 73, 68, 58, 32, 123, 53, 68, 51, 69, 48, 50, 65, 66, 45, 54, 49, 57, 48, 45, 49, 49, 68, 51, 45, 66, 66, 66, 66, 45, 48, 48, 67, 48, 52, 70, 55, 57, 53, 54, 56, 51, 125, 13, 10, 83, 101, 115, 115, 105, 111, 110, 73, 68, 58, 32, 57, 48, 54, 55, 50, 55, 57, 53, 48, 13, 10, 65, 112, 112, 73, 68, 58, 32, 50, 13, 10, 82, 101, 113, 117, 101, 115, 116, 70, 108, 97, 103, 115, 58, 32, 49, 54, 13, 10, 67, 111, 110, 116, 101, 120, 116, 58, 32, 80, 103, 73, 65, 65, 65, 73, 65, 65, 65, 67, 78, 53, 103, 73, 65, 65, 65, 65, 65, 65, 65, 69, 65, 65, 65, 66, 66, 65, 71, 52, 65, 100, 65, 66, 112, 65, 70, 81, 65, 81, 119, 66, 68, 65, 68, 69, 65, 77, 81, 65, 52, 65, 71, 77, 65, 76, 103, 66, 54, 65, 71, 107, 65, 99, 65, 65, 65, 65, 65, 65, 65, 65, 65, 65, 65, 65, 65, 65, 65, 65, 65, 65, 65, 65, 65, 65, 65, 65, 65, 65, 65, 65, 65, 65, 65, 65, 65, 65, 65, 65, 65, 65, 65, 65, 65, 65, 65, 65, 65, 65, 65, 65, 65, 65, 65, 65, 65, 65, 65, 65, 65, 65, 65, 65, 65, 65, 65, 65, 65, 65, 65, 65, 65, 65, 65, 65, 65, 65, 65, 65, 65, 65, 65, 65, 65, 65, 65, 65, 65, 65, 65, 65, 65, 65, 65, 65, 65, 65, 65, 65, 65, 65, 65, 65, 65, 65, 65, 65, 65, 65, 65, 65, 65, 65, 65, 65, 65, 65, 65, 65, 65, 65, 65, 65, 65, 65, 65, 65, 65, 65, 65, 65, 65, 65, 65, 65, 65, 65, 65, 65, 65, 65, 65, 65, 65, 65, 65, 65, 65, 65, 65, 65, 65, 65, 65, 65, 65, 65, 65, 65, 65, 65, 65, 65, 65, 65, 65, 65, 65, 65, 65, 65, 65, 65, 65, 65, 65, 65, 65, 65, 65, 65, 65, 65, 65, 65, 65, 65, 65, 65, 65, 65, 65, 65, 65, 65, 65, 65, 65, 65, 65, 65, 65, 65, 65, 65, 65, 65, 65, 65, 65, 65, 65, 65, 65, 65, 65, 65, 65, 65, 65, 65, 65, 65, 65, 65, 65, 65, 65, 65, 65, 65, 65, 65, 65, 65, 65, 65, 65, 65, 65, 65, 65, 65, 65, 65, 65, 65, 65, 65, 65, 65, 65, 65, 65, 65, 65, 65, 65, 65, 65, 65, 65, 65, 65, 65, 65, 65, 65, 65, 65, 65, 65, 65, 65, 65, 65, 65, 65, 65, 65, 65, 65, 65, 65, 65, 65, 65, 65, 65, 65, 65, 65, 65, 65, 65, 65, 65, 65, 65, 65, 65, 65, 65, 65, 65, 65, 65, 65, 65, 65, 65, 65, 65, 65, 65, 65, 65, 65, 65, 65, 65, 65, 65, 65, 65, 65, 65, 65, 65, 65, 65, 65, 65, 65, 65, 65, 65, 65, 65, 65, 65, 65, 65, 65, 65, 65, 65, 65, 65, 65, 65, 65, 65, 65, 65, 65, 65, 65, 65, 65, 65, 65, 65, 65, 65, 65, 65, 65, 65, 65, 65, 65, 65, 65, 65, 65, 65, 65, 65, 65, 65, 65, 65, 65, 65, 65, 65, 65, 65, 65, 65, 65, 65, 65, 65, 65, 65, 65, 65, 65, 65, 65, 65, 65, 65, 65, 65, 65, 65, 65, 65, 65, 65, 65, 65, 65, 65, 65, 65, 65, 65, 65, 65, 65, 65, 65, 65, 65, 65, 65, 65, 65, 65, 65, 65, 65, 65, 65, 65, 65, 65, 65, 65, 65, 65, 65, 65, 65, 65, 65, 65, 65, 65, 65, 65, 65, 65, 65, 65, 65, 65, 65, 65, 65, 65, 65, 65, 65, 65, 65, 65, 65, 65, 65, 65, 65, 65, 65, 65, 65, 65, 65, 65, 65, 65, 65, 65, 65, 65, 65, 65, 65, 65, 65, 65, 65, 65, 65, 65, 65, 65, 65, 65, 65, 65, 65, 65, 65, 65, 65, 65, 65, 65, 65, 65, 65, 65, 65, 65, 65, 65, 65, 65, 65, 65, 65, 65, 65, 65, 65, 65, 65, 65, 65, 65, 65, 65, 65, 65, 65, 65, 65, 65, 65, 65, 65, 65, 65, 65, 65, 65, 65, 65, 65, 65, 65, 65, 65, 65, 65, 65, 65, 65, 65, 65, 65, 65];
        // let second_invite_msg_payload = [65, 65, 65, 65, 65, 65, 65, 65, 65, 65, 65, 65, 65, 65, 65, 65, 65, 65, 65, 65, 65, 65, 65, 65, 65, 65, 65, 65, 65, 65, 65, 65, 65, 65, 65, 65, 65, 65, 65, 65, 65, 65, 65, 65, 65, 65, 65, 65, 65, 65, 65, 65, 65, 65, 65, 65, 65, 65, 65, 65, 65, 65, 65, 65, 65, 65, 65, 65, 65, 65, 65, 65, 65, 65, 65, 65, 65, 65, 65, 65, 65, 65, 65, 65, 65, 65, 65, 65, 65, 65, 65, 65, 65, 65, 65, 65, 65, 65, 65, 65, 65, 65, 65, 65, 65, 65, 65, 65, 65, 65, 65, 65, 65, 65, 65, 65, 65, 65, 65, 65, 65, 65, 65, 65, 65, 65, 65, 65, 65, 65, 65, 65, 65, 65, 65, 65, 65, 61, 61, 13, 10, 13, 10, 0];

        // let mut first_msg_payload = P2PPayload::new(1, 0);
        // first_msg_payload.header_length = 20;
        // first_msg_payload.add_tlv(TLVFactory::get_untransfered_data_size(144));
        // first_msg_payload.set_payload(first_invite_msg_payload.to_vec());

        // let mut first_p2p_packet = P2PTransportPacket::new(0, Some(first_msg_payload));
        // first_p2p_packet.op_code = 3;
        // first_p2p_packet.header_length= 24;
        // first_p2p_packet.payload_length = 1226;
        // first_p2p_packet.add_tlv(TLVFactory::get_client_peer_info());

        // let mut second_msg_payload = P2PPayload::new(0, 0);
        // second_msg_payload.header_length = 8;
        // second_msg_payload.set_payload(second_invite_msg_payload.to_vec());

        // let mut second_p2p_packet = P2PTransportPacket::new(0+1226, Some(second_msg_payload));
        // second_p2p_packet.header_length = 8;
        // second_p2p_packet.payload_length = 152;

        // let first_p2p_pending_packet = PendingPacket::new(first_p2p_packet, MSNUser::default(), MSNUser::new(String::from("glandu@recv.com")));
        // let second_p2p_pending_packet = PendingPacket::new(second_p2p_packet, MSNUser::default(), MSNUser::new(String::from("glandu@recv.com")));

        // let (p2p_sender, mut p2p_receiver) = broadcast::channel::<P2PEvent>(10);

        // let mut p2p_session = P2PSession::new(p2p_sender);
        // //p2p_session.set_initialized(true);

        // p2p_session.on_message_received(first_p2p_pending_packet);
        // p2p_session.on_message_received(second_p2p_pending_packet);

        // let slp_response = p2p_receiver.recv().await.unwrap();

        // if let P2PEvent::Message(msg) = slp_response {
        //     assert_eq!(msg.sender.get_msn_addr(), String::from("glandu@recv.com"));
        //     assert_eq!(msg.receiver.get_msn_addr(), MSNUser::default().get_msn_addr());

        //     let mut ack_p2p_packet = P2PTransportPacket::new(0+1226+152, None);
        //     ack_p2p_packet.set_ack(msg.packet.get_next_sequence_number());
        //     info!("DEBUGG: {:?}", &msg.packet);
        //     p2p_session.on_message_received(PendingPacket::new(ack_p2p_packet, MSNUser::default(), MSNUser::new(String::from("glandu@recv.com"))));
        // }

        // let slp_response = p2p_receiver.recv().await.unwrap();
        // if let P2PEvent::Message(msg) = slp_response {
        //     assert_eq!(msg.sender.get_msn_addr(), String::from("glandu@recv.com"));
        //     assert_eq!(msg.receiver.get_msn_addr(), MSNUser::default().get_msn_addr());

        //     info!("DEBUGG: {:?}", &msg.packet);
        //     assert!(msg.packet.get_payload_length() > 0);
        // }
    }
}
