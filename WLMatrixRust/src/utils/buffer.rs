use log::info;

use crate::models::p2p::tlv::TLV;




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