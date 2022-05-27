use std::collections::HashMap;

use chashmap::CHashMap;
use log::info;
use rand::Rng;
use tokio::sync::broadcast::Sender;

use super::{pending_packet::PendingPacket, p2p_transport_packet::P2PTransportPacket, factories::{P2PTransportPacketFactory, SlpPayloadFactory, P2PPayloadFactory}, p2p_payload::P2PPayload, slp_payload::SlpPayload};

pub struct P2PSession {

    sender : Sender<PendingPacket>,
    //p2ppayload package number & P2PTransport packet
    chunked_packets: HashMap<u16, PendingPacket>,

    pending_packets : Vec<PendingPacket>,

    /* Session has been initialized */
    initialized: bool,
    /* The current sequence number */
    sequence_number: u32

}

impl P2PSession {

    pub fn new(sender: Sender<PendingPacket>) -> Self {

        let mut rng = rand::thread_rng();
        let seq_number = rng.gen::<u32>();

        return P2PSession { sender, chunked_packets: HashMap::new(), pending_packets: Vec::new(), initialized: false, sequence_number: seq_number };
    }

    pub fn set_seq_number(&mut self, seq_number: u32) {
        self.sequence_number = seq_number;
    }

    pub fn set_initialized(&mut self, initialized: bool) {
        self.initialized = initialized;
    }

    fn is_in_chunks(&self, msg: &PendingPacket) -> bool {
        let mut found : bool = false;
        if let Some(package_num) = msg.packet.get_payload_package_number(){
            found = self.chunked_packets.contains_key(&package_num);
        }
        return found;
    }

    fn get_from_chunks(&self, package_number: u16) -> Option<&PendingPacket> {
        return self.chunked_packets.get(&package_number);
    }

    fn pop_from_chunks(&mut self, package_number: u16) -> Option<PendingPacket> {
        return self.chunked_packets.remove(&package_number);
    }

    fn add_or_append_to_chunks(&mut self, msg: &PendingPacket) -> u16 {

        if let Some(package_num) = msg.packet.get_payload_package_number(){
            if let Some(found) = self.chunked_packets.get_mut(&package_num) {
                found.add_chunk(msg.packet.to_owned());
            } else {
                self.chunked_packets.insert(package_num, msg.to_owned()); 
            }
            return package_num;
        }
        return 0;
    }

    pub fn on_message_received(&mut self, msg: PendingPacket) {

        let is_chunk = self.handle_chunks(&msg);
        if !is_chunk {

            let is_initialized = self.handle_handshake(&msg);
            if is_initialized {

                if !self.pending_packets.is_empty() {
                    self.handle_pending_packets();
                }

                //main section
                let packet = msg.get_packet().expect("Packet was not complete");
                if !packet.is_syn() && packet.is_rak() {
                    self.reply_ack(&msg);
                }

                if let Some(payload) = packet.get_payload() {
                    if let Ok(slp) = payload.get_payload_as_slp() {
                        self.reply_slp(&msg, slp);
                    }
                }
            } else {
                // save the packet while we wait for handshake
                self.pending_packets.push(msg);
            }

        }   
    }
    
    fn handle_pending_packets(&mut self) {

        let mut packets = Vec::new();
        packets.append(&mut self.pending_packets);
        
        for packet in packets {
            self.on_message_received(packet);
        }

    }

    fn handle_handshake(&mut self, msg: &PendingPacket) -> bool {
        if !self.initialized {
            let packet = msg.get_packet().expect("Packet was not complete");
            if packet.is_syn() {
                if packet.is_rak() {
                    //We need to send a syn + ack + rak and wait for handshake
                    self.reply_handshake(&msg);
                } else {
                    //Bypassed handshake
                    self.initialized = true;

                }
            } else if packet.is_ack() {
                //ack received for our rak (maybe check with number later)
                self.initialized = true;
            }
        }

        return self.initialized;
    }

