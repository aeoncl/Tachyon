/* Amazing documentation for the P2P stack by msnp-sharp project
 * https://code.google.com/archive/p/msnp-sharp/wikis/KB_MSNC12_BinaryHeader.wiki
 */

use anyhow::anyhow;
use std::{
    fmt::Display,
    str::{from_utf8, FromStr},
};

use base64::{engine::general_purpose, Engine};
use linked_hash_map::LinkedHashMap;
use log::warn;
use num::FromPrimitive;

use crate::shared::models::endpoint_id::EndpointId;
use crate::{
    msnp::error::PayloadError,
    shared::models::{msn_object::MsnObject, msn_user::MsnUser, uuid::Uuid},
};

use crate::p2p::v2::app_id::AppID;
use crate::p2p::v2::slp::slp_context::{PreviewData, SlpContext};

#[derive(Clone, Debug)]
pub struct SlpPayload {
    pub first_line: String,
    pub headers: LinkedHashMap<String, String>,
    pub body: LinkedHashMap<String, String>,
}

impl SlpPayload {
    pub fn new() -> Self {
        return SlpPayload {
            first_line: String::new(),
            headers: LinkedHashMap::new(),
            body: LinkedHashMap::new(),
        };
    }

    pub fn add_header(&mut self, name: String, value: String) {
        self.headers.insert(name, value);
    }

    pub fn get_header(&self, name: &String) -> Option<&String> {
        return self.headers.get(name);
    }

    pub fn add_body_property(&mut self, name: String, value: String) {
        self.body.insert(name, value);
    }

    pub fn get_body_property(&self, name: &str) -> Option<&str> {
        return self.body.get(name).map(|s| s.as_str());
    }

    pub fn get_content_type(&self) -> Option<&String> {
        return self.get_header(&String::from("Content-Type"));
    }

    //TODO Error handling here
    pub fn get_sender(&self) -> Option<MsnUser> {
        if let Some(from) = self.get_header(&String::from("From")) {
            let from_trimmed = from.to_owned()[9..from.len() - 1].to_string();

            let sender = MsnUser::new(EndpointId::from_str(&from_trimmed).unwrap());
            return Some(sender);
        }
        return None;
    }

    pub fn get_receiver(&self) -> Option<MsnUser> {
        if let Some(to) = self.get_header(&String::from("To")) {
            let to_trimmed = to.to_owned()[9..to.len() - 1].to_string();
            let receiver = MsnUser::new(EndpointId::from_str(&to_trimmed).unwrap());
            return Some(receiver);
        }
        return None;
    }

    pub fn get_context_as_preview_data(&self) -> Option<PreviewData> {
        if let Some(context) = self.get_body_property(&String::from("Context")) {
            if let Ok(decoded) = general_purpose::STANDARD.decode(context) {
                return PreviewData::from_slp_context(&decoded);
            } else {
                warn!("Couldn't decode base64 slp context: {}", context);
            }
        }

        return None;
    }

    pub fn get_euf_guid(&self) -> Result<Option<EufGUID>, PayloadError> {
        let euf_guid = self.get_body_property(&String::from("EUF-GUID"));
        if euf_guid.is_none() {
            return Ok(None);
        }

        let euf_guid = euf_guid.unwrap();
        let euf_guid = EufGUID::try_from(euf_guid)?;
        return Ok(Some(euf_guid));
    }

    pub fn get_app_id(&self) -> Result<Option<AppID>, PayloadError> {
        let app_id = self.get_body_property(&String::from("AppID"));
        if app_id.is_none() {
            return Ok(None);
        }

        let app_id = app_id.unwrap();
        let app_id = u32::from_str(app_id)?;
        let app_id: Option<AppID> = FromPrimitive::from_u32(app_id);
        return Ok(app_id);
    }

    pub fn get_call_id(&self) -> Result<Option<Uuid>, PayloadError> {
        let call_id = self.get_header(&"Call-ID".into());
        if call_id.is_none() {
            return Ok(None);
        }

        let mut call_id = call_id.unwrap().as_str();
        call_id = call_id.trim().strip_prefix("{").unwrap_or(call_id);

        call_id = call_id.strip_suffix("}").unwrap_or(call_id);

        return Ok(Some(Uuid::from_str(call_id).unwrap())); //TODO HANDLE ERROR CORRECTLY
    }

