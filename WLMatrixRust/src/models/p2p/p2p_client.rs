use std::{
    collections::HashMap,
    mem,
    sync::{
        Arc,
        atomic::{AtomicBool, AtomicI32, Ordering}, Mutex,
    },
};
use std::io::Write;

use log::{debug, info, warn};
use matrix_sdk::media::MediaEventContent;
use rand::Rng;
use tokio::sync::mpsc::UnboundedSender;

use crate::models::{
    msn_user::MSNUser,
    p2p::{
        app_id::AppID,
        events::content::{
            file_received_event_content::FileReceivedEventContent,
            file_transfer_accepted_event_content::FileTransferAcceptedEventContent, msn_object_requested_event_content::MSNObjectRequestedEventContent,
        },
        slp_payload::EufGUID,
    }, uuid::UUID,
};
use crate::models::tachyon_error::PayloadError;
use crate::P2P_REPO;

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
    slp_payload::{SlpPayload, self},
};

#[derive(Debug)]
pub struct InnerP2PClient {
    sender: UnboundedSender<P2PEvent>,

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
    pub fn new(sender: UnboundedSender<P2PEvent>) -> Self {
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
                transport_session_status: AtomicI32::new(P2PSessionStatus::NONE as i32),
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

        P2P_REPO.set_seq_number(seq_number);
        let mut seq_num = self
            .inner
            .sequence_number
            .lock()
            .expect("seq_number to be unlocked");

        let old_value = mem::replace(&mut *seq_num, seq_number);
        
        // debug!(
        //     "Replacing seq_number: {} ({:01x}) with {} ({:01x}) (diff +{})",
        //     old_value,
        //     old_value,
        //     seq_number,
        //     seq_number,
        //     seq_number - old_value
        // );
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
            } else if payload.is_file_transfer() || payload.is_msn_obj_transfer() {
                info!(
                    "file transfer packet received!, retrieveing pending file: {}",
                    &payload.session_id
                );


                let pl = payload.get_payload_bytes().clone();
                let mut file = std::fs::File::create("C:\\temp\\out.raw").unwrap();
                file.write_all(&pl);

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

            let mut payload_to_add = P2PPayload::new(payload.tf_combination, payload.session_id);
            payload_to_add.package_number = payload.get_package_number();
            payload_to_add.payload = chunk.to_vec();
            payload_to_add.session_id = payload.session_id;

            let mut to_add: P2PTransportPacket = P2PTransportPacket::new(0, None);

            if i < chunks.len() - 1 {
                //We need to add the remaining bytes TLV
                payload_to_add.add_tlv(TLVFactory::get_untransfered_data_size(
                    remaining_bytes.try_into().unwrap(),
                ));
                info!("remainingBytes: {}", remaining_bytes);
            }

            if i == 0 {
                to_add.op_code = to_split.op_code;
                to_add.tlvs = to_split.tlvs.clone();
                to_add.set_payload(Some(payload_to_add));
                //We are creating the first packet
            } else {
                payload_to_add.tf_combination = payload_to_add.tf_combination - 1;
                to_add.set_payload(Some(payload_to_add));
                //We are creating other packets
            }

            to_add.sequence_number = self.get_seq_number();
            self.set_seq_number(self.get_seq_number() + to_add.get_payload_length());

            out.push(to_add);
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
            },
            P2PSessionType::MSNObject(ref obj) => {
                slp_request = Some(
                    SlpPayloadFactory::get_msn_object_request(
                        &inviter, &invitee, obj, session_id,
                    ).unwrap(),
                );
            }
            _ => {}
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
    ) -> Result<Option<SlpPayload>, PayloadError> {
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
                //TODO STOP sending when we receive this
                return Err(PayloadError::PayloadBytesMissing);
            }
            _ => {
                info!("not handled slp payload: {:?}", slp_payload);
                return Err(PayloadError::PayloadBytesMissing);
            }
        }
    }

    fn handle_sessionreqbody(
        &mut self,
        slp_payload: &SlpPayload,
        sender: &MSNUser,
        receiver: &MSNUser
    ) -> Result<Option<SlpPayload>, PayloadError> {
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
                .expect("AppID to be valid");

            let session_id = slp_payload
            .get_body_property(&String::from("SessionID"))
            .ok_or(PayloadError::MandatoryPartNotFound { name: "SessionID".to_string(), payload: slp_payload.to_string() })?
            .parse::<u32>()?;


            match euf_guid {
                EufGUID::FileTransfer => {
                    if app_id.expect("AppId to be present here") == AppID::FileTransfer {
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

                    self.inner
                        .pending_files
                        .lock()
                        .expect("pending_files to be unlocked")
                        .insert(
                            session_id,
                            File::new(context.size as usize, context.friendly.as_ref().unwrap_or(&String::new()).to_string()),
                        );


                        self.inner.sender.send(P2PEvent::MSNObjectRequested(MSNObjectRequestedEventContent{
                            msn_object: context,
                            session_id: session_id,
                            call_id: slp_payload.get_call_id().unwrap().unwrap(),
                            inviter: sender.clone(),
                            invitee: receiver.clone()
                        }));
                },
                EufGUID::SharePhoto => {
                    
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

    pub fn send_msn_object(&mut self, session_id: u32, call_id: UUID, file: Vec<u8>, sender: MSNUser, receiver: MSNUser) {

            let data_preparation_message = P2PPayloadFactory::get_data_preparation_message(session_id);
            let data_preparation_packet = P2PTransportPacket::new(0, Some(data_preparation_message));
            self.reply(&sender, &receiver, data_preparation_packet);

            let mut msn_obj_message = P2PPayloadFactory::get_msn_obj(session_id);
            msn_obj_message.set_payload(file);
            let msn_obj_packet = P2PTransportPacket::new(0, Some(msn_obj_message));
            self.reply(&sender, &receiver, msn_obj_packet);

            let bye = SlpPayloadFactory::get_session_bye(&sender, &receiver, call_id ,session_id.to_string()).unwrap();

            self.reply_slp(&sender, &receiver, bye)


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
    use std::str::{from_utf8_unchecked, FromStr};

    use tokio::sync::{broadcast, mpsc};

    use crate::{sockets::{msnp_command::MSNPCommandParser, switchboard_command_handler::SwitchboardCommandHandler, events::socket_event::SocketEvent, command_handler::CommandHandler}, models::p2p::p2p_transport_packet::P2PTransportPacket};

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

        let (sb_sender, mut sb_receiver) = mpsc::unbounded_channel::<SocketEvent>();

        let mut handler = SwitchboardCommandHandler::new(sb_sender);

        for command in test1 {
            handler.handle_command(&command).await;
        }

        let test = sb_receiver.recv().await.unwrap();

        let bp = 0;
    }


    #[test]
    fn test_audio_msg(){
        let test_data: [u8; 1428]  = [0x14, 0x00, 0x00, 0x00, 0x08, 0x00, 0x00, 
        0x0c, 0x59, 0x7d, 0x54, 0x3f, 0x08, 0x01, 0x00, 
        0x00, 0x97, 0x09, 0xb4, 0xbb, 0x00, 0x00, 0x00, 
        0x00, 0x78, 0x05, 0x00, 0x00, 0x08, 0x00, 0x05, 
        0x70, 0x59, 0x7d, 0x54, 0x4b, 0x14, 0x05, 0x00, 
        0x01, 0x97, 0x09, 0xb4, 0xbb, 0x01, 0x08, 0x00, 
        0x00, 0x00, 0x00, 0x00, 0x00, 0x05, 0x08, 0x00, 
        0x00, 0x52, 0x49, 0x46, 0x46, 0x5c, 0x0a, 0x00, 
        0x00, 0x57, 0x41, 0x56, 0x45, 0x66, 0x6d, 0x74, 
        0x20, 0x14, 0x00, 0x00, 0x00, 0x8e, 0x02, 0x01, 
        0x00, 0x80, 0x3e, 0x00, 0x00, 0xd0, 0x07, 0x00, 
        0x00, 0x28, 0x00, 0x00, 0x00, 0x02, 0x00, 0x40, 
        0x01, 0x66, 0x61, 0x63, 0x74, 0x04, 0x00, 0x00, 
        0x00, 0x40, 0x51, 0x00, 0x00, 0x64, 0x61, 0x74, 
        0x61, 0x28, 0x0a, 0x00, 0x00, 0x5c, 0xf0, 0x78, 
        0x5e, 0x86, 0x85, 0x0d, 0x33, 0x7e, 0x2a, 0xcb, 
        0x45, 0xa2, 0xa7, 0x8e, 0xc3, 0xb2, 0x22, 0x8a, 
        0xff, 0x48, 0x1c, 0xbd, 0x82, 0xe8, 0xfd, 0x8a, 
        0xc5, 0xb2, 0xa1, 0xf2, 0x0d, 0xfc, 0x17, 0xa1, 
        0x96, 0xf0, 0x23, 0xff, 0xf1, 0x63, 0x31, 0x30, 
        0x4d, 0xad, 0xea, 0x32, 0x35, 0xb2, 0x19, 0xb4, 
        0xd5, 0xe6, 0xef, 0x4d, 0x1d, 0xd0, 0x94, 0x71, 
        0x58, 0x75, 0x4b, 0x4a, 0x09, 0x20, 0x08, 0xf0, 
        0xec, 0x90, 0xa2, 0xe0, 0xa1, 0x10, 0xa2, 0x7f, 
        0x6b, 0x55, 0xf6, 0xaf, 0xfe, 0x65, 0x17, 0xb5, 
        0x09, 0x4f, 0xcb, 0x89, 0xc7, 0xfd, 0x46, 0xe8, 
        0xb5, 0x20, 0x11, 0xbc, 0x6a, 0x51, 0x22, 0xc6, 
        0x40, 0x43, 0x64, 0x66, 0x47, 0xc3, 0xe5, 0x34, 
        0x5b, 0xc0, 0x14, 0x1b, 0x0b, 0x76, 0x07, 0xa0, 
        0x47, 0x90, 0x38, 0xff, 0xf1, 0x63, 0x23, 0xc8, 
        0x97, 0x58, 0x3d, 0x0c, 0xb3, 0xfe, 0x6c, 0xa0, 
        0x87, 0xca, 0xa1, 0x20, 0x83, 0x5e, 0xaa, 0x98, 
        0x29, 0x33, 0x43, 0xe4, 0x0a, 0x85, 0x1f, 0x49, 
        0x61, 0x5f, 0x39, 0x1d, 0x58, 0xa8, 0x55, 0x2c, 
        0xf7, 0x95, 0xd5, 0x5f, 0xf6, 0x62, 0xcc, 0xf9, 
        0xa3, 0x90, 0x55, 0x01, 0x18, 0x83, 0x53, 0x52, 
        0x62, 0xe7, 0xd6, 0x29, 0x51, 0x5b, 0x30, 0x29, 
        0xf4, 0x7d, 0x48, 0x0e, 0x13, 0x5b, 0xd1, 0x91, 
        0xe7, 0x2b, 0xbd, 0x17, 0xed, 0x43, 0x6f, 0x68, 
        0xba, 0xf2, 0x1d, 0x7f, 0xf4, 0x64, 0xe9, 0x8a, 
        0x7a, 0x6f, 0xe6, 0x13, 0x95, 0x44, 0xab, 0xcb, 
        0x3a, 0x77, 0x86, 0x46, 0x7b, 0xb1, 0xad, 0x0e, 
        0x15, 0x06, 0x20, 0x54, 0x0b, 0xe8, 0x7b, 0xfc, 
        0x89, 0x81, 0x90, 0x46, 0x2e, 0x8a, 0x54, 0xb5, 
        0x1f, 0xc1, 0xa1, 0xa8, 0x3d, 0x60, 0xe7, 0xe8, 
        0x89, 0x07, 0xf3, 0x84, 0x61, 0x01, 0xf2, 0x99, 
        0x59, 0xfc, 0x4b, 0xb0, 0x2d, 0x32, 0xcf, 0xf2, 
        0x0a, 0x52, 0x80, 0x11, 0x20, 0x1f, 0x92, 0xa9, 
        0x72, 0x69, 0x24, 0x78, 0x00, 0x06, 0x3f, 0x15, 
        0xb9, 0xe1, 0xe3, 0x5a, 0x3d, 0x5c, 0x74, 0xe0, 
        0xc4, 0x52, 0xb9, 0x93, 0xa3, 0x44, 0x58, 0xf2, 
        0x7a, 0x34, 0xf3, 0x64, 0x21, 0xca, 0xed, 0xa1, 
        0x47, 0x5b, 0x02, 0x06, 0xf1, 0xa3, 0x3a, 0x03, 
        0x97, 0xba, 0x52, 0x87, 0x40, 0x3f, 0xba, 0x6a, 
        0x25, 0x7f, 0xff, 0xff, 0xf5, 0x5e, 0xc4, 0xc4, 
        0x22, 0x93, 0xd1, 0x73, 0xa6, 0xab, 0xfe, 0x1e, 
        0x82, 0x45, 0x78, 0xaf, 0xa7, 0xa8, 0x68, 0x6b, 
        0xeb, 0x5d, 0x0a, 0xd2, 0x67, 0x4d, 0x29, 0x10, 
        0xc9, 0x1b, 0x54, 0x27, 0x65, 0xb5, 0xa0, 0xf3, 
        0x4c, 0xcb, 0x5c, 0xa3, 0xf4, 0x60, 0xe4, 0x01, 
        0x89, 0x61, 0xa9, 0x16, 0x12, 0xaf, 0xff, 0xfe, 
        0x1b, 0x9e, 0xff, 0xeb, 0xda, 0x7c, 0x09, 0xe1, 
        0xb0, 0xc5, 0x30, 0x16, 0xdf, 0xf7, 0x00, 0x40, 
        0x46, 0x43, 0x4c, 0xcd, 0x67, 0x16, 0xe3, 0xca, 
        0x8c, 0xe1, 0xfe, 0xe9, 0xfc, 0x60, 0xe7, 0x1c, 
        0x06, 0x83, 0xf9, 0x86, 0x8e, 0x90, 0xff, 0xe2, 
        0x7c, 0x92, 0xa9, 0x02, 0xcd, 0xfa, 0x4b, 0xf4, 
        0x71, 0x40, 0xe5, 0x8a, 0x55, 0xee, 0xa0, 0x0e, 
        0xea, 0x9f, 0x52, 0xec, 0x14, 0x02, 0x81, 0x0a, 
        0x92, 0xa2, 0xb4, 0x7f, 0xf8, 0x5e, 0xa0, 0xa3, 
        0x90, 0x3c, 0x8a, 0x32, 0x37, 0x84, 0xb3, 0xf3, 
        0xf5, 0x1d, 0x6c, 0xeb, 0x17, 0x37, 0xb7, 0xbd, 
        0xa7, 0x37, 0x00, 0x2e, 0x59, 0x58, 0x4f, 0x80, 
        0x1c, 0x78, 0x95, 0xcc, 0x38, 0xd1, 0x33, 0x88, 
        0x50, 0x05, 0xfe, 0x82, 0x71, 0x5c, 0xea, 0x64, 
        0x24, 0x43, 0xfa, 0x82, 0x3c, 0x01, 0x0a, 0x3c, 
        0xa3, 0xb0, 0x80, 0x06, 0x3c, 0xdd, 0xa9, 0xe4, 
        0xff, 0x82, 0x27, 0xa6, 0x82, 0x09, 0x2d, 0x40, 
        0x90, 0x93, 0x45, 0xd5, 0x7a, 0x29, 0x9e, 0x81, 
        0xdf, 0x59, 0x77, 0xcd, 0xf2, 0x60, 0xa7, 0x13, 
        0x0c, 0x87, 0xf3, 0x87, 0xfd, 0x34, 0xbf, 0x80, 
        0x68, 0xeb, 0xb3, 0xd4, 0x37, 0x29, 0x63, 0x44, 
        0x0d, 0xbd, 0x16, 0x07, 0x13, 0x1c, 0x65, 0x72, 
        0x71, 0x79, 0x88, 0xa8, 0xca, 0x5d, 0x3e, 0xb4, 
        0x57, 0xa2, 0x02, 0x57, 0xfd, 0x61, 0x25, 0x57, 
        0xc8, 0xa1, 0xfe, 0x66, 0xa4, 0x24, 0xff, 0xc6, 
        0xdc, 0xa1, 0x01, 0x45, 0xff, 0x1e, 0x19, 0x01, 
        0xcd, 0x13, 0x24, 0x9e, 0x29, 0x28, 0x09, 0x65, 
        0xf6, 0x2d, 0x4a, 0x9c, 0xe0, 0x49, 0xf5, 0x50, 
        0x1f, 0x54, 0x8e, 0xdf, 0xf3, 0x5a, 0x6a, 0xaf, 
        0x96, 0xc3, 0xaa, 0x83, 0x62, 0x6a, 0xf1, 0x0b, 
        0xf7, 0x54, 0x0b, 0x40, 0xbb, 0x90, 0x88, 0xd7, 
        0x44, 0x0f, 0x02, 0xae, 0x60, 0x17, 0xef, 0xed, 
        0xc1, 0x42, 0xef, 0xc7, 0x7c, 0x3c, 0x54, 0x55, 
        0xbf, 0x3e, 0xef, 0xff, 0xf0, 0x58, 0x35, 0x8f, 
        0x42, 0x83, 0xd5, 0x4e, 0x8d, 0x08, 0x8f, 0x8c, 
        0x6e, 0x92, 0xca, 0xea, 0xaa, 0xa5, 0x63, 0x2b, 
        0x7c, 0xa9, 0x81, 0x00, 0x5c, 0x42, 0x16, 0xaa, 
        0x60, 0x77, 0x03, 0x18, 0xeb, 0x7a, 0xda, 0xe0, 
        0x7e, 0xe9, 0x71, 0x57, 0xfd, 0x5c, 0x04, 0x0b, 
        0xa1, 0x4b, 0xc0, 0x1c, 0xe1, 0x41, 0x0a, 0x2f, 
        0x28, 0x45, 0x43, 0xff, 0xfe, 0x03, 0x96, 0x0e, 
        0x8d, 0xaf, 0xec, 0x56, 0x64, 0xf0, 0xa1, 0x63, 
        0xe0, 0x25, 0xa3, 0x42, 0xdd, 0x87, 0xc1, 0xb3, 
        0xa5, 0x31, 0xae, 0xbf, 0xf9, 0x5b, 0x56, 0x8d, 
        0x36, 0x3e, 0x8a, 0x9a, 0xa2, 0xc4, 0x6d, 0xcb, 
        0x89, 0x34, 0x93, 0xff, 0xb0, 0x6a, 0x10, 0x3f, 
        0xa4, 0xdb, 0x2b, 0x3d, 0x33, 0x1b, 0x30, 0x3b, 
        0x72, 0x0d, 0x4b, 0x4e, 0x3b, 0x84, 0x80, 0x53, 
        0x4c, 0xfa, 0x7f, 0xff, 0xf9, 0x5f, 0xd4, 0xbc, 
        0x45, 0xa7, 0xe0, 0xc2, 0x34, 0xa3, 0xfd, 0xbb, 
        0x2c, 0x01, 0xff, 0x83, 0xc0, 0x24, 0x12, 0x0f, 
        0x36, 0x0c, 0x30, 0x03, 0xbb, 0x10, 0x95, 0x54, 
        0x11, 0x69, 0x1d, 0x31, 0x8e, 0x0a, 0x5d, 0x54, 
        0x83, 0x0d, 0x07, 0xff, 0xf5, 0x58, 0x99, 0x8c, 
        0x2c, 0x53, 0xf0, 0x83, 0xeb, 0x3b, 0xfe, 0xf1, 
        0x68, 0x00, 0xaf, 0xe9, 0x08, 0x99, 0x4a, 0x85, 
        0x7e, 0x34, 0x82, 0xe1, 0x86, 0x59, 0xda, 0x6c, 
        0x11, 0xfa, 0x20, 0x27, 0x79, 0xef, 0x5e, 0xe0, 
        0x47, 0x63, 0x7f, 0xff, 0xfe, 0x5b, 0x71, 0xb9, 
        0x2d, 0xa0, 0xaa, 0x43, 0xc0, 0x62, 0x35, 0x89, 
        0xe4, 0x0f, 0xc6, 0x21, 0xfe, 0xab, 0x1a, 0xc2, 
        0x0d, 0x44, 0x19, 0x16, 0x5e, 0x33, 0x86, 0xb1, 
        0x9b, 0x54, 0x4b, 0xe1, 0xbd, 0xde, 0x51, 0xed, 
        0x6e, 0xb1, 0x80, 0xbf, 0xf5, 0x5d, 0xb9, 0xf8, 
        0x18, 0xae, 0x78, 0xce, 0x84, 0x0c, 0x5f, 0xde, 
        0x28, 0xb1, 0x47, 0xff, 0xd4, 0x28, 0xc3, 0xbd, 
        0x1b, 0xa2, 0x08, 0x40, 0x28, 0xa3, 0x95, 0x65, 
        0xd5, 0xee, 0x32, 0x5f, 0x95, 0x57, 0x3f, 0xe8, 
        0x75, 0x32, 0x5b, 0x9f, 0xf7, 0x5b, 0x79, 0xfc, 
        0x00, 0x11, 0x2a, 0x8a, 0x8d, 0xfb, 0x2a, 0xd8, 
        0x70, 0x8d, 0xbf, 0xfe, 0x96, 0xcb, 0x6c, 0x73, 
        0xa7, 0x2c, 0xee, 0x24, 0x4d, 0x00, 0x95, 0x02, 
        0x2e, 0xec, 0x4f, 0x39, 0x2a, 0xca, 0x48, 0x89, 
        0x3c, 0x01, 0x37, 0x9b, 0xfd, 0x5b, 0xb9, 0xcc, 
        0x01, 0x00, 0x1f, 0x30, 0x9c, 0xcc, 0xed, 0xd9, 
        0x40, 0x2c, 0x17, 0xf8, 0x18, 0x7f, 0x14, 0x0e, 
        0xf7, 0x5e, 0xa8, 0xca, 0x2a, 0xfe, 0xa3, 0x9c, 
        0x95, 0x7c, 0xf4, 0x62, 0xdd, 0x91, 0x31, 0x58, 
        0x51, 0xcc, 0x8f, 0xff, 0xf0, 0x5c, 0x6d, 0xbe, 
        0x0d, 0x11, 0x93, 0x56, 0x5f, 0x16, 0xa6, 0xb0, 
        0xa7, 0x53, 0x1b, 0x84, 0xfa, 0x88, 0x5d, 0x76, 
        0xb2, 0xa6, 0xf7, 0x98, 0x60, 0x45, 0x8f, 0x7d, 
        0x91, 0x30, 0x8b, 0x36, 0xf1, 0x81, 0x9d, 0x04, 
        0x2c, 0x80, 0x35, 0x7f, 0xf1, 0x61, 0x81, 0x1e, 
        0x4d, 0x04, 0xbd, 0x49, 0xbc, 0x08, 0xa0, 0x8d, 
        0xf0, 0x26, 0x26, 0x89, 0xc5, 0x34, 0x40, 0xfb, 
        0x02, 0x00, 0x49, 0x5a, 0x2f, 0x7c, 0x24, 0x80, 
        0x7c, 0x07, 0xac, 0x05, 0xd1, 0x4e, 0x02, 0xaf, 
        0x84, 0x82, 0xff, 0xff, 0xfe, 0x63, 0xc1, 0x9d, 
        0xaf, 0xca, 0x80, 0xc3, 0x55, 0x3f, 0xf8, 0xb3, 
        0x03, 0xa8, 0x8a, 0x00, 0x26, 0xdf, 0xef, 0x04, 
        0x90, 0x88, 0x78, 0x8b, 0x73, 0x31, 0x4c, 0xd6, 
        0xa5, 0xbb, 0x96, 0x6a, 0x21, 0x1a, 0x14, 0x38, 
        0x40, 0xdc, 0x23, 0xa3, 0xfa, 0x62, 0xeb, 0x74, 
        0xde, 0x85, 0x14, 0x9d, 0x03, 0xff, 0xfe, 0x90, 
        0x52, 0x42, 0x52, 0xd0, 0x9c, 0x6a, 0x43, 0x89, 
        0x9a, 0xd2, 0x4b, 0x01, 0x0c, 0x02, 0xbf, 0xf2, 
        0xb5, 0x4f, 0xe5, 0x01, 0x1c, 0x9d, 0x07, 0x20, 
        0x5f, 0xbf, 0xb4, 0x8f, 0xfa, 0x60, 0xe8, 0x24, 
        0x25, 0x50, 0xc1, 0x66, 0x59, 0xff, 0x97, 0x0d, 
        0xc2, 0x88, 0x7d, 0x01, 0x12, 0x44, 0x06, 0x02, 
        0x72, 0x76, 0x03, 0x50, 0xb8, 0x1b, 0x60, 0x0d, 
        0x79, 0xd8, 0x66, 0x67, 0x43, 0xc8, 0xda, 0xd2, 
        0x70, 0x0f, 0x92, 0x45, 0xf5, 0x58, 0x08, 0x24, 
        0xaf, 0x0f, 0x99, 0x16, 0x00, 0x97, 0xff, 0x7e, 
        0x73, 0x09, 0x69, 0xc8, 0x29, 0x5c, 0xb4, 0xfd, 
        0xf1, 0x2c, 0x82, 0x6e, 0x09, 0x36, 0x92, 0x1d, 
        0x17, 0xe8, 0x77, 0x6e, 0x30, 0xfb, 0x51, 0xea, 
        0x1f, 0xa0, 0x22, 0x85, 0xf1, 0x60, 0xa7, 0xc5, 
        0x7e, 0x39, 0x14, 0x8d, 0x7f, 0xfc, 0x94, 0x20, 
        0x19, 0xa6, 0x50, 0x0e, 0xca, 0xf1, 0xa6, 0xe6, 
        0x03, 0xeb, 0x80, 0x41, 0x00, 0x67, 0x8a, 0x56, 
        0x6f, 0x95, 0x8c, 0x97, 0xfc, 0x44, 0x78, 0x6d, 
        0x48, 0x3d, 0x60, 0x1f, 0xf9, 0x5c, 0x6e, 0x04, 
        0x32, 0x5d, 0x54, 0x04, 0x4e, 0xa4, 0x5f, 0x38, 
        0xd0, 0x1f, 0xe8, 0x51, 0xb3, 0xb3, 0x3a, 0x4d, 
        0x89, 0x18, 0x0f, 0xfa, 0x89, 0x94, 0x0e, 0xca, 
        0x82, 0x7b, 0x4c, 0xa0, 0x6c];

        let str: &str = unsafe { from_utf8_unchecked(&test_data) };

        let test = P2PTransportPacket::from_str(&str).unwrap();
        let test1 = 2;


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
