use anyhow::anyhow;
use core::fmt;
use std::fmt::Display;
use std::ops::Sub;
use std::str::from_utf8_unchecked;
use byteorder::{BigEndian, ByteOrder};
use log::info;
use rand::random;
use crate::{msnp::error::PayloadError, shared::traits::IntoBytes};
use crate::p2p::v2::factories::TLVFactory;
use crate::p2p::v2::slp::raw_slp_payload::RawSlpPayload;
use super::tlv::{TLVList, ValueType};

#[derive(Clone)]
pub struct RawP2PPayload {
    pub header_length: usize,
    /// Upper nibble (bits 7–4) of the `tf` byte at offset 1.
    pub transfer_type: u8,
    /// Lower nibble (bits 3–0) of the `tf` byte at offset 1.
    pub flag: u8,
    pub package_number: u16,
    pub session_id: u32,
    pub tlvs: TLVList,
    pub payload: Vec<u8>,
}

impl fmt::Debug for RawP2PPayload {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.debug_struct("P2PPayload")
            .field("header_length", &self.header_length)
            .field(
                "tf_byte",
                &format!(
                    "0x{:02X} (T={}, F={})",
                    self.tf_byte(),
                    self.transfer_type,
                    self.flag
                ),
            )
            .field("package_number", &self.package_number)
            .field("session_id", &self.session_id)
            .field("tlvs", &self.tlvs)
            .field("payload_bytes", &self.payload)
            .finish()
    }
}

impl RawP2PPayload {
    pub fn new(transfer_type: u8, flag: u8, session_id: u32) -> Self {
        return RawP2PPayload {
            header_length: 0,
            transfer_type: transfer_type & 0x0F,
            flag: flag & 0x0F,
            package_number: 0,
            session_id,
            tlvs: TLVList::new(),
            payload: Vec::new(),
        };
    }

    /// Returns the combined `tf` byte reconstructed from
    /// `transfer_type` (upper nibble) and `flag` (lower nibble).
    pub fn tf_byte(&self) -> u8 {
        (self.transfer_type << 4) | (self.flag & 0x0F)
    }

    /// Deserializes a `P2PPayload` from raw bytes.
    ///
    /// `total_data_length` is the total Data Layer packet size as reported by the
    /// Transport Layer header (i.e. the `payload_length` field of `P2PTransportPacket`).
    /// It determines how many bytes of the provided slice belong to this payload
    /// (header + TLVs + body).
    pub fn deserialize(bytes: &[u8], total_data_length: usize) -> Result<Self, PayloadError> {
        if bytes.len() < 8 {
            return Err(PayloadError::BinaryPayloadParsingError {
                payload: bytes.to_owned(),
                source: anyhow!(
                    "P2PPayload input is too short: expected at least 8 bytes, got {}",
                    bytes.len()
                ),
            });
        }

        let header_length = bytes[0] as usize;

        if header_length < 8 {
            return Err(PayloadError::BinaryPayloadParsingError {
                payload: bytes.to_owned(),
                source: anyhow!(
                    "P2PPayload header length must be >= 8 but was {}",
                    header_length
                ),
            });
        }

        let tf_combination = bytes[1];
        let transfer_type = tf_combination >> 4;
        let flag = tf_combination & 0x0F;
        let package_number = BigEndian::read_u16(&bytes[2..4]);
        let session_id = BigEndian::read_u32(&bytes[4..8]);
        let tlvs_length = header_length - 8;
        let tlvs = if tlvs_length > 0 {
            TLVList::from_bytes(&bytes[8..8 + tlvs_length])
        } else {
            TLVList::new()
        };

        if total_data_length > bytes.len() {
            return Err(PayloadError::BinaryPayloadParsingError {
                payload: bytes.to_owned(),
                source: anyhow!(
                    "P2PPayload was chunked: total_data_length is {} but only {} bytes available",
                    total_data_length,
                    bytes.len()
                ),
            });
        }

        let payload = bytes[8 + tlvs_length..total_data_length].to_owned();
        Ok(RawP2PPayload {
            header_length,
            transfer_type,
            flag,
            package_number,
            session_id,
            tlvs,
            payload,
        })
    }

    pub fn add_tlv(&mut self, tlv: super::tlv::TLV) {
        self.tlvs.push(tlv);
    }

