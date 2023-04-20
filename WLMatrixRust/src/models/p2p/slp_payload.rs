
/* Amazing documentation for the P2P stack by msnp-sharp project
 * https://code.google.com/archive/p/msnp-sharp/wikis/KB_MSNC12_BinaryHeader.wiki
 */

use std::{str::{FromStr, from_utf8}, fmt::Display};

use base64::{Engine, engine::general_purpose};
use linked_hash_map::LinkedHashMap;
use log::warn;
use substring::Substring;

use crate::models::{errors::Errors, msn_user::MSNUser};

use super::slp_context::{PreviewData, MsnObject, SlpContext};

#[derive(Clone, Debug)]
pub struct SlpPayload {

    pub first_line: String,
    pub headers: LinkedHashMap<String, String>,
    pub body: LinkedHashMap<String, String>
}

impl SlpPayload {
    pub fn new() -> Self {
        return SlpPayload{ first_line: String::new(), headers: LinkedHashMap::new(), body: LinkedHashMap::new()};
    }

    pub fn add_header(&mut self, name: String, value: String){
        self.headers.insert(name, value);
    }

    pub fn get_header(&self, name: &String) -> Option<&String> {
        return self.headers.get(name);
    }

    pub fn add_body_property(&mut self, name: String, value: String){
        self.body.insert(name, value);
    }

    pub fn get_body_property(&self, name: &String) -> Option<&String> {
        return self.body.get(name);
    }

    pub fn get_content_type(&self) -> Option<&String> {
        return self.get_header(&String::from("Content-Type"));
    }

    pub fn get_sender(&self) -> Option<MSNUser> {
        if let Some(from) = self.get_header(&String::from("From")) {
            let from_trimmed = from.to_owned().substring(1, from.len()-1).to_string();
            return Some(MSNUser::from_mpop_addr_string(from_trimmed).unwrap_or(MSNUser::default()));
        }
        return None;
    }

    pub fn get_receiver(&self) -> Option<MSNUser> {
        if let Some(to) = self.get_header(&String::from("To")) {
            let to_trimmed = to.to_owned().substring(1, to.len()-1).to_string();
            return Some(MSNUser::from_mpop_addr_string(to_trimmed).unwrap_or(MSNUser::default()));
        }
        return None;
    }

    pub fn get_context_as_preview_data(&self) -> Option<Box<PreviewData>> {
        if let Some(context) = self.get_body_property(&String::from("Context")) {
            if let Ok(decoded) = general_purpose::STANDARD.decode(context) {
                return PreviewData::from_slp_context(&decoded);
            } else {
                warn!("Couldn't decode base64 slp context: {}", context);
            }            
        }


        return None;
    }

    pub fn get_euf_guid(&self) -> Result<Option<EufGUID>, Errors> {
        let euf_guid = self.get_body_property(&String::from("EUF-GUID"));
        if euf_guid.is_none() {
            return Ok(None);
        }

        let euf_guid = euf_guid.unwrap();
        let euf_guid = EufGUID::try_from(euf_guid.as_str())?;
        return Ok(Some(euf_guid));
    }

    pub fn get_context_as_msnobj() -> Option<MsnObject> {
        return None;
    }

    pub fn is_invite(&self) -> bool {
        return self.first_line.contains("INVITE");
    }

    pub fn is_200_ok(&self) -> bool {
        return self.first_line.contains("200 OK");
    }

}

impl FromStr for SlpPayload {
    type Err = Errors;

    fn from_str(s: &str) -> Result<Self, Self::Err> {

        if let Some((headers, body)) = s.split_once("\r\n\r\n") {
            let mut out = SlpPayload::new();
            let headers_split: Vec<&str> = headers.split("\r\n").collect();

            out.first_line = headers_split.get(0).ok_or(Errors::PayloadDeserializeError)?.to_string();

            for i in 1..headers_split.len() {
                let current = headers_split.get(i).unwrap().to_string();

                if let Some((name, value)) =  current.split_once(":"){
                    out.add_header(name.trim().to_string(), value.trim().to_string());
                }
            }

            let body_split: Vec<&str> = body.split("\r\n").collect();
            for i in 0..body_split.len() {
                let current = body_split.get(i).unwrap().to_string();
                if let Some((name, value)) =  current.split_once(":"){
                    out.add_body_property(name.trim().to_string(), value.trim().to_string());
                }
            }

            return Ok(out);
        }
       return Err(Errors::PayloadDeserializeError);
    }
}

impl TryFrom<&Vec<u8>> for SlpPayload {
    type Error = Errors;

    fn try_from(value: &Vec<u8>) -> Result<Self, Self::Error> {
        let str = from_utf8(value)?;
        return SlpPayload::from_str(str);
    }
}

impl Display for SlpPayload {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {

        let mut out = self.first_line.clone();
        out.push_str("\r\n");

        let mut body = String::new();
        for (key, value) in &self.body {
            body.push_str(format!("{key}: {value}\r\n", key=&key, value=&value).as_str());
        }
        body.push_str("\r\n");
        body.push_str("\0");

        
        let mut headers = String::new();

        for (key, value) in &self.headers {
            headers.push_str(format!("{key}: {value}\r\n", key=&key, value=&value).as_str());
        }
        
        headers.push_str(format!("Content-Length: {value}\r\n", value=body.len()).as_str()); 

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
    Activity
}

impl Display for EufGUID {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        let mut out = "{00000000-0000-0000-0000-000000000000}";
        match self {
            EufGUID::MSNObject => {
                out = "{A4268EEC-FEC5-49E5-95C3-F126696BDBF6}";
            },
            EufGUID::FileTransfer => {
                out = "{5D3E02AB-6190-11D3-BBBB-00C04F795683}";
            },
            EufGUID::MediaReceiveOnly => {
                out = "{1C9AA97E-9C05-4583-A3BD-908A196F1E92}";
            },
            EufGUID::MediaSession => {
                out = "{4BD96FC0-AB17-4425-A14A-439185962DC8}"
            },
            EufGUID::SharePhoto => {
                out = "{41D3E74E-04A2-4B37-96F8-08ACDB610874}";
            },
            EufGUID::Activity => {
                out = "{6A13AF9C-5308-4F35-923A-67E8DDA40C2F}";
            }
        }
        return write!(f, "{}", &out);
    }
}

impl TryFrom<&str> for EufGUID {
    type Error = Errors;

    fn try_from(value: &str) -> Result<Self, Self::Error> {
        match value {
            "{A4268EEC-FEC5-49E5-95C3-F126696BDBF6}" => {
                return Ok(EufGUID::MSNObject);
            },
            "{5D3E02AB-6190-11D3-BBBB-00C04F795683}" => {
                return Ok(EufGUID::FileTransfer);
            },
            "{1C9AA97E-9C05-4583-A3BD-908A196F1E92}" => {
                return Ok(EufGUID::MediaReceiveOnly);
            },
            "{4BD96FC0-AB17-4425-A14A-439185962DC8}" => {
                return Ok(EufGUID::MediaSession);
            },
            "{41D3E74E-04A2-4B37-96F8-08ACDB610874}" => {
                return Ok(EufGUID::SharePhoto);
            },
            "{6A13AF9C-5308-4F35-923A-67E8DDA40C2F}" => {
                return Ok(EufGUID::Activity);
            },
            _=> {
                return Err(Errors::PayloadDeserializeError);
            }
        }
    }
}

mod tests {
    use crate::models::p2p::slp_payload::SlpPayload;

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
       payload.add_body_property(String::from("EUF-GUID"), String::from("{A4268EEC-FEC5-49E5-95C3-F126696BDBF6}"));

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