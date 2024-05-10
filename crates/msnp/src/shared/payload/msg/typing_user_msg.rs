use std::str::FromStr;
use anyhow::anyhow;
use crate::msnp::error::PayloadError;
use crate::shared::models::email_address::EmailAddress;
use crate::shared::payload::msg::raw_msg_payload::factories::RawMsgPayloadFactory;
use crate::shared::payload::msg::raw_msg_payload::RawMsgPayload;
use crate::shared::payload::msg::text_msg::TextMessageContent;
use crate::shared::traits::{MSGPayload, MSNPPayload};

pub struct TypingUserMessageContent {

    pub typing_user: EmailAddress

}

impl MSGPayload for TypingUserMessageContent {
    type Err = PayloadError;

    fn try_from_raw(mut raw_msg_payload: RawMsgPayload) -> Result<Self, Self::Err> where Self: Sized {

        if "text/x-msmsgscontrol" != &raw_msg_payload.content_type {
            return Err(PayloadError::PayloadPropertyParseError {
                property_name: "Content-Type".to_string(),
                raw_value: format!("{:?}", raw_msg_payload),
                payload_type: "MSG".to_string(),
                source: anyhow!("Content Type doesnt match expectation for this type of message"),
            });
        }

        let raw_typing_user = raw_msg_payload.headers.remove("TypingUser").ok_or(PayloadError::MandatoryPartNotFound{ name: "TypingUser".to_string(), payload: "".to_string() })?;
        let typing_user = EmailAddress::from_str(&raw_typing_user)?;

        Ok(Self { typing_user })

    }

    fn into_bytes(self) -> Vec<u8> {
        let raw = RawMsgPayloadFactory::get_typing_user(self.typing_user.as_str());
        raw.into_bytes()
    }
}