    pub fn set_payload(&mut self, payload: Vec<u8>) {
        self.payload = payload;
    }

    pub fn is_chunked_packet(&self) -> bool {
        self.get_missing_bytes_count() > 0
    }

    pub fn chunk(self, chunk_size: usize) -> Vec<RawP2PPayload> {
        let payload = self.payload;
        let transfer_type = self.transfer_type;
        let session_id = self.session_id;
        let package_number = random();
        let tlvs = self.tlvs;
        let mut remaining_bytes: u64 = payload.len() as u64;

        let mut out = Vec::with_capacity((remaining_bytes / chunk_size as u64) as usize);


        let chunk_count = payload.len().div_ceil(chunk_size);

        for (index, chunk) in payload.chunks(chunk_size).enumerate() {

            let flag: u8 = if index == 0 {
                1
            } else {
                0
            };

            let mut payload = RawP2PPayload::new(transfer_type, flag, session_id);
            remaining_bytes = remaining_bytes.saturating_sub(chunk_size as u64);

            if index == 0 {
                payload.tlvs = tlvs.clone();
            }

            if index < chunk_count - 1 {
                payload.add_tlv(TLVFactory::get_untransfered_data_size(remaining_bytes));

            }

            payload.package_number = package_number;
            payload.payload = chunk.to_vec();

            out.push(payload);
        }

        out
    }

    pub fn get_tlv_for_type(&self, value_type: &ValueType) -> Option<&super::tlv::TLV> {
        self.tlvs.get_for_type(value_type)
    }

    pub fn get_package_number(&self) -> u16 {
        return self.package_number;
    }

    fn get_remaining_bytes_tlv(&self) -> Option<&super::tlv::TLV> {
        self.tlvs.get_untransfered_data_size()
    }

    pub fn get_missing_bytes_count(&self) -> u64 {
        self.tlvs.get_missing_bytes_count()
    }

    pub fn get_payload_as_slp(&self) -> Result<RawSlpPayload, PayloadError> {
        if !self.payload.is_empty() {
            if self.flag <= 0x01 && self.session_id == 0x0000 {
                return RawSlpPayload::try_from(&self.payload);
            }
        }
        return Err(PayloadError::PayloadDoesNotContainsSLP);
    }

    pub fn get_payload_bytes(&self) -> &Vec<u8> {
        return &self.payload;
    }

    pub fn is_file_transfer(&self) -> bool {
        if !self.payload.is_empty() {
            if (self.flag == 6 || self.flag == 7) && self.session_id > 0 {
                return true;
            }
        }
        false
    }

    pub fn is_msn_obj_transfer(&self) -> bool {
        if !self.payload.is_empty() {
            if (self.flag == 4 || self.flag == 5) && self.session_id > 0 {
                return true;
            }
        }
        false
    }

    pub fn append(&mut self, payload: &mut RawP2PPayload) -> usize {
        let added_size = payload.payload.len();
        self.payload.append(payload.payload.as_mut());
        return added_size;
    }

    pub fn append_raw(&mut self, payload: &[u8]) -> usize {
        let added_size = payload.len();
        self.payload.extend_from_slice(&payload);
        return added_size;
    }

    /// Computes the total serialized byte length of this Data Layer packet
    /// without actually building the byte vector.
    ///
    /// Layout: header_length(1) + tf(1) + pkg_num(2) + sid(4) + tlvs + padding + payload
    pub fn serialized_len(&self) -> usize {
        8 + self.tlvs.serialized_len() + self.payload.len()
    }
}

impl IntoBytes for RawP2PPayload {
    fn into_bytes(self) -> Vec<u8> {
        let mut out: Vec<u8> = Vec::new();

        out.push(self.tf_byte());

        let mut buffer: [u8; 2] = [0, 0];
        BigEndian::write_u16(&mut buffer, self.package_number);
        out.extend_from_slice(&buffer);

        let mut buffer: [u8; 4] = [0, 0, 0, 0];
        BigEndian::write_u32(&mut buffer, self.session_id);
        out.extend_from_slice(&buffer);

        // TLV block (includes padding)
        out.extend_from_slice(&self.tlvs.to_bytes());

        // header_length = everything so far + 1 (for the header_length byte itself)
        out.insert(0, (out.len() + 1) as u8);

        out.extend(self.payload);

        out
    }
}


