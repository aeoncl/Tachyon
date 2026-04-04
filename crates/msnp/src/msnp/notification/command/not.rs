use crate::msnp::error::PayloadError;
use crate::msnp::raw_command_parser::RawCommand;
use crate::shared::traits::{IntoBytes, TryFromBytes, TryFromRawCommand};
use crate::soap::error::SoapMarshallError;
use crate::soap::traits::xml::ToXml;
use yaserde::ser::to_string_with_config;
use yaserde_derive::{YaDeserialize, YaSerialize};

//NOT {payload_size}\r\n{payload}
pub struct NotServer {
    pub payload: NotificationPayloadType
}


pub enum NotificationPayloadType {
    Normal(NotificationPayload),
    Raw(String)
}


impl IntoBytes for NotificationPayloadType {
    fn into_bytes(self) -> Vec<u8> {
        match self {
            NotificationPayloadType::Normal(content) => { content.into_bytes() }
            NotificationPayloadType::Raw(raw) => { raw.into_bytes() }
        }
    }
}

impl TryFromRawCommand for NotServer {
    type Err = PayloadError;

    fn try_from_raw(_raw: RawCommand) -> Result<Self, Self::Err> where Self: Sized {
        todo!()
    }

}

impl IntoBytes for NotServer {
    fn into_bytes(self) -> Vec<u8> {

        let mut payload = self.payload.into_bytes();
        let mut out = format!("NOT {}\r\n", payload.len()).into_bytes();
        out.append(&mut payload);

        out
    }
}


impl TryFromBytes for NotificationPayload {
    type Err = PayloadError;

    fn try_from_bytes(_bytes: Vec<u8>) -> Result<Self, Self::Err> where Self: Sized {
        todo!()
    }

}

impl IntoBytes for NotificationPayload {
    fn into_bytes(self) -> Vec<u8> {
        self.to_xml().expect("To work").into_bytes()
    }
}

#[derive(Default, YaSerialize, YaDeserialize)]
#[yaserde(rename = "NOTIFICATION")]
pub struct NotificationPayload {
    #[yaserde(rename = "id", attribute)]
    id: i32,
    #[yaserde(rename = "ver", attribute)]
    ver: Option<i32>,
    #[yaserde(rename = "siteid", attribute)]
    site_id: i32,
    #[yaserde(rename = "siteurl", attribute)]
    site_url: String,
    #[yaserde(rename = "TO")]
    to: Recipient,
    #[yaserde(rename = "MSG")]
    message: Message
}

impl ToXml for NotificationPayload {
    type Error = SoapMarshallError;

    fn to_xml(&self) -> Result<String, Self::Error> {

        let yaserde_cfg = yaserde::ser::Config{
            perform_indent: false,
            write_document_declaration: false,
            indent_string: None
        };

        to_string_with_config(self, &yaserde_cfg).map_err(|e| SoapMarshallError::SerializationError { message: e})
    }
}



#[derive(Default, YaSerialize, YaDeserialize)]
#[yaserde(rename = "MSG")]
pub struct Message{
    #[yaserde(rename = "id", attribute)]
    id: i32,
    #[yaserde(rename = "SUBSCR", default)]
    subscriber: Url,
    #[yaserde(rename = "ACTION", default)]
    action: Url,
    #[yaserde(rename = "BODY", default)]
    body: MessageBody
}

#[derive(YaSerialize, YaDeserialize)]
#[yaserde(rename = "BODY", flatten)]
pub enum MessageBodyContent {
    #[yaserde(flatten)]
    String(RawStringMessageBody),
    #[yaserde(flatten)]
    Text(TextMessageBody),
    #[yaserde(flatten)]
    TextXml(TextXmlMessageBody)
}

impl Default for MessageBodyContent {
    fn default() -> Self {
        Self::String(RawStringMessageBody::default())
    }
}

#[derive(Default, YaSerialize, YaDeserialize)]
pub struct RawStringMessageBody{

    #[yaserde(text, default)]
    content: String

}

impl RawStringMessageBody {
    pub fn new(str: String) -> Self {
        Self { content: str }
    }
}

#[derive(Default, YaSerialize, YaDeserialize)]
pub struct TextMessageBody{