    pub fn get_context_as_msnobj(&self) -> Option<MsnObject> {
        let context = self.get_body_property("Context");
        match context {
            None => None,
            Some(context) => MsnObject::from_slp_context(&context.as_bytes()),
        }
    }

    pub fn is_invite(&self) -> bool {
        return self.first_line.contains("INVITE");
    }

    pub fn is_200_ok(&self) -> bool {
        return self.first_line.contains("200 OK");
    }
}

impl FromStr for SlpPayload {
    type Err = PayloadError;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        let (headers, body) =
            s.split_once("\r\n\r\n")
                .ok_or(PayloadError::StringPayloadParsingError {
                    payload: s.to_string(),
                    source: anyhow!("There was no header body boundary in Slp Payload"),
                })?;
        let mut out = SlpPayload::new();
        let headers_split: Vec<&str> = headers.split("\r\n").collect();

        out.first_line = headers_split
            .get(0)
            .ok_or(PayloadError::StringPayloadParsingError {
                payload: s.to_string(),
                source: anyhow!(
                    "Could not get the first line from Slp Payload: {:?}",
                    &headers_split
                ),
            })?
            .to_string();

        for i in 1..headers_split.len() {
            let current = headers_split
                .get(i)
                .ok_or(PayloadError::StringPayloadParsingError {
                    payload: s.to_string(),
                    source: anyhow!(
                        "Could not get header at index {} in headers: {:?}",
                        &i,
                        &headers_split
                    ),
                })?
                .to_string();

            if let Some((name, value)) = current.split_once(":") {
                out.add_header(name.trim().to_string(), value.trim().to_string());
            }
        }

        let body_split: Vec<&str> = body.split("\r\n").collect();
        for i in 0..body_split.len() {
            let current = body_split
                .get(i)
                .ok_or(PayloadError::StringPayloadParsingError {
                    payload: s.to_string(),
                    source: anyhow!(
                        "Could not get body element at index: {} for body: {:?}",
                        &i,
                        &body_split
                    ),
                })?
                .to_string();
            if let Some((name, value)) = current.split_once(":") {
                out.add_body_property(name.trim().to_string(), value.trim().to_string());
            }
        }

        Ok(out)
    }
}

impl TryFrom<&Vec<u8>> for SlpPayload {
    type Error = PayloadError;

    fn try_from(value: &Vec<u8>) -> Result<Self, Self::Error> {
        let str = from_utf8(value).map_err(|e| PayloadError::BinaryPayloadParsingError {
            payload: value.to_owned(),
            source: anyhow!(e),
        })?;
        return SlpPayload::from_str(str);
    }
}

impl Display for SlpPayload {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        let mut out = self.first_line.clone();
        out.push_str("\r\n");

        let mut body = String::new();
        for (key, value) in &self.body {
            body.push_str(format!("{key}: {value}\r\n", key = &key, value = &value).as_str());
        }
        body.push_str("\r\n");
        body.push_str("\0");

        let mut headers = String::new();

        for (key, value) in &self.headers {
            headers.push_str(format!("{key}: {value}\r\n", key = &key, value = &value).as_str());
        }

        headers.push_str(format!("Content-Length: {value}\r\n", value = body.len()).as_str());

        out.push_str(headers.as_str());
        out.push_str("\r\n");
        out.push_str(body.as_str());
        return write!(f, "{}", out);
    }
}

#[derive(PartialEq, Debug)]
pub enum EufGUID {
    MSNObject,
    FileTransfer,
    MediaReceiveOnly,
    MediaSession,
    SharePhoto,
    Activity,
}

impl Display for EufGUID {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        //let mut out = "{00000000-0000-0000-0000-000000000000}";
        let out = match self {
            EufGUID::MSNObject => "{A4268EEC-FEC5-49E5-95C3-F126696BDBF6}",
            EufGUID::FileTransfer => "{5D3E02AB-6190-11D3-BBBB-00C04F795683}",
            EufGUID::MediaReceiveOnly => "{1C9AA97E-9C05-4583-A3BD-908A196F1E92}",
            EufGUID::MediaSession => "{4BD96FC0-AB17-4425-A14A-439185962DC8}",
            EufGUID::SharePhoto => "{41D3E74E-04A2-4B37-96F8-08ACDB610874}",
            EufGUID::Activity => "{6A13AF9C-5308-4F35-923A-67E8DDA40C2F}",
        };
        return write!(f, "{}", &out);
    }
}

