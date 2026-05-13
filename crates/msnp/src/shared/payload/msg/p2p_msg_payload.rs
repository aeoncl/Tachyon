use crate::msnp::error::PayloadError;
use crate::p2p::v2::p2p_transport_packet::P2PTransportPacket;
use crate::shared::models::endpoint_id::EndpointId;
use crate::shared::payload::msg::raw_msg_payload::{MsgContentType, RawMsgPayload};
use crate::shared::traits::{IntoBytes, IntoRawMsgPayload, TryFromRawMsgPayload};
use anyhow::anyhow;
use std::str::FromStr;

pub struct P2PMessagePayload {
    pub sender: EndpointId,
    pub receiver: EndpointId,
    pub payload: P2PTransportPacket,
    pub sender_display_name: Option<String>,
}

impl P2PMessagePayload {
    pub fn new(sender: EndpointId, receiver: EndpointId, payload: P2PTransportPacket, sender_display_name: Option<String>) -> Self {
        Self {
            sender,
            receiver,
            payload,
            sender_display_name,
        }
    }
}

impl TryFromRawMsgPayload for P2PMessagePayload {
    type Err = PayloadError;

    fn try_from_raw(mut raw: RawMsgPayload) -> Result<Self, Self::Err>
    where
        Self: Sized
    {
        if MsgContentType::P2P != raw.get_content_type().unwrap() {
            return Err(PayloadError::PayloadPropertyParseError {
                property_name: "Content-Type".to_string(),
                raw_value: format!("{:?}", raw),
                payload_type: "MSG".to_string(),
                source: anyhow!("Content Type doesnt match expectation for this type of message"),
            });
        }

        let raw_dest = raw.headers.remove("P2P-Dest").ok_or(PayloadError::MandatoryPartNotFound { name: "P2P-Dest".to_string(), payload: "".to_string() })?;
        let raw_src = raw.headers.remove("P2P-Src").ok_or(PayloadError::MandatoryPartNotFound { name: "P2P-Src".to_string(), payload: "".to_string() })?;

        let dest = EndpointId::from_str(&raw_dest).map_err(|e| PayloadError::PayloadPropertyParseError {
            property_name: "P2P-Dest".to_string(),
            raw_value: raw_dest.to_string(),
            payload_type: "P2P".to_string(),
            source: anyhow!(e),
        } )?;

        let src = EndpointId::from_str(&raw_src).map_err(|e| PayloadError::PayloadPropertyParseError {
            property_name: "P2P-Src".to_string(),
            raw_value: raw_dest.to_string(),
            payload_type: "P2P".to_string(),
            source: anyhow!(e),
        } )?;

        let display_name = raw.headers.remove("P4-Context");

        let payload  = P2PTransportPacket::try_from(raw.body.iter().as_slice())?;

        Ok(P2PMessagePayload {
            sender: src,
            receiver: dest,
            payload,
            sender_display_name: display_name,
        })
    }
}

impl IntoRawMsgPayload for P2PMessagePayload {
    fn into_raw(self) -> RawMsgPayload {
        let mut out = RawMsgPayload::new(MsgContentType::P2P, false);
        out.add_header("P2P-Dest", &self.receiver.to_string());
        out.add_header("P2P-Src", &self.sender.to_string());

        if let Some(display_name) = self.sender_display_name {
            out.add_header("P4-Context", display_name.as_str())
        }

        out.set_body(self.payload.to_string().into_bytes());
        out
    }
}

impl IntoBytes for P2PMessagePayload {
    fn into_bytes(self) -> Vec<u8> {
        self.into_raw().into_bytes()
    }
}