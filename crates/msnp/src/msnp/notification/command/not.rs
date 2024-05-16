use std::fmt::Display;

use yaserde::ser::to_string_with_config;
use yaserde_derive::{YaDeserialize, YaSerialize};
use crate::msnp::error::PayloadError;
use crate::msnp::raw_command_parser::RawCommand;
use crate::shared::traits::{MSNPCommand, MSNPPayload};
use crate::soap::error::SoapMarshallError;
use crate::soap::traits::xml::ToXml;

//NOT {payload_size}\r\n{payload}
pub struct NotServer {
    pub payload: NotificationPayload
}

impl MSNPCommand for NotServer {
    type Err = PayloadError;

    fn try_from_raw(raw: RawCommand) -> Result<Self, Self::Err> where Self: Sized {
        todo!()
    }

    fn into_bytes(self) -> Vec<u8> {

        let mut payload = self.payload.into_bytes();
        let mut out = format!("NOT {}\r\n", payload.len()).into_bytes();
        out.append(&mut payload);

        out
    }
}


impl MSNPPayload for NotificationPayload {
    type Err = PayloadError;

    fn try_from_bytes(bytes: Vec<u8>) -> Result<Self, Self::Err> where Self: Sized {
        todo!()
    }

    fn into_bytes(self) -> Vec<u8> {
        self.to_xml().expect("To work").into_bytes()
    }
}

#[derive(Default, YaSerialize, YaDeserialize)]
#[yaserde(rename = "NOTIFICATION")]
pub struct NotificationPayload {
    #[yaserde(rename = "id", attribute)]
    id: i32,
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
    body: String
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
    #[yaserde(rename = "VIA")]
    via: Via

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
    use yaserde::ser::to_string;


    use crate::shared::models::uuid::Uuid;
    use crate::soap::traits::xml::ToXml;

    use super::{Message, NotificationData, NotificationPayload, Recipient, Url, Via};

    pub struct NotificationFactory;

    impl NotificationFactory {

        pub fn get_abch_updated(uuid: &Uuid, msn_addr: &str) -> NotificationPayload {
            let recipient_pid = format!("0x{}:0x{}", uuid.get_least_significant_bytes_as_hex(), uuid.get_most_significant_bytes_as_hex());
            let recipient = Recipient{ pid: recipient_pid, name: msn_addr.to_string(), via: Via{ agent: String::from("messenger") } };
    
            let now = Local::now().format("%Y-%m-%dT%H:%M:%SZ").to_string();
    
            let body = NotificationData{ service: Some(String::from("ABCHInternal")), cid: Some(uuid.to_decimal_cid()), last_modified_date: Some(now), has_new_item: true, circle_id: None };

            let notification_data_ser = body.to_xml().unwrap();

            let message = Message{ id: 0, subscriber: Url{ url: String::from("s.htm")}, action: Url{ url: String::from("a.htm")}, body: notification_data_ser };
    
            NotificationPayload{ id: 0, site_id: 45705, site_url: String::from("http://contacts.msn.com"), to: recipient, message }
        }

        pub fn get_circle_updated(me_uuid: &Uuid, me_msn_addr: &str, circle_id: &str) -> NotificationPayload {
            let recipient_pid = format!("0x{}:0x{}", me_uuid.get_least_significant_bytes_as_hex(), me_uuid.get_most_significant_bytes_as_hex());
            let recipient = Recipient{ pid: recipient_pid, name: me_msn_addr.to_string(), via: Via{ agent: String::from("messenger") } };

            let body = NotificationData{ service: None, cid: None, last_modified_date: None, has_new_item: true, circle_id: Some(circle_id.to_string()) };

            let notification_data_ser = body.to_xml().unwrap();

            let message = Message{ id: 0, subscriber: Url{ url: String::from("s.htm")}, action: Url{ url: String::from("a.htm")}, body: notification_data_ser };

            NotificationPayload{ id: 0, site_id: 45705, site_url: String::from("http://contacts.msn.com"), to: recipient, message }

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

    use crate::{msnp::notification::command::not::factories::NotificationFactory, shared::models::msn_user::MsnUser};
    use crate::shared::models::email_address::EmailAddress;
    use crate::soap::traits::xml::ToXml;

    #[test]
    fn ab_notification_test_2() {
        let msn_user = MsnUser::without_endpoint_guid(EmailAddress::from_str("aeon.shl@shl.local").unwrap());
        let notif = NotificationFactory::get_abch_updated(&msn_user.uuid, &msn_user.endpoint_id.email_addr.0);

        let notif_legacy = NotificationFactory::test(&msn_user.uuid, &msn_user.endpoint_id.email_addr.0);
        assert_eq!(notif.to_xml().unwrap().as_str(), notif_legacy.replace("\r\n", ""));
    }
}