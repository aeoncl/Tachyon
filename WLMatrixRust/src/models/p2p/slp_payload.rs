
/* Amazing documentation for the P2P stack by msnp-sharp project
 * https://code.google.com/archive/p/msnp-sharp/wikis/KB_MSNC12_BinaryHeader.wiki
 */

use std::{str::{FromStr, from_utf8}, fmt::Display};

use linked_hash_map::LinkedHashMap;
use substring::Substring;

use crate::models::{errors::Errors, msn_user::MSNUser};

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




