#[derive(Clone, Debug)]
pub struct TLV {
    pub length: usize,
    pub value_type: u8,
    pub value: Vec<u8>
}

impl TLV {

    pub fn new(value_type: u8, length: usize, value: Vec<u8>) -> Self {
        return TLV{length, value_type, value};
    }

    pub fn empty() -> Self {
        return TLV::new(0, 0, Vec::new());
    }

    pub fn as_vec(&self) -> Vec<u8> {
        let mut out = Vec::new();
        out.push(self.value_type.clone());
        out.push(self.length.clone() as u8);
        out.append(&mut self.value.clone());

        return out;
    }

    pub fn is_type(&self, value_type: &ValueType) -> bool {
        match value_type {
            &ValueType::AckSequenceNumber => {
                return self.value_type == 0x02 && self.length == 0x04;
            },
            &ValueType::ClientPeerInfo => {
                return self.value_type == 0x01 && self.length == 0xc;
            },
            &ValueType::SizeOfUntransferData => {
                return self.value_type == 0x01 && self.length == 0x08;
            },
            _ => {
                return false;
            }
        }
    }

}

pub enum ValueType {
    SizeOfUntransferData,
    AckSequenceNumber,
    ClientPeerInfo,
    NakSequenceNumber 
}

pub fn extract_tlvs(tlvs_bytes: &[u8], tlvs_length: usize) -> Vec<TLV> {
    //  info!("TLV bytes: {:?}", &tlvs_bytes);
      let mut tlvs_treated_count = 0;
      let mut out = Vec::new();
      while tlvs_treated_count < tlvs_bytes.len(){
      //    info!("TLV Treated count {}", &tlvs_treated_count);
  
          let start_index = tlvs_treated_count;
          let value_type = tlvs_bytes.get(start_index).unwrap().to_owned();
  
          if value_type != 0 {
              let length = tlvs_bytes.get(start_index+1).unwrap().to_owned() as usize;
  
        //      info!("current tlv start_index: {}, type: {:x}, length: {:x}", &start_index, &value_type, &length);
  
              let payload_start_index = start_index+2;
              let payload_end_index = payload_start_index+length;
  
              let value = tlvs_bytes[payload_start_index..payload_end_index].to_owned();
        //      info!("current tlv value: {:?}", &value);
  
              out.push(TLV::new(value_type, length, value));
              tlvs_treated_count += length+2;
          } else {
              //we are done
              let padding_count = tlvs_length - tlvs_treated_count;
          //    info!("padding_bytes_count: {}", &padding_count);
              break;
          }
      }
          return out;
  }