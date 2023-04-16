use core::fmt;
use std::{str::from_utf8_unchecked, fmt::Display};

use byteorder::{BigEndian, ByteOrder};
use log::info;

use crate::{models::{errors::Errors}};

use super::{tlv::{TLV, ValueType, extract_tlvs}, slp_payload::SlpPayload};


#[derive(Clone)]
pub struct P2PPayload {

    pub header_length: usize,
    pub tf_combination : u8,
    pub package_number: u16,
    pub session_id: u32,
    pub tlvs: Vec<TLV>,
    pub payload: Vec<u8>
}

impl fmt::Debug for P2PPayload {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.debug_struct("P2PPayload")
         .field("header_length", &self.header_length)
         .field("tf_combination", &self.tf_combination)
         .field("package_number", &self.package_number)
         .field("session_id", &self.session_id)
         .field("tlvs", &self.tlvs)
        // .field("payload_str",  unsafe {&from_utf8_unchecked(&self.payload.as_slice())})
         .field("payload_bytes", &self.payload)
         .finish()
    }
}

impl P2PPayload {

    pub fn new(tf_combination: u8, session_id: u32) -> Self{
        return P2PPayload{ header_length: 0, tf_combination, package_number: 0, session_id, tlvs: Vec::new(), payload: Vec::new() };
    }

    pub fn deserialize(bytes: &[u8], payload_length: usize) -> Result<Self, Errors> {
        let header_length = bytes.get(0).unwrap_or(&0).to_owned() as usize;

        if header_length < 8 {
            return Err(Errors::PayloadDeserializeError);
        }

        let tf_combination = bytes.get(1).unwrap_or(&0).to_owned();
        let package_number = BigEndian::read_u16(&bytes[2..4]);
        let session_id = BigEndian::read_u32(&bytes[4..8]);
        let tlvs_length = header_length - 8;
        let mut tlvs: Vec<TLV> = Vec::new();

        if tlvs_length > 0 {
            let tlvs_bytes = &bytes[8..8+tlvs_length];
            tlvs = extract_tlvs(tlvs_bytes, tlvs_length);
        }

        let mut payload_length_to_take = payload_length;
        if payload_length > bytes.len() {
            return Err(Errors::PayloadNotComplete);
        }

        let payload = bytes[8+tlvs_length..payload_length_to_take].to_owned();
        return Ok(P2PPayload{ header_length, tf_combination, package_number, session_id, tlvs, payload });
    }

    pub fn add_tlv(&mut self, tlv: TLV) {
        self.tlvs.push(tlv);
    }

    pub fn set_payload(&mut self, payload: Vec<u8>) {
        self.payload = payload;
    }

    pub fn is_chucked_packet(&self) -> bool {
       return self.get_missing_bytes_count() > 0;
    }

    pub fn get_tlv_for_type(&self, value_type: &ValueType) -> Option<&TLV> {
        for tlv in &self.tlvs {
            if tlv.is_type(value_type) {
                return Some(tlv);
            }
        }
        return None;
    }

    pub fn get_package_number(&self) -> u16{
        return self.package_number;
    }

    fn get_remaining_bytes_tlv(&self) -> Option<&TLV> {
        return self.get_tlv_for_type(&ValueType::SizeOfUntransferData);
    }

    pub fn get_missing_bytes_count(&self) -> u64 {
        if let Some(tlv) = self.get_remaining_bytes_tlv() {
            let remaining_bytes_count = BigEndian::read_u64(tlv.value.as_slice());
            return remaining_bytes_count;
        }
        return 0;
    }

    pub fn get_payload_as_slp(&self) -> Result<SlpPayload, Errors> {

        if !self.payload.is_empty() {
            if self.tf_combination <= 0x01 && self.session_id == 0x0000 {
                return SlpPayload::try_from(&self.payload);
            }
        }
        return Err(Errors::PayloadDoesNotContainsSLP);
    }

    pub fn get_payload_bytes(&self) -> &Vec<u8> {
        return &self.payload;
    }

    pub fn is_file_transfer(&self) -> bool {
        info!("is file transfer");
        if !self.payload.is_empty() {
            info!("tf: {}, session_id: {}", &self.tf_combination, &self.session_id);

            if self.tf_combination == 6 || self.tf_combination == 7 && self.session_id > 0 {
                return true;
            }
        }
        return false;
    }

    pub fn append(&mut self, payload: &mut P2PPayload) -> usize {
        let added_size = payload.payload.len();
        self.payload.append(payload.payload.as_mut());
        return added_size;
    }

    pub fn append_raw(&mut self, payload: &[u8]) -> usize {
        let added_size = payload.len();
        self.payload.extend_from_slice(&payload);
        return added_size;
    }
 
}

impl Display for P2PPayload {

    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        let mut out: Vec<u8> = Vec::new();

        out.push(self.tf_combination.clone());

        let mut buffer : [u8;2] = [0,0];
        BigEndian::write_u16(&mut buffer, self.package_number);
        out.append(&mut buffer.to_vec());

        let mut buffer : [u8;4] = [0,0,0,0];
        BigEndian::write_u32(&mut buffer, self.session_id);
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

        out.insert(0, (out.len() + 1) as u8);

        out.append(&mut self.payload.clone());

        let out_str = unsafe { from_utf8_unchecked(&out) };
        return write!(f, "{}", out_str);

    }
}

#[cfg(test)]
mod tests {
    use crate::models::p2p::factories::TLVFactory;


#[test]
fn test() {
    //137 With header
  let test = [65, 65, 65, 65, 65, 65, 65, 65, 65, 65, 65, 65, 65, 65, 65, 65, 65, 65, 65, 65, 65, 65, 65, 65, 65, 65, 65, 65, 65, 65, 65, 65, 65, 65, 65, 65, 65, 65, 65, 65, 65, 65, 65, 65, 65, 65, 65, 65, 65, 65, 65, 65, 65, 65, 65, 65, 65, 65, 65, 65, 65, 65, 65, 65, 65, 65, 65, 65, 65, 65, 65, 65, 65, 65, 65, 65, 65, 65, 65, 65, 65, 65, 65, 65, 65, 65, 65, 65, 65, 65, 65, 65, 65, 65, 65, 65, 65, 65, 65, 65, 65, 65, 65, 65, 65, 65, 65, 65, 65, 65, 65, 65, 65, 65, 65, 65, 65, 65, 65, 65, 65, 65, 61, 61, 13, 10, 13, 10, 0];
  //TLV { length: 8, value_type: 1, value: [0, 0, 0, 0, 0, 0, 0, 129] }]

  let tlv = TLVFactory::get_untransfered_data_size(129);

  println!("tlv: {:?}", &tlv);


}

}