impl TryFrom<&str> for EufGUID {
    type Error = PayloadError;

    fn try_from(value: &str) -> Result<Self, Self::Error> {
        match value.to_uppercase().as_str() {
            "{A4268EEC-FEC5-49E5-95C3-F126696BDBF6}" => {
                return Ok(EufGUID::MSNObject);
            }
            "{5D3E02AB-6190-11D3-BBBB-00C04F795683}" => {
                return Ok(EufGUID::FileTransfer);
            }
            "{1C9AA97E-9C05-4583-A3BD-908A196F1E92}" => {
                return Ok(EufGUID::MediaReceiveOnly);
            }
            "{4BD96FC0-AB17-4425-A14A-439185962DC8}" => {
                return Ok(EufGUID::MediaSession);
            }
            "{41D3E74E-04A2-4B37-96F8-08ACDB610874}" => {
                return Ok(EufGUID::SharePhoto);
            }
            "{6A13AF9C-5308-4F35-923A-67E8DDA40C2F}" => {
                return Ok(EufGUID::Activity);
            }
            _ => {
                return Err(PayloadError::StringPayloadParsingError {
                    payload: value.to_string(),
                    source: anyhow!("Unknown EUF-GUID"),
                });
            }
        }
    }
}

pub struct SlpPayloadFactory;

impl SlpPayloadFactory {
    pub fn get_session_bye(
        sender: &MsnUser,
        receiver: &MsnUser,
        call_id: Uuid,
        session_id: String,
    ) -> Result<SlpPayload, PayloadError> {
        let mut out = SlpPayload::new();
        out.first_line = format!(
            "BYE MSNMSGR:{mpop_id} MSNSLP/1.0",
            mpop_id = receiver.endpoint_id
        );
        out.add_header(
            String::from("To"),
            format!("<msnmsgr:{mpop_id}>", mpop_id = receiver.endpoint_id),
        );
        out.add_header(
            String::from("From"),
            format!("<msnmsgr:{mpop_id}>", mpop_id = sender.endpoint_id),
        );
        out.add_header(
            String::from("Via"),
            format!(
                "MSNSLP/1.0/TLP ;branch={{{branch_uuid}}}",
                branch_uuid = Uuid::new().to_string()
            ),
        );
        out.add_header(String::from("CSeq"), String::from("0"));
        out.add_header(
            String::from("Call-ID"),
            format!("{{{call_id}}}", call_id = call_id.to_string()),
        );
        out.add_header(String::from("Max-Forwards"), String::from("0"));
        out.add_header(
            String::from("Content-Type"),
            String::from("application/x-msnmsgr-sessionclosebody"),
        );
        out.add_body_property(String::from("SessionID"), session_id);
        return Ok(out);
    }

    pub fn get_200_ok_session(invite: &SlpPayload) -> Result<SlpPayload, PayloadError> {
        let mut out = SlpPayload::new();
        out.first_line = String::from("MSNSLP/1.0 200 OK");
        out.add_header(
            String::from("To"),
            invite
                .get_header(&String::from("From"))
                .ok_or(PayloadError::MandatoryPartNotFound {
                    name: String::from("From"),
                    payload: format!("{:?}", &invite),
                })?
                .to_owned(),
        );

        out.add_header(
            String::from("From"),
            invite
                .get_header(&String::from("To"))
                .ok_or(PayloadError::MandatoryPartNotFound {
                    name: String::from("To"),
                    payload: format!("{:?}", &invite),
                })?
                .to_owned(),
        );

        out.add_header(
            String::from("Via"),
            invite
                .get_header(&String::from("Via"))
                .ok_or(PayloadError::MandatoryPartNotFound {
                    name: String::from("Via"),
                    payload: format!("{:?}", &invite),
                })?
                .to_owned(),
        );

        let cseq = invite
            .headers
            .get("CSeq")
            .ok_or(PayloadError::MandatoryPartNotFound {
                name: String::from("CSeq"),
                payload: format!("{:?}", &invite),
            })?
            .parse::<i32>()?
            + 1;

        out.add_header(String::from("CSeq"), cseq.to_string());

        out.add_header(
            String::from("Call-ID"),
            invite
                .get_header(&String::from("Call-ID"))
                .ok_or(PayloadError::MandatoryPartNotFound {
                    name: String::from("Call-ID"),
                    payload: format!("{:?}", &invite),
                })?
                .to_owned(),
        );

        out.add_header(String::from("Max-Forwards"), String::from("0"));
        out.add_header(
            String::from("Content-Type"),
            String::from("application/x-msnmsgr-sessionreqbody"),
        );

        out.add_body_property(
            String::from("SessionID"),
            invite
                .get_body_property(&String::from("SessionID"))
                .ok_or(PayloadError::MandatoryPartNotFound {
                    name: String::from("SessionID"),
                    payload: format!("{:?}", &invite),
                })?
                .to_owned(),
        );

        return Ok(out);
    }

