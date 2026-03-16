use crate::msnp::error::CommandError;
use crate::msnp::raw_command_parser::RawCommand;
use crate::shared::traits::{IntoBytes, TryFromRawCommand};

// SB >> ACK 2
pub struct AckServer {
    tr_id: u128
}

impl AckServer {
    pub fn new(tr_id: u128) -> Self {
        Self { tr_id }
    }
}

impl TryFromRawCommand for AckServer {
    type Err = CommandError;

    fn try_from_raw(_raw: RawCommand) -> Result<Self, Self::Err> {
        todo!()
    }

}

impl IntoBytes for AckServer {
    fn into_bytes(self) -> Vec<u8> {
        format!("ACK {}\r\n", self.tr_id).into_bytes()
    }

}