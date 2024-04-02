use std::fmt::Display;

use crate::shared::traits::SerializeMsnp;

pub struct OkCommand {
    pub operand: String,
    pub tr_id: u128
}

impl Display for OkCommand {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{operand} {tr_id} OK\r\n", operand = self.operand, tr_id = self.tr_id)
    }
}

impl SerializeMsnp for OkCommand {

    fn serialize_msnp(&self) -> Vec<u8> {
        self.to_string().into_bytes()
    }
}