    fn handle_chunks(&mut self, msg: &PendingPacket) -> bool {
        let is_in_chunks = self.is_in_chunks(&msg);

        if is_in_chunks || !msg.is_complete() {

            let package_number = self.add_or_append_to_chunks(&msg);

                if msg.is_complete() {
                    //this is the last chunk
                    self.on_message_complete(package_number);
                }
        }

        return is_in_chunks || !msg.is_complete();
    }

    fn on_message_complete(&mut self, package_number: u16) {
        let packet = self.pop_from_chunks(package_number).expect("on_message_complete did not in fact contain a message");
        self.on_message_received(packet);
    }

    fn handle_slp_payload(&mut self, slp_payload: &SlpPayload) -> P2PPayload {

        let content_type = slp_payload.get_content_type().unwrap(); //todo unwrap_or error slp message
            match content_type.as_str() {
                "application/x-msnmsgr-transreqbody" => {
                   let slp_payload_response = SlpPayloadFactory::get_200_ok_direct_connect(&slp_payload).unwrap(); //todo unwrap_or error slp message

                   let mut p2p_payload_response = P2PPayloadFactory::get_sip_text_message();
                   p2p_payload_response.set_payload(slp_payload_response.to_string().as_bytes().to_owned());
                    return p2p_payload_response;
                },
                "application/x-msnmsgr-sessionreqbody" => {
                    let slp_payload_response = SlpPayloadFactory::get_200_ok_session(slp_payload).unwrap(); //todo unwrap_or error slp message
                    let mut p2p_payload_response = P2PPayloadFactory::get_sip_text_message();
                    p2p_payload_response.set_payload(slp_payload_response.to_string().as_bytes().to_owned());
                    return p2p_payload_response;
                },
                "application/x-msnmsgr-transrespbody" => {
                    let bridge = slp_payload.get_body_property(&String::from("Bridge")).unwrap();
                    let slp_payload_response = SlpPayloadFactory::get_500_error_direct_connect(slp_payload, bridge.to_owned()).unwrap(); //todo unwrap_or error slp message
                    let mut p2p_payload_response = P2PPayloadFactory::get_sip_text_message();
                    p2p_payload_response.set_payload(slp_payload_response.to_string().as_bytes().to_owned());
                    return p2p_payload_response;
                },
                _ => {
                    info!("not handled slp payload yet: {:?}", slp_payload);
                   return P2PPayload::new(0, 0);
                }
            }
    }

    fn reply_slp(&mut self, request: &PendingPacket, slp_request: SlpPayload) {
        let slp_payload_resp = self.handle_slp_payload(&slp_request);
        let slp_transport_resp = P2PTransportPacket::new(0, Some(slp_payload_resp));
        self.reply(request, slp_transport_resp);
    }

    fn reply_ack(&mut self, request: &PendingPacket) {
        self.reply(request, P2PTransportPacketFactory::get_ack(request.packet.get_next_sequence_number()));
    }

    fn reply_handshake(&mut self, request: &PendingPacket) {
        self.reply(request, P2PTransportPacketFactory::get_syn_ack(request.packet.get_next_sequence_number()));
    }

    fn reply(&mut self, request: &PendingPacket, msg_to_send: P2PTransportPacket) {
        let mut msg_to_send = msg_to_send.clone();
        msg_to_send.sequence_number = self.sequence_number.clone();

        //setting next sequence number
        self.sequence_number = self.sequence_number + msg_to_send.get_payload_length();

        self.sender.send(PendingPacket::new(msg_to_send, request.receiver.clone(), request.sender.clone()));
    }


}



#[cfg(test)]
mod tests {
    use log::info;
    use tokio::sync::broadcast;

    use crate::models::{p2p::{pending_packet::PendingPacket, p2p_transport_packet::P2PTransportPacket, factories::P2PTransportPacketFactory}, msn_user::MSNUser};