impl Display for RawP2PPayload {

    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        let mut out: Vec<u8> = Vec::new();

        out.push(self.tf_byte().clone());

        let mut buffer : [u8;2] = [0,0];
        BigEndian::write_u16(&mut buffer, self.package_number);
        out.append(&mut buffer.to_vec());

        let mut buffer : [u8;4] = [0,0,0,0];
        BigEndian::write_u32(&mut buffer, self.session_id);
        out.append(&mut buffer.to_vec());

        for tlv in &self.tlvs {
            let mut tlv_serialized : Vec<u8> = tlv.to_bytes();
            out.append(&mut tlv_serialized);
        }

        if !self.tlvs.is_empty() {
            let mut last = self.tlvs.iter().last().unwrap().clone();
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

        out.insert(0, (out.len() + 1) as u8);

        out.append(&mut self.payload.clone());

        let out_str = unsafe { from_utf8_unchecked(&out) };
        return write!(f, "{}", out_str);

    }
}

#[cfg(test)]
mod tests {
    use byteorder::{BigEndian, ByteOrder};

    use crate::shared::traits::IntoBytes;

    use super::RawP2PPayload;
    use crate::p2p::v2::{factories::TLVFactory, tlv::TLVList};

    /// Helper: serialize a P2PPayload to bytes, then deserialize it back.
    /// Asserts that key fields survive the roundtrip.
    fn roundtrip(payload: RawP2PPayload) -> RawP2PPayload {
        // into_bytes consumes self, so clone first to know the serialized length
        let clone = payload.clone();
        let clone_bytes = clone.into_bytes();
        let total_len = clone_bytes.len();

        let bytes = payload.into_bytes();
        assert_eq!(
            bytes.len(),
            total_len,
            "serialized length should be consistent"
        );

        RawP2PPayload::deserialize(&bytes, total_len)
            .expect("roundtrip deserialization should succeed")
    }

    // ---------------------------------------------------------------
    // 1. Minimal packet: no TLVs, no payload
    // ---------------------------------------------------------------
    #[test]
    fn roundtrip_minimal() {
        let original = RawP2PPayload::new(0, 0x01, 0x0000);

        let result = roundtrip(original);

        assert_eq!(
            result.header_length, 8,
            "header should be 8 bytes (fixed fields only, no TLVs)"
        );
        assert_eq!(result.transfer_type, 0);
        assert_eq!(result.flag, 0x01);
        assert_eq!(result.tf_byte(), 0x01);
        assert_eq!(result.package_number, 0);
        assert_eq!(result.session_id, 0x0000);
        assert!(result.tlvs.is_empty());
        assert!(result.payload.is_empty());
    }

    // ---------------------------------------------------------------
    // 2. Packet with payload but no TLVs (SLP-style)
    // ---------------------------------------------------------------
    #[test]
    fn roundtrip_with_payload_no_tlvs() {
        let mut original = RawP2PPayload::new(0, 0x01, 0x0000);
        original.set_payload(b"INVITE MSNMSGR:test@example.com MSNSLP/1.0\r\n".to_vec());

        let result = roundtrip(original);

        assert_eq!(result.header_length, 8);
        assert_eq!(result.tf_byte(), 0x01);
        assert_eq!(result.session_id, 0x0000);
        assert!(result.tlvs.is_empty());
        assert_eq!(
            result.payload,
            b"INVITE MSNMSGR:test@example.com MSNSLP/1.0\r\n"
        );
    }

    // ---------------------------------------------------------------
    // 3. Packet with one ACK TLV (6 bytes: T=1 + L=1 + V=4)
    //    6 bytes of TLV data → not 4-aligned.
    //    Padding: (4 - 6 % 4) % 4 = (4 - 2) % 4 = 2 padding bytes
    //    header_length = 1 + 7 + 6 + 2 = 16
    // ---------------------------------------------------------------
    #[test]
    fn roundtrip_with_ack_tlv() {
        let mut original = RawP2PPayload::new(0, 0x01, 0x0000);
        original.add_tlv(TLVFactory::get_ack(0x12345678));
        original.set_payload(b"hello".to_vec());

        let bytes = original.clone().into_bytes();
        let result = roundtrip(original);

        // ACK TLV is 2+4=6 bytes → padding = 2 → header = 1+7+6+2 = 16
        assert_eq!(result.header_length, 16);
        assert_eq!(result.tlvs.len(), 1);
        assert_eq!(result.tlvs[0].value_type, 0x02);
        assert_eq!(result.tlvs[0].length, 4);
        let ack_seq = BigEndian::read_u32(&result.tlvs[0].value);
        assert_eq!(ack_seq, 0x12345678);
        assert_eq!(result.payload, b"hello");

        // Verify the raw bytes have 2 padding zeros before the payload
        // header occupies bytes 0..16, payload starts at byte 16
        assert_eq!(bytes[14], 0x00, "padding byte 1 should be zero");
        assert_eq!(bytes[15], 0x00, "padding byte 2 should be zero");
    }

