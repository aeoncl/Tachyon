use std::str::FromStr;
use anyhow::anyhow;
use num_derive::FromPrimitive;
use num_traits::FromPrimitive;
use crate::msnp::error::PayloadError;
use crate::shared::models::msn_object::MsnObject;
use crate::shared::payload::msg::raw_msg_payload::factories::RawMsgPayloadFactory;
use crate::shared::payload::msg::raw_msg_payload::{MsgContentType, RawMsgPayload};
use crate::shared::traits::{MSGPayload, MSNPPayload};

pub struct DatacastMessageContent {
    data: Datacast
}

impl DatacastMessageContent {
    pub fn get_type(&self) -> DatacastType {
        self.data.get_type()
    }
}

impl MSGPayload for DatacastMessageContent {
    type Err = PayloadError;

    fn try_from_raw(mut raw_msg_payload: RawMsgPayload) -> Result<Self, Self::Err> where Self: Sized {
        if MsgContentType::Datacast != raw_msg_payload.get_content_type().unwrap() {
            return Err(PayloadError::PayloadPropertyParseError {
                property_name: "Content-Type".to_string(),
                raw_value: format!("{:?}", raw_msg_payload),
                payload_type: "MSG".to_string(),
                source: anyhow!("Content Type doesnt match expectation for this type of message"),
            });
        }

        let raw_datacast_type = u8::from_str(&raw_msg_payload.headers.remove("ID").ok_or(PayloadError::MandatoryPartNotFound{ name: "ID".to_string(), payload: "".to_string() })?)?;
        let datacast_type =  DatacastType::from_u8(raw_datacast_type).ok_or(anyhow!("Unknown datacast type: {}", raw_datacast_type))?;

        let content = match datacast_type {
            DatacastType::Nudge => {
                DatacastMessageContent {
                    data: Datacast::Nudge,
                }
            }
            DatacastType::MsnObject => {
                DatacastMessageContent {
                    data: Datacast::MsnObject(MsnObject::from_str(raw_msg_payload.get_body_as_str()?)?),
                }
            }
            DatacastType::ActionMsg => {
                DatacastMessageContent {
                    data: Datacast::ActionMsg(String::from_utf8(raw_msg_payload.body)?)
                }
            }
        };

        Ok(content)
    }

    fn into_bytes(self) -> Vec<u8> {
        match self.data {
            Datacast::Nudge => {
                RawMsgPayloadFactory::get_nudge().into_bytes()
            }
            Datacast::MsnObject(obj) => {
                RawMsgPayloadFactory::get_msnobj_datacast(&obj).into_bytes()
            }
            Datacast::ActionMsg(msg) => {
                RawMsgPayloadFactory::get_action_msg(msg, false).into_bytes()
            }
        }
    }
}

pub enum Datacast {
    Nudge,
    MsnObject(MsnObject),
    ActionMsg(String)
}

#[derive(FromPrimitive, PartialEq, Eq)]
pub enum DatacastType {
    Nudge = 1,
    MsnObject = 3,
    ActionMsg = 4,
}


impl Datacast {
    pub fn get_type(&self) -> DatacastType{
        match self {
            Datacast::Nudge => {
                DatacastType::Nudge
            }
            Datacast::MsnObject(_) => {
                DatacastType::MsnObject
            }
            Datacast::ActionMsg(_) => {
                DatacastType::ActionMsg
            }
        }
    }

}