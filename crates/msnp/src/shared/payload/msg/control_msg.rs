use std::str::FromStr;
use crate::msnp::error::PayloadError;
use crate::shared::models::email_address::EmailAddress;
use crate::shared::payload::msg::raw_msg_payload::factories::RawMsgPayloadFactory;
use crate::shared::payload::msg::raw_msg_payload::RawMsgPayload;
use crate::shared::traits::{IntoBytes, TryFromRawMsgPayload};

pub struct ControlMessagePayload {
    pub typing_user: EmailAddress
}

impl ControlMessagePayload {
    pub fn new(typing_user: EmailAddress) -> Self {
        Self { typing_user }
    }
}

impl IntoBytes for ControlMessagePayload {
    fn into_bytes(self) -> Vec<u8> {
        RawMsgPayloadFactory::get_typing_user(&self.typing_user).into_bytes()
    }
}


impl TryFromRawMsgPayload for ControlMessagePayload {
    type Err = PayloadError;

    fn try_from_raw(raw_msg_payload: RawMsgPayload) -> Result<Self, Self::Err>
    where
        Self: Sized
    {
        let typing_user = raw_msg_payload.get_header("TypingUser").ok_or(PayloadError::MandatoryPartNotFound { name: "TypingUser".to_string(), payload: "MSG".to_string() })?.to_string();
        let typing_user_email = EmailAddress::from_str(&typing_user)?;

        Ok(Self {
            typing_user: typing_user_email,
        })
    }
}