    // ---------------------------------------------------------------
    // 4. Packet with untransfered data size TLV (10 bytes: T=1 + L=1 + V=8)
    //    10 bytes → (4 - 10 % 4) % 4 = (4 - 2) % 4 = 2 padding bytes
    //    header_length = 1 + 7 + 10 + 2 = 20
    // ---------------------------------------------------------------
    #[test]
    fn roundtrip_with_untransfered_data_tlv() {
        let mut original = RawP2PPayload::new(0, 0x01, 0x0000);
        original.add_tlv(TLVFactory::get_untransfered_data_size(0xDEADBEEFCAFE));
        original.set_payload(b"AAAA".to_vec());

        let bytes = original.clone().into_bytes();
        let result = roundtrip(original);

        // TLV is 2+8=10 bytes → padding = 2 → header = 1+7+10+2 = 20
        assert_eq!(result.header_length, 20);
        assert_eq!(result.tlvs.len(), 1);
        assert_eq!(result.tlvs[0].value_type, 0x01);
        assert_eq!(result.tlvs[0].length, 8);
        let remaining = BigEndian::read_u64(&result.tlvs[0].value);
        assert_eq!(remaining, 0xDEADBEEFCAFE);
        assert_eq!(result.payload, b"AAAA");

        // Verify padding bytes before payload
        assert_eq!(bytes[18], 0x00, "padding byte 1 should be zero");
        assert_eq!(bytes[19], 0x00, "padding byte 2 should be zero");
    }

    // ---------------------------------------------------------------
    // 5. Binary payload with nulls and high bytes
    // ---------------------------------------------------------------
    #[test]
    fn roundtrip_binary_payload() {
        let mut original = RawP2PPayload::new(0, 0x07, 0xABCD1234);
        original.package_number = 42;
        original.set_payload(vec![0x00, 0xFF, 0x80, 0x01, 0x00, 0x00, 0xFE, 0x7F]);

        let result = roundtrip(original);

        assert_eq!(result.transfer_type, 0);
        assert_eq!(result.flag, 0x07);
        assert_eq!(result.tf_byte(), 0x07);
        assert_eq!(result.package_number, 42);
        assert_eq!(result.session_id, 0xABCD1234);
        assert_eq!(
            result.payload,
            vec![0x00, 0xFF, 0x80, 0x01, 0x00, 0x00, 0xFE, 0x7F]
        );
    }

    // ---------------------------------------------------------------
    // 6. Client peer info TLV (14 bytes: T=1 + L=1 + V=12)
    //    14 bytes → (4 - 14 % 4) % 4 = (4 - 2) % 4 = 2 padding bytes
    //    header_length = 1 + 7 + 14 + 2 = 24
    // ---------------------------------------------------------------
    #[test]
    fn roundtrip_with_peer_info_tlv() {
        let mut original = RawP2PPayload::new(0, 0x01, 0x0000);
        original.add_tlv(TLVFactory::get_client_peer_info());

        let bytes = original.clone().into_bytes();
        let result = roundtrip(original);

        // Peer info TLV is 2+12=14 bytes → padding = 2 → header = 1+7+14+2 = 24
        assert_eq!(result.header_length, 24);
        assert_eq!(result.tlvs.len(), 1);
        assert_eq!(result.tlvs[0].value_type, 0x01);
        assert_eq!(result.tlvs[0].length, 12);
        assert!(result.payload.is_empty());

        // Verify padding
        assert_eq!(bytes[22], 0x00, "padding byte 1 should be zero");
        assert_eq!(bytes[23], 0x00, "padding byte 2 should be zero");
    }

