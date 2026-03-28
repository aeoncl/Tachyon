use std::fmt::{Display, Formatter};
use std::io::{Read, Write};
use std::str::{from_utf8, FromStr};

use chrono::{DateTime, Utc};
use xml::attribute::OwnedAttribute;
use xml::namespace::Namespace;
use yaserde::de::Deserializer;
use yaserde::ser::{to_string_with_config, Serializer};
use yaserde::{YaDeserialize, YaSerialize};
use yaserde_derive::{YaDeserialize, YaSerialize};

use crate::msnp::error::PayloadError;
use crate::shared::config::yaserde::CONFIG_NO_DECL;
use crate::shared::models::email_address::EmailAddress;
use crate::shared::models::uuid::Uuid;
use crate::shared::payload::msg::raw_msg_payload::factories::RawMsgPayloadFactory;
use crate::shared::payload::msg::raw_msg_payload::MsgContentType;
use crate::shared::traits::{IntoBytes, TryFromBytes};
use crate::soap::error::SoapMarshallError;
use crate::soap::traits::xml::ToXml;

#[derive(Debug, Default, YaSerialize, YaDeserialize, Clone)]
#[yaserde(
rename = "M",
)]
pub struct MailDataMessage {
    //Type, but has only been seen set to 11
    // Speculation:
    // 11 = OIM, 12 = SMS & 13 = Federated
    // SMS shares the same OIM syntax as OIM; but it starts to spin up a platform conversation with MobileDeviceType = 1; OIM Starts it with 0
    // Federated seems to want another type of OIM as the normal one fails validation : message from [redacted] failed validation. dropping & deleting this offline message.
    #[yaserde(rename = "T", default)]
    pub message_type: i32,
    //Paid, has only been seen set to 6
    #[yaserde(rename = "S", default)]
    pub paid: i32,
    //The date/time stamp for when the message was received by the server
    #[yaserde(rename = "RT", default)]
    pub received_timestamp: Option<String>,
    //ReadState, but most likely is set to 1 if the message has been read before ("Read Set").
    #[yaserde(rename = "RS", default)]
    pub read_state: u8,
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
    //FolderID, has so far only been observed as either a GUID with a single 9 at the end, or as ".!!OIM" (in case you are already online when receiving the notification).
    #[yaserde(rename = "F", default)]
    pub folder_id: String,
    //This field contains the friendlyname of the person, wrapped in a special encoding. This encoding is defined in RFC 2047, but to get you started there is a quick overview of the format below
    //When this field is found in a non-initial notification it will contain a space in the data field. You must filter this space (trim the string) in order to correctly decode this field!
    //    =?charset?encoding-type?data?=
    //    charset: The character set which should be used for the decoded text. This charset is usually UTF-8, although it may vary widly. For example, koi8-r is popular for encoding Russian text, while jcode is used for Japanese text.
    //    encoding-type: This can either be "B" or "Q". So far only "B" has been observed, but be prepared to handle "Q" as well. B uses Base 64 to encode its data, while Q means Quoted-Printable, which is similar to URL encoding, but uses "=" instead of "%".
    //    data: The data, either encoded with Base64 or Quoted-Printable.
    #[yaserde(rename = "N", default)]
    pub sender_display_name: String,
    //Subject, has only been observed to contain one space.
    #[yaserde(rename = "SU", default)]
    pub subject: String,
}

impl MailDataMessage {
    pub fn new(timestamp: DateTime<Utc>, sender: EmailAddress, sender_display_name: String, message_id: String, subject: String, message_size: usize, read: bool) -> Self{
        //2005-11-15T22:24:27.000Z
        let ts = timestamp.format("%Y-%m-%dT%H:%M:%S%.3fZ").to_string();

        let encoded = crate::shared::converters::rfc2047::encode(&sender_display_name);

        Self {
            message_type: 11,
            paid: 6,
            received_timestamp: Some(ts),
            read_state: {if read { 1 } else { 0 }},
            size: message_size as u32,
            sender_email_addr: sender.to_string(),
            message_id,
            folder_id: "00000000-0000-0000-0000-000000000009".to_string(),
            sender_display_name: encoded,
            subject,
        }
    }
}

impl ToXml for MailDataMessage {
    type Error = SoapMarshallError;

    fn to_xml(&self) -> Result<String, Self::Error>  {
        to_string_with_config(self, &CONFIG_NO_DECL).map_err(|e| SoapMarshallError::SerializationError { message: e})
    }
}

#[derive(Debug, YaSerialize, YaDeserialize, Clone)]
#[yaserde(
rename = "Q",
)]
pub struct Quota {
    #[yaserde(rename = "QTM", default)]
    pub quota_total_max_kb: i32,
    #[yaserde(rename = "QNM", default)]
    pub quota_now_max_kb: i32,
}

impl Default for Quota {
    fn default() -> Self {
        Self {
            quota_total_max_kb: 409600,
            quota_now_max_kb: 204800,
        }
    }
}

