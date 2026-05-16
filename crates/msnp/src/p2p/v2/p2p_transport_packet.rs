use core::fmt;
use std::fmt::{Debug, Display, Formatter};
use std::str::{from_utf8_unchecked, FromStr};

use crate::{msnp::error::PayloadError, shared::traits::IntoBytes};
use anyhow::anyhow;
use byteorder::{BigEndian, ByteOrder};

use super::{
    factories::TLVFactory,
    opcode::OperationCode,
    raw_p2p_payload::RawP2PPayload,
    tlv::{TLVList, ValueType},
};

/* P2PHeaderV2 */
/* For the official client to use P2PV2 headers, firstly, our fake client must have the P2PV2 Extended Capability
AND our fake client must be MPOP enabled. (which means adding endpoint data in NLN UBX payload AND making join the endpoint in switchboards. */
#[derive(Clone)]
pub struct P2PTransportPacket {
    pub header_length: usize,
    op_code: TransportOperationCode,
    pub payload_length: usize,
    pub sequence_number: u32,
    pub tlvs: TLVList,
    payload: Option<RawP2PPayload>,
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

    pub fn new(sequence_number: u32, payload: Option<RawP2PPayload>) -> Self {
        return P2PTransportPacket {
            header_length: 0,
            op_code: TransportOperationCode::default(),
            payload_length: 0,
            sequence_number,
            tlvs: TLVList::new(),
            payload,
        };
    }

    pub fn get_payload(&self) -> Option<&RawP2PPayload> {
        return self.payload.as_ref();
    }

    pub fn get_payload_as_mut(&mut self) -> Option<&mut RawP2PPayload> {
        return self.payload.as_mut();
    }

    pub fn set_payload(&mut self, payload: Option<RawP2PPayload>) {
        self.payload = payload;
    }

    pub fn get_next_sequence_number(&self) -> u32 {
        return self.sequence_number + self.payload_length as u32;
    }

    pub fn get_sequence_number(&self) -> u32 {
        return self.sequence_number;
    }

    pub fn get_payload_length(&self) -> u32 {
        if let Some(payload) = &self.payload {
            return payload.serialized_len() as u32;
        }
        return 0;
    }

    pub fn is_rak(&self) -> bool {
        self.op_code.is_rak()
    }

    pub fn is_syn(&self) -> bool {
        self.op_code.is_syn()
    }

    pub fn op_code(&self) -> TransportOperationCode {
        self.op_code.clone()
    }

    pub fn add_tlv(&mut self, tlv: super::tlv::TLV) {
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

    pub fn set_syn(&mut self, client_info: super::tlv::TLV) {
        self.op_code().set_syn();
        self.tlvs.push(client_info);
    }

    pub fn set_ack(&mut self, sequence_number: u32) {
        let ack_tlv = TLVFactory::get_ack(sequence_number);
        self.tlvs.push(ack_tlv);
    }

    pub fn set_rak(&mut self) {
        self.op_code().set_rak()
    }

    pub fn get_next_ack_sequence_number(&self) -> Option<u32> {
        if self.is_rak() {
            if let Some(ack_tlv) = self.get_ack_tlv() {
                let seq_number = BigEndian::read_u32(ack_tlv.value.as_slice());
                return Some(seq_number + self.payload_length as u32);
            }
        }
        return None;
    }

    pub fn get_ack_tlv(&self) -> Option<&super::tlv::TLV> {
        self.tlvs.get_ack()
    }

    pub fn get_client_info_tlv(&self) -> Option<&super::tlv::TLV> {
        self.tlvs.get_client_info()
    }

    pub fn get_tlv_for_type(&self, value_type: &ValueType) -> Option<&super::tlv::TLV> {
        self.tlvs.get_for_type(value_type)
    }

    pub fn is_payload_chunked(&self) -> bool {
        if let Some(payload) = self.payload.as_ref() {
            return payload.is_chunked_packet();
        }
        return false;
    }

    pub fn get_payload_package_number(&self) -> Option<u16> {
        if let Some(payload) = self.payload.as_ref() {
            return Some(payload.package_number.to_owned());
        }
        return None;
    }

    //FIXME make this consume chunk to avoid copying payload
    pub fn append_chunk(&mut self, chunk: &P2PTransportPacket) {
        if let Some(chunk_payload) = chunk.get_payload() {
            let mut chunk_payload = chunk_payload.to_owned();
            if let Some(payload) = self.payload.as_mut() {
                let added_len = payload.append(&mut chunk_payload);
                self.payload_length += added_len;
            }
        }
    }

    pub fn append_raw_chunk(&mut self, chunk: &[u8]) {
        if let Some(payload) = self.payload.as_mut() {
            let added_len = payload.append_raw(&chunk);
            self.payload_length += added_len;
        }
    }
}

impl TryFrom<&[u8]> for P2PTransportPacket {
    type Error = PayloadError;

