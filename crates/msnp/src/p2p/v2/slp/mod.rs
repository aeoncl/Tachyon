use std::convert::Infallible;
use std::fmt::{write, Display, Formatter};
use std::str::FromStr;
use anyhow::anyhow;
use lazy_static_include::syn::parse::End;
use linked_hash_map::LinkedHashMap;
use log::warn;
use crate::msnp::error::PayloadError;
use crate::p2p::v2::slp::raw_slp_payload::{RawSlpPayload, TryFromRawSlpPayload};
use crate::p2p::v2::slp::session_slp_payload::{SessionClosePayload, SessionInviteRequestPayload, SessionInviteResponsePayload};
use crate::p2p::v2::slp::SlpPayload::{SessionInviteRequest, SessionInviteResponse};
use crate::p2p::v2::slp::transport_slp_payload::{TransportInviteResponseErrorPayload, TransportInviteRequestPayload, TransportInviteResponseSuccessPayload, TransportDestinationAddressUpdatePayload};
use crate::shared::models::endpoint_id::EndpointId;
use crate::shared::models::uuid::{Error, Uuid};
use crate::shared::traits::TryFromBytes;

pub mod session_slp_context;
pub mod raw_slp_payload;
pub mod session_slp_payload;
pub mod transport_slp_payload;
pub mod app_id;

pub enum SlpPayload {
    SessionInviteRequest(SessionInviteRequestPayload),
    SessionInviteResponse(SessionInviteResponsePayload),
    SessionClose(SessionClosePayload),
    TransportInviteRequest(TransportInviteRequestPayload),
    TransportInviteResponseSuccess(TransportInviteResponseSuccessPayload),
    TransportInviteResponseError(TransportInviteResponseErrorPayload),
    TransportDestinationAddressUpdate(TransportDestinationAddressUpdatePayload),
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
                    SessionInviteRequest(SessionInviteRequestPayload::try_from_raw_slp_payload(raw)?)
                } else if raw.first_line.starts_with("MSNSLP/1.0") {
                    SessionInviteResponse(SessionInviteResponsePayload::try_from_raw_slp_payload(raw)?)
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
            },
            "application/x-msnmsgr-transdestaddrupdate" => {
                todo!()
            },
            _ => {
                SlpPayload::Raw(raw)
            }
        };

        Ok(payload)

    }
}

pub struct ViaHeader(Uuid);

impl ViaHeader {

    pub fn new(uuid: Uuid) -> Self {
        Self(uuid)
    }

    pub fn random() -> Self {
        Self(Uuid::new())
    }

}

impl FromStr for ViaHeader {
    type Err = PayloadError;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        let branch_uuid = match s.strip_prefix("MSNSLP/1.0/TLP ;branch=") {
            None => {
                Err(anyhow!("Slp `Via` header did not start with `MSNSLP/1.0/TLP ;branch=`. raw header value: `{}`", s))
            }
            Some(branch_uuid_raw) => {
                let branch_uuid_trimmed = branch_uuid_raw.trim().trim_start_matches("{").trim_end_matches("}");
                match Uuid::from_str(branch_uuid_trimmed) {
                    Ok(uuid) => {
                        Ok(uuid)
                    }
                    Err(err) => {
                        Err(anyhow!("Invalid `Via` header branch uuid: {}. error: {}", branch_uuid_trimmed, err))
                    }
                }
            }
        }?;

        Ok(Self::new(branch_uuid))
    }
}

impl Display for ViaHeader {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        write!(f, "MSNSLP/1.0/TLP ;branch={{{branch_uuid}}}", branch_uuid = self.0)
    }
}

pub struct SlpHeaders {
    receiver: EndpointId,
    sender: EndpointId,
    via: ViaHeader,
    c_seq: u32,
    call_id: Uuid,
    max_forwards: u32,
}

impl SlpHeaders {