    pub fn get_file_transfer_request(
        sender: &MsnUser,
        receiver: &MsnUser,
        context: &PreviewData,
        session_id: u32,
    ) -> Result<SlpPayload, PayloadError> {
        let mut out = SlpPayload::new();
        out.first_line = format!("INVITE MSNMSGR:{} MSNSLP/1.0", receiver.endpoint_id);
        out.add_header(
            String::from("To"),
            format!("<msnmsgr:{mpop_id}>", mpop_id = receiver.endpoint_id),
        );
        out.add_header(
            String::from("From"),
            format!("<msnmsgr:{mpop_id}>", mpop_id = sender.endpoint_id),
        );
        out.add_header(
            String::from("Via"),
            format!(
                "MSNSLP/1.0/TLP ;branch={{{branch_uuid}}}",
                branch_uuid = Uuid::new().to_string()
            ),
        );

        out.add_header(String::from("CSeq"), String::from("0"));
        out.add_header(
            String::from("Call-ID"),
            format!("{{{call_id}}}", call_id = Uuid::new().to_string()),
        );
        out.add_header(String::from("Max-Forwards"), String::from("0"));
        out.add_header(
            String::from("Content-Type"),
            String::from("application/x-msnmsgr-sessionreqbody"),
        );

        out.add_body_property(String::from("EUF-GUID"), EufGUID::FileTransfer.to_string());
        out.add_body_property(String::from("SessionID"), session_id.to_string());
        out.add_body_property(String::from("AppID"), String::from("2"));
        out.add_body_property(String::from("RequestFlags"), String::from("16"));

        out.add_body_property(String::from("Context"), context.to_string());

        return Ok(out);
    }

    pub fn get_msn_object_request(
        sender: &MsnUser,
        receiver: &MsnUser,
        context: &MsnObject,
        session_id: u32,
    ) -> Result<SlpPayload, PayloadError> {
        let context_b64 = general_purpose::STANDARD.encode(context.to_string_not_encoded());

        let mut out = SlpPayload::new();
        out.first_line = format!("INVITE MSNMSGR:{} MSNSLP/1.0", receiver.endpoint_id);
        out.add_header(
            String::from("To"),
            format!("<msnmsgr:{mpop_id}>", mpop_id = receiver.endpoint_id),
        );
        out.add_header(
            String::from("From"),
            format!("<msnmsgr:{mpop_id}>", mpop_id = sender.endpoint_id),
        );
        out.add_header(
            String::from("Via"),
            format!(
                "MSNSLP/1.0/TLP ;branch={{{branch_uuid}}}",
                branch_uuid = Uuid::new().to_string()
            ),
        );

        out.add_header(String::from("CSeq"), String::from("0"));
        out.add_header(
            String::from("Call-ID"),
            format!("{{{call_id}}}", call_id = Uuid::new().to_string()),
        );
        out.add_header(String::from("Max-Forwards"), String::from("0"));
        out.add_header(
            String::from("Content-Type"),
            String::from("application/x-msnmsgr-sessionreqbody"),
        );

        out.add_body_property(String::from("EUF-GUID"), EufGUID::MSNObject.to_string());
        out.add_body_property(String::from("SessionID"), session_id.to_string());
        out.add_body_property(String::from("AppID"), String::from("20"));
        out.add_body_property(String::from("RequestFlags"), String::from("18"));
        out.add_body_property(String::from("Context"), context_b64);
        return Ok(out);
    }

