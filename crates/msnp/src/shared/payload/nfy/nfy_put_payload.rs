use std::collections::HashMap;
use std::fmt::{Display, Formatter, write};
use std::mem;
use std::str::{from_utf8, FromStr, Utf8Error};

use anyhow::anyhow;
use linked_hash_map::LinkedHashMap;
use log::warn;
use strum_macros::{Display, EnumString};

use crate::msnp::error::PayloadError;
use crate::msnp::switchboard::command::msg::MsgPayload;
use crate::shared::models::network_id_email::NetworkIdEmail;
use crate::shared::payload::msg::raw_msg_payload::{MsgContentType, RawMsgPayload};
use crate::shared::traits::MSNPPayload;

#[cfg(test)]
mod tests {
    use std::str::FromStr;

    use crate::shared::models::network_id_email::NetworkIdEmail;
    use super::{NfyContentType, NfyEnvelope, RawNfyPayload};
    use crate::shared::traits::MSNPPayload;

    #[test]
    fn test_put_envelope_serialize() {
        let put_env = NfyEnvelope {
            routing: "1.0".to_string(),
            from: NetworkIdEmail::from_str("9:00000000-0000-0000-0000-000000000009@live.com").unwrap(),
            to: NetworkIdEmail::from_str("1:aeon@lukewarmmail.com").unwrap(),
            reliability: "1.0".to_string(),
            stream: 0,
            segment: None,
            flags: None,
        };

        let ser = put_env.to_string();

        assert_eq!("Routing: 1.0\r\nTo: 1:aeon@lukewarmmail.com\r\nFrom: 9:00000000-0000-0000-0000-000000000009@live.com\r\n\r\nReliability: 1.0\r\nStream: 0\r\n\r\n", &ser)
    }

    #[test]
    fn test_put_envelope_serialize_with_segment() {
        let put_env = NfyEnvelope {
            routing: "1.0".to_string(),
            from: NetworkIdEmail::from_str("9:00000000-0000-0000-0000-000000000009@live.com").unwrap(),
            to: NetworkIdEmail::from_str("1:aeon@lukewarmmail.com").unwrap(),
            reliability: "1.0".to_string(),
            stream: 0,
            segment: Some(2),
            flags: None,
        };

        let ser = put_env.to_string();

        assert_eq!("Routing: 1.0\r\nTo: 1:aeon@lukewarmmail.com\r\nFrom: 9:00000000-0000-0000-0000-000000000009@live.com\r\n\r\nReliability: 1.0\r\nStream: 0\r\nSegment: 2\r\n\r\n", &ser)
    }

    #[test]
    fn test_put_envelope_deser_with_segment() {
        let raw = "Routing: 1.0\r\nTo: 1:aeon@lukewarmmail.com\r\nFrom: 9:00000000-0000-0000-0000-000000000009@live.com\r\n\r\nReliability: 1.0\r\nStream: 0\r\nSegment: 2\r\n\r\n";
        let deser = NfyEnvelope::from_str(raw).unwrap();

        assert_eq!("1.0", &deser.routing);
        assert_eq!("1:aeon@lukewarmmail.com", &deser.to.to_string());
        assert_eq!("9:00000000-0000-0000-0000-000000000009@live.com", &deser.from.to_string());
        assert_eq!("1.0", &deser.reliability);
        assert_eq!(0, deser.stream);
        assert_eq!(Some(2), deser.segment);
    }


    #[test]
    fn test_deser_put_payload() {
        let raw = b"Routing: 1.0\r\nTo: 1:aeon@lukewarmmail.com\r\nFrom: 9:00000000-0000-0000-0000-000000000009@live.com\r\n\r\nReliability: 1.0\r\nStream: 0\r\n\r\nNotification: 1.0\r\nNotifNum: 0\r\nUri: /circle\r\nNotifType: Full\r\nContent-Type: application/circles+xml\r\nContent-Length: 4\r\n\r\n1234";
        let deser = RawNfyPayload::try_from_bytes(raw.to_vec()).unwrap();

        let envelope = &deser.envelope;
        assert_eq!("1.0", &envelope.routing);
        assert_eq!("1:aeon@lukewarmmail.com", &envelope.to.to_string());
        assert_eq!("9:00000000-0000-0000-0000-000000000009@live.com", &envelope.from.to_string());
        assert_eq!("1.0", &envelope.reliability);
        assert_eq!(0, envelope.stream);
        assert_eq!(None, envelope.segment);

        assert_eq!("0", deser.get_header("NotifNum").unwrap());
        assert_eq!("/circle", deser.get_header("Uri").unwrap());
        assert_eq!("Full", deser.get_header("NotifType").unwrap());
        assert!(matches!(deser.content_type, NfyContentType::Circle));
        assert_eq!("4", deser.get_header("Content-Length").unwrap());
        assert_eq!(vec![b'1',b'2',b'3',b'4'], deser.body)
    }

