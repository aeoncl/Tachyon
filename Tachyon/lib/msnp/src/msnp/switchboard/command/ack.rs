use crate::shared::traits::SerializeMsnp;

// SB >> ACK 2
pub struct AckServer {
    tr_id: u128
}

impl SerializeMsnp for AckServer {
    fn serialize_msnp(&self) -> Vec<u8> {
        format!("ACK {}\r\n", self.tr_id).into_bytes()
    }
}