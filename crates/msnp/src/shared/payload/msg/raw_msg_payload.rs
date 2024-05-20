use std::{collections::HashMap, str::FromStr};
use std::collections::BTreeMap;
use std::fmt::{Debug, Display, format, Formatter};
use std::str::{from_utf8, Utf8Error};
use std::string::FromUtf8Error;

use anyhow::anyhow;
use lazy_static::lazy_static;
use lazy_static_include::syn::Lit::Str;
use linked_hash_map::LinkedHashMap;
use log::warn;
use new_mime_guess::Mime;
use strum_macros::{Display, EnumString};

use crate::msnp::error::PayloadError;
use crate::shared::traits::MSNPPayload;

#[derive(Clone, EnumString, Display, PartialEq, Debug)]
pub enum MsgContentType {
    #[strum(serialize = "text/plain; charset=UTF-8", ascii_case_insensitive)]
    TextPlain,

    #[strum(serialize = "text/x-msmsgsprofile; charset=UTF-8", ascii_case_insensitive)]
    Profile,

    #[strum(serialize = "text/x-msmsgsinitialmdatanotification; charset=UTF-8", ascii_case_insensitive)]
    InitialMailDataNotification,

    #[strum(serialize = "application/x-msmsgssystemmessage; charset=UTF-8", ascii_case_insensitive)]
    SystemMessage,

    #[strum(serialize = "text/x-msmsgscontrol", ascii_case_insensitive)]
    Control,

    #[strum(serialize = "text/x-msnmsgr-datacast", ascii_case_insensitive)]
    Datacast,

    #[strum(serialize = "application/x-msnmsgrp2p", ascii_case_insensitive)]
    P2P,

    None
}

impl Default for MsgContentType {
    fn default() -> Self {
        MsgContentType::None
    }
}

#[derive(Clone, Debug)]
pub struct RawMsgPayload {
    enable_trailing_terminators : bool,
    pub headers: LinkedHashMap<String, String>,
    pub body: Vec<u8>
}

impl RawMsgPayload {

    pub fn new(content_type: MsgContentType, enable_trailing_terminators: bool) -> Self {
        let mut headers = LinkedHashMap::new();

        headers.insert("MIME-Version".into(), "1.0".into());
        headers.insert("Content-Type".into(), content_type.to_string());

        return RawMsgPayload {
            headers,
            body: Vec::new(),
            enable_trailing_terminators
        };
    }

    pub fn get_content_type(&self) -> Result<MsgContentType, PayloadError>  {
        let raw_content_type = self.get_header("Content-Type").ok_or(anyhow!("Content-Type not present in MSG Payload"))?;
        Ok(MsgContentType::from_str(raw_content_type).map_err(|e| anyhow!(e))?)
    }

    pub fn add_header(&mut self, name: &str, value: &str){
        self.headers.insert(name.to_string(), value.to_string());
    }

    pub fn add_header_owned(&mut self, name: String, value: String) {
        self.headers.insert(name, value);
    }

    pub fn get_header(&self, name: &str) -> Option<&str> {
        return self.headers.get(name).map(|e| e.as_str());
    }
    pub fn set_body(&mut self, body : Vec<u8>  ) {
        self.body = body;
    }
    pub fn set_body_str(&mut self, body: &str) {
        self.body = body.as_bytes().to_owned();
    }
    pub fn set_body_string(&mut self, body: String) {
        self.body = body.into_bytes();
    }
    pub fn get_body_as_str(&self) -> Result<&str, Utf8Error> {
        from_utf8(&self.body)
    }

}

impl MSNPPayload for RawMsgPayload {
    type Err = PayloadError;

    fn try_from_bytes(mut bytes: Vec<u8>) -> Result<Self, Self::Err> {

        let header_body_split_index = bytes.windows(4).enumerate().find_map(|(index, content)| {
            if content[0] as char == '\r' && content[1] as char == '\n' && content[2]  as char  == '\r' && content[3] as char == '\n'  {
                Some(index+1)
            } else {
                None
            }
        });

        if header_body_split_index.is_none() {
            return Err(PayloadError::AnyError(anyhow!("Malformed Payload")));
        }

        let mut out = RawMsgPayload::default();

        let header_body_split_index = header_body_split_index.expect("to never fail");

        let (headers, body) = bytes.split_at_mut(header_body_split_index);

        let headers = from_utf8(headers)?;
        let headers_split : Vec<&str> = headers.split("\r\n").collect();

        for current in headers_split {
            match current.split_once(":") {
                None => {
                    warn!("Malformed header, ignoring...: {}", current);
                }
                Some((name, value)) => {
                    out.add_header(name.trim(), value.trim());
                }
            }
        }

        out.body = body[3..].to_vec();

        out.get_content_type()?;

        Ok(out)
    }

