use std::str::FromStr;
use crate::msnp::error::CommandError;
use crate::msnp::raw_command_parser::RawCommand;
use crate::shared::payload::nfy::nfy_put_payload::RawNfyPayload;
use crate::shared::traits::{TryFromRawCommand, TryFromBytes, IntoBytes};

pub struct PutClient {
    pub tr_id: u128,
    pub payload: RawNfyPayload
}

impl PutClient {
    pub fn get_ok_command(&self) -> PutServer {
        PutServer{
            tr_id: self.tr_id,
        }
    }
}

impl TryFromRawCommand for PutClient {
    type Err = CommandError;

    fn try_from_raw(raw: RawCommand) -> Result<Self, Self::Err> where Self: Sized {
        let mut split = raw.command_split;
        let _operand = split.pop_front();

        let raw_tr_id = split.pop_front().ok_or(CommandError::MissingArgument(raw.command.clone(), "tr_id".into(), 1))?;
        let tr_id = u128::from_str(&raw_tr_id)?;

        let payload = RawNfyPayload::try_from_bytes(raw.payload)?;

        Ok(PutClient {
            tr_id,
            payload,
        })
    }

}

impl IntoBytes for PutClient {

    fn into_bytes(self) -> Vec<u8> {
        let mut payload = self.payload.into_bytes();

        let mut out = format!("PUT {} {}\r\n", self.tr_id, payload.len()).into_bytes();

        out.append(&mut payload);

        out
    }
}


//Todo handle errors ?
pub struct PutServer {
    tr_id: u128
}

impl TryFromRawCommand for PutServer {
    type Err = CommandError;

    fn try_from_raw(_raw: RawCommand) -> Result<Self, Self::Err> where Self: Sized {
        todo!()
    }

}

impl IntoBytes for PutServer {
    fn into_bytes(self) -> Vec<u8> {
        format!("PUT {} OK 0\r\n", self.tr_id).into_bytes()
    }
}