    #[yaserde(rename = "TEXT", default)]
    content: String

}

impl TextMessageBody {
    pub fn new(str: String) -> Self {
        Self { content: str }
    }
}

#[derive(Default, YaSerialize, YaDeserialize)]
pub struct TextXmlMessageBody{

    #[yaserde(rename = "TEXTXML", default)]
    content: String

}

impl TextXmlMessageBody {
    pub fn new(str: String) -> Self {
        Self { content: str }
    }
}




#[derive(Default, YaSerialize, YaDeserialize)]
#[yaserde(rename = "BODY")]
pub struct MessageBody {
    #[yaserde(rename = "lang", attribute)]
    lang: Option<String>,

    #[yaserde(rename = "icon", attribute)]
    icon: Option<String>,

    #[yaserde(flatten, default)]
    content: MessageBodyContent

}

#[derive(Default, YaSerialize, YaDeserialize)]
pub struct Url {
    #[yaserde(rename = "url", attribute)]
    url: String
}

#[derive(Default, YaSerialize, YaDeserialize)]
#[yaserde(rename = "TO")]
pub struct Recipient {
    /*0x%recipient_low%:0x%recipient_high% */
    #[yaserde(rename = "pid", attribute)]
    pid: String,
    /* recipient email */
    #[yaserde(rename = "name", attribute)]
    name: String,

    #[yaserde(rename = "email", attribute)]
    email: Option<String>,

    #[yaserde(rename = "VIA")]
    via: Option<Via>

}

#[derive(Default, YaSerialize, YaDeserialize)]
#[yaserde(rename = "VIA")]
pub struct Via {
    #[yaserde(rename = "agent", attribute)]
    agent: String

}

#[derive(Default, YaSerialize, YaDeserialize)]
#[yaserde(rename = "NotificationData",
    namespace="xsd: http://www.w3.org/2001/XMLSchema",
    namespace="xsi: http://www.w3.org/2001/XMLSchema-instance"
)]
pub struct NotificationData {

    #[yaserde(rename = "Service")]
    service: Option<String>,
    #[yaserde(rename = "CID")]
    cid: Option<i64>,
    #[yaserde(rename = "LastModifiedDate")]
    last_modified_date: Option<String>,
    #[yaserde(rename = "HasNewItem")]
    has_new_item: bool,
    #[yaserde(rename = "CircleId")]
    circle_id: Option<String>,

}

impl ToXml for NotificationData {
    type Error = SoapMarshallError;

    fn to_xml(&self) -> Result<String, Self::Error> {
        let yaserde_cfg = yaserde::ser::Config{
            perform_indent: false,
            write_document_declaration: false,
            indent_string: None
        };

        to_string_with_config(self, &yaserde_cfg).map_err(|e| SoapMarshallError::SerializationError { message: e})
    }
}



pub mod factories {
    use chrono::Local;

    use crate::shared::models::email_address::EmailAddress;
    use crate::shared::models::uuid::Uuid;
    use crate::soap::traits::xml::ToXml;

    use super::{Message, MessageBody, MessageBodyContent, NotificationData, NotificationPayload, RawStringMessageBody, Recipient, TextMessageBody, Url, Via};

    pub struct NotificationFactory;

    impl NotificationFactory {

        pub fn get_abch_updated(uuid: &Uuid, msn_addr: &EmailAddress) -> NotificationPayload {
            let recipient_pid = format!("0x{}:0x{}", uuid.get_least_significant_bytes_as_hex(), uuid.get_most_significant_bytes_as_hex());
            let recipient = Recipient{ pid: recipient_pid, name: msn_addr.to_string(), email: None, via: Some(Via{ agent: String::from("messenger") }) };
    
            let now = Local::now().format("%Y-%m-%dT%H:%M:%SZ").to_string();
    
            let body = NotificationData{ service: Some(String::from("ABCHInternal")), cid: Some(uuid.to_decimal_cid()), last_modified_date: Some(now), has_new_item: true, circle_id: None };

            let notification_data_ser = body.to_xml().unwrap();

            let message = Message{ id: 0, subscriber: Url{ url: String::from("s.htm")}, action: Url{ url: String::from("a.htm")}, body:  MessageBody{
                lang: None,
                icon: None,
                content:  MessageBodyContent::String(RawStringMessageBody::new(notification_data_ser)),
            } };
    
            NotificationPayload{ id: 0, ver: None, site_id: 45705, site_url: String::from("http://contacts.msn.com"), to: recipient, message }
        }