    pub fn new(receiver: EndpointId, sender: EndpointId, via: ViaHeader, c_seq: u32, call_id: Uuid, max_forwards: u32) -> Self {
        Self {
            receiver,
            sender,
            via,
            c_seq,
            call_id,
            max_forwards,
        }
    }

    pub fn new_minimal(receiver: EndpointId, sender: EndpointId, c_seq: u32) -> Self {
        Self {
            receiver,
            sender,
            via: ViaHeader::random(),
            c_seq,
            call_id: Default::default(),
            max_forwards: 0,
        }
    }

    pub fn try_from_headers(mut headers: LinkedHashMap<String, String>) -> Result<Self,PayloadError> {
        let to_raw =headers.remove("To").ok_or(anyhow!("Missing slp `To` header"))?;
        let to_raw_trimmed = to_raw[9..to_raw.len() - 1].to_string();
        let to = EndpointId::from_str(&to_raw_trimmed).map_err(|e|anyhow!("Invalid slp `To` header: {} error: {}", &to_raw, e))?;

        let from_raw =headers.remove("From").ok_or(anyhow!("Missing slp `From` header"))?;
        let from_raw_trimmed = from_raw[9..from_raw.len() - 1].to_string();
        let from = EndpointId::from_str(&from_raw_trimmed).map_err(|e|anyhow!("Invalid slp `From` header: {} error: {}", &from_raw, e))?;

        let via = {
            let raw_via = headers.remove("Via").ok_or(anyhow!("Missing slp `Via` header"))?;
            ViaHeader::from_str(&raw_via)?
        };

        let c_seq = headers.remove("CSeq").ok_or(anyhow!("Missing slp `CSeq` header"))?.parse::<u32>()?;

        let call_id = {
            let mut raw_call_id = headers.remove("Call-ID").ok_or(anyhow!("Missing slp `Call-ID` header"))?;
            let mut trimmed = raw_call_id.trim().strip_prefix("{").unwrap_or(raw_call_id.as_str());
            trimmed = trimmed.strip_suffix("}").unwrap_or(trimmed);
            Uuid::from_str(trimmed).map_err(|e|anyhow!("Invalid `Call-ID` header: {}. error: {}", &raw_call_id, e))?
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

impl Into<LinkedHashMap<String, String>> for SlpHeaders {
    fn into(self) -> LinkedHashMap<String, String> {
       let mut out = LinkedHashMap::new();
        out.insert("To".to_string(), format!("<msnmsgr:{endpoint_id}>", endpoint_id = self.receiver.to_string()));
        out.insert("From".to_string(), format!("<msnmsgr:{endpoint_id}>", endpoint_id = self.sender.to_string()));
        out.insert("Via".to_string(), self.via.to_string());
        out.insert("CSeq".to_string(), self.c_seq.to_string());
        out.insert("Call-ID".to_string(), format!("{{{}}}", self.call_id.to_string()));
        out.insert("Max-Forwards".to_string(), self.max_forwards.to_string());
        out
    }
}

pub enum SlpStatus {
    Ok,
    NotFound,
    Err,
    Decline,
    UnknownYet(String),
}

impl Display for SlpStatus {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        match self {
            SlpStatus::Ok => write!(f, "200 OK"),
            SlpStatus::NotFound => write!(f, "404 Not Found"),
            SlpStatus::Err => write!(f, "500 Internal Error"),
            SlpStatus::Decline => write!(f, "603 Decline"),
            SlpStatus::UnknownYet(unknown) => { write!(f, "{}", unknown)}

        }
    }
}

impl FromStr for SlpStatus {
    type Err = Infallible;

    fn from_str(s: &str) -> Result<Self, <SlpStatus as FromStr>::Err> {
        match s.trim() {
            "200 OK" => Ok(SlpStatus::Ok),
            "404 Not Found" => Ok(SlpStatus::NotFound),
            "500 Internal Error" => Ok(SlpStatus::Err),
            "603 Decline" => Ok(SlpStatus::Decline),
            _ => Ok(SlpStatus::UnknownYet(s.to_string())),
        }
    }
}