    // ---------------------------------------------------------------
    // 7. Two TLVs: ACK (6 bytes) + NAK (6 bytes) = 12 bytes
    //    12 bytes → already 4-aligned → 0 padding bytes
    //    header_length = 1 + 7 + 12 + 0 = 20
    // ---------------------------------------------------------------
    #[test]
    fn roundtrip_with_two_tlvs_aligned() {
        let mut original = RawP2PPayload::new(0, 0x01, 0x0000);
        original.add_tlv(TLVFactory::get_ack(100));
        original.add_tlv(TLVFactory::get_nak(200));

        let bytes = original.clone().into_bytes();
        let result = roundtrip(original);

        // Two TLVs: 6 + 6 = 12 → 4-aligned → no padding → header = 1+7+12+0 = 20
        assert_eq!(result.header_length, 20);
        assert_eq!(result.tlvs.len(), 2);

        assert_eq!(result.tlvs[0].value_type, 0x02);
        let ack_val = BigEndian::read_u32(&result.tlvs[0].value);
        assert_eq!(ack_val, 100);

        assert_eq!(result.tlvs[1].value_type, 0x03);
        let nak_val = BigEndian::read_u32(&result.tlvs[1].value);
        assert_eq!(nak_val, 200);

        // No padding — total length should be header only
        assert_eq!(bytes.len(), 20, "no padding, total = header only");
    }

    // ---------------------------------------------------------------
    // 8. Data preparation message (per spec): tf=0x01, session_id!=0, payload=4 null bytes
    // ---------------------------------------------------------------
    #[test]
    fn roundtrip_data_preparation_message() {
        let mut original = RawP2PPayload::new(0, 0x01, 0x6C99FBC2);
        original.set_payload(vec![0x00, 0x00, 0x00, 0x00]);

        let result = roundtrip(original);

        assert_eq!(result.tf_byte(), 0x01);
        assert_eq!(result.session_id, 0x6C99FBC2);
        assert_eq!(result.payload, vec![0x00, 0x00, 0x00, 0x00]);
        assert_eq!(
            result.payload.len(),
            4,
            "data prep payload should be exactly 4 null bytes"
        );
    }

    // ---------------------------------------------------------------
    // 9. Verify serialized header_length byte is correct for various TLV configs
    // ---------------------------------------------------------------
    #[test]
    fn serialized_header_length_byte_is_correct() {
        // No TLVs → header_length = 8
        let no_tlvs = RawP2PPayload::new(0, 0x01, 0);
        let bytes = no_tlvs.into_bytes();
        assert_eq!(bytes[0], 8, "header_length byte should be 8 when no TLVs");

        // ACK TLV (6 bytes) + 2 padding → header_length = 16
        let mut with_ack = RawP2PPayload::new(0, 0x01, 0);
        with_ack.add_tlv(TLVFactory::get_ack(1));
        let bytes = with_ack.into_bytes();
        assert_eq!(
            bytes[0], 16,
            "header_length byte should be 16 for ACK TLV + padding"
        );

        // Peer info TLV (14 bytes) + 2 padding → header_length = 24
        let mut with_peer = RawP2PPayload::new(0, 0x01, 0);
        with_peer.add_tlv(TLVFactory::get_client_peer_info());
        let bytes = with_peer.into_bytes();
        assert_eq!(
            bytes[0], 24,
            "header_length byte should be 24 for peer info TLV + padding"
        );

        // Two ACK TLVs (6+6=12, aligned) → header_length = 20
        let mut with_two = RawP2PPayload::new(0, 0x01, 0);
        with_two.add_tlv(TLVFactory::get_ack(1));
        with_two.add_tlv(TLVFactory::get_ack(2));
        let bytes = with_two.into_bytes();
        assert_eq!(
            bytes[0], 20,
            "header_length byte should be 20 for two ACK TLVs, no padding"
        );
    }

