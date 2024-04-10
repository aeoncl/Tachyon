use std::{convert::Infallible, fmt::Display, io::Read, str::FromStr};
use actix_web::dev::Payload;
use anyhow::anyhow;

use matrix_sdk::ruma::presence::PresenceState;
use strum_macros::{EnumString, ToString};
use substring::Substring;
use yaserde::{de::{self, from_str}, ser::to_string};
use yaserde_derive::{YaDeserialize, YaSerialize};

use crate::models::{capabilities::ClientCapabilities};
use crate::models::tachyon_error::PayloadError;

#[derive(Debug, Clone, Default, YaSerialize, YaDeserialize)]

pub struct EndpointData {

    #[yaserde(rename = "id", attribute)]
    pub machine_guid: Option<String>,
    #[yaserde(rename = "Capabilities")]
    pub capabilities: ClientCapabilities,

}

impl EndpointData {
    pub fn new(machine_guid: Option<String>, capabilities: ClientCapabilities) -> Self {
        return EndpointData{
            machine_guid,
            capabilities,
        };
    }
}

impl FromStr for EndpointData {
    type Err = PayloadError;

    fn from_str(s: &str) -> Result<Self, Self::Err> { 
        from_str::<EndpointData>(s).map_err(|e| PayloadError::StringPayloadParsingError { payload: s.to_string(), sauce: anyhow!("Couldn't parse EndpointData Payload") })
    }
}

impl Display for EndpointData {


    //Todo remove this fmt method, you can configure yaserde to remove the XML tag
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        if let Ok(serialized) = to_string(self) {
            let wesh = serialized.substring(38, serialized.len());
            return write!(f, "{}", wesh);
        } else {
            return Err(std::fmt::Error);
        }
     
    }
}


#[derive(Debug, Clone, Default, YaSerialize, YaDeserialize)]

pub struct PrivateEndpointData {

    #[yaserde(rename = "id", attribute)]
    pub machine_guid: Option<String>,
    #[yaserde(rename = "EpName")]
    pub ep_name: String,
    #[yaserde(rename = "Idle")]
    pub idle: bool,
    #[yaserde(rename = "ClientType")]
    pub client_type: ClientType,
    #[yaserde(rename = "State")]
    pub state: PresenceStatus
}

impl PrivateEndpointData {
    pub fn new(machine_guid: Option<String>, ep_name: String, idle: bool, client_type: ClientType, state: PresenceStatus) -> Self {
        return PrivateEndpointData { machine_guid, ep_name, idle, client_type, state };
    }
}


impl FromStr for PrivateEndpointData {
    type Err = PayloadError;

    fn from_str(s: &str) -> Result<Self, PayloadError> {
       from_str::<PrivateEndpointData>(s).map_err(|e| PayloadError::StringPayloadParsingError { payload: s.to_string(), sauce: anyhow!("Couldn't deserialize Private Endpoint Data: error: {}", e) } )
    }
}

impl Display for PrivateEndpointData {

    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        if let Ok(serialized) = to_string(self) {
            let wesh = serialized.substring(38, serialized.len());
            return write!(f, "{}", wesh);
        } else {
            return Err(std::fmt::Error);
        }
     
    }
}

#[derive(Debug, Clone)]

pub struct MPOPEndpoint {
    pub endpoint_data: EndpointData,
    pub private_endpoint_data: PrivateEndpointData
}

impl MPOPEndpoint {
    pub fn new(endpoint_data: EndpointData, private_endpoint_data: PrivateEndpointData) -> Self {
        return MPOPEndpoint {
            endpoint_data,
            private_endpoint_data,
        };
    }
}

impl Display for MPOPEndpoint {

    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {

        let endpoint_data = self.endpoint_data.to_string();
        let private_endpoint_data = self.private_endpoint_data.to_string();
        return write!(f, "{}{}", endpoint_data, private_endpoint_data);
    }
}


#[derive(Debug, Clone)]
pub enum ClientType {

    Computer = 1,
    Website = 2,
    Mobile = 3,
    Xbox = 4,
    Other = 5

}

impl Default for ClientType {
    fn default() -> Self {
        return ClientType::Other;
    }
}

impl TryFrom<i32> for ClientType {

    type Error = Infallible;

    fn try_from(v: i32) -> Result<Self, Self::Error> {

        match v {
            x if x == ClientType::Computer as i32 => Ok(ClientType::Computer),
            x if x == ClientType::Website as i32 => Ok(ClientType::Website),
            x if x == ClientType::Mobile as i32 => Ok(ClientType::Mobile),
            x if x == ClientType::Xbox as i32 => Ok(ClientType::Xbox),
            x if x == ClientType::Other as i32 => Ok(ClientType::Other),
            _ => {
                Ok(ClientType::Other)
            }
        }
    }


}

