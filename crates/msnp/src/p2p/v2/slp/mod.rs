use std::convert::Infallible;
use std::fmt::{write, Display, Formatter};
use std::str::FromStr;
use anyhow::anyhow;
use linked_hash_map::LinkedHashMap;
use log::warn;
use crate::msnp::error::PayloadError;
use crate::p2p::v2::slp::raw_slp_payload::{RawSlpPayload, TryFromRawSlpPayload};
use crate::p2p::v2::slp::session_slp_payload::{SessionClosePayload, SessionReqInvitePayload, SessionResponseInvitePayload};
use crate::p2p::v2::slp::SlpPayload::{SessionReqInvite, SessionResponseInvite};
use crate::p2p::v2::slp::transport_slp_payload::{TransportErrorResponseInvitePayload, TransportReqInvitePayload, TransportSuccessResponseInvitePayload};
use crate::shared::models::endpoint_id::EndpointId;
use crate::shared::models::uuid::Uuid;
use crate::shared::traits::TryFromBytes;

pub mod session_slp_context;
pub mod raw_slp_payload;
pub mod session_slp_payload;
pub mod transport_slp_payload;
pub mod app_id;

pub enum SlpPayload {
    SessionReqInvite(SessionReqInvitePayload),
    SessionResponseInvite(SessionResponseInvitePayload),
    SessionClose(SessionClosePayload),
    TransportReqInvite(TransportReqInvitePayload),
    TransportSuccessResponseInvite(TransportSuccessResponseInvitePayload),
    TransportErrorResponseInvite(TransportErrorResponseInvitePayload),
    Raw(RawSlpPayload),
}

impl TryFromBytes for SlpPayload {
    type Err = PayloadError;

    fn try_from_bytes(bytes: Vec<u8>) -> Result<Self, Self::Err>
    where
        Self: Sized
    {
        let utf8_sanitized = String::from_utf8(bytes)?;

        let raw = RawSlpPayload::from_str(&utf8_sanitized)?;

        let content_type = raw.get_content_type().ok_or(anyhow!("Missing slp `Content-Type` header"))?.trim();

        let payload = match content_type {
            "application/x-msnmsgr-sessionreqbody" => {
                if raw.first_line.starts_with("INVITE") {
                    SessionReqInvite(SessionReqInvitePayload::try_from_raw_slp_payload(raw)?)
                } else if raw.first_line.starts_with("MSNSLP/1.0") {
                    SessionResponseInvite(SessionResponseInvitePayload::try_from_raw_slp_payload(raw)?)
                } else {
                    warn!("Unknown SLP sessionreqbody");
                    SlpPayload::Raw(raw)
                }
            },
            "application/x-msnmsgr-sessionclosebody" => {
                SlpPayload::SessionClose(SessionClosePayload::try_from_raw_slp_payload(raw)?)
            },
            "application/x-msnmsgr-transreqbody" => {
                todo!()
            },
            "application/x-msnmsgr-transrespbody" => {
                todo!()
            }
            _ => {
                SlpPayload::Raw(raw)
            }
        };

        Ok(payload)

    }
}

pub struct SlpHeaders {
    receiver: EndpointId,
    sender: EndpointId,
    via: String,
    c_seq: u32,
    call_id: Uuid,
    max_forwards: u32,
}

impl SlpHeaders {
    pub fn try_from_headers(mut headers: LinkedHashMap<String, String>) -> Result<Self,PayloadError> {
        let from_raw =headers.remove("From").ok_or(anyhow!("Missing slp `From` header"))?;
        let from_raw_trimmed = from_raw[9..from_raw.len() - 1].to_string();
        let from = EndpointId::from_str(&from_raw_trimmed).map_err(anyhow!("Invalid slp `From` header: {}", &from_raw))?;

        let to_raw =headers.remove("To").ok_or(anyhow!("Missing slp `To` header"))?;
        let to_raw_trimmed = to_raw[9..from_raw.len() - 1].to_string();
        let to = EndpointId::from_str(&to_raw_trimmed).map_err(anyhow!("Invalid slp `To` header: {}", &from_raw))?;

        let via = headers.remove("Via").ok_or(anyhow!("Missing slp `Via` header"))?;

        let c_seq = headers.remove("CSeq").ok_or(anyhow!("Missing slp `CSeq` header"))?.parse::<u32>()?;

        let call_id = {
            let mut raw_call_id = headers.remove("Call-ID").ok_or(anyhow!("Missing slp `Call-ID` header"))?.as_str();
            raw_call_id = raw_call_id.trim().strip_prefix("{").unwrap_or(raw_call_id);
            raw_call_id = raw_call_id.strip_suffix("}").unwrap_or(raw_call_id);
            Uuid::from_str(raw_call_id).map_err(anyhow!("Invalid `Call-ID` header: {}", &raw_call_id))?
        };

        let max_forwards = headers.remove("Max-Forwards").ok_or(anyhow!("Missing slp `Max-Forwards` header"))?.parse::<u32>()?;

        Ok(SlpHeaders {
            receiver: to,
            sender: from,
            via,
            c_seq,
            call_id,
            max_forwards,
        })
    }
}

pub enum SlpStatus {
    Ok,
    Err,
    UnknownYet(String),
}

impl Display for SlpStatus {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        match self {
            SlpStatus::Ok => write!(f, "200 OK"),
            SlpStatus::Err => write!(f, "500 Internal Error"),
            SlpStatus::UnknownYet(unknown) => { write!(f, "{}", unknown)}
        }
    }
}

impl FromStr for SlpStatus {
    type Err = Infallible;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        match s.trim() {
            "200 OK" => Ok(SlpStatus::Ok),
            "500 Internal Error" => Ok(SlpStatus::Err),
            _ => Ok(SlpStatus::UnknownYet(s.to_string())),
        }
    }
}