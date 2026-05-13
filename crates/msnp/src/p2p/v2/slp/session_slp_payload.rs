use std::str::FromStr;
use anyhow::anyhow;
use base64::Engine;
use base64::engine::general_purpose;
use num_traits::FromPrimitive;
use crate::msnp::error::PayloadError;
use crate::msnp::switchboard::models::session_id::SessionId;
use crate::p2p::v2::slp::app_id::AppID;
use crate::p2p::v2::slp::raw_slp_payload::{EufGUID, RawSlpPayload, TryFromRawSlpPayload};
use crate::p2p::v2::slp::session_slp_context::{PreviewData, SlpContext};
use crate::p2p::v2::slp::{SlpHeaders, SlpStatus};
use crate::shared::models::msn_object::MsnObject;


pub struct SessionReqInvitePayload {
    headers: SlpHeaders,
    session_id: u32,
    app_id: AppID,
    request_flags: u32,
    context: SessionReqInviteContext,
}

impl TryFromRawSlpPayload for SessionReqInvitePayload {
    type Err = PayloadError;

    fn try_from_raw_slp_payload(mut payload: RawSlpPayload) -> Result<Self, Self::Err> {

        let headers = SlpHeaders::try_from_headers(payload.headers)?;

        let session_id = payload.body.remove("SessionID").ok_or(anyhow!("Missing slp `SessionID` body"))?.parse::<u32>()?;

        let app_id: AppID = {
            let raw_app_id = payload.body.remove("AppID").ok_or(anyhow!("Missing slp `AppID` body"))?.parse::<u32>()?;
            FromPrimitive::from_u32(raw_app_id).ok_or(anyhow!("Unknown slp `AppID` body: {}", raw_app_id))?
        };

        let request_flags = payload.body.remove("RequestFlags").ok_or(anyhow!("Missing slp `RequestFlags` body"))?.parse::<u32>()?;

        let euf_guid = {
            let raw_euf_guid = payload.body.remove("EUF-GUID").ok_or(anyhow!("Missing slp `EUF-GUID` body"))?;
            EufGUID::try_from(raw_euf_guid).map_err(|_| anyhow!("Unknown `EUF-GUID`: {}", raw_euf_guid))?
        };

        let context = {
            let raw_context = payload.body.remove("Context").ok_or(anyhow!("Missing slp `Context` body"))?;

            match euf_guid {
                EufGUID::MSNObject => {
                    let decoded = general_purpose::STANDARD.decode(raw_context).map_err(|_| anyhow!("Invalid base64 `MsnObject` context body: {}", raw_context))?;
                    let utf8_decoded = String::from_utf8(decoded).map_err(|_| anyhow!("Invalid UTF8 `MsnObject` context body: {}", raw_context))?;
                    SessionReqInviteContext::MsnObject(MsnObject::from_str(&utf8_decoded)?)
                }
                EufGUID::FileTransfer => {
                    let decoded = general_purpose::STANDARD.decode(raw_context).map_err(|_| anyhow!("Invalid base64 `FileTransfer` context body: {}", raw_context))?;
                    SessionReqInviteContext::FileTransfer(PreviewData::from_slp_context(&decoded))
                }
                EufGUID::MediaReceiveOnly => {
                    SessionReqInviteContext::MediaReceiveOnly
                }
                EufGUID::MediaSession => {
                    SessionReqInviteContext::MediaSession
                }
                EufGUID::SharePhoto => {
                    SessionReqInviteContext::SharePhoto
                }
                EufGUID::Activity => {
                    SessionReqInviteContext::Activity
                }
            }
        };

        Ok(Self {
            headers,
            session_id,
            app_id,
            request_flags,
            context,
        })
    }
}

impl SessionReqInvitePayload {
    pub fn euf_guid(&self) -> EufGUID {
        self.context.into()
    }
}


pub struct SessionResponseInvitePayload {
    headers: SlpHeaders,
    status: SlpStatus,
    session_id: u32,
}

impl TryFromRawSlpPayload for SessionResponseInvitePayload {
    type Err = PayloadError;

    fn try_from_raw_slp_payload(payload: RawSlpPayload) -> Result<Self, Self::Err> {
        todo!()
    }
}


pub struct SessionClosePayload {
    headers: SlpHeaders,
    session_id: u32,
}

impl TryFromRawSlpPayload for SessionClosePayload {
    type Err = PayloadError;

    fn try_from_raw_slp_payload(payload: RawSlpPayload) -> Result<Self, Self::Err> {
        todo!()
    }
}


pub enum SessionReqInviteContext {
    MsnObject(MsnObject),
    FileTransfer(PreviewData),
    MediaReceiveOnly,
    MediaSession,
    SharePhoto,
    Activity,
}

impl Into<EufGUID> for SessionReqInviteContext {
    fn into(self) -> EufGUID {
        match self {
            SessionReqInviteContext::MsnObject(_) => { EufGUID::MSNObject }
            SessionReqInviteContext::FileTransfer(_) => { EufGUID::FileTransfer }
            SessionReqInviteContext::MediaReceiveOnly => { EufGUID::MediaReceiveOnly }
            SessionReqInviteContext::MediaSession => { EufGUID::MediaSession }
            SessionReqInviteContext::SharePhoto => { EufGUID::SharePhoto }
            SessionReqInviteContext::Activity => { EufGUID::Activity }
        }
    }
}

