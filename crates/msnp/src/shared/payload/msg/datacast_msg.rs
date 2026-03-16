use std::collections::HashMap;
use std::str::FromStr;
use anyhow::anyhow;
use log::{debug, warn};
use num_derive::FromPrimitive;
use num_traits::FromPrimitive;
use crate::msnp::error::PayloadError;
use crate::shared::models::msn_object::MsnObject;
use crate::shared::payload::msg::raw_msg_payload::factories::RawMsgPayloadFactory;
use crate::shared::payload::msg::raw_msg_payload::{MsgContentType, RawMsgPayload};
use crate::shared::traits::{TryFromRawMsgPayload, TryFromBytes, IntoBytes};

pub struct DatacastMessagePayload {
    pub data: Datacast
}

impl DatacastMessagePayload {
    pub fn get_type(&self) -> DatacastType {
        self.data.get_type()
    }
}

impl TryFromRawMsgPayload for DatacastMessagePayload {
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


        //FIXME: DATACAST ID IS IN BODY, not in HEADER
        /*
        16-03-2026T01:06:24.514 [DEBUG] - SB << | MSG 113 N 1325MIME-Version: 1.0
Content-Type: text/x-msnmsgr-datacast
Message-ID: {5CFDB40C-9737-4788-B186-9A22A350564B}
Chunks: 3

ID: 2
Data: <msnobj Creator="aeonshl@shlasouf.local"..etc
         */


        let body_str = raw_msg_payload.get_body_as_string()?;

        debug!("DatacastBody: Parsing body: {}", body_str);

        let body_split : Vec<&str> = body_str.split("\r\n").collect();

        debug!("body split: {:?}", &body_split);

        let mut body_map = {
            let mut body = HashMap::new();
            for current in body_split {
                match current.split_once(":") {
                    None => {
                        warn!("DatacastBody: Malformed header, ignoring...: {}", current);
                    }
                    Some((name, value)) => {
                        body.insert(name.trim(), value.trim());
                    }
                }
            }
            body
        };


        let raw_datacast_type = u8::from_str(body_map.remove("ID").ok_or(PayloadError::MandatoryPartNotFound{ name: "ID".to_string(), payload: "".to_string() })?)?;
        let datacast_type =  DatacastType::from_u8(raw_datacast_type).ok_or(anyhow!("Unknown datacast type: {}", raw_datacast_type))?;
        let data = body_map.remove("Data").ok_or(PayloadError::MandatoryPartNotFound{ name: "Data".to_string(), payload: "".to_string() })?;

        let content = match datacast_type {
            DatacastType::Nudge => {
                DatacastMessagePayload {
                    data: Datacast::Nudge,
                }
            }
            DatacastType::MsnObject => {
                DatacastMessagePayload {
                    data: Datacast::MsnObject(MsnObject::from_str(data)?),
                }
            }
            DatacastType::ActionMsg => {
                DatacastMessagePayload {
                    data: Datacast::ActionMsg(data.to_string())
                }
            }
            DatacastType::Wink => {
                DatacastMessagePayload {
                    data: Datacast::Wink(MsnObject::from_str(data)?),
                }
            }
        };

        Ok(content)
    }
}

impl IntoBytes for DatacastMessagePayload {
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
            Datacast::Wink(_) => {
                todo!()
            }
        }    }
}

pub enum Datacast {
    Nudge,
    Wink(MsnObject),
    MsnObject(MsnObject),
    ActionMsg(String)
}

#[derive(FromPrimitive, PartialEq, Eq, Debug)]
pub enum DatacastType {
    Nudge = 1,
    Wink = 2,
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
            Datacast::Wink(_) => {
                DatacastType::Wink
            }
        }
    }

}