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
        let mut payload = self.payload.serialize_msnp();
        let cmd = format!("MSG {} {} {}\r\n", self.sender, self.display_name, payload.len());

        let mut out = Vec::with_capacity(cmd.len()+payload.len());
        out.extend_from_slice(cmd.as_bytes());
        out.append(&mut payload);

        out
    }
}

pub enum MsgPayload {
    Raw(RawMsgPayload),
}

impl SerializeMsnp for MsgPayload {
    fn serialize_msnp(&self) -> Vec<u8> {
        match self {
            MsgPayload::Raw(payload) => { payload.serialize_msnp() }
        }
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
