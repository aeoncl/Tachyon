use crate::msnp::error::CommandError;
use crate::msnp::raw_command_parser::RawCommand;
use crate::shared::traits::MSNPCommand;

// SB >> ACK 2
pub struct NakServer {
    tr_id: u128
}

impl NakServer {
    pub fn new(tr_id: u128) -> Self {
        Self { tr_id }
    }
}

impl MSNPCommand for NakServer {
    type Err = CommandError;

    fn try_from_raw(_raw: RawCommand) -> Result<Self, Self::Err> {
        todo!()
    }

    fn into_bytes(self) -> Vec<u8> {
        format!("NAK {}\r\n", self.tr_id).into_bytes()
    }
}