    /*
     MIME-Version: 1.0\r\n
     Content-Type: text/x-msmsgsoimnotification; charset=UTF-8\r\n
     Header: Value\r\n
     Header2: Value\r\n
     \r\n
     Body
     \r\n (sometimes)
     */
    fn into_bytes(mut self) -> Vec<u8> {
        let mut out = Vec::new();

        for (key, value) in self.headers {
            out.append(&mut format!("{}: {}\r\n", key, value).into_bytes());
        }

        out.extend_from_slice(b"\r\n");

        out.append(&mut self.body);

        if self.enable_trailing_terminators {
            out.extend_from_slice(b"\r\n");

        }

        out
    }
}

impl Default for RawMsgPayload {
    fn default() -> Self {
        RawMsgPayload {
            enable_trailing_terminators: false,
            headers: Default::default(),
            body: vec![],
        }
    }
}

pub mod factories {
    use std::str::FromStr;
    use base64::Engine;
    use chrono::{DateTime, Local, Utc};
    use mime::Mime;

    use crate::{p2p::v2::p2p_transport_packet::P2PTransportPacket, shared::models::{msn_object::MsnObject, msn_user::MsnUser, uuid::Puid}};
    use crate::shared::converters::filetime::FileTime;
    use crate::shared::models::email_address::EmailAddress;
    use crate::shared::models::oim::MetaData;
    use crate::shared::models::ticket_token::TicketToken;
    use crate::soap::traits::xml::ToXml;

    use super::{MsgContentType, RawMsgPayload};

    pub struct RawMsgPayloadFactory;

    impl RawMsgPayloadFactory {

        //Flags: 1610613827
        pub fn get_msmsgs_profile(puid: &Puid, msn_addr: &EmailAddress, ticket_token: &TicketToken) -> RawMsgPayload {
            let mut out = RawMsgPayload::new(MsgContentType::Profile, false);
            let now = Local::now().timestamp_millis();
            out.add_header("LoginTime", &now.to_string());
            out.add_header("EmailEnabled", "1");
            out.add_header("MemberIdHigh", &puid.get_most_significant_bytes().to_string());
            out.add_header("MemberIdLow", &puid.get_least_significant_bytes().to_string());
            out.add_header("lang_preference", "1033");
            out.add_header("preferredEmail", msn_addr.as_str());
            out.add_header("country", "");
            out.add_header("PostalCode", "");
            out.add_header("Gender", "");
            out.add_header("Kid", "0");
            out.add_header("Age", "");
            out.add_header("BDayPre", "");
            out.add_header("Birthday", "");
            out.add_header("Wallet", "");
            out.add_header("Flags", "536872513");
            out.add_header("sid", "507");
            out.add_header("MSPAuth", &ticket_token.0);
            out.add_header("ClientIP", "");
            out.add_header("ClientPort","");
            out.add_header("ABCHMigrated", "1");
            out.add_header("MPOPEnabled", "1");
            out.add_header("BetaInvites", "1");

            return out;
        }

        pub fn get_initial_mail_data_empty_notification() -> RawMsgPayload {
            let mut out = RawMsgPayload::new(MsgContentType::InitialMailDataNotification, false);
            out.set_body_str("Mail-Data: <MD><E><I>0</I><IU>0</IU><O>0</O><OU>0</OU></E><Q><QTM>409600</QTM><QNM>204800</QNM></Q></MD>\r\nInbox-Unread: 0\r\nFolders-Unread: 0\r\nInbox-URL: /cgi-bin/HoTMaiL\r\nFolders-URL: /cgi-bin/folders\r\nPost-URL: http://127.0.0.1:8080/email\r\n");
            return out;
        }

        pub fn get_initial_mail_data_too_large_notification() -> RawMsgPayload {
            let mut out = RawMsgPayload::new(MsgContentType::InitialMailDataNotification, false);
            out.set_body_str("Mail-Data: too-large\r\nInbox-Unread: 0\r\nFolders-Unread: 0\r\nInbox-URL: /cgi-bin/HoTMaiL\r\nFolders-URL: /cgi-bin/folders\r\nPost-URL: http://127.0.0.1:8080/email\r\n");
            return out;
        }

