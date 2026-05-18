use crate::switchboard::models::switchboard_handle::SwitchboardHandle;
use crate::tachyon::client::tachyon_client::TachyonClient;
use matrix_sdk::ruma::RoomId;
use msnp::p2p::v2::p2p_transport_packet::{P2PTransportPacket, TransportOperationCode};
use msnp::p2p::v2::raw_p2p_payload::RawP2PPayload;
use msnp::shared::models::endpoint_id::EndpointId;
use msnp::shared::models::msn_user::MsnUser;
use msnp::shared::payload::msg::p2p_msg_payload::P2PMessagePayload;
use std::sync::{Arc, Mutex};
use std::time::Duration;
use anyhow::anyhow;
use chrono::{DateTime, Utc};
use dashmap::DashMap;
use lazy_static_include::syn::parse::End;
use log::{debug, error};
use ruma::int;
use msnp::p2p::v2::factories::{P2PPayloadFactory, P2PTransportPacketFactory, TLVFactory};
use msnp::p2p::v2::slp::raw_slp_payload::{RawSlpPayload, SlpPayloadFactory};
use msnp::shared::models::uuid::Uuid;
use msnp::shared::traits::IntoBytes;
use crate::p2p::client::transport::TransportStatus::HandshakeComplete;

impl TachyonClient {
    pub fn get_or_create_transport(&self, room_id: &RoomId, inviter: &MsnUser) -> Transport {
        match self.inner.transports.get(room_id) {
            None => {
                let switchboard_handle = self.switchboards().get_or_initialize(room_id, inviter);
                let transport = Transport::new(TransportSender::SBBridge(switchboard_handle), inviter.endpoint_id.clone(), self.own_user().endpoint_id);
                self.inner.transports.insert(room_id.to_owned(), transport.clone());
                transport
            }
            Some(transport) => {
                transport.value().clone()
            }
        }
    }

    pub fn remove_transport(self, room_id: &RoomId) {
        self.inner.transports.remove(room_id);
    }

}

pub enum TransportSender {
    SBBridge(SwitchboardHandle),
    TCPv1()
}

impl TransportSender {
    pub async fn send_packet(&self, sender: &EndpointId, sender_display_name: &str, receiver: &EndpointId, packet: P2PTransportPacket) {
        match self {
            TransportSender::SBBridge(handle) => {
                let msg = P2PMessagePayload::new(sender.to_owned(), receiver.clone(), packet, Some(sender_display_name.to_string()));
                handle.receive_msg(&sender.email_addr, sender_display_name, msg).await;
            }
            TransportSender::TCPv1() => {
                todo!("TCP not yet implemented")
            }
        }
    }
}

#[derive(PartialEq, Clone)]
enum TransportStatus {
    Initial,
    HandshakeOngoing,
    HandshakeComplete,
    NegotiatingBridge(Uuid),
    Ready
}


type PackageNumber = u16;

struct TransportInner {
    transport_id: u32,
    sequence_number: tokio::sync::Mutex<u32>,
    status: tokio::sync::RwLock<TransportStatus>,
    transport_sender: tokio::sync::Mutex<TransportSender>,
    chunks_unwraped: DashMap<PackageNumber, Vec<P2PTransportPacket>>,
    receiver: EndpointId,
    sender: EndpointId
}

#[derive(Clone)]
pub struct Transport {
    inner: Arc<TransportInner>
}

const PAYLOAD_MAX_LEN: usize = 2048;

impl Transport {
    pub fn new(initial_transport: TransportSender, sender: EndpointId, receiver: EndpointId) -> Transport {
        let sequence_number: u32 = 0;
        let transport_id: u32 = rand::random();
        Transport {
            inner: Arc::new(TransportInner {
                transport_id,
                sequence_number: tokio::sync::Mutex::new(sequence_number),
                status: tokio::sync::RwLock::new(TransportStatus::Initial),
                transport_sender: tokio::sync::Mutex::new(initial_transport),
                chunks_unwraped: Default::default(),
                receiver,
                sender,
            }),
        }
    }