    #[test]
    fn test_ser_put_payload() {

        let put_env = NfyEnvelope {
            routing: "1.0".to_string(),
            from: NetworkIdEmail::from_str("9:00000000-0000-0000-0000-000000000009@live.com").unwrap(),
            to: NetworkIdEmail::from_str("1:aeon@lukewarmmail.com").unwrap(),
            reliability: "1.0".to_string(),
            stream: 0,
            segment: None,
            flags: None,
        };


        let mut raw = RawNfyPayload::new(put_env, NfyContentType::Circle, false);
        raw.set_body(vec![b'1', b'2', b'3', b'4']);
        raw.add_header("Notification", "1.0");
        raw.add_header("NotifNum", "0");
        raw.add_header("Uri", "/circle");
        raw.add_header("NotifType", "Full");

        let ser = String::from_utf8(raw.into_bytes()).unwrap();

        let expectation = "Routing: 1.0\r\nTo: 1:aeon@lukewarmmail.com\r\nFrom: 9:00000000-0000-0000-0000-000000000009@live.com\r\n\r\nReliability: 1.0\r\nStream: 0\r\n\r\nNotification: 1.0\r\nNotifNum: 0\r\nUri: /circle\r\nNotifType: Full\r\nContent-Type: application/circles+xml\r\nContent-Length: 4\r\n\r\n1234";

        assert_eq!(expectation, &ser);
    }

}


#[derive(Clone, EnumString, Display, PartialEq, Debug)]
pub enum NfyContentType {
    #[strum(serialize = "application/circles+xml", ascii_case_insensitive)]
    Circle,
    None,
}

#[derive(Debug)]
pub struct NfyEnvelope {
    pub routing: String,
    pub from: NetworkIdEmail,
    pub to: NetworkIdEmail,

    pub reliability: String,
    pub stream: u32,
    pub segment: Option<u32>,
    pub flags: Option<String>,
}

impl NfyEnvelope {
    pub fn swap_sides(&mut self){
        mem::swap(&mut self.from,&mut self.to);
    }
    pub fn from_parts(routing_info: Vec<&str>, reliability_info: Vec<&str>) -> Result<Self, PayloadError> {
        let mut routing = None;
        let mut to = None;
        let mut from = None;

        for current in routing_info {
            let (key, value) = current.split_once(":").ok_or(anyhow!("Malformed PUT header"))?;
            match key.trim().to_lowercase().as_str() {
                "routing" => {
                    routing = Some(value.trim().to_string());
                }
                "to" => {
                    to = Some(NetworkIdEmail::from_str(value.trim()).unwrap());
                }
                "from" => {
                    from = Some(NetworkIdEmail::from_str(value.trim()).unwrap());
                }
                _ => {
                    warn!("Unknown routing info PUT header: {} {}", key, value)
                }
            }
        }

        let mut reliability = None;
        let mut stream = None;
        let mut segment = None;
        let mut flags = None;

        for current in reliability_info {
            let (key, value) = current.split_once(":").ok_or(anyhow!("Malformed PUT header"))?;
            match key.trim().to_lowercase().as_str() {
                "reliability" => {
                    reliability = Some(value.trim().to_string());
                }
                "stream" => {
                    stream = Some(u32::from_str(value.trim())?);
                }
                "segment" => {
                    segment = Some(u32::from_str(value.trim())?);
                },
                "flags" => {
                    flags = Some(value.trim().to_string());
                }
                _ => {
                    warn!("Unknown reliability PUT header: {} {}", key, value)
                }
            }
        }

        Ok(NfyEnvelope {
            routing: routing.ok_or(anyhow!("Missing routing field from PUT"))?,
            from: from.ok_or(anyhow!("Missing from field from PUT"))?,
            to: to.ok_or(anyhow!("Missing to field from PUT"))?,
            reliability: reliability.ok_or(anyhow!("Missing reliability field from PUT"))?,
            stream: stream.ok_or(anyhow!("Missing stream field from PUT"))?,
            segment,
            flags,
        }
        )
    }
}

impl FromStr for NfyEnvelope {
    type Err = PayloadError;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        let split: Vec<&str> = s.trim().split("\r\n\r\n").collect();

        if split.len() < 2 {
            Err(anyhow!("PutEnvelope split was too short"))?;
        }

        let routing_info: Vec<&str> = split.get(0).unwrap().split("\r\n").collect();
        let reliability_info: Vec<&str> = split.get(1).unwrap().split("\r\n").collect();

        NfyEnvelope::from_parts(routing_info, reliability_info)
    }
}

impl Display for NfyEnvelope {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        write!(f, "Routing: {}\r\nTo: {}\r\nFrom: {}\r\n\r\nReliability: {}\r\nStream: {}\r\n", self.routing, self.to, self.from, self.reliability, self.stream);

        match self.segment {
            None => {}
            Some(segment) => {
                write!(f, "Segment: {}\r\n", segment);
            }
        };

        match self.flags.as_ref() {
            None => {}
            Some(flags) => {
                write!(f, "Flags: {}\r\n", flags);
            }
        };

        write!(f, "\r\n");

        Ok(())
    }
}


#[derive(Debug)]
pub struct RawNfyPayload {
    pub envelope: NfyEnvelope,
    pub content_type: NfyContentType,
    pub headers: LinkedHashMap<String, String>,
    pub body: Vec<u8>,
    enable_trailing_terminators: bool,
}

