use std::{collections::HashMap, str::FromStr};
use lazy_static::lazy_static;

use crate::models::errors::Errors;

use super::errors;

lazy_static! {
    static ref TEMPLATE : String = String::from("MIME-Version: 1.0\r\nContent-Type: ");
    static ref CHARSET : String = String::from("; charset=UTF-8");
}
#[derive(Clone, Debug)]

pub struct MsgPayload {

    pub content_type: String,
    enable_charset : bool,
    enable_trailing_terminators : bool,
    pub headers: HashMap<String, String>,
    pub body: String
}

impl MsgPayload {

    pub fn new(content_type: &str) -> Self {
        return MsgPayload{ content_type: content_type.to_string(), headers: HashMap::new(), body: String::new(), enable_charset: true, enable_trailing_terminators: true };
    }

    pub fn add_header(&mut self, name: String, value: String){
        self.headers.insert(name, value);
    }

    pub fn get_header(&self, name: &String) -> Option<&String> {
        return self.headers.get(name);
    }

    pub fn set_body(&mut self, body: String) {
        self.body = body;
    }

    pub fn disable_charset(&mut self) {
        self.enable_charset = false;
    }

    pub fn disable_trailing_terminators(&mut self) {
        self.enable_trailing_terminators = false;
    }

    pub fn serialize(&self) -> String {
        let mut headers = String::new();
        for (key, value) in &self.headers {
            headers.push_str(format!("{}: {}\r\n", &key, &value).as_str());
        }


        let mut out: String;
        if self.enable_charset {
            out = format!("{template}{content_type}{charset}\r\n{headers}\r\n{body}", template=TEMPLATE.as_str(), content_type=&self.content_type, charset=CHARSET.as_str(), headers=headers, body=&self.body);
        } else {
            out = format!("{template}{content_type}\r\n{headers}\r\n{body}", template=TEMPLATE.as_str(), content_type=&self.content_type, headers=headers, body=&self.body);
        }

        if self.enable_trailing_terminators {
            out.push_str("\r\n");
        }

        return out;
    }
}

impl FromStr for MsgPayload {
    type Err = errors::Errors;

    fn from_str(s: &str) -> Result<Self, Self::Err> {

        let mut out = MsgPayload::new("");

        //info!("debug testtt");

        let s = String::from(s);
        //info!("debug testtt1");

        if let Some((headers, body)) = s.split_once("\r\n\r\n") {

            let split_str : Vec<&str> = headers.split("\r\n").collect();
           // info!("debug split: {:?}", &split_str);
            let count = split_str.len();
            for i in 0..count {
                let current = split_str.get(i).unwrap().to_string();

                if let Some((name, value)) =  current.split_once(":"){

                        if name == String::from("Content-Type") {
                            let value_split : Vec<&str> = value.split(";").collect();
                            let mime_type = value_split.get(0).ok_or(Errors::PayloadDeserializeError)?.trim().to_string();
                            out.content_type = mime_type;
                        } else {
                            if name != String::from("MIME-Version") {
                                out.add_header(name.trim().to_string(), value.trim().to_string());
                            }
                        }

                }
            }
            out.body = body.to_owned();
            return Ok(out);

        }
        return Err(Errors::PayloadDeserializeError);
    }
}

pub mod factories {
    use chrono::Local;

    use crate::models::{uuid::{PUID}, msn_user::MSNUser, p2p::p2p_transport_packet::P2PTransportPacket };

    use super::MsgPayload;

    pub struct MsgPayloadFactory;

    impl MsgPayloadFactory {
        pub fn get_msmsgs_profile(puid: &PUID, msn_addr: String, matrix_token: String) -> MsgPayload {
            let mut out = MsgPayload::new("text/x-msmsgsprofile");
            let now = Local::now().timestamp_millis();
            out.add_header(String::from("LoginTime"), now.to_string());
            out.add_header(String::from("EmailEnabled"), String::from("1"));
            out.add_header(String::from("MemberIdHigh"), puid.get_most_significant_bytes().to_string());
            out.add_header(String::from("MemberIdLow"), puid.get_least_significant_bytes().to_string());
            out.add_header(String::from("lang_preference"), String::from("1033"));
            out.add_header(String::from("preferredEmail"), msn_addr);
            out.add_header(String::from("country"), String::new());
            out.add_header(String::from("PostalCode"), String::new());
            out.add_header(String::from("Gender"), String::new());
            out.add_header(String::from("Kid"), String::from("0"));
            out.add_header(String::from("Age"), String::new());
            out.add_header(String::from("BDayPre"), String::new());
            out.add_header(String::from("Birthday"), String::new());
            out.add_header(String::from("Wallet"), String::new());
            out.add_header(String::from("Flags"), String::from("1610613827"));
            out.add_header(String::from("sid"), String::from("507"));
            out.add_header(String::from("MSPAuth"), matrix_token);
            out.add_header(String::from("ClientIP"), String::new());
            out.add_header(String::from("ClientPort"), String::from("0"));
            out.add_header(String::from("ABCHMigrated"), String::from("1"));
            out.add_header(String::from("MPOPEnabled"), String::from("1"));
            out.disable_trailing_terminators();

            return out;
        }

