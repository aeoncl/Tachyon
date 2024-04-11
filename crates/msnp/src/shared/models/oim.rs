use yaserde_derive::{YaDeserialize, YaSerialize};
use crate::shared::models::uuid::Uuid;

#[derive(Debug, Default, YaSerialize, YaDeserialize, Clone)]
#[yaserde(
rename = "MetadataMessage",
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
    pub rs: bool,
    //The size of the message, including headers
    #[yaserde(rename = "SZ", default)]
    pub size: i32,
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
#[derive(Debug, Default, YaSerialize, YaDeserialize, Clone)]
#[yaserde(
rename = "Q",
)]
pub struct Q {
    #[yaserde(rename = "QTM", default)]
    pub qtm: i32,
    #[yaserde(rename = "QNM", default)]
    pub qnm: i32,
}
#[derive(Debug, Default, YaSerialize, YaDeserialize, Clone)]
#[yaserde(
rename = "MetaData",
)]
pub struct MetaData {
    #[yaserde(rename = "M", default)]
    pub messages: Vec<MetadataMessage>,
    #[yaserde(rename = "Q", default)]
    pub q: Q,
}