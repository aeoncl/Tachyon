use crate::msnp::error::PayloadError;
use crate::p2p::v2::slp::app_id::AppID;
use crate::p2p::v2::slp::raw_slp_payload::{EufGUID, IntoRawSlpPayload, RawSlpPayload, TryFromRawSlpPayload};
use crate::p2p::v2::slp::session_slp_context::{PreviewData, SlpContext};
use crate::p2p::v2::slp::{SlpHeaders, SlpStatus};
use crate::shared::models::msn_object::MsnObject;
use anyhow::anyhow;
use base64::engine::general_purpose;
use base64::Engine;
use num_traits::FromPrimitive;
use std::fmt::{Display, Formatter};
use std::str::FromStr;


pub struct SessionInviteRequestPayload {
    headers: SlpHeaders,
    session_id: u32,
    app_id: AppID,
    request_flags: u32,
    context: SessionReqInviteContext,
}

impl SessionInviteRequestPayload {
    pub fn new(headers: SlpHeaders, session_id: u32, app_id: AppID, request_flags: u32, context: SessionReqInviteContext) -> Self {
        Self {
            headers,
            session_id,
            app_id,
            request_flags,
            context,
        }

    }

    pub fn new_file_transfer() {
        todo!()
    }

    pub fn headers(&self) -> &SlpHeaders {
        &self.headers
    }

    pub fn session_id(&self) -> u32 {
        self.session_id
    }

    pub fn app_id(&self) -> AppID {
        self.app_id.clone()
    }

    pub fn request_flags(&self) -> u32 {
        self.request_flags
    }

    pub fn context(&self) -> &SessionReqInviteContext {
        &self.context
    }
}