        pub fn get_initial_mail_data_notification(mail_data: MetaData) -> RawMsgPayload {

            let mut out = RawMsgPayload::new(MsgContentType::InitialMailDataNotification, false);
            out.set_body_str(format!("Mail-Data: {}\r\nInbox-Unread: 0\r\nFolders-Unread: 0\r\nInbox-URL: /cgi-bin/HoTMaiL\r\nFolders-URL: /cgi-bin/folders\r\nPost-URL: http://127.0.0.1:8080/email\r\n", mail_data.to_xml().unwrap()).as_str());
            return out;
        }

        pub fn get_system_msg(msg_type: String, arg1: String, arg2: String) -> RawMsgPayload {
            let mut out = RawMsgPayload::new(MsgContentType::SystemMessage, false);
            out.set_body_str(format!("Type: {msg_type}\r\nArg1: {arg1}\r\nArg2: {arg2}", msg_type = msg_type, arg1 = arg1, arg2 = arg2).as_str());
            return out;
        }

        pub fn get_message(text: &str) -> RawMsgPayload {
            let mut out = RawMsgPayload::new(MsgContentType::TextPlain, false);
            out.add_header("X-MMS-IM-Format","FN=MS%20Sans%20Serif; EF=; CO=0; PF=0; RL=0");
            out.set_body_str(text);
            return out;
        }

        pub fn get_typing_user(typing_user_msn_addr: &str) -> RawMsgPayload {
            let mut out = RawMsgPayload::new(MsgContentType::Control, true);
            out.add_header("TypingUser", typing_user_msn_addr);
            return out;
        }

        pub fn get_action_msg(text: String, plugin_context: bool) -> RawMsgPayload {
            let mut out = RawMsgPayload::new(MsgContentType::Datacast, true);
            if plugin_context {
                out.add_header("PlugIn-Context", "1");
            }
            out.body = format!("ID: 4\r\nData: {}\r\n", text).into_bytes();
            return out;
        }

        pub fn get_nudge() -> RawMsgPayload {
            let mut out = RawMsgPayload::new(MsgContentType::Datacast, true);
            out.body = b"ID: 1".into();
            return out;
        }

        pub fn get_msnobj_datacast(msn_object: &MsnObject) -> RawMsgPayload {
            let mut out = RawMsgPayload::new(MsgContentType::Datacast, true);
            out.body = format!("ID: 3\r\nData: {}\r\n", msn_object.to_string_not_encoded()).into_bytes();
            out
        }

        pub fn get_p2p(source: &MsnUser, destination: &MsnUser, payload: &P2PTransportPacket) -> RawMsgPayload {
            let mut out = RawMsgPayload::new(MsgContentType::P2P, false);
            out.add_header("P2P-Dest", &destination.endpoint_id.to_string());
            out.add_header("P2P-Src",  &source.endpoint_id.to_string());
            //TODO change this payload to serialize into bytes
            out.body = payload.to_string().into_bytes();
            return out;
        }

        pub fn get_oim(recv_datetime: DateTime<Utc>, from: &str, from_display_name: &str, to: &str, run_id: &str, seq_num: u32, message_id: &str, content: &str, content_type: MsgContentType) -> RawMsgPayload {

            let mut out = RawMsgPayload::new(content_type, true);

            let recv_datetime_local : DateTime<Local> = DateTime::from(recv_datetime.clone());
            let recv_datetime_formatted = recv_datetime_local.format("%a, %d %b %Y %H:%M:%S %z").to_string();


            let filetime = FileTime::from_utc_datetime(recv_datetime.clone());

            let encoded = crate::shared::converters::rfc2047::encode(from_display_name);

            out.add_header("X-Message-Info", "");
            out.add_header_owned("Received".into(), format!("from Tachyon by 127.0.0.1 with Matrix;{}", &recv_datetime_formatted));
            out.add_header_owned("From".into(), format!("{friendly} <{sender}>", friendly = encoded, sender = from));
            out.add_header("To", to);
            out.add_header("Subject", "");
            out.add_header("X-OIM-originatingSource", "127.0.0.1");
            out.add_header("X-OIMProxy", "MSNMSGR");
            out.add_header("Content-Transfer-Encoding", "base64");
            out.add_header("X-OIM-Message-Type", "OfflineMessage");
            out.add_header("X-OIM-Run-Id", run_id);
            out.add_header_owned("X-OIM-Sequence-Num".into(), format!("{}", &seq_num));
            out.add_header("Message-ID", message_id);

            let test = recv_datetime_local.format("%d %b %Y %H:%M:%S%.3f (%Z)").to_string();
            out.add_header_owned("X-OriginalArrivalTime".into(), format!("{date} FILETIME={filetime}", date = test, filetime = filetime));
            out.add_header_owned("Date".into(), recv_datetime_local.format("%d %b %Y %H:%M:%S %z").to_string());
            out.add_header("Return-Path", "ndr@oim.messenger.msn.com");

            out.set_body_string(base64::engine::general_purpose::STANDARD.encode(content));

            out
        }


    }

}