    // ---------------------------------------------------------------
    // 10. Deserialize matches protocol spec example 1 (data prep acknowledgement)
    //     From the spec: 08 01 00 00 6c 99 fb c2 00 00 00 00
    // ---------------------------------------------------------------
    #[test]
    fn deserialize_spec_example_1() {
        let raw: &[u8] = &[
            0x08, // header_length = 8
            0x01, // tf_combination = 1 (T=0, F=1)
            0x00, 0x00, // package_number = 0
            0x6C, 0x99, 0xFB, 0xC2, // session_id
            0x00, 0x00, 0x00, 0x00, // payload (4 null bytes = data prep)
        ];

        let result =
            RawP2PPayload::deserialize(raw, raw.len()).expect("should deserialize spec example 1");

        assert_eq!(result.header_length, 8);
        assert_eq!(result.transfer_type, 0);
        assert_eq!(result.flag, 0x01);
        assert_eq!(result.tf_byte(), 0x01);
        assert_eq!(result.package_number, 0);
        assert_eq!(result.session_id, 0x6C99FBC2);
        assert!(result.tlvs.is_empty());
        assert_eq!(result.payload, vec![0x00, 0x00, 0x00, 0x00]);
    }

    // ---------------------------------------------------------------
    // 11. Verify extract_tlvs correctly stops at padding zeros
    // ---------------------------------------------------------------
    #[test]
    fn extract_tlvs_with_padding() {
        // ACK TLV (02 04 XX XX XX XX) = 6 bytes, then 2 padding bytes
        let tlv_bytes: &[u8] = &[
            0x02, 0x04, 0x00, 0x01, 0x02, 0x03, // ACK TLV
            0x00, 0x00, // padding
        ];

        let tlvs = TLVList::from_bytes(tlv_bytes);

        assert_eq!(
            tlvs.len(),
            1,
            "should extract exactly 1 TLV, stopping at padding"
        );
        assert_eq!(tlvs[0].value_type, 0x02);
        assert_eq!(tlvs[0].length, 4);
        assert_eq!(tlvs[0].value, vec![0x00, 0x01, 0x02, 0x03]);
    }

    // ---------------------------------------------------------------
    // 12. serialized_len() matches into_bytes().len() for various configs
    // ---------------------------------------------------------------
    #[test]
    fn serialized_len_matches_no_tlvs_no_payload() {
        let p = RawP2PPayload::new(0, 0x01, 0);
        assert_eq!(p.serialized_len(), 8);
        assert_eq!(p.clone().into_bytes().len(), 8);
    }

    #[test]
    fn serialized_len_matches_with_payload_no_tlvs() {
        let mut p = RawP2PPayload::new(0, 0x01, 0);
        p.set_payload(vec![0xDE, 0xAD, 0xBE, 0xEF]);
        assert_eq!(p.serialized_len(), 12);
        assert_eq!(p.clone().into_bytes().len(), 12);
    }

    #[test]
    fn serialized_len_matches_with_ack_tlv() {
        let mut p = RawP2PPayload::new(0, 0x01, 0);
        p.add_tlv(TLVFactory::get_ack(42));
        p.set_payload(b"data".to_vec());
        // ACK TLV = 6 bytes, padding = 2 → header = 16, + 4 payload = 20
        assert_eq!(p.serialized_len(), 20);
        assert_eq!(p.clone().into_bytes().len(), 20);
    }

    #[test]
    fn serialized_len_matches_with_untransfered_data_tlv() {
        let mut p = RawP2PPayload::new(0, 0x01, 0);
        p.add_tlv(TLVFactory::get_untransfered_data_size(9999));
        // TLV = 10 bytes, padding = 2 → header = 20, + 0 payload = 20
        assert_eq!(p.serialized_len(), 20);
        assert_eq!(p.clone().into_bytes().len(), 20);
    }

    #[test]
    fn serialized_len_matches_with_peer_info_tlv() {
        let mut p = RawP2PPayload::new(0, 0x01, 0);
        p.add_tlv(TLVFactory::get_client_peer_info());
        p.set_payload(vec![0xAA; 100]);
        // TLV = 14 bytes, padding = 2 → header = 24, + 100 payload = 124
        assert_eq!(p.serialized_len(), 124);
        assert_eq!(p.clone().into_bytes().len(), 124);
    }

    #[test]
    fn serialized_len_matches_two_aligned_tlvs() {
        let mut p = RawP2PPayload::new(0, 0x01, 0);
        p.add_tlv(TLVFactory::get_ack(1));
        p.add_tlv(TLVFactory::get_nak(2));
        // Two TLVs = 12 bytes, already aligned → header = 20, + 0 payload = 20
        assert_eq!(p.serialized_len(), 20);
        assert_eq!(p.clone().into_bytes().len(), 20);
    }
}
