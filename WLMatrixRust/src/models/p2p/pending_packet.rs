use core::fmt;

use byteorder::{LittleEndian, ByteOrder};
use crate::models::{msn_user::MSNUser, errors::Errors};

use super::p2p_transport_packet::P2PTransportPacket;


#[derive(Clone, Debug)]
pub struct PendingPacket {
    pub packet: P2PTransportPacket,
    chunks: Vec<P2PTransportPacket>,
    pub sender: MSNUser,
    pub receiver: MSNUser
}

impl PendingPacket {
    
    pub fn new(packet: P2PTransportPacket, sender: MSNUser, receiver: MSNUser) -> Self {
        return PendingPacket{ packet, sender, receiver, chunks: Vec::new() };
    }

    pub fn add_chunk(&mut self, packet: P2PTransportPacket) {
        self.chunks.push(packet);
    }

    pub fn get_packet(&self) -> Result<P2PTransportPacket, Errors> {
        if !self.packet.is_payload_chunked() {
            return Ok(self.packet.to_owned());
        }

        if !self.is_complete() {
            return Err(Errors::PayloadNotComplete);
        }

        return Ok(self.merge_chunks());

    }

    pub fn get_last_chunk_next_seq_number(&self) -> u32 {
        let last_chunk = self.chunks.last();
        if last_chunk.is_none() {
           return self.packet.get_next_sequence_number();
        } else {
           return last_chunk.unwrap().get_next_sequence_number();
        }
    }

    fn merge_chunks(&self) -> P2PTransportPacket {
        let mut original = self.packet.to_owned();
        for chunk in &self.chunks {
            original.append_chunk(chunk);
        }
        return original;
    }

    pub fn is_complete(&self) -> bool {
        if !self.packet.is_payload_chunked() {
            return true;
        }

        if let Some(last) = self.chunks.last() {
            return !last.is_payload_chunked();
        }

        return false;
    }
}