        pub fn get_circle_updated(me_uuid: &Uuid, me_msn_addr: &str, circle_id: &str) -> NotificationPayload {
            let recipient_pid = format!("0x{}:0x{}", me_uuid.get_least_significant_bytes_as_hex(), me_uuid.get_most_significant_bytes_as_hex());
            let recipient = Recipient{ pid: recipient_pid, name: me_msn_addr.to_string(), email: None, via: Some(Via{ agent: String::from("messenger") }) };

            let body = NotificationData{ service: None, cid: None, last_modified_date: None, has_new_item: true, circle_id: Some(circle_id.to_string()) };

            let notification_data_ser = body.to_xml().unwrap();

            let message = Message{ id: 0, subscriber: Url{ url: String::from("s.htm")}, action: Url{ url: String::from("a.htm")}, body: MessageBody{
                lang: None,
                icon: None,
                content:  MessageBodyContent::String(RawStringMessageBody::new(notification_data_ser)),
            }  };

            NotificationPayload{ id: 0, ver: None, site_id: 45705, site_url: String::from("http://contacts.msn.com"), to: recipient, message }

        }

        pub fn text(uuid: &Uuid, msn_addr: &str, msg: &str) -> NotificationPayload {
            let recipient_pid = format!("0x{}:0x{}", uuid.get_least_significant_bytes_as_hex(), uuid.get_most_significant_bytes_as_hex());
            let recipient = Recipient{ pid: recipient_pid, name: msn_addr.to_string(), email: None, via: Some(Via{ agent: String::from("messenger") }) };

            let message = Message{ id: 1, subscriber: Url{ url: String::from("s.htm")}, action: Url{ url: String::from("a.htm")}, body: MessageBody{
                lang: None,
                icon: None,
                content:  MessageBodyContent::String(RawStringMessageBody::new(format!("<XML>{}</XML>", msg))),
            }     };

            NotificationPayload {
                id: 1,
                ver: None,
                site_id: 45705,
                site_url: "http://alerts.msn.com".to_string(),
                to: recipient,
                message: message,
            }
        }

        pub fn alert(uuid: &Uuid, msn_addr: &EmailAddress, msg: &str, site_url: &str, subscribe_url: &str, action_url: &str, icon: Option<&str>, id: i32) -> NotificationPayload {
            let recipient_pid = format!("0x{}:0x{}", uuid.get_least_significant_bytes_as_hex(), uuid.get_most_significant_bytes_as_hex());
            let recipient = Recipient{ pid: recipient_pid, name: msn_addr.to_string(), email: None, via: None };

            let message = Message{ id: 1, subscriber: Url{ url: subscribe_url.to_string()}, action: Url{ url: action_url.to_string()}, body:  MessageBody{
                lang: Some("3076".to_string()),
                icon: icon.or(Some("")).map(|s| s.to_string()),
                content:  MessageBodyContent::Text(TextMessageBody::new(msg.to_string())),
            }
            };

            NotificationPayload {
                id,
                ver: Some(2),
                site_id: 199999999,
                site_url: site_url.to_string(),
                to: recipient,
                message,
            }
        }

