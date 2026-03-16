use std::collections::VecDeque;
use crate::msnp::raw_command_parser::RawCommand;
use crate::shared::payload::msg::raw_msg_payload::RawMsgPayload;

pub trait TryFromBytes {
    type Err;
    fn try_from_bytes(bytes: Vec<u8>) -> Result<Self, Self::Err>
    where
        Self: Sized;
}

pub trait TryFromRawMsgPayload {
    type Err;
    fn try_from_raw(raw_msg_payload: RawMsgPayload) -> Result<Self, Self::Err>
    where
        Self: Sized;
}

pub trait TryFromRawCommand {
    type Err;
    fn try_from_raw(raw: RawCommand) -> Result<Self, Self::Err>
    where
        Self: Sized;
}

pub trait TryFromSplit {
    type Err;
    fn try_from_split(split: VecDeque<String>, command: &str) -> Result<Self, Self::Err>
    where
        Self: Sized;
}

pub trait IntoBytes {
    fn into_bytes(self) -> Vec<u8>;
}
