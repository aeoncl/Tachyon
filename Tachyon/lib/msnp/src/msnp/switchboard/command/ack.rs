use crate::msnp::error::CommandError;
use crate::msnp::raw_command_parser::RawCommand;
use crate::shared::traits::MSNPCommand;

// SB >> ACK 2
pub struct AckServer {
    tr_id: u128
}

impl MSNPCommand for AckServer {
    type Err = CommandError;

    fn try_from_raw(raw: RawCommand) -> Result<Self, Self::Err> {
        todo!()
    }

    fn to_bytes(self) -> Vec<u8> {
        format!("ACK {}\r\n", self.tr_id).into_bytes()
    }
}