impl TryFromRawSlpPayload for SessionInviteRequestPayload {
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
            EufGUID::from_str(&raw_euf_guid).map_err(|e| anyhow!("Unknown `EUF-GUID`: {}. error: {}", raw_euf_guid, e))?
        };

        let context = {
            let raw_context = payload.body.remove("Context").ok_or(anyhow!("Missing slp `Context` body"))?;

            //Todo refactor this so that SessionReqInviteContext handles the conversion itself of each of it's branches.
            match euf_guid {
                EufGUID::MSNObject => {
                    let decoded = general_purpose::STANDARD.decode(raw_context).map_err(|e| anyhow!("Invalid base64 `MsnObject` context. error: {}", e))?;
                    let utf8_decoded = String::from_utf8(decoded).map_err(|e| anyhow!("Invalid UTF8 `MsnObject` context body. error: {}", e))?;
                    SessionReqInviteContext::MsnObject(MsnObject::from_str(&utf8_decoded)?)
                }
                EufGUID::FileTransfer => {
                    let decoded = general_purpose::STANDARD.decode(raw_context).map_err(|e| anyhow!("Invalid base64 `FileTransfer` contex body. error: {}", e))?;
                    SessionReqInviteContext::FileTransfer(PreviewData::from_slp_context(&decoded)
                        .ok_or(anyhow!("Invalid `PreviewData` context body: {:?}", decoded))?)
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

impl IntoRawSlpPayload for SessionInviteRequestPayload {
    fn into_raw_slp_payload(self) -> RawSlpPayload {
        let mut out = RawSlpPayload::new();
        out.first_line = format!("INVITE MSNMSGR:{} MSNSLP/1.0", &self.headers.receiver);

        out.add_body_property(String::from("EUF-GUID"), self.euf_guid().to_string());
        out.add_body_property(String::from("SessionID"), self.session_id.to_string());
        out.add_body_property(String::from("AppID"), format!("{}", self.app_id as u32));
        out.add_body_property(String::from("RequestFlags"), format!("{}", self.request_flags));
        out.add_body_property(String::from("Context"), self.context.to_string());

        out.headers = self.headers.into();
        out.add_header("Content-Type".to_string(), "application/x-msnmsgr-sessionreqbody".to_string());

        out
    }
}

impl SessionInviteRequestPayload {
    pub fn euf_guid(&self) -> EufGUID {
        self.context.euf_guid()
    }
}

pub struct SessionInviteResponsePayload {
    headers: SlpHeaders,
    status: SlpStatus,
    session_id: u32,
}

impl TryFromRawSlpPayload for SessionInviteResponsePayload {
    type Err = PayloadError;

    fn try_from_raw_slp_payload(mut payload: RawSlpPayload) -> Result<Self, Self::Err> {
        let headers = SlpHeaders::try_from_headers(payload.headers)?;

        let status = {
            let raw_status = payload.first_line.trim().strip_prefix("MSNSLP/1.0 ").ok_or(anyhow!("Unexpected SLP first line did not start with `MSNSLP/1.0 ` :, {}", payload.first_line))?;
            SlpStatus::from_str(raw_status).expect("to be infaillible")
        };

        let session_id = payload.body.remove("SessionID").ok_or(anyhow!("Missing slp `SessionID` body"))?.parse::<u32>()?;

        Ok(
            Self {
                headers,
                status,
                session_id,
            }
        )
    }
}

impl IntoRawSlpPayload for SessionInviteResponsePayload {
    fn into_raw_slp_payload(self) -> RawSlpPayload {
        let mut out = RawSlpPayload::new();
        out.first_line = format!("MSNSLP/1.0 {}", &self.status);

        out.add_body_property(String::from("SessionID"), self.session_id.to_string());

        out.headers = self.headers.into();
        out.add_header("Content-Type".to_string(), "application/x-msnmsgr-sessionreqbody".to_string());

        out
    }
}

pub struct SessionClosePayload {
    headers: SlpHeaders,
    session_id: u32,
}

impl TryFromRawSlpPayload for SessionClosePayload {
    type Err = PayloadError;

    fn try_from_raw_slp_payload(mut payload: RawSlpPayload) -> Result<Self, Self::Err> {
        let headers = SlpHeaders::try_from_headers(payload.headers)?;
        let session_id = payload.body.remove("SessionID").ok_or(anyhow!("Missing slp `SessionID` body"))?.parse::<u32>()?;

        Ok(Self {
            headers,
            session_id,
        })
    }
}

impl IntoRawSlpPayload for SessionClosePayload {
    fn into_raw_slp_payload(self) -> RawSlpPayload {
        let mut out = RawSlpPayload::new();
        out.first_line = format!("BYE MSNMSGR:{} MSNSLP/1.0", &self.headers.receiver);

        out.add_body_property(String::from("SessionID"), self.session_id.to_string());

        out.headers = self.headers.into();
        out.add_header("Content-Type".to_string(), "application/x-msnmsgr-sessionclosebody".to_string());

        out
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

impl SessionReqInviteContext {
    pub fn euf_guid(&self) -> EufGUID {
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

//TODO refactor this because right now some SLP handles base64 encoding, some don't.
impl Display for SessionReqInviteContext {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {

        match self {
            SessionReqInviteContext::MsnObject(object) => {
                let serialized = object.to_string();
                let base64 = general_purpose::STANDARD.encode(serialized);
                write!(f, "{}", base64)
            }
            SessionReqInviteContext::FileTransfer(preview_data) => {
                let base64 = preview_data.to_string();
                write!(f, "{}", base64)
            }
            SessionReqInviteContext::MediaReceiveOnly => {
                todo!("MediaReceiveOnly serialization not yet implemented")
            }
            SessionReqInviteContext::MediaSession => {
                todo!("MediaSession serialization not yet implemented")
            }
            SessionReqInviteContext::SharePhoto => {
                todo!("SharePhoto serialization not yet implemented")
            }
            SessionReqInviteContext::Activity => {
                todo!("Activity serialization not yet implemented")
            }
        }

    }
}

#[cfg(test)]
mod tests {
    use crate::p2p::v2::slp::app_id::AppID;
    use crate::p2p::v2::slp::raw_slp_payload::{IntoRawSlpPayload, SlpPayloadFactory, TryFromRawSlpPayload};
    use crate::p2p::v2::slp::session_slp_context::PreviewData;
    use crate::p2p::v2::slp::session_slp_payload::{SessionReqInviteContext, SessionInviteRequestPayload};
    use crate::p2p::v2::slp::{SlpHeaders, ViaHeader};
    use crate::shared::models::email_address::EmailAddress;
    use crate::shared::models::endpoint_id::EndpointId;
    use crate::shared::models::uuid::Uuid;
    use std::str::FromStr;

    #[test]
    fn serialization_test_2() {
        let file_transfer_expected = concat!(r#"INVITE MSNMSGR:aeon1@test.com;{EA020650-AE67-5B4A-8B99-72EB07A5DD84} MSNSLP/1.0
To: <msnmsgr:aeon1@test.com;{EA020650-AE67-5B4A-8B99-72EB07A5DD84}>
From: <msnmsgr:aeon@test.com;{1F8BFCBF-4F72-587D-8A18-489C017B448B}>
Via: MSNSLP/1.0/TLP ;branch={2F22593B-C23E-4F8B-BFD8-3FFCC76731C3}
CSeq: 0
Call-ID: {D7AFE190-7AA9-4453-9845-51CCF0ADF312}
Max-Forwards: 0
Content-Type: application/x-msnmsgr-sessionreqbody
Content-Length: 876

EUF-GUID: {5D3E02AB-6190-11D3-BBBB-00C04F795683}
SessionID: 123
AppID: 2
RequestFlags: 16
Context: PgIAAAIAAAB7AAAAAAAAAAEAAABiAGwAYQBiAGwAYQBiAGwAYQAuAGcAaQBmAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAA==

"#, "\0").replace("\n", "\r\n");

        let sender = EndpointId::from_email_addr(EmailAddress::from_str("aeon@test.com").unwrap());
        let receiver = EndpointId::from_email_addr(EmailAddress::from_str("aeon1@test.com").unwrap());

        let headers = SlpHeaders::new(receiver, sender, ViaHeader::new(Uuid::from_str("2F22593B-C23E-4F8B-BFD8-3FFCC76731C3").unwrap()), 0, Uuid::from_str("D7AFE190-7AA9-4453-9845-51CCF0ADF312").unwrap(), 0);
        let payload = SessionInviteRequestPayload::new(headers, 123, AppID::FileTransfer, 16, SessionReqInviteContext::FileTransfer(PreviewData::new(123, "blablabla.gif".to_string())));

        let payload_serialized = payload.into_raw_slp_payload().to_string();
        assert_eq!(file_transfer_expected, payload_serialized)
    }

    #[test]
    fn serialization_test() {
        let sender = EndpointId::from_email_addr(EmailAddress::from_str("aeon@test.com").unwrap());
        let receiver = EndpointId::from_email_addr(EmailAddress::from_str("aeon1@test.com").unwrap());

        let preview_data = PreviewData::new(123, "blablabla.gif".to_string());
        let session_id = 123;
        let call_id = Uuid::new();
        let factory_raw = SlpPayloadFactory::get_file_transfer_request(&sender, &receiver, &preview_data, session_id, &call_id).unwrap();

        let serialized = factory_raw.to_string();

        println!("{}", serialized);
        let model = SessionInviteRequestPayload::try_from_raw_slp_payload(factory_raw).unwrap();
        let model_serialized = model.into_raw_slp_payload().to_string();

        assert_eq!(serialized, model_serialized)
    }


}