        pub fn alert_test(uuid: &Uuid, msn_addr: &str, msg: &str) -> String {

            let recipient_pid = format!("0x{}:0x{}", uuid.get_least_significant_bytes_as_hex(), uuid.get_most_significant_bytes_as_hex());


            let test = format!(r#"<NOTIFICATION id="1342902633" ver="2" siteid="199999999" siteurl="http://127.0.0.1:8080/ads"><TO pid="{pid}" name="{email}" email="{email}"/><MSG id="1"><SUBSCR url="http://127.0.0.1:8080/tachyon?s=true" /><ACTION url="http://127.0.0.1:8080/tachyon?a=true" /><BODY lang="3076" icon="spongebob-icon_32x32.png"><TEXT>{text}</TEXT></BODY></MSG></NOTIFICATION>"#, pid = recipient_pid, email = msn_addr, text=msg);


            test
        }


        pub fn test(uuid: &Uuid, msn_addr: &str) -> String {
            let now = Local::now().format("%Y-%m-%dT%H:%M:%SZ").to_string();

            let mut template = String::from("<NOTIFICATION id=\"0\" siteid=\"45705\" siteurl=\"http://contacts.msn.com\">\r\n<TO pid=\"0x%recipient_low%:0x%recipient_high%\" name=\"%recipient_email%\">\r\n<VIA agent=\"messenger\" />\r\n</TO>\r\n<MSG id=\"0\">\r\n<SUBSCR url=\"s.htm\" />\r\n<ACTION url=\"a.htm\" />\r\n<BODY>\r\n&lt;NotificationData xmlns:xsd=\"http://www.w3.org/2001/XMLSchema\" xmlns:xsi=\"http://www.w3.org/2001/XMLSchema-instance\"&gt;\r\n&lt;Service&gt;%service%&lt;/Service&gt;\r\n&lt;CID&gt;%cid%&lt;/CID&gt;\r\n&lt;LastModifiedDate&gt;%last_modified_date%&lt;/LastModifiedDate&gt;\r\n&lt;HasNewItem&gt;%has_new_item%&lt;/HasNewItem&gt;\r\n&lt;/NotificationData&gt;\r\n</BODY>\r\n</MSG>\r\n</NOTIFICATION>");
            template = template.replace("%recipient_low%", uuid.get_least_significant_bytes_as_hex().as_str());
            template = template.replace("%recipient_high%", uuid.get_most_significant_bytes_as_hex().as_str());
            template = template.replace("%recipient_email%", msn_addr);
            template = template.replace("%cid%", uuid.to_decimal_cid().to_string().as_str());
            template = template.replace("%last_modified_date%", now.as_str());
            template = template.replace("%has_new_item%", "true");
            template = template.replace("%service%", "ABCHInternal");

            template
        }

    }

}


#[cfg(test)]
mod tests {
    use std::str::FromStr;

    use crate::shared::models::email_address::EmailAddress;
    use crate::soap::traits::xml::ToXml;
    use crate::{msnp::notification::command::not::factories::NotificationFactory, shared::models::msn_user::MsnUser};

    #[test]
    fn ab_alert_notification_test() {
        let msn_user = MsnUser::without_endpoint_guid(EmailAddress::from_str("aeon.shl@shl.local").unwrap());
        let notif = NotificationFactory::alert(&msn_user.uuid, &msn_user.endpoint_id.email_addr, "this is an alert", "http://127.0.0.1:8080/ads", "http://127.0.0.1:8080/tachyon?s=true", "http://127.0.0.1:8080/tachyon?a=true", Some("spongebob-icon_32x32.png"));

        let notif_ser = notif.to_xml().unwrap();

        let notif_legacy = r#"<NOTIFICATION id="1342902633" ver="2" siteid="199999999" siteurl="http://127.0.0.1:8080/ads"><TO pid="0x97f337aa885ae2bc:0x4d555fd33077064c" name="aeon.shl@shl.local" /><MSG id="1"><SUBSCR url="http://127.0.0.1:8080/tachyon?s=true" /><ACTION url="http://127.0.0.1:8080/tachyon?a=true" /><BODY lang="3076" icon="spongebob-icon_32x32.png"><TEXT>this is an alert</TEXT></BODY></MSG></NOTIFICATION>"#;

        assert_eq!(&notif_ser, notif_legacy);
    }

    #[test]
    fn ab_notification_test_2() {
        let msn_user = MsnUser::without_endpoint_guid(EmailAddress::from_str("aeon.shl@shl.local").unwrap());
        let notif = NotificationFactory::get_abch_updated(&msn_user.uuid, &msn_user.endpoint_id.email_addr);

        println!("{}", notif.to_xml().unwrap());

        let notif_legacy = NotificationFactory::test(&msn_user.uuid, &msn_user.endpoint_id.email_addr.as_str());
        assert_eq!(notif.to_xml().unwrap().as_str(), notif_legacy.replace("\r\n", ""));
    }
}