    fn try_from(bytes: &[u8]) -> Result<Self, Self::Error> {
        let header_length = bytes.get(0).unwrap_or(&0).to_owned() as usize;

        if header_length < 8 {
            return Err(PayloadError::BinaryPayloadParsingError {
                payload: bytes.to_owned(),
                source: anyhow!(
                    "Header of P2PTransport packet must be of size 8, but was: {}",
                    &header_length
                ),
            });
        }

        let op_code = bytes.get(1).unwrap_or(&0).to_owned().into();
        let payload_length = BigEndian::read_u16(&bytes[2..4]) as usize;
        let sequence_number = BigEndian::read_u32(&bytes[4..8]);
        let tlvs_length = header_length - 8;
        let tlvs = if tlvs_length > 0 {
            TLVList::from_bytes(&bytes[8..8 + tlvs_length])
        } else {
            TLVList::new()
        };

        let mut payload = None;
        if payload_length > 0 {
            //info!("P2PPayload present!");
            payload = Some(RawP2PPayload::deserialize(
                &bytes[8 + tlvs_length..bytes.len()],
                payload_length,
            )?);
        }

        return Ok(P2PTransportPacket {
            header_length,
            op_code,
            payload_length,
            sequence_number,
            tlvs,
            payload,
        });
    }
}

impl IntoBytes for P2PTransportPacket {
    fn into_bytes(self) -> Vec<u8> {
        let mut out: Vec<u8> = Vec::new();
        out.push(self.op_code.into());

        // Placeholder for payload_length
        out.push(0);
        out.push(0);

        let mut buffer: [u8; 4] = [0, 0, 0, 0];
        BigEndian::write_u32(&mut buffer, self.sequence_number);
        out.extend_from_slice(&buffer);

        // TLV block (includes padding)
        out.extend_from_slice(&self.tlvs.to_bytes());

        // header_length = everything so far + 1 (for the header_length byte itself)
        out.insert(0, (out.len() + 1) as u8);

        // Serialize the Data Layer payload and write its length into the header
        if let Some(payload) = self.payload {
            let payload_bytes = payload.into_bytes();
            let payload_length = payload_bytes.len() as u16;

            let mut buffer: [u8; 2] = [0, 0];
            BigEndian::write_u16(&mut buffer, payload_length);
            out[2] = buffer[0];
            out[3] = buffer[1];

            out.extend(payload_bytes);
        }

        // Footer (4 zero bytes for switchboard transport)
        out.extend_from_slice(&[0u8; 4]);

        out
    }
}

impl FromStr for P2PTransportPacket {
    type Err = PayloadError;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        let bytes = s.as_bytes();
        return P2PTransportPacket::try_from(bytes);
    }
}

#[derive(Clone)]
pub struct TransportOperationCode(u8);

impl Default for TransportOperationCode {
    fn default() -> Self {
        Self(0)
    }
}

impl Into<u8> for TransportOperationCode {
    fn into(self) -> u8 {
        self.0
    }
}

impl From<u8> for TransportOperationCode {
    fn from(value: u8) -> Self {
        Self(value)
    }
}

impl Debug for TransportOperationCode {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        write!(f, "TransportOperationCode({flag}", flag = self.0)?;
        if self.is_syn() {
            write!(f, " | SYN")?;
        }

        if self.is_rak() {
            write!(f, "| RAK")?;
        }

        write!(f, ")")
    }
}

impl TransportOperationCode {
    pub fn set_syn(&mut self) {
        self.0 |= OperationCode::Syn as u8;
    }

    pub fn set_rak(&mut self) {
        self.0 |= OperationCode::RequestForAck as u8;
    }

    pub fn is_rak(&self) -> bool {
        let has_rak_flag = &self.0 & OperationCode::RequestForAck as u8;
        return has_rak_flag == OperationCode::RequestForAck as u8;
    }

    pub fn is_syn(&self) -> bool {
        let is_syn_flag = &self.0 & OperationCode::Syn as u8;
        return is_syn_flag == OperationCode::Syn as u8;
    }
}
