use strum_macros::{Display, EnumString};

use crate::msnp::error::CommandError;
use crate::msnp::raw_command_parser::RawCommand;
use crate::shared::payload::nfy::nfy_put_payload::RawNfyPayload;
use crate::shared::traits::{MSNPCommand, MSNPPayload};

pub struct NfyServer {
    pub operation: NfyOperation,
    pub payload: RawNfyPayload
}


#[derive(EnumString, Display)]
pub enum NfyOperation {
    #[strum(serialize = "PUT")]
    Put,
    #[strum(serialize = "DEL")]
    Del
}

impl MSNPCommand for NfyServer {
    type Err = CommandError;

    fn try_from_raw(raw: RawCommand) -> Result<Self, Self::Err> where Self: Sized {
        todo!()
    }

    fn into_bytes(self) -> Vec<u8> {
        let mut payload = self.payload.into_bytes();

        let mut out = format!("NFY {} {}\r\n", self.operation, payload.len()).into_bytes();
        out.append(&mut payload);

        out
    }
}