impl yaserde::YaSerialize for ClientType {

    fn serialize<W: std::io::Write>(&self, writer: &mut yaserde::ser::Serializer<W>) -> Result<(), String> {
        let _ret = writer.write(xml::writer::XmlEvent::start_element("ClientType"));
        let _ret = writer.write(xml::writer::XmlEvent::characters(
          &(self.clone() as i32).to_string(),
        ));
        let _ret = writer.write(xml::writer::XmlEvent::end_element());
        Ok(())
    }

    fn serialize_attributes(&self, attributes: Vec<xml::attribute::OwnedAttribute>, namespace: xml::namespace::Namespace) -> Result<(Vec<xml::attribute::OwnedAttribute>, xml::namespace::Namespace), String> {
        Ok((attributes, namespace))
    }
}

impl yaserde::YaDeserialize for ClientType {

    fn deserialize<R: Read>(reader: &mut de::Deserializer<R>) -> Result<Self, String>{

        if let xml::reader::XmlEvent::StartElement { name, .. } = reader.peek()?.to_owned() {
          let expected_name = "ClientType".to_owned();
          if name.local_name != expected_name {
            return Err(format!(
              "Wrong StartElement name: {}, expected: {}",
              name, expected_name
            ));
          }
          let _next = reader.next_event();
        } else {
          return Err("StartElement missing".to_string());
        }
  
        if let xml::reader::XmlEvent::Characters(text) = reader.peek()?.to_owned() {

          let text_parsed : i32 = FromStr::from_str(text.as_str()).unwrap();

          Ok(ClientType::try_from(text_parsed).unwrap())
        } else {
          Err("Characters missing".to_string())
        }
    }

}




#[derive(Debug, Clone, ToString, EnumString, YaSerialize, YaDeserialize, PartialEq, Eq)]
pub enum PresenceStatus {

    /* Online */
    NLN,
    /* Busy */
    BSY,
    /* Away */
    AWY,
    /* Hidden */
    HDN,
    /* Be Right Back */
    BRB,
    /* Idle */
    IDL,
    /* Phone */
    PHN,
    /* Lunch */
    LUN,
    /* Disconnected */
    FLN
}

impl Default for PresenceStatus {

    fn default() -> Self {
        return PresenceStatus::HDN;
    }
}

impl From<PresenceState> for PresenceStatus {

    fn from(matrix_presence: PresenceState) -> Self {
        match matrix_presence {
            PresenceState::Online => {
                PresenceStatus::NLN
            },
            PresenceState::Unavailable => {
                PresenceStatus::AWY
            },
            PresenceState::Offline => {
                PresenceStatus::default()
            }
            _ => {
                PresenceStatus::default()
            }
        }
    }
}

impl Into<PresenceState> for PresenceStatus {


    fn into(self) -> PresenceState {
        match self {
            PresenceStatus::NLN => {
                PresenceState::Online
            },
            PresenceStatus::HDN | PresenceStatus::FLN => {
                PresenceState::Offline
            },
            _ => {
                PresenceState::Unavailable
            }
        }
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

impl Display for NotificationPayload {

    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        if let Ok(serialized) = to_string(self) {
            let wesh = serialized.substring(38, serialized.len());
            return write!(f, "{}", wesh);
        } else {
            return Err(std::fmt::Error);
        }
     
    }
}


#[derive(Default, YaSerialize, YaDeserialize)]
#[yaserde(rename = "MSG")]
pub struct Message{
    #[yaserde(rename = "id", attribute)]
    id: i32,
    #[yaserde(rename = "SUBSCR", attribute)]
    subscriber: Url,
    #[yaserde(rename = "ACTION", attribute)]
    action: Url,
    #[yaserde(rename = "BODY", attribute)]
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
    service: String,
    #[yaserde(rename = "CID")]
    cid: i64,
    #[yaserde(rename = "LastModifiedDate")]
    last_modified_date: String,
    #[yaserde(rename = "HasNewItem")]
    has_new_item: bool

}

pub mod factories {
    use chrono::Local;
    use yaserde::ser::to_string;

    use crate::models::uuid::UUID;

    use super::{Message, NotificationData, NotificationPayload, Recipient, Url, Via};

    pub struct NotificationFactory;

    impl NotificationFactory {

        pub fn get_abch_updated(uuid: &UUID, msn_addr: String) -> NotificationPayload {
            let recipient_pid = format!("0x{}:0x{}", uuid.get_least_significant_bytes_as_hex(), uuid.get_most_significant_bytes_as_hex());
            let recipient = Recipient{ pid: recipient_pid, name: msn_addr, via: Via{ agent: String::from("messenger") } };
    
            let now = Local::now().format("%Y-%m-%dT%H:%M:%SZ").to_string();
    
            let body = NotificationData{ service: String::from("ABCHInternal"), cid: uuid.to_decimal_cid(), last_modified_date: now, has_new_item: true };
    
            let body_serialized = html_escape::encode_text(to_string(&body).unwrap().as_str()).into_owned();
    
            let message = Message{ id: 0, subscriber: Url{ url: String::from("s.htm")}, action: Url{ url: String::from("a.htm")}, body: body_serialized };
    
            return NotificationPayload{ id: 0, site_id: 45705, site_url: String::from("http://contacts.msn.com"), to: recipient, message: message };
        }