        pub fn get_initial_mail_data_notification() -> MsgPayload {
            let mut out = MsgPayload::new("text/x-msmsgsinitialmdatanotification");
            out.set_body(String::from("Mail-Data: <MD><E><I>0</I><IU>0</IU><O>0</O><OU>0</OU></E><Q><QTM>409600</QTM><QNM>204800</QNM></Q></MD>\r\nInbox-Unread: 1\r\nFolders-Unread: 0\r\nInbox-URL: /cgi-bin/HoTMaiL\r\nFolders-URL: /cgi-bin/folders\r\nPost-URL: http://127.0.0.1:8080/email\r\n"));
            out.disable_trailing_terminators();
            return out;
        }

        pub fn get_system_msg(msg_type: String, arg1: String) -> MsgPayload {
            let mut out = MsgPayload::new("application/x-msmsgssystemmessage");
            out.set_body(format!("Type: {msg_type}\r\nArg1: {arg1}", msg_type = msg_type, arg1 = arg1));
            out.disable_trailing_terminators();
            return out;
        }

        pub fn get_message(text: String) -> MsgPayload {
            let mut out = MsgPayload::new("text/plain");
            out.add_header(String::from("X-MMS-IM-Format"), String::from("FN=MS%20Sans%20Serif; EF=; CO=0; PF=0; RL=0"));
            out.set_body(text);
            out.disable_trailing_terminators();
            return out;
        }

        pub fn get_typing_user(typing_user_msn_addr: String) -> MsgPayload {
            let mut out = MsgPayload::new("text/x-msmsgscontrol");
            out.add_header(String::from("TypingUser"), typing_user_msn_addr);
            out.disable_charset();
            return out;
        }

        pub fn get_action_msg(text: String, plugin_context: bool) -> MsgPayload {
            let mut out = MsgPayload::new("text/x-msnmsgr-datacast");
            if plugin_context {
                out.add_header(String::from("PlugIn-Context"), String::from("1"));
            }
            out.body = format!("ID: 4\r\nData: {}\r\n", text);
            out.disable_charset();
            return out;
        }

        pub fn get_nudge() -> MsgPayload {
            let mut out = MsgPayload::new("text/x-msnmsgr-datacast");
            out.body = String::from("ID: 1");
            out.disable_charset();
            return out;
        }

        pub fn get_p2p(source: &MSNUser, destination: &MSNUser, payload: &P2PTransportPacket) -> MsgPayload {
            let mut out = MsgPayload::new("application/x-msnmsgrp2p");
            out.add_header(String::from("P2P-Dest"), format!("{msn_addr};{{{endpoint_guid}}}", msn_addr = &destination.get_msn_addr(), endpoint_guid = &destination.get_endpoint_guid()));
            out.add_header(String::from("P2P-Src"), format!("{msn_addr};{{{endpoint_guid}}}", msn_addr = &source.get_msn_addr(), endpoint_guid = &source.get_endpoint_guid()));

            out.body = payload.to_string();
            out.disable_charset();
            out.disable_trailing_terminators();
            return out;
        }

    }

}

#[cfg(test)]
mod tests {
    use std::str::FromStr;

    use crate::models::msg_payload::MsgPayload;


    #[test]
    fn test() {
        let mut payload = MsgPayload::new("content-type");
        payload.add_header(String::from("headerName"),String::from("headerValue"));
        payload.disable_trailing_terminators();
        let serialized = payload.serialize();
        assert_eq!(serialized,String::from("MIME-Version: 1.0\r\nContent-Type: content-type; charset=UTF-8\r\nheaderName: headerValue\r\n\r\n")); 
    }

    #[test]
    fn test_deserialize() {
        let test = "MIME-Version: 1.0\r\nContent-Type: text/plain; charset=UTF-8\r\nX-MMS-IM-Format: FN=Segoe%20UI; EF=; CO=0; CS=1; PF=0\r\n\r\nfaefeafa";
        let mut result = MsgPayload::from_str(test).unwrap();
        result.disable_trailing_terminators();
        assert_eq!(result.body, String::from("faefeafa"));
        assert_eq!(result.content_type, String::from("text/plain"));
        assert!(result.get_header(&String::from("X-MMS-IM-Format")) == Some(&String::from("FN=Segoe%20UI; EF=; CO=0; CS=1; PF=0")));

        let serialized = result.serialize();
        assert_eq!(test, serialized.as_str());
    }
}