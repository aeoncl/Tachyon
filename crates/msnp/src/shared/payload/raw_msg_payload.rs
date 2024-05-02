use std::{collections::HashMap, str::FromStr};
use std::fmt::{Display, format, Formatter};
use anyhow::anyhow;

use lazy_static::lazy_static;
use lazy_static_include::syn::Lit::Str;

use crate::msnp::error::PayloadError;
use crate::shared::traits::MSNPPayload;

const TEMPLATE: &str = "MIME-Version: 1.0\r\nContent-Type: ";
const CHARSET: &str = "; charset=UTF-8";

#[derive(Clone, Debug)]

pub struct RawMsgPayload {

    pub content_type: String,
    enable_charset : bool,
    enable_trailing_terminators : bool,
    pub headers: HashMap<String, String>,
    pub body: String
}

impl RawMsgPayload {

    pub fn new(content_type: &str) -> Self {
        return RawMsgPayload { content_type: content_type.to_string(), headers: HashMap::new(), body: String::new(), enable_charset: true, enable_trailing_terminators: true };
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

    pub fn set_body(&mut self, body: &str) {
        self.body = body.to_string();
    }

    pub fn set_body_owned(&mut self, body: String) {
        self.body = body;
    }

    pub fn disable_charset(&mut self) {
        self.enable_charset = false;
    }

    pub fn disable_trailing_terminators(&mut self) {
        self.enable_trailing_terminators = false;
    }



}

impl MSNPPayload for RawMsgPayload {
    type Err = PayloadError;

    fn try_from_bytes(bytes: Vec<u8>) -> Result<Self, Self::Err> {
        todo!()
    }

    fn to_bytes(self) -> Vec<u8> {
        self.to_string().into_bytes()
    }
}

impl Display for RawMsgPayload {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        let mut headers = String::with_capacity(self.headers.len() * 50);

        for (key, value) in &self.headers {
            headers.push_str(format!("{}: {}\r\n", key, value).as_str());
        }

        if self.enable_charset {
            write!(f, "{template}{content_type}{charset}\r\n{headers}\r\n{body}", template=TEMPLATE, content_type=&self.content_type, charset=CHARSET, headers=headers, body=&self.body)?;
        } else {
            write!(f, "{template}{content_type}\r\n{headers}\r\n{body}", template=TEMPLATE, content_type=&self.content_type, headers=headers, body=&self.body)?;
        };

        if self.enable_trailing_terminators {
            write!(f, "\r\n")?;
        }
        Ok(())
    }



}


impl Default for RawMsgPayload {
    fn default() -> Self {
        RawMsgPayload::new("")
    }
}

impl FromStr for RawMsgPayload {
    type Err = PayloadError;

    fn from_str(s: &str) -> Result<Self, Self::Err> {

        let mut out = RawMsgPayload::new("");

        if let Some((headers, body)) = s.split_once("\r\n\r\n") {

            let split_str : Vec<&str> = headers.split("\r\n").collect();
           // info!("debug split: {:?}", &split_str);
            let count = split_str.len();
            for i in 0..count {
                let current = split_str.get(i).expect("MsgPayload index to be in bounds");

                if let Some((name, value)) =  current.split_once(":"){

                        if name =="Content-Type" {
                            let value_split : Vec<&str> = value.split(";").collect();
                            let mime_type = value_split.get(0)
                                .ok_or(PayloadError::StringPayloadParsingError { payload: s.to_string(), source: anyhow!("Could not extract content-type from MSG Payload") })?
                                .trim();
                            
                            out.content_type = mime_type.to_string();
                        } else {
                            if name != "MIME-Version" {
                                out.add_header(name.trim(), value.trim());
                            }
                        }

                }
            }
            out.body = body.to_string();
            return Ok(out);

        }
        return Err(PayloadError::StringPayloadParsingError { payload: s.to_string(), source: anyhow!("MSG Payload did not contain body header separator") });
    }
}

pub mod factories {
    use std::fmt::format;
    use std::num::TryFromIntError;
    use base64::Engine;
    use byteorder::{BigEndian, ByteOrder, LittleEndian};
    use chrono::{DateTime, Local, Utc};
    use uuid::Uuid;


