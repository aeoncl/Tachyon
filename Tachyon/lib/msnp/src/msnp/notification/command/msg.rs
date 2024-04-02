use std::str::FromStr;
use crate::msnp::error::PayloadError;
use crate::shared::payload::raw_msg_payload::RawMsgPayload;
use crate::shared::traits::SerializeMsnp;

pub struct MsgServer {

    pub sender: String,
    pub display_name: String,
    pub payload: MsgPayload
}

impl SerializeMsnp for MsgServer {
    fn serialize_msnp(&self) -> Vec<u8> {
        todo!()
    }
}

pub enum MsgPayload {
    InitialProfile(InitialProfilePayload),
    InitialMailData
}

impl SerializeMsnp for MsgPayload {
    fn serialize_msnp(&self) -> Vec<u8> {
        todo!()
    }
}

pub struct InitialProfilePayload {
    raw: RawMsgPayload
}

impl FromStr for InitialProfilePayload {
    type Err = PayloadError;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        let raw = RawMsgPayload::from_str(s)?;
        Ok(Self {
            raw
        })
    }
}