    pub fn get_200_ok_indirect_connect(
        invite: &SlpPayload,
    ) -> Result<SlpPayload, PayloadError> {
        let mut out = SlpPayloadFactory::get_200_ok_direct_connect(invite)?;
        out.add_body_property(String::from("Bridge"), String::from("SBBridge"));
        return Ok(out);
    }

    pub fn get_200_ok_direct_connect(invite: &SlpPayload) -> Result<SlpPayload, PayloadError> {
        let mut out = SlpPayload::new();
        out.first_line = String::from("MSNSLP/1.0 200 OK");

        out.add_header(
            String::from("To"),
            invite
                .get_header(&String::from("From"))
                .ok_or(PayloadError::MandatoryPartNotFound {
                    name: String::from("From"),
                    payload: format!("{:?}", &invite),
                })?
                .to_owned(),
        );

        out.add_header(
            String::from("From"),
            invite
                .get_header(&String::from("To"))
                .ok_or(PayloadError::MandatoryPartNotFound {
                    name: String::from("To"),
                    payload: format!("{:?}", &invite),
                })?
                .to_owned(),
        );

        out.add_header(
            String::from("Via"),
            invite
                .get_header(&String::from("Via"))
                .ok_or(PayloadError::MandatoryPartNotFound {
                    name: String::from("Via"),
                    payload: format!("{:?}", &invite),
                })?
                .to_owned(),
        );

        let cseq = invite
            .headers
            .get("CSeq")
            .ok_or(PayloadError::MandatoryPartNotFound {
                name: String::from("CSeq"),
                payload: format!("{:?}", &invite),
            })?
            .parse::<i32>()?
            + 1;

        out.add_header(String::from("CSeq"), cseq.to_string());

        out.add_header(
            String::from("Call-ID"),
            invite
                .get_header(&String::from("Call-ID"))
                .ok_or(PayloadError::MandatoryPartNotFound {
                    name: String::from("Call-ID"),
                    payload: format!("{:?}", &invite),
                })?
                .to_owned(),
        );

        out.add_header(String::from("Max-Forwards"), String::from("0"));
        out.add_header(
            String::from("Content-Type"),
            String::from("application/x-msnmsgr-transrespbody"),
        );

        if let Some(session_id) = invite.get_body_property(&String::from("SessionID")) {
            out.add_body_property(String::from("SessionID"), session_id.to_owned());
        }

        out.add_body_property(String::from("Listening"), String::from("true"));
        out.add_body_property(
            String::from("NeedConnectingEndpointInfo"),
            String::from("false"),
        );
        out.add_body_property(String::from("Conn-Type"), String::from("DirectConnect"));
        out.add_body_property(String::from("TCP-Conn-Type"), String::from("DirectConnect"));
        out.add_body_property(String::from("IPv6-global"), String::from(""));
        out.add_body_property(String::from("UPnPNat"), String::from("false"));
        out.add_body_property(String::from("Capabilities-Flags"), String::from("1"));
        out.add_body_property(
            String::from("IPv4Internal-Addrs"),
            String::from("127.0.0.1"),
        );
        out.add_body_property(String::from("IPv4Internal-Port"), String::from("1865"));
        out.add_body_property(
            String::from("Nat-Trav-Msg-Type"),
            String::from("WLX-Nat-Trav-Msg-Direct-Connect-Resp"),
        );
        out.add_body_property(String::from("Bridge"), String::from("TCPv1"));
        out.add_body_property(
            String::from("Hashed-Nonce"),
            String::from("{2B95F56D-9CA0-9A64-82CE-ADC1F3C55845}"),
        );
        return Ok(out);
    }

