use chrono::{DateTime, Local};
use email_encoding::headers::writer::EmailWriter;
use log::Metadata;
use yaserde::ser::{Config, to_string, to_string_with_config};
use yaserde_derive::{YaDeserialize, YaSerialize};
use crate::shared::config::yaserde::CONFIG_NO_DECL;

use crate::shared::models::email_address::EmailAddress;
use crate::shared::models::uuid::Uuid;
use crate::soap::error::SoapMarshallError;
use crate::soap::traits::xml::ToXml;

#[derive(Debug, Default, YaSerialize, YaDeserialize, Clone)]
#[yaserde(
rename = "M",
)]
pub struct MetadataMessage {
    //Unknown, but has only been seen set to 11
    #[yaserde(rename = "T", default)]
    pub t: i32,
    //Unknown, but has only been seen set to 6
    #[yaserde(rename = "S", default)]
    pub s: i32,
    //The date/time stamp for when the message was received by the server
    #[yaserde(rename = "RT", default)]
    pub received_timestamp: Option<String>,
    //Unknown, but most likely is set to 1 if the message has been read before ("Read Set").
    #[yaserde(rename = "RS", default)]
    pub rs: u8,
    //The size of the message, including headers
    #[yaserde(rename = "SZ", default)]
    pub size: u32,
    #[yaserde(rename = "E", default)]
    pub sender_email_addr: String,
    //This is the ID of the message, which should be used later on to retrieve the message. Note that the ID is a GUID in the form XXXXXXXX-XXXX-XXXX-XXXX-XXXXXXXXXXXX.
    // It was previously (the change was first noticed in March 2007) in the format of "MSGunix-timestamp.millseconds" (for example MSG1132093467.11)
    // and the Message ID format could change again anytime.
    #[yaserde(rename = "I", default)]
    pub message_id: String,
    //Unknown, but has so far only been observed as either a GUID with a single 9 at the end, or as ".!!OIM" (in case you are already online when receiving the notification).
    #[yaserde(rename = "F", default)]
    pub f: String,
    //This field contains the friendlyname of the person, wrapped in a special encoding. This encoding is defined in RFC 2047, but to get you started there is a quick overview of the format below
    //When this field is found in a non-initial notification it will contain a space in the data field. You must filter this space (trim the string) in order to correctly decode this field!
    //    =?charset?encoding-type?data?=
    //    charset: The character set which should be used for the decoded text. This charset is usually UTF-8, although it may vary widly. For example, koi8-r is popular for encoding Russian text, while jcode is used for Japanese text.
    //    encoding-type: This can either be "B" or "Q". So far only "B" has been observed, but be prepared to handle "Q" as well. B uses Base 64 to encode its data, while Q means Quoted-Printable, which is similar to URL encoding, but uses "=" instead of "%".
    //    data: The data, either encoded with Base64 or Quoted-Printable.
    #[yaserde(rename = "N", default)]
    pub sender_display_name: String,
    //Unknown, has only been observed to contain one space.
    #[yaserde(rename = "SU", default)]
    pub su: String,
}

impl MetadataMessage {
    pub fn new(timetamp: DateTime<Local>, sender: EmailAddress, sender_display_name: String, message_id: String, message_size: usize) -> Self{
        //2005-11-15T22:24:27.000Z
        let ts = timetamp.format("%Y-%m-%dT%H:%M:%S%.3fZ").to_string();

        let mut encoded = String::new();
        {
            let mut writer = EmailWriter::new(&mut encoded, 0, 0, false);
            email_encoding::headers::rfc2047::encode(&sender_display_name, &mut writer).unwrap();
        }

        Self {
            t: 11,
            s: 6,
            received_timestamp: Some(ts),
            rs: 0,
            size: message_size as u32,
            sender_email_addr: sender.0,
            message_id,
            f: "00000000-0000-0000-0000-000000000009".to_string(),
            sender_display_name: encoded,
            su: " ".to_string(),
        }
    }
}

impl ToXml for MetadataMessage {
    type Error = SoapMarshallError;

    fn to_xml(&self) -> Result<String, Self::Error>  {
        to_string_with_config(self, &CONFIG_NO_DECL).map_err(|e| SoapMarshallError::SerializationError { message: e})
    }
}

#[derive(Debug, YaSerialize, YaDeserialize, Clone)]
#[yaserde(
rename = "Q",
)]
pub struct Q {
    #[yaserde(rename = "QTM", default)]
    pub qtm: i32,
    #[yaserde(rename = "QNM", default)]
    pub qnm: i32,
}

impl Default for Q {
    fn default() -> Self {
        Self {
            qtm: 409600,
            qnm: 204800,
        }
    }
}

#[derive(Debug, Default, YaSerialize, YaDeserialize, Clone)]
#[yaserde(
rename = "MD",
)]
pub struct MetaData {
    #[yaserde(rename = "E", default)]
    pub e: E,
    #[yaserde(rename = "Q", default)]
    pub q: Q,
    #[yaserde(rename = "M", default)]
    pub messages: Vec<MetadataMessage>
}

#[derive(Debug, YaSerialize, YaDeserialize, Clone)]
#[yaserde(
rename = "E",
)]
pub struct E {
    #[yaserde(rename = "I", default)]
    pub i: i32,
    #[yaserde(rename = "IU", default)]
    pub iu: i32,
    #[yaserde(rename = "O", default)]
    pub o: i32,
    #[yaserde(rename = "OU", default)]
    pub ou: i32
}

impl Default for E {
    fn default() -> Self {
        Self {
            i: 0,
            iu: 0,
            o: 0,
            ou: 0,
        }
    }
}


impl ToXml for MetaData {
    type Error = SoapMarshallError;

    fn to_xml(&self) -> Result<String, Self::Error>  {
        to_string_with_config(self, &CONFIG_NO_DECL).map_err(|e| SoapMarshallError::SerializationError { message: e})
    }
}

#[cfg(test)]
mod tests {
    use std::str::FromStr;

    use chrono::Local;

    use crate::shared::models::email_address::EmailAddress;
    use crate::shared::models::oim::{MetaData, MetadataMessage};
    use crate::soap::traits::xml::ToXml;

    #[test]
    fn ser_metadata_message() {

        let msg = MetadataMessage::new(Local::now(), EmailAddress::from_str("aeon@test.com").unwrap(), "Aeon".into(), "msgid".into(), 123);
        let str = msg.to_xml().unwrap();
        println!("{}", &str);

    }

    #[test]
    fn ser_metadata() {
        let msg = MetadataMessage::new(Local::now(), EmailAddress::from_str("aeon@test.com").unwrap(), "Aeon".into(), "msgid".into(), 123);
        let msg1 = MetadataMessage::new(Local::now(), EmailAddress::from_str("aeon2@test.com").unwrap(), "Aeon2".into(), "msgid2".into(), 123);

        let metadata = MetaData {
            messages: vec![msg, msg1],
            ..Default::default()
        };

        let str = metadata.to_xml().unwrap();
        println!("{}", &str);

    }

}