use std::collections::VecDeque;
use crate::msnp::raw_command_parser::RawCommand;
use crate::shared::payload::msg::raw_msg_payload::RawMsgPayload;

pub trait MSNPPayload {
    type Err;
    fn try_from_bytes(bytes: Vec<u8>) -> Result<Self, Self::Err> where Self : Sized;
    fn into_bytes(self) -> Vec<u8>;
}

pub trait MSGPayload {
    type Err;
    fn try_from_raw(raw_msg_payload: RawMsgPayload) -> Result<Self, Self::Err> where Self : Sized;
    fn into_bytes(self) -> Vec<u8>;
}

pub trait MSNPCommand {
    type Err;
    fn try_from_raw(raw: RawCommand) -> Result<Self, Self::Err> where Self : Sized;
    fn into_bytes(self) -> Vec<u8>;
}

pub trait MSNPCommandPart {
    type Err;
    fn try_from_split(split: VecDeque<String>, command: &str) -> Result<Self, Self::Err> where Self : Sized;
}

pub trait IntoBytes {
    fn into_bytes(self) -> Vec<u8>;

}