    use super::P2PSession;


#[actix_rt::test]
async fn test_chunked_payload(){
    let part1_msg: [u8;1833] = [24, 3, 4, 202, 138, 185, 205, 99, 1, 12, 0, 2, 0, 0, 0, 14, 48, 48, 15, 1, 0, 0, 0, 0, 20, 1, 0, 0, 0, 0, 0, 0, 1, 8, 0, 0, 0, 0, 0, 0, 0, 131, 0, 0, 73, 78, 86, 73, 84, 69, 32, 77, 83, 78, 77, 83, 71, 82, 58, 97, 101, 111, 110, 116, 101, 115, 116, 51, 64, 115, 104, 108, 46, 108, 111, 99, 97, 108, 59, 123, 55, 55, 99, 52, 54, 97, 56, 102, 45, 51, 51, 97, 51, 45, 53, 50, 56, 50, 45, 57, 97, 53, 100, 45, 57, 48, 53, 101, 99, 100, 51, 101, 98, 48, 54, 57, 125, 32, 77, 83, 78, 83, 76, 80, 47, 49, 46, 48, 13, 10, 84, 111, 58, 32, 60, 109, 115, 110, 109, 115, 103, 114, 58, 97, 101, 111, 110, 116, 101, 115, 116, 51, 64, 115, 104, 108, 46, 108, 111, 99, 97, 108, 59, 123, 55, 55, 99, 52, 54, 97, 56, 102, 45, 51, 51, 97, 51, 45, 53, 50, 56, 50, 45, 57, 97, 53, 100, 45, 57, 48, 53, 101, 99, 100, 51, 101, 98, 48, 54, 57, 125, 62, 13, 10, 70, 114, 111, 109, 58, 32, 60, 109, 115, 110, 109, 115, 103, 114, 58, 97, 101, 111, 110, 116, 101, 115, 116, 64, 115, 104, 108, 46, 108, 111, 99, 97, 108, 59, 123, 102, 53, 50, 57, 55, 51, 98, 54, 45, 99, 57, 50, 54, 45, 52, 98, 97, 100, 45, 57, 98, 97, 56, 45, 55, 99, 49, 101, 56, 52, 48, 101, 52, 97, 98, 48, 125, 62, 13, 10, 86, 105, 97, 58, 32, 77, 83, 78, 83, 76, 80, 47, 49, 46, 48, 47, 84, 76, 80, 32, 59, 98, 114, 97, 110, 99, 104, 61, 123, 55, 65, 49, 54, 65, 50, 67, 69, 45, 70, 68, 66, 70, 45, 52, 49, 69, 56, 45, 57, 55, 57, 56, 45, 54, 55, 56, 53, 52, 67, 69, 51, 68, 53, 55, 57, 125, 13, 10, 67, 83, 101, 113, 58, 32, 48, 32, 13, 10, 67, 97, 108, 108, 45, 73, 68, 58, 32, 123, 56, 50, 57, 66, 70, 50, 52, 50, 45, 68, 57, 68, 67, 45, 52, 66, 66, 48, 45, 57, 70, 52, 48, 45, 52, 68, 48, 48, 65, 52, 49, 52, 49, 68, 57, 48, 125, 13, 10, 77, 97, 120, 45, 70, 111, 114, 119, 97, 114, 100, 115, 58, 32, 48, 13, 10, 67, 111, 110, 116, 101, 110, 116, 45, 84, 121, 112, 101, 58, 32, 97, 112, 112, 108, 105, 99, 97, 116, 105, 111, 110, 47, 120, 45, 109, 115, 110, 109, 115, 103, 114, 45, 115, 101, 115, 115, 105, 111, 110, 114, 101, 113, 98, 111, 100, 121, 13, 10, 67, 111, 110, 116, 101, 110, 116, 45, 76, 101, 110, 103, 116, 104, 58, 32, 56, 56, 51, 13, 10, 13, 10, 69, 85, 70, 45, 71, 85, 73, 68, 58, 32, 123, 53, 68, 51, 69, 48, 50, 65, 66, 45, 54, 49, 57, 48, 45, 49, 49, 68, 51, 45, 66, 66, 66, 66, 45, 48, 48, 67, 48, 52, 70, 55, 57, 53, 54, 56, 51, 125, 13, 10, 83, 101, 115, 115, 105, 111, 110, 73, 68, 58, 32, 49, 57, 57, 57, 51, 52, 50, 50, 52, 54, 13, 10, 65, 112, 112, 73, 68, 58, 32, 50, 13, 10, 82, 101, 113, 117, 101, 115, 116, 70, 108, 97, 103, 115, 58, 32, 49, 54, 13, 10, 67, 111, 110, 116, 101, 120, 116, 58, 32, 80, 103, 73, 65, 65, 65, 73, 65, 65, 65, 68, 109, 106, 81, 65, 65, 65, 65, 65, 65, 65, 65, 69, 65, 65, 65, 66, 104, 65, 71, 85, 65, 98, 119, 66, 117, 65, 67, 65, 65, 99, 65, 66, 112, 65, 72, 103, 65, 90, 81, 66, 115, 65, 67, 52, 65, 99, 65, 66, 122, 65, 71, 81, 65, 65, 65, 65, 65, 65, 65, 65, 65, 65, 65, 65, 65, 65, 65, 65, 65, 65, 65, 65, 65, 65, 65, 65, 65, 65, 65, 65, 65, 65, 65, 65, 65, 65, 65, 65, 65, 65, 65, 65, 65, 65, 65, 65, 65, 65, 65, 65, 65, 65, 65, 65, 65, 65, 65, 65, 65, 65, 65, 65, 65, 65, 65, 65, 65, 65, 65, 65, 65, 65, 65, 65, 65, 65, 65, 65, 65, 65, 65, 65, 65, 65, 65, 65, 65, 65, 65, 65, 65, 65, 65, 65, 65, 65, 65, 65, 65, 65, 65, 65, 65, 65, 65, 65, 65, 65, 65, 65, 65, 65, 65, 65, 65, 65, 65, 65, 65, 65, 65, 65, 65, 65, 65, 65, 65, 65, 65, 65, 65, 65, 65, 65, 65, 65, 65, 65, 65, 65, 65, 65, 65, 65, 65, 65, 65, 65, 65, 65, 65, 65, 65, 65, 65, 65, 65, 65, 65, 65, 65, 65, 65, 65, 65, 65, 65, 65, 65, 65, 65, 65, 65, 65, 65, 65, 65, 65, 65, 65, 65, 65, 65, 65, 65, 65, 65, 65, 65, 65, 65, 65, 65, 65, 65, 65, 65, 65, 65, 65, 65, 65, 65, 65, 65, 65, 65, 65, 65, 65, 65, 65, 65, 65, 65, 65, 65, 65, 65, 65, 65, 65, 65, 65, 65, 65, 65, 65, 65, 65, 65, 65, 65, 65, 65, 65, 65, 65, 65, 65, 65, 65, 65, 65, 65, 65, 65, 65, 65, 65, 65, 65, 65, 65, 65, 65, 65, 65, 65, 65, 65, 65, 65, 65, 65, 65, 65, 65, 65, 65, 65, 65, 65, 65, 65, 65, 65, 65, 65, 65, 65, 65, 65, 65, 65, 65, 65, 65, 65, 65, 65, 65, 65, 65, 65, 65, 65, 65, 65, 65, 65, 65, 65, 65, 65, 65, 65, 65, 65, 65, 65, 65, 65, 65, 65, 65, 65, 65, 65, 65, 65, 65, 65, 65, 65, 65, 65, 65, 65, 65, 65, 65, 65, 65, 65, 65, 65, 65, 65, 65, 65, 65, 65, 65, 65, 65, 65, 65, 65, 65, 65, 65, 65, 65, 65, 65, 65, 65, 65, 65, 65, 65, 65, 65, 65, 65, 65, 65, 65, 65, 65, 65, 65, 65, 65, 65, 65, 65, 65, 65, 65, 65, 65, 65, 65, 65, 65, 65, 65, 65, 65, 65, 65, 65, 65, 65, 65, 65, 65, 65, 65, 65, 65, 65, 65, 65, 65, 65, 65, 65, 65, 65, 65, 65, 65, 65, 65, 65, 65, 65, 65, 65, 65, 65, 65, 65, 65, 65, 65, 65, 65, 65, 65, 65, 65, 65, 65, 65, 65, 65, 65, 65, 65, 65, 65, 65, 65, 65, 65, 65, 65, 65, 65, 65, 65, 65, 65, 65, 65, 65, 65, 65, 65, 65, 65, 65, 65, 65, 65, 65, 65, 65, 65, 65, 65, 65, 65, 65, 65, 65, 65, 65, 65, 65, 65, 65, 65, 65, 65, 65, 65, 65, 65, 65, 65, 65, 65, 65, 65, 65, 65, 65, 65, 65, 65, 65, 65, 65, 65, 65, 65, 65, 65, 65, 65, 65, 65, 65, 65, 65, 65, 65, 65, 65, 65, 65, 65, 65, 65, 65, 65, 65, 65, 65, 65, 65, 65, 65, 65, 65, 65, 65, 65, 65, 65, 65, 65, 65, 65, 65, 65, 65, 65, 65, 65, 65, 65, 65, 65, 65, 65, 65, 65, 65, 65, 65, 65, 65, 65, 65, 65, 65, 65, 65, 65, 65, 65, 65, 65, 65, 65, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0];
    let part2_msg: [u8;151] = [8, 0, 0, 139, 138, 185, 210, 45, 8, 0, 0, 0, 0, 0, 0, 0, 65, 65, 65, 65, 65, 65, 65, 65, 65, 65, 65, 65, 65, 65, 65, 65, 65, 65, 65, 65, 65, 65, 65, 65, 65, 65, 65, 65, 65, 65, 65, 65, 65, 65, 65, 65, 65, 65, 65, 65, 65, 65, 65, 65, 65, 65, 65, 65, 65, 65, 65, 65, 65, 65, 65, 65, 65, 65, 65, 65, 65, 65, 65, 65, 65, 65, 65, 65, 65, 65, 65, 65, 65, 65, 65, 65, 65, 65, 65, 65, 65, 65, 65, 65, 65, 65, 65, 65, 65, 65, 65, 65, 65, 65, 65, 65, 65, 65, 65, 65, 65, 65, 65, 65, 65, 65, 65, 65, 65, 65, 65, 65, 65, 65, 65, 65, 65, 65, 65, 65, 65, 65, 65, 65, 61, 61, 13, 10, 13, 10, 0, 0, 0, 0, 0];

    let p2p_transport_packet1 = P2PTransportPacket::try_from(part1_msg.as_ref()).unwrap();
    let p2p_transport_packet2 = P2PTransportPacket::try_from(part2_msg.as_ref()).unwrap();

    let (p2p_sender, mut p2p_receiver) = broadcast::channel::<PendingPacket>(10);

    let mut p2p_session = P2PSession::new(p2p_sender);

    
    p2p_session.on_message_received(PendingPacket::new(p2p_transport_packet1, MSNUser::default(), MSNUser::default()));
    p2p_session.on_message_received(PendingPacket::new(p2p_transport_packet2, MSNUser::default(), MSNUser::default()));

    let syn_ack = p2p_receiver.recv().await.unwrap();
    assert!(syn_ack.is_complete());
    
    let syn_ack_packet = syn_ack.get_packet().unwrap();
    assert!(syn_ack_packet.is_syn());
    assert!(syn_ack_packet.is_rak());
    assert!(syn_ack_packet.is_ack());


    let mut ack = P2PTransportPacketFactory::get_ack(syn_ack_packet.get_next_sequence_number());
    let ack_packet = PendingPacket::new(ack, MSNUser::default(), MSNUser::default());

    p2p_session.on_message_received(ack_packet.clone());


    let invite_response = p2p_receiver.recv().await.unwrap();
    assert!(invite_response.is_complete());
    let invite_p2p_transport = invite_response.get_packet().unwrap();
    let invite_slp_payload = invite_p2p_transport.get_payload().unwrap().get_payload_as_slp().unwrap();
    assert!(invite_slp_payload.first_line.contains("200 OK"));
    
    println!("{}", invite_slp_payload.to_string());



}


}