    pub fn get_200_ok_direct_connect_bad_port(
        invite: &SlpPayload,
    ) -> Result<SlpPayload, PayloadError> {
        let mut out = SlpPayload::new();
        out.first_line = String::from("MSNSLP/1.0 200 OK");

        out.add_header(
            String::from("To"),
            invite
                .get_header(&String::from("From"))
                .ok_or(PayloadError::MandatoryPartNotFound {
                    name: String::from("From"),
                    payload: format!("{:?}", &invite),
                })?
                .to_owned(),
        );

        out.add_header(
            String::from("From"),
            invite
                .get_header(&String::from("To"))
                .ok_or(PayloadError::MandatoryPartNotFound {
                    name: String::from("To"),
                    payload: format!("{:?}", &invite),
                })?
                .to_owned(),
        );

        out.add_header(
            String::from("Via"),
            invite
                .get_header(&String::from("Via"))
                .ok_or(PayloadError::MandatoryPartNotFound {
                    name: String::from("Via"),
                    payload: format!("{:?}", &invite),
                })?
                .to_owned(),
        );

        let cseq = invite
            .headers
            .get("CSeq")
            .ok_or(PayloadError::MandatoryPartNotFound {
                name: String::from("CSeq"),
                payload: format!("{:?}", &invite),
            })?
            .parse::<i32>()?
            + 1;
        out.add_header(String::from("CSeq"), cseq.to_string());

        out.add_header(
            String::from("Call-ID"),
            invite
                .get_header(&String::from("Call-ID"))
                .ok_or(PayloadError::MandatoryPartNotFound {
                    name: String::from("Call-ID"),
                    payload: format!("{:?}", &invite),
                })?
                .to_owned(),
        );

        out.add_header(String::from("Max-Forwards"), String::from("0"));
        out.add_header(
            String::from("Content-Type"),
            String::from("application/x-msnmsgr-transrespbody"),
        );

        if let Some(session_id) = invite.get_body_property(&String::from("SessionID")) {
            out.add_body_property(String::from("SessionID"), session_id.to_owned());
        }

        out.add_body_property(String::from("Listening"), String::from("true"));
        out.add_body_property(
            String::from("NeedConnectingEndpointInfo"),
            String::from("false"),
        );
        out.add_body_property(String::from("Conn-Type"), String::from("Firewall"));
        out.add_body_property(String::from("TCP-Conn-Type"), String::from("Firewall"));
        out.add_body_property(String::from("IPv6-global"), String::from(""));
        out.add_body_property(String::from("UPnPNat"), String::from("false"));
        out.add_body_property(String::from("Capabilities-Flags"), String::from("1"));
        out.add_body_property(
            String::from("IPv4External-Addrs"),
            String::from("127.0.0.1"),
        );
        out.add_body_property(String::from("IPv4External-Port"), String::from("1866"));
        out.add_body_property(
            String::from("Nat-Trav-Msg-Type"),
            String::from("WLX-Nat-Trav-Msg-Direct-Connect-Resp"),
        );
        out.add_body_property(String::from("Bridge"), String::from("TCPv1"));
        out.add_body_property(
            String::from("Hashed-Nonce"),
            String::from("{2B95F56D-9CA0-9A64-82CE-ADC1F3C55845}"),
        );
        return Ok(out);
    }

    pub fn get_500_error_direct_connect(
        invite: &SlpPayload,
        bridge: String,
    ) -> Result<SlpPayload, PayloadError> {
        let mut out = SlpPayload::new();
        out.first_line = String::from("MSNSLP/1.0 500 Internal Error");

        out.add_header(
            String::from("To"),
            invite
                .get_header(&String::from("From"))
                .ok_or(PayloadError::MandatoryPartNotFound {
                    name: String::from("From"),
                    payload: format!("{:?}", &invite),
                })?
                .to_owned(),
        );

        out.add_header(
            String::from("From"),
            invite
                .get_header(&String::from("To"))
                .ok_or(PayloadError::MandatoryPartNotFound {
                    name: String::from("To"),
                    payload: format!("{:?}", &invite),
                })?
                .to_owned(),
        );

        out.add_header(
            String::from("Via"),
            invite
                .get_header(&String::from("Via"))
                .ok_or(PayloadError::MandatoryPartNotFound {
                    name: String::from("Via"),
                    payload: format!("{:?}", &invite),
                })?
                .to_owned(),
        );

        let cseq = invite
            .headers
            .get("CSeq")
            .ok_or(PayloadError::MandatoryPartNotFound {
                name: String::from("CSeq"),
                payload: format!("{:?}", &invite),
            })?
            .parse::<i32>()?
            + 1;

        out.add_header(String::from("CSeq"), cseq.to_string());
        out.add_header(
            String::from("Call-ID"),
            invite
                .get_header(&String::from("Call-ID"))
                .ok_or(PayloadError::MandatoryPartNotFound {
                    name: String::from("Call-ID"),
                    payload: format!("{:?}", &invite),
                })?
                .to_owned(),
        );

        out.add_header(String::from("Max-Forwards"), String::from("0"));
        out.add_header(
            String::from("Content-Type"),
            String::from("application/x-msnmsgr-transrespbody"),
        );

        out.add_body_property(String::from("Bridge"), String::from(bridge));
        out.add_body_property(
            String::from("Nonce"),
            String::from("{00000000-0000-0000-0000-000000000000}"),
        );
        out.add_body_property(String::from("Capabilities-Flags"), String::from("0"));

        return Ok(out);
    }