    use crate::{p2p::v2::p2p_transport_packet::P2PTransportPacket, shared::models::{msn_object::MsnObject, msn_user::MsnUser, uuid::Puid}};
    use crate::shared::converters::filetime::FileTime;
    use crate::shared::models::email_address::EmailAddress;
    use crate::shared::models::oim::MetaData;
    use crate::shared::models::ticket_token::TicketToken;
    use crate::soap::traits::xml::ToXml;

    use super::RawMsgPayload;

    pub struct MsgPayloadFactory;

    impl MsgPayloadFactory {
        pub fn get_msmsgs_profile(puid: &Puid, msn_addr: &EmailAddress, ticket_token: &TicketToken) -> RawMsgPayload {
            let mut out = RawMsgPayload::new("text/x-msmsgsprofile");
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
            out.add_header("Flags", "1610613827");
            out.add_header("sid", "507");
            out.add_header("MSPAuth", &ticket_token.0);
            out.add_header("ClientIP", "");
            out.add_header("ClientPort","");
            out.add_header("ABCHMigrated", "1");
            out.add_header("MPOPEnabled", "1");

            out.disable_trailing_terminators();

            return out;
        }

        pub fn get_initial_mail_data_empty_notification() -> RawMsgPayload {
            let mut out = RawMsgPayload::new("text/x-msmsgsinitialmdatanotification");
            out.set_body("Mail-Data: <MD><E><I>0</I><IU>0</IU><O>0</O><OU>0</OU></E><Q><QTM>409600</QTM><QNM>204800</QNM></Q></MD>\r\nInbox-Unread: 0\r\nFolders-Unread: 0\r\nInbox-URL: /cgi-bin/HoTMaiL\r\nFolders-URL: /cgi-bin/folders\r\nPost-URL: http://127.0.0.1:8080/email\r\n");
            out.disable_trailing_terminators();
            return out;
        }

        pub fn get_initial_mail_data_too_large_notification() -> RawMsgPayload {
            let mut out = RawMsgPayload::new("text/x-msmsgsinitialmdatanotification");
            out.set_body("Mail-Data: too-large\r\nInbox-Unread: 0\r\nFolders-Unread: 0\r\nInbox-URL: /cgi-bin/HoTMaiL\r\nFolders-URL: /cgi-bin/folders\r\nPost-URL: http://127.0.0.1:8080/email\r\n");
            out.disable_trailing_terminators();
            return out;
        }

        pub fn get_initial_mail_data_notification(mail_data: MetaData) -> RawMsgPayload {
            let mut out = RawMsgPayload::new("text/x-msmsgsinitialmdatanotification");
            out.set_body(format!("Mail-Data: {}\r\nInbox-Unread: 0\r\nFolders-Unread: 0\r\nInbox-URL: /cgi-bin/HoTMaiL\r\nFolders-URL: /cgi-bin/folders\r\nPost-URL: http://127.0.0.1:8080/email\r\n", mail_data.to_xml().unwrap()).as_str());
            out.disable_trailing_terminators();
            return out;
        }

        pub fn get_system_msg(msg_type: String, arg1: String, arg2: String) -> RawMsgPayload {
            let mut out = RawMsgPayload::new("application/x-msmsgssystemmessage");
            out.set_body(format!("Type: {msg_type}\r\nArg1: {arg1}\r\nArg2: {arg2}", msg_type = msg_type, arg1 = arg1, arg2 = arg2).as_str());
            out.disable_trailing_terminators();
            return out;
        }

        pub fn get_message(text: &str) -> RawMsgPayload {
            let mut out = RawMsgPayload::new("text/plain");
            out.add_header("X-MMS-IM-Format","FN=MS%20Sans%20Serif; EF=; CO=0; PF=0; RL=0");
            out.set_body(text);
            out.disable_trailing_terminators();
            return out;
        }

        pub fn get_typing_user(typing_user_msn_addr: &str) -> RawMsgPayload {
            let mut out = RawMsgPayload::new("text/x-msmsgscontrol");
            out.add_header("TypingUser", typing_user_msn_addr);
            out.disable_charset();
            return out;
        }

        pub fn get_action_msg(text: String, plugin_context: bool) -> RawMsgPayload {
            let mut out = RawMsgPayload::new("text/x-msnmsgr-datacast");
            if plugin_context {
                out.add_header("PlugIn-Context", "1");
            }
            out.body = format!("ID: 4\r\nData: {}\r\n", text);
            out.disable_charset();
            return out;
        }

        pub fn get_nudge() -> RawMsgPayload {
            let mut out = RawMsgPayload::new("text/x-msnmsgr-datacast");
            out.body = String::from("ID: 1");
            out.disable_charset();
            return out;
        }