impl RawNfyPayload {
    pub fn new(envelope: NfyEnvelope, content_type: NfyContentType, enable_trailing_terminators: bool) -> Self {
        let mut headers = LinkedHashMap::new();

        return RawNfyPayload {
            envelope,
            content_type,
            headers,
            body: Vec::new(),
            enable_trailing_terminators,
        };
    }

    pub fn new_circle(from: NetworkIdEmail, to: NetworkIdEmail) -> Self {
        let envelope = NfyEnvelope{
            routing: "1.0".to_string(),
            from,
            to,
            reliability: "1.0".to_string(),
            stream: 0,
            segment: None,
            flags: None,
        };

        let mut out = Self::new(envelope, NfyContentType::Circle, false);
        out.add_header("Notification", "1.0");
        out.add_header("NotifNum", "0");
        out.add_header("Uri", "/circle");
        out.add_header("NotifType", "Full");

        out
    }

    pub fn new_circle_partial(from: NetworkIdEmail, to: NetworkIdEmail) -> Self {
        let envelope = NfyEnvelope{
            routing: "1.0".to_string(),
            from,
            to,
            reliability: "1.0".to_string(),
            stream: 0,
            segment: None,
            flags: None,
        };

        let mut out = Self::new(envelope, NfyContentType::Circle, false);
        out.add_header("Notification", "1.0");
        out.add_header("NotifNum", "0");
        out.add_header("Uri", "/circle");
        out.add_header("NotifType", "Partial");

        out

    }


    pub fn add_header(&mut self, name: &str, value: &str) {
        self.headers.insert(name.to_string(), value.to_string());
    }

    pub fn add_header_owned(&mut self, name: String, value: String) {
        self.headers.insert(name, value);
    }

    pub fn get_header(&self, name: &str) -> Option<&str> {
        return self.headers.get(name).map(|e| e.as_str());
    }
    pub fn set_body(&mut self, body: Vec<u8>) {
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

impl MSNPPayload for RawNfyPayload {
    type Err = PayloadError;

    fn try_from_bytes(mut bytes: Vec<u8>) -> Result<Self, Self::Err> {

        let mut found_count = 0;

        let header_body_split_index = bytes.windows(4).enumerate().find_map(|(index, content)| {
            if content[0] as char == '\r' && content[1] as char == '\n' && content[2] as char == '\r' && content[3] as char == '\n' {
                if found_count < 2 {
                    found_count+=1;
                    None
                } else {
                    Some(index + 1)
                }
            } else {
                None
            }
        });

        if header_body_split_index.is_none() {
            return Err(PayloadError::AnyError(anyhow!("Malformed Payload")));
        }

        let header_body_split_index = header_body_split_index.expect("to never fail");

        let (headers, body) = bytes.split_at_mut(header_body_split_index);

        let headers = from_utf8(headers)?;

        let headers_split: Vec<&str> = headers.split("\r\n\r\n").collect();

        if headers_split.len() < 3 {
            Err(anyhow!("PutPayload headers len too short"))?
        }

        let routing_info: Vec<&str> = headers_split.get(0).unwrap().split("\r\n").collect();
        let reliability_info: Vec<&str> = headers_split.get(1).unwrap().split("\r\n").collect();
        let payload_headers: Vec<&str> = headers_split.get(2).unwrap().split("\r\n").collect();

        let envelope = NfyEnvelope::from_parts(routing_info, reliability_info)?;

        let mut headers: LinkedHashMap<String, String> = LinkedHashMap::new();


        let mut content_type = None;

        for current in payload_headers {
            match current.split_once(":") {
                None => {
                    warn!("Malformed header, ignoring...: {}", current);
                }
                Some((name, value)) => {
                    let name = name.trim();
                    let value = value.trim();
                    if name == "Content-Type" {
                        content_type = Some(NfyContentType::from_str(value).map_err(|e| anyhow!("Couldnt parse content type {}", e))?);
                    } else {
                        headers.insert(name.to_string(), value.to_string());
                    }

                }
            }
        }

        let body = body[3..].to_vec();

        let out = RawNfyPayload {
            envelope,
            content_type: content_type.ok_or(anyhow!("Missing content-type"))?,
            headers,
            body,
            enable_trailing_terminators: false,
        };

        Ok(out)
    }

    fn into_bytes(mut self) -> Vec<u8> {
        let mut out = self.envelope.to_string().into_bytes();

        for (key, value) in self.headers {
            let key_lower = key.to_lowercase();
            if &key_lower == "content-length" || &key_lower == "content-type" {
                continue;
            }
            out.append(&mut format!("{}: {}\r\n", key, value).into_bytes());
        }

        out.append(&mut format!("Content-Type: {}\r\n", self.content_type).into_bytes());
        out.append(&mut format!("Content-Length: {}\r\n", self.body.len()).into_bytes());

        out.extend_from_slice(b"\r\n");

        out.append(&mut self.body);

        if self.enable_trailing_terminators {
            out.extend_from_slice(b"\r\n");
        }

        out
    }
}