#[cfg(test)]
mod tests {
    use std::str::from_utf8;

    use byteorder::{ByteOrder, LittleEndian};

    use crate::shared::models::uuid::Uuid;
    use crate::shared::payload::msg::raw_msg_payload::factories::RawMsgPayloadFactory;
    use crate::shared::payload::msg::raw_msg_payload::{MsgContentType, RawMsgPayload};
    use crate::shared::traits::MSNPPayload;

    #[test]
    fn test_padding() {
        let mut buf: [u8; 8] = [0; 8];

        LittleEndian::write_u64(&mut buf, 11644473600000);

        let lsb_as_bytes = &buf[0..4];
        let msb_as_bytes = &buf[4..8];

        let lsb = LittleEndian::read_u32(lsb_as_bytes);
        let msb = LittleEndian::read_u32(msb_as_bytes);

        let lsb_ser = hex::encode_upper(&lsb_as_bytes);
        let msb_ser = hex::encode_upper(&msb_as_bytes);


        let mut lsb_deser = hex::decode(lsb_ser).unwrap();
        let msb_deser = hex::decode(msb_ser).unwrap();


        assert_eq!(lsb_as_bytes, lsb_deser.as_slice());
        assert_eq!(msb_as_bytes, msb_deser.as_slice());


        lsb_deser.extend_from_slice(msb_deser.as_slice());

        let out = LittleEndian::read_u64(lsb_deser.as_slice());



        assert_eq!(11644473600000, out);


    }

    #[test]
    fn test_oim_ser() {
        let oim = RawMsgPayloadFactory::get_oim(chrono::Local::now().to_utc(), "from@shlasouf.local", "displayname", "to@shlasouf.local", &Uuid::new().to_string(), 1, "id1", "Hello !!!!", MsgContentType::TextPlain);
        let oim_ser = oim.into_bytes();
        let test = from_utf8(oim_ser.as_slice()).unwrap();
        print!("{}", test);
    }

    #[test]
    fn test() {
        let mut payload = RawMsgPayload::new(MsgContentType::TextPlain, false);
        payload.add_header("headerName","headerValue");
        let serialized = String::from_utf8(payload.into_bytes()).unwrap();
        assert_eq!("MIME-Version: 1.0\r\nContent-Type: text/plain; charset=UTF-8\r\nheaderName: headerValue\r\n\r\n", &serialized);
    }

    #[test]
    fn test_deserialize() {
        let test = b"MIME-Version: 1.0\r\nContent-Type: text/plain; charset=UTF-8\r\nX-MMS-IM-Format: FN=Segoe%20UI; EF=; CO=0; CS=1; PF=0\r\n\r\nfaefeafa";
        let mut result = RawMsgPayload::try_from_bytes(test.to_vec()).unwrap();

        assert_eq!(result.get_body_as_str().unwrap(), "faefeafa");
        assert_eq!(result.get_content_type().unwrap(), MsgContentType::TextPlain);
        assert_eq!(Some("FN=Segoe%20UI; EF=; CO=0; CS=1; PF=0"), result.get_header("X-MMS-IM-Format"));

        let serialized = result.into_bytes();
        assert_eq!("MIME-Version: 1.0\r\nContent-Type: text/plain; charset=UTF-8\r\nX-MMS-IM-Format: FN=Segoe%20UI; EF=; CO=0; CS=1; PF=0\r\n\r\nfaefeafa", from_utf8(serialized.as_slice()).unwrap());
    }
}