        pub fn get_msnobj_datacast(msn_object: &MsnObject) -> RawMsgPayload {
            let mut out = RawMsgPayload::new("text/x-msnmsgr-datacast");

            out.body = format!("ID: 3\r\nData: {}\r\n", msn_object.to_string_not_encoded());
            out.disable_charset();
            out
        }

        pub fn get_p2p(source: &MsnUser, destination: &MsnUser, payload: &P2PTransportPacket) -> RawMsgPayload {
            let mut out = RawMsgPayload::new("application/x-msnmsgrp2p");
            out.add_header("P2P-Dest", &destination.endpoint_id.to_string());
            out.add_header("P2P-Src",  &source.endpoint_id.to_string());

            out.body = payload.to_string();
            out.disable_charset();
            out.disable_trailing_terminators();
            return out;
        }

        pub fn get_oim(recv_datetime: DateTime<Utc>, from: &str, from_display_name: &str, to: &str, run_id: &str, seq_num: u32, message_id: &str, content: &str, content_type: &str) -> RawMsgPayload {

            let mut out = RawMsgPayload::new(content_type);

            let recv_datetime_local : DateTime<Local> = DateTime::from(recv_datetime.clone());
            let recv_datetime_formatted = recv_datetime_local.format("%a, %d %b %Y %H:%M:%S %z").to_string();


            let filetime = FileTime::from_utc_datetime(recv_datetime.clone());

            let mut encoded = crate::shared::converters::rfc2047::encode(from_display_name);


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

            out.set_body_owned(base64::engine::general_purpose::STANDARD.encode(content));

            out
        }


    }

}

#[cfg(test)]
mod tests {
    use std::str::FromStr;
    use byteorder::{ByteOrder, LittleEndian};
    use chrono::{DateTime, FixedOffset, Local, NaiveDateTime, Utc};
    use filetime_type::FileTime;
    use crate::shared::models::uuid::Uuid;
    use crate::shared::payload::raw_msg_payload::factories::MsgPayloadFactory;

    use crate::shared::payload::raw_msg_payload::RawMsgPayload;
    use crate::shared::traits::MSNPPayload;




    #[test]
    fn test_filetime() {

        let test = chrono::Utc::now().format("%d %b %Y %H:%M:%S").to_string();

        let test2= DateTime::parse_from_str("15 Nov 2005 22:24:27.020 (UTC)", "%d %b %Y %H:%M:%S%.3f (%Z)");

        if test2.is_err() {
            let err = test2.err().unwrap();
            println!("{}", err);
        }
        test2.unwrap();

        todo!()
    }


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
        let oim = MsgPayloadFactory::get_oim(chrono::Local::now(), "from@shlasouf.local", "displayname","to@shlasouf.local", &Uuid::new().to_string(), 1, "id1", "Hello !!!!", "text/plain");

        let test = oim.to_string();
        print!("{}", test);


    }

    #[test]
    fn test() {
        let mut payload = RawMsgPayload::new("content-type");
        payload.add_header("headerName","headerValue");
        payload.disable_trailing_terminators();
        let serialized = payload.to_bytes();
        assert_eq!(b"MIME-Version: 1.0\r\nContent-Type: content-type; charset=UTF-8\r\nheaderName: headerValue\r\n\r\n", serialized.as_slice());
    }

    #[test]
    fn test_deserialize() {
        let test = "MIME-Version: 1.0\r\nContent-Type: text/plain; charset=UTF-8\r\nX-MMS-IM-Format: FN=Segoe%20UI; EF=; CO=0; CS=1; PF=0\r\n\r\nfaefeafa";
        let mut result = RawMsgPayload::from_str(test).unwrap();
        result.disable_trailing_terminators();
        assert_eq!(result.body, "faefeafa");
        assert_eq!(result.content_type, "text/plain");
        assert_eq!(Some("FN=Segoe%20UI; EF=; CO=0; CS=1; PF=0"), result.get_header("X-MMS-IM-Format"));

        let serialized = result.to_bytes();
        assert_eq!(b"MIME-Version: 1.0\r\nContent-Type: text/plain; charset=UTF-8\r\nX-MMS-IM-Format: FN=Segoe%20UI; EF=; CO=0; CS=1; PF=0\r\n\r\nfaefeafa", serialized.as_slice());
    }
}