    pub async fn receive_data_packet(&self, sender: &EndpointId, sender_display_name: &str, receiver: &EndpointId, packet: RawP2PPayload) {

        if packet.payload.len() > PAYLOAD_MAX_LEN {
            //We need to chunk
            let chunks = packet.chunk(PAYLOAD_MAX_LEN);
            for chunk in chunks {
                self.receive_single_data_packet(sender, sender_display_name, receiver, chunk).await
            }

        } else {
            self.receive_single_data_packet(sender, sender_display_name, receiver, packet).await
        }

    }

    async fn receive_single_data_packet(&self, sender: &EndpointId, sender_display_name: &str, receiver: &EndpointId, packet: RawP2PPayload) {
        if packet.session_id != 0 {
            self.wait_for_transport_ready(Duration::from_secs(20)).await.unwrap();
        }

        let transport_packet = P2PTransportPacket::new(0, Some(packet));
        self.receive_single_packet(transport_packet).await
    }

    async fn wait_for_transport_ready(&self, timeout: Duration) -> Result<(), anyhow::Error> {

        let mut interval = tokio::time::interval(Duration::from_secs(1));
        let end = Utc::now().timestamp() + (timeout.as_secs() as i64);
        loop {
            interval.tick().await;
            if *self.inner.status.read().await == TransportStatus::Ready {
                break;
            }

            if end > Utc::now().timestamp() {
                return Err(anyhow!("Transport still not ready after timeout limit: {:?}", timeout));
            }

        }

        Ok(())
    }

    pub async fn handle_transport_request(&self, transport_req: RawSlpPayload) {

        debug!("Handle Transport Request: {:?}", &transport_req);

        if transport_req.get_content_type().unwrap().as_str() == "application/x-msnmsgr-transreqbody" {

            let slp_transport_req_error_response = SlpPayloadFactory::get_500_error_direct_connect(
                &transport_req,
                String::from("TCPv1"),
            ).unwrap();

            let mut slp_transport_err_packet = P2PPayloadFactory::get_sip_text_message();
            slp_transport_err_packet.set_payload(slp_transport_req_error_response.into_bytes());
            let transport_packet = P2PTransportPacket::new(0, Some(slp_transport_err_packet));
            self.receive_single_packet(transport_packet).await;

            let mut write_lock = self.inner.status.write().await;
            *write_lock = TransportStatus::Ready;

        }
    }

    async fn receive_single_packet(&self, mut transport_packet: P2PTransportPacket) {
        let mut sequence_lock = self.inner.sequence_number.lock().await;
        let transport_sender_lock = self.inner.transport_sender.lock().await;

        let current_sequence_number = *sequence_lock;
        transport_packet.sequence_number = current_sequence_number;

        transport_packet = self.initiate_handshake_if_needed(transport_packet).await;


        let next_sequence_number = current_sequence_number + transport_packet.get_payload_length();;

        debug!("Client<-Transport: {:?}", &transport_packet);
        transport_sender_lock.send_packet(&self.inner.sender, self.inner.sender.email_addr.as_str(), &self.inner.receiver, transport_packet).await;

        *sequence_lock = next_sequence_number
    }

    pub async fn request_for_ack(&self) {
        self.receive_single_packet(P2PTransportPacketFactory::get_rak()).await;
    }

    async fn initiate_handshake_if_needed(&self, mut transport_packet: P2PTransportPacket) -> P2PTransportPacket {
        let status_lock = self.inner.status.read().await.clone();
        if status_lock == TransportStatus::Initial {
            transport_packet.set_syn(TLVFactory::get_client_peer_info());
            transport_packet.set_rak();

            let mut write_lock = self.inner.status.write().await;
            *write_lock = TransportStatus::HandshakeOngoing;
        };




        transport_packet
    }