#[derive(Debug, Default, YaSerialize, YaDeserialize, Clone)]
#[yaserde(
rename = "MD",
)]
pub struct MailData {
    #[yaserde(rename = "E", default)]
    pub inbox_metadata: InboxMetadata,
    #[yaserde(rename = "Q", default)]
    pub quota: Quota,
    #[yaserde(rename = "M", default)]
    pub messages: Vec<MailDataMessage>
}

#[derive(Debug, YaSerialize, YaDeserialize, Clone)]
#[yaserde(
rename = "E",
)]
pub struct InboxMetadata {
    #[yaserde(rename = "I", default)]
    //Inbox total
    pub inbox_count: i32,
    #[yaserde(rename = "IU", default)]
    //Inbox unread mail
    pub inbox_unread: i32,
    #[yaserde(rename = "O", default)]
    //Sent + Junk + Drafts
    pub others_count: i32,
    #[yaserde(rename = "OU", default)]
    //Sent + Junk + Drafts Unread
    pub others_unread_count: i32
}

impl Default for InboxMetadata {
    fn default() -> Self {
        Self {
            inbox_count: 0,
            inbox_unread: 0,
            others_count: 0,
            others_unread_count: 0,
        }
    }
}


impl ToXml for MailData {
    type Error = SoapMarshallError;

    fn to_xml(&self) -> Result<String, Self::Error>  {
        to_string_with_config(self, &CONFIG_NO_DECL).map_err(|e| SoapMarshallError::SerializationError { message: e})
    }
}

#[derive(Debug, Clone, Default)]
pub struct OIM {
    pub recv_datetime: DateTime<Utc>,
    pub sender: EmailAddress,
    pub sender_display_name: Option<String>,
    pub receiver: EmailAddress,
    pub run_id: Uuid,
    pub seq_number: u32,
    pub message_id: String,
    pub content: String,
    pub content_type: MsgContentType,
    pub read: bool
}

impl FromStr for OIM {
    type Err = PayloadError;

    fn from_str(_s: &str) -> Result<Self, Self::Err> {
       // let msg_payload : RawMsgPayload = RawMsgPayload::from_str(s)?;
        todo!("Deserializing an OIM is not yet implemented")
    }
}

impl Display for OIM {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        let msg_payload = RawMsgPayloadFactory::get_oim(self.recv_datetime, self.sender.as_str(), self.sender_display_name.as_ref().map(|e| e.as_str()).unwrap_or(""), self.receiver.as_str(), self.run_id.to_string().as_str(), self.seq_number, self.message_id.as_str(), self.content.as_str(), self.content_type.clone());
        let bytes = msg_payload.into_bytes();

        write!(f, "{}", from_utf8(&bytes).expect("OIM payload to never contain binary data"))
    }
}

impl YaSerialize for OIM {
    fn serialize<W: Write>(&self, writer: &mut Serializer<W>) -> Result<(), String> {


        let start_name = writer.get_start_event_name().unwrap_or("OIM".into());
        let _ret = writer.write(xml::writer::XmlEvent::start_element(start_name.as_str()));

        let _ret = writer.write(xml::writer::XmlEvent::characters(
            &self.to_string(),
        ));
        let _ret = writer.write(xml::writer::XmlEvent::end_element());

        Ok(())
    }

    fn serialize_attributes(&self, attributes: Vec<OwnedAttribute>, namespace: Namespace) -> Result<(Vec<OwnedAttribute>, Namespace), String> {
        Ok((attributes, namespace))
    }
}


impl YaDeserialize for OIM {
    fn deserialize<R: Read>(reader: &mut Deserializer<R>) -> Result<Self, String> {
        if let xml::reader::XmlEvent::StartElement { name: _, attributes: _, namespace: _ } = reader.peek()?.to_owned() {
            let _next = reader.next_event();
        }
        if let xml::reader::XmlEvent::Characters(text) = reader.peek()?.to_owned() {
            let oim = OIM::from_str(text.as_str()).map_err(|e| e.to_string())?;
            Ok(oim)
        } else {
            Err("Characters missing".to_string())
        }
    }
}



#[cfg(test)]
mod tests {
    use std::str::FromStr;

    use chrono::Local;

    use crate::shared::models::email_address::EmailAddress;
    use crate::shared::models::oim::{MailData, MailDataMessage};
    use crate::soap::traits::xml::ToXml;

    #[test]
    fn ser_metadata_message() {

        let msg = MailDataMessage::new(Local::now().to_utc(), EmailAddress::from_str("aeon@test.com").unwrap(), "Aeon".into(), "msgid".into(), String::new(), 123, false);
        let str = msg.to_xml().unwrap();
        println!("{}", &str);

    }

    #[test]
    fn ser_metadata() {
        let msg = MailDataMessage::new(Local::now().to_utc(), EmailAddress::from_str("aeon@test.com").unwrap(), "Aeon".into(), "msgid".into(), String::new(), 123, false);
        let msg1 = MailDataMessage::new(Local::now().to_utc(), EmailAddress::from_str("aeon2@test.com").unwrap(), "Aeon2".into(), "msgid2".into(),String::new(),123, false);

        let metadata = MailData {
            messages: vec![msg, msg1],
            ..Default::default()
        };

        let str = metadata.to_xml().unwrap();
        println!("{}", &str);

    }

}