    pub fn get_transport_request(sender: &MsnUser, receiver: &MsnUser) -> SlpPayload {
        let mut out = SlpPayload::new();

        out.first_line = format!(
            "INVITE MSNMSGR:{mpop_id} MSNSLP/1.0",
            mpop_id = &receiver.endpoint_id
        );

        out.add_header(
            String::from("To"),
            format!("<msnmsgr:{mpop_id}>", mpop_id = &receiver.endpoint_id),
        );
        out.add_header(
            String::from("From"),
            format!("<msnmsgr:{mpop_id}>", mpop_id = &sender.endpoint_id),
        );
        out.add_header(
            String::from("Via"),
            format!(
                "MSNSLP/1.0/TLP ;branch={{{branch_uuid}}}",
                branch_uuid = Uuid::new().to_string()
            ),
        );

        out.add_header(String::from("CSeq"), String::from("0"));
        out.add_header(
            String::from("Call-ID"),
            format!("{{{call_id}}}", call_id = Uuid::new().to_string()),
        );
        out.add_header(String::from("Max-Forwards"), String::from("0"));
        out.add_header(
            String::from("Content-Type"),
            String::from("application/x-msnmsgr-transreqbody"),
        );

        out.add_body_property(String::from("NetID"), String::from("251789322"));
        out.add_body_property(String::from("Conn-Type"), String::from("Firewall"));
        out.add_body_property(String::from("TCP-Conn-Type"), String::from("Firewall"));
        out.add_body_property(String::from("UPnPNat"), String::from("false"));
        out.add_body_property(String::from("ICF"), String::from("false"));
        out.add_body_property(
            String::from("IPv4Internal-Addrs"),
            String::from("127.0.0.1"),
        );
        out.add_body_property(String::from("IPv4Internal-Port"), String::from("1865"));
        out.add_body_property(String::from("IPv6-global"), String::from(""));
        out.add_body_property(String::from("Capabilities-Flags"), String::from("1"));
        out.add_body_property(
            String::from("Nat-Trav-Msg-Type"),
            String::from("WLX-Nat-Trav-Msg-Direct-Connect-Req"),
        );
        out.add_body_property(String::from("Bridges"), String::from("SBBridge"));
        out.add_body_property(
            String::from("Hashed-Nonce"),
            String::from("{D14FBAF3-CA6B-CC91-F93A-E76F24903F60}"),
        );

        return out;
    }
}

mod tests {

    use crate::p2p::v2::slp::slp_payload::SlpPayload;

    use super::EufGUID;

    #[test]
    fn test_euf_guid_try_from_str() {
        let test = EufGUID::try_from("{A4268EEC-FEC5-49E5-95C3-F126696BDBF6}").unwrap();
        assert_eq!(test, EufGUID::MSNObject);
    }

    #[test]
    fn test_euf_guid_to_str() {
        let test = EufGUID::MSNObject.to_string();
        assert_eq!("{A4268EEC-FEC5-49E5-95C3-F126696BDBF6}", test.as_str());
    }

    #[test]
    fn test_slp_payload_get_euf_guid_success() {
        let mut payload = SlpPayload::new();
        payload.add_body_property(
            String::from("EUF-GUID"),
            String::from("{A4268EEC-FEC5-49E5-95C3-F126696BDBF6}"),
        );

        let result = payload.get_euf_guid().unwrap().unwrap();
        assert_eq!(result, EufGUID::MSNObject);
    }

    #[test]
    fn test_slp_payload_get_euf_guid_none() {
        let payload = SlpPayload::new();
        let result = payload.get_euf_guid().unwrap();
        assert_eq!(result.is_none(), true);
    }
}