    async fn unwrap_handshake(&self, packet: &P2PTransportPacket) {

        let mut handshake_complete = false;

        {
            let guard = self.inner.status.read().await.clone();
            match guard {
                TransportStatus::Initial => {
                    if packet.is_syn() && packet.is_rak() {
                        self.receive_single_packet(P2PTransportPacketFactory::get_syn_ack(packet.get_next_sequence_number())).await;
                        debug!("Transport ({transport_id}): Received initial SYN | RAK handshake, as expected.", transport_id = self.inner.transport_id);
                    } else {
                        error!("Transport ({transport_id}): Received packet without SYN | RAK while having no handshake setup.", transport_id = self.inner.transport_id);
                    }
                }
                TransportStatus::HandshakeOngoing => {
                    if  packet.is_syn() && packet.is_rak() && packet.is_ack() {
                        debug!("Transport ({transport_id}): Received response to our SYN | RAK handshake, as expected.", transport_id = self.inner.transport_id);
                        self.receive_single_packet(P2PTransportPacketFactory::get_ack(packet.get_next_sequence_number())).await;
                        handshake_complete = true;
                    }
                }
                _ => {

                }
            };
        }


        if handshake_complete {
            debug!("Transport ({transport_id}): Handshake Complete.", transport_id = self.inner.transport_id);
            let mut write_guard = self.inner.status.write().await;
            *write_guard = HandshakeComplete
        }

    }

    // Unwraps packets that were sent by the client throught the transport.
    // Unchunks SLP packets
    // Handles SYN handshake
    // handles RAK

    pub async fn unwrap_packet(&self, packet: P2PTransportPacket) -> Result<(Option<UnwrappedP2PPacket>) , anyhow::Error> {
        debug!("Client->Transport: {:?}", &packet);

        self.unwrap_handshake(&packet).await;

        debug!("Unwrapped handshake");
        if !packet.is_syn() && packet.is_rak() {
            // Simple RAK
            self.receive_single_packet(P2PTransportPacketFactory::get_ack(packet.get_next_sequence_number())).await;
        }

        if packet.get_payload().is_none() {
            return Ok(None);
        }

        if let Some(payload) = packet.get_payload() {
            //Only unchunk transport layer packets
            if payload.session_id == 0 {
                debug!("Unchunking transport layer packet");
                let is_in_chunks = self.inner.chunks_unwraped.contains_key(&payload.package_number);
                if payload.is_chunked_packet() {
                    debug!("Chunked -> returning none");
                    self.inner.chunks_unwraped.get_mut(&payload.package_number).unwrap().push(packet);
                    Ok(None)
                } else if is_in_chunks && !payload.is_chunked_packet() {
                    debug!("Now complete -> return reformed");
                    //Reform previously chunked packet
                    let (_, mut chunks) = self.inner.chunks_unwraped.remove(&payload.package_number).unwrap();

                    let reformed = chunks.drain(..).reduce( |mut acc, mut e| {
                        acc.append_chunk(&e);
                        acc
                    }
                    ).expect("not to be empty");

                    let slp = reformed.get_payload().unwrap().get_payload_as_slp().unwrap();
                    Ok(Some(UnwrappedP2PPacket::Slp(slp, reformed.op_code())))
                } else {
                    debug!("Packet not chunked, return.");
                    //Packet is not chunked and is not in chunks, so it's really a non chunked packet.
                    let slp = payload.get_payload_as_slp().unwrap();
                    Ok(Some(UnwrappedP2PPacket::Slp(slp, packet.op_code())))
                }
            } else {
                //We don't handle chunking in the transport for Session Scoped packets
                //FIXME: do not clone the payload.
                Ok(Some(UnwrappedP2PPacket::DataPacket(payload.clone(), packet.op_code())))
            }
        } else {
            Ok(None)
        }
    }
}

pub enum UnwrappedP2PPacket {
    Slp(RawSlpPayload, TransportOperationCode),
    DataPacket(RawP2PPayload, TransportOperationCode)
}