        pub fn test(uuid: &UUID, msn_addr: String) -> String {
            let now = Local::now().format("%Y-%m-%dT%H:%M:%SZ").to_string();

            let mut template = String::from("<NOTIFICATION id=\"0\" siteid=\"45705\" siteurl=\"http://contacts.msn.com\">\r\n<TO pid=\"0x%recipient_low%:0x%recipient_high%\" name=\"%recipient_email%\">\r\n<VIA agent=\"messenger\" />\r\n</TO>\r\n<MSG id=\"0\">\r\n<SUBSCR url=\"s.htm\" />\r\n<ACTION url=\"a.htm\" />\r\n<BODY>\r\n&lt;NotificationData xmlns:xsd=\"http://www.w3.org/2001/XMLSchema\" xmlns:xsi=\"http://www.w3.org/2001/XMLSchema-instance\"&gt;\r\n&lt;Service&gt;%service%&lt;/Service&gt;\r\n&lt;CID&gt;%cid%&lt;/CID&gt;\r\n&lt;LastModifiedDate&gt;%last_modified_date%&lt;/LastModifiedDate&gt;\r\n&lt;HasNewItem&gt;%has_new_item%&lt;/HasNewItem&gt;\r\n&lt;/NotificationData&gt;\r\n</BODY>\r\n</MSG>\r\n</NOTIFICATION>");
            template = template.replace("%recipient_low%", uuid.get_least_significant_bytes_as_hex().as_str());
            template = template.replace("%recipient_high%", uuid.get_most_significant_bytes_as_hex().as_str());
            template = template.replace("%recipient_email%", msn_addr.as_str());
            template = template.replace("%cid%", uuid.to_decimal_cid().to_string().as_str());
            template = template.replace("%last_modified_date%", now.as_str());
            template = template.replace("%has_new_item%", "true");
            template = template.replace("%service%", "ABCHInternal");

            return template;
        }

    }

}



#[cfg(test)]
mod tests {
    use std::str::FromStr;

    use crate::{generated::payloads::{ClientType, EndpointData, factories::NotificationFactory, PresenceStatus, PrivateEndpointData}, models::msn_user::MSNUser};

    #[test]
 fn test_private_endpoint_data() {
        //Arrange
        let command = "<PrivateEndpointData><EpName>M1CROW8Vl</EpName><Idle>true</Idle><ClientType>2</ClientType><State>AWY</State></PrivateEndpointData>";

        //Act
        let parsed = PrivateEndpointData::from_str(command).unwrap();

        //Assert
        assert_eq!(parsed.machine_guid, None);
        assert_eq!(parsed.ep_name, String::from("M1CROW8Vl"));
        assert_eq!(parsed.idle, true);
        assert!(matches!(parsed.client_type,ClientType::Website));
        assert!(matches!(parsed.state,PresenceStatus::AWY));

    }

    #[test]
    fn test_endpoint_data() {
        //Arrange
        let command = "<EndpointData id=\"machine_guid\"><Capabilities>2789003324:48</Capabilities></EndpointData>";

        //Act
        let parsed = EndpointData::from_str(command).unwrap();

        //Assert
        assert_eq!(parsed.machine_guid, Some("machine_guid".to_string()));
        assert_eq!(parsed.capabilities.to_string(), String::from("2789003324:48"));
    }

    #[test]
    fn test_serialize_private_endpoint_data() {
           //Arrange
           let command = PrivateEndpointData{ machine_guid: None, ep_name: String::from("M1CROW8Vl"), idle: true, client_type: ClientType::Website, state: PresenceStatus::AWY };

           //Act
           let parsed = command.to_string();
   
           //Assert
           assert_eq!(parsed, String::from("<PrivateEndpointData><EpName>M1CROW8Vl</EpName><Idle>true</Idle><ClientType>2</ClientType><State>AWY</State></PrivateEndpointData>"));
   
    }

    #[test]
    fn ab_notification_test() {
        let msn_user = MSNUser::new("aeon.shl@shl.local".to_string());
        let notif = NotificationFactory::get_abch_updated(&msn_user.get_uuid(), msn_user.get_msn_addr());

        let notif_legacy = NotificationFactory::test(&msn_user.get_uuid(), msn_user.get_msn_addr());
        assert_eq!(notif.to_string(), notif_legacy);
    }
}