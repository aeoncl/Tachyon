use core::fmt;
use std::{str::{from_utf8_unchecked, FromStr}, fmt::Display};

use byteorder::{BigEndian, ByteOrder};
use log::info;

use crate::{models::errors::Errors};

use super::{p2p_payload::P2PPayload, opcode::OperationCode, factories::TLVFactory, tlv::{ValueType, TLV, extract_tlvs}};


/* P2PHeaderV2 */
/* For the official client to use P2PV2 headers, firstly, our fake client must have the P2PV2 Extended Capability 
AND our fake client must be MPOP enabled. (which means adding endpoint data in NLN UBX payload AND making join the endpoint in switchboards. */
    #[derive(Clone)]
    pub struct P2PTransportPacket {
    
        pub header_length: usize,
        pub op_code : u8,
        pub payload_length: usize,
        pub sequence_number: u32,
        pub tlvs: Vec<TLV>,
        payload: Option<P2PPayload>
    }

    impl fmt::Debug for P2PTransportPacket {
        fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
            f.debug_struct("P2PTransportPacket")
             .field("header_length", &self.header_length)
             .field("sequence_number", &self.sequence_number)

             .field("op_code", &self.op_code)
             .field("is_syn", &self.is_syn())
             .field("is_ack", &self.is_ack())
             .field("is_rak", &self.is_rak())
             .field("payload_length", &self.payload_length)
             .field("is_payload_chunked", &self.is_payload_chunked())
             .field("tlvs", &self.tlvs)
             .field("payload", &self.payload)
             .finish()
        }
    }
    
    impl P2PTransportPacket {

        pub fn extract_payload_length(p2p_transport_data: &[u8]) -> usize {
            return BigEndian::read_u16(&p2p_transport_data[2..4]) as usize;
        }

        pub fn new(sequence_number: u32, payload: Option<P2PPayload>) -> Self {
            return P2PTransportPacket { header_length: 0, op_code: 0, payload_length: 0, sequence_number, tlvs: Vec::new(), payload};
        }

        pub fn get_payload(&self) -> Option<&P2PPayload> {
            return self.payload.as_ref();
        }

        pub fn set_payload(&mut self, payload: Option<P2PPayload>) {
            self.payload = payload;
        }
        
        pub fn to_vec(&self) -> Vec<u8> {
                let mut out: Vec<u8> = Vec::new();
                out.push(self.op_code.clone());
        
                let mut buffer : [u8;4] = [0,0,0,0];
                BigEndian::write_u32(&mut buffer, self.sequence_number);
                out.append(&mut buffer.to_vec());
        
                for tlv in &self.tlvs {
                    let mut tlv_serialized : Vec<u8> = tlv.as_vec();
                    out.append(&mut tlv_serialized);
                }
        
                if !self.tlvs.is_empty() {
                    let mut last = self.tlvs.last().unwrap().clone();
                    let mut padding : Vec<u8> = Vec::new();
                    let mut value = last.value;
        
                    let mut trailing_nul_bytes_count = 0;
        
                    while value.pop().unwrap_or(0x01) == 0x00 {
                        trailing_nul_bytes_count+=1;
                    }
        
                    let necessary_padding = 4 - trailing_nul_bytes_count;
        
                    if(necessary_padding>0) {
                        for i in 0..necessary_padding {
                            padding.push(0x0);
                        }
                    }
        
                    info!("Padding length: {}", padding.len());
        
                    out.append(&mut padding);
                }
        
                let mut payload_bytes = Vec::new();
                if let Some(payload) = &self.payload {
                    let payload = payload.to_string();
                    let payload_length = payload.len() as u16;
                    
                    payload_bytes = payload.as_bytes().to_vec();
        
                    let mut buffer : [u8;2] = [0,0];
                    BigEndian::write_u16(&mut buffer, payload_length);
                    out.insert(1, buffer[0]);
                    out.insert(2, buffer[1]);
                } else {
                    let mut buffer : [u8;2] = [0,0];
                    out.insert(1, buffer[0]);
                    out.insert(2, buffer[1]);
                }
        
                out.insert(0, (out.len() + 1) as u8);
        
                out.append(&mut payload_bytes);
        
                let padding : [u8;4] = [0,0,0,0];
                out.append(&mut padding.to_vec());
        
                return out;
            
        }

        pub fn get_next_sequence_number(&self) -> u32 {
            return self.sequence_number + self.payload_length as u32;
        }

        pub fn get_sequence_number(&self) -> u32 {
            return self.sequence_number;
        }
    
        pub fn get_payload_length(&self) -> u32 {
            if let Some(payload) = &self.payload {
                let serialized = payload.to_string();
                return serialized.len() as u32;
            } 
            return 0;
        }
    
        pub fn is_rak(&self) -> bool {
          let has_rak_flag = &self.op_code & OperationCode::RequestForAck as u8;
          return has_rak_flag == OperationCode::RequestForAck as u8;
        }
    
        pub fn is_syn(&self) -> bool {
            let is_syn_flag = &self.op_code & OperationCode::Syn as u8;
            return is_syn_flag == OperationCode::Syn as u8;
        }

        pub fn add_tlv(&mut self, tlv: TLV) {
            self.tlvs.push(tlv);
        }

        pub fn is_slp_msg(&self) -> bool {
            if self.payload.is_none() {
                return false;
            }

            let payload = self.payload.as_ref().unwrap();

            return payload.session_id == 0;
        }
    
        pub fn is_ack(&self) -> bool {
            return self.get_ack_tlv().is_some();
        }
    
        pub fn set_syn(&mut self, client_info: TLV) {
            self.op_code = self.op_code + OperationCode::Syn as u8;
            self.tlvs.push(client_info);
        }
    
        pub fn set_ack(&mut self, sequence_number: u32){
            let ack_tlv = TLVFactory::get_ack(sequence_number);
            self.tlvs.push(ack_tlv);
        }
    
        pub fn set_rak(&mut self) {
            self.op_code += OperationCode::RequestForAck as u8;
        }
    
        pub fn get_next_ack_sequence_number(&self) -> Option<u32> {
            if self.is_rak() {
                if let Some(ack_tlv) = self.get_ack_tlv(){
                    let seq_number = BigEndian::read_u32(ack_tlv.value.as_slice());
                    return Some(seq_number + self.payload_length as u32);
                }
            }
            return None;
        }
    
        pub fn get_ack_tlv(&self) -> Option<&TLV> {
            return self.get_tlv_for_type(&ValueType::AckSequenceNumber);
        }
    
        pub fn get_client_info_tlv(&self) -> Option<&TLV> {
            return self.get_tlv_for_type(&ValueType::ClientPeerInfo);
        }
    
        pub fn get_tlv_for_type(&self, value_type: &ValueType) -> Option<&TLV> {
            for tlv in &self.tlvs {
                if tlv.is_type(value_type) {
                    return Some(tlv);
                }
            }
            return None;
        }

        pub fn is_payload_chunked(&self) -> bool {
            if let Some(payload) = self.payload.as_ref() {
                return payload.is_chucked_packet();
            }
            return false;
        }

        pub fn get_payload_package_number(&self) -> Option<u16> {
            if let Some(payload) = self.payload.as_ref() {
                return Some(payload.package_number.to_owned());
            }
            return None;
        }

        pub fn append_chunk(&mut self, chunk: &P2PTransportPacket) {
            if let Some(chunk_payload) = chunk.get_payload() {
                let mut chunk_payload = chunk_payload.to_owned();
                if let Some(payload) = self.payload.as_mut(){
                    let added_len = payload.append(&mut chunk_payload);
                    self.payload_length += added_len;
                }
            }
        }

        pub fn append_raw_chunk(&mut self, chunk: &[u8]) {

            if let Some(payload) = self.payload.as_mut(){
                let added_len = payload.append_raw(&chunk);
                self.payload_length += added_len;
            }
        }

    }
    
    impl TryFrom<&[u8]> for P2PTransportPacket {
        type Error = Errors;

        fn try_from(bytes: &[u8]) -> Result<Self, Self::Error> {
            let header_length = bytes.get(0).unwrap_or(&0).to_owned() as usize;

            if header_length < 8 {
                return Err(Errors::PayloadNotComplete);
            }
            
            let op_code = bytes.get(1).unwrap_or(&0).to_owned();
            let payload_length = BigEndian::read_u16(&bytes[2..4]) as usize;
            let sequence_number = BigEndian::read_u32(&bytes[4..8]);
            let tlvs_length = header_length - 8;
            let mut tlvs: Vec<TLV> = Vec::new();
    
            if tlvs_length > 0 {
                let tlvs_bytes = &bytes[8..8+tlvs_length];
                tlvs = extract_tlvs(tlvs_bytes, tlvs_length);
            }
    
            let mut payload = None;
            if payload_length > 0 {
                //info!("P2PPayload present!");
                payload = Some(P2PPayload::deserialize(&bytes[8+tlvs_length..bytes.len()], payload_length)?);
            } 
    
            return Ok(P2PTransportPacket{ header_length, op_code, payload_length, sequence_number, tlvs, payload});    
        }

    }

    impl FromStr for P2PTransportPacket {
        type Err = Errors;
    
        fn from_str(s: &str) -> Result<Self, Self::Err> {
            let bytes = s.as_bytes();
            return P2PTransportPacket::try_from(bytes);
        }
    }
    
    impl Display for P2PTransportPacket {
        
        fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
            let mut out: Vec<u8> = self.to_vec();
            let mut out_str = unsafe { from_utf8_unchecked(&out) }.to_string();
            return write!(f, "{}", out_str);
        }
    }