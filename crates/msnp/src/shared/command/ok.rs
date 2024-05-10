use std::fmt::Display;

use crate::msnp::error::CommandError;
use crate::msnp::raw_command_parser::RawCommand;
use crate::shared::traits::MSNPCommand;

pub struct OkCommand {
    pub operand: String,
    pub tr_id: u128
}

impl Display for OkCommand {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{operand} {tr_id} OK\r\n", operand = self.operand, tr_id = self.tr_id)
    }
}

impl MSNPCommand for OkCommand {
    type Err = CommandError;

    fn try_from_raw(raw: RawCommand) -> Result<Self, Self::Err> {
        todo!()
    }

    fn into_bytes(self) -> Vec<u8> {
        self.to_string().into_bytes()    }
}