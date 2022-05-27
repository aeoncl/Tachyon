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
