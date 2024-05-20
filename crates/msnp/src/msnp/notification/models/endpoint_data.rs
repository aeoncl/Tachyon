use std::{convert::Infallible, fmt::Display, io::Read, str::FromStr};

use yaserde::{de::{self, from_str}, ser::{to_string, to_string_with_config}};
use yaserde_derive::{YaDeserialize, YaSerialize};

use crate::{msnp::error::PayloadError, shared::models::{capabilities::ClientCapabilities, presence_status::PresenceStatus}};

use anyhow::anyhow;
use crate::msnp::notification::models::endpoint_guid::EndpointGuid;

#[derive(Debug, Clone, Default, YaSerialize, YaDeserialize)]

pub struct EndpointData {

    #[yaserde(rename = "id", attribute)]
    pub machine_guid: Option<EndpointGuid>,
    #[yaserde(rename = "Capabilities")]
    pub capabilities: ClientCapabilities,

}

impl EndpointData {
    pub fn new(machine_guid: Option<EndpointGuid>, capabilities: ClientCapabilities) -> Self {
        return EndpointData{
            machine_guid,
            capabilities,
        };
    }
}

impl FromStr for EndpointData {
    type Err = PayloadError;

    fn from_str(s: &str) -> Result<Self, Self::Err> { 
        from_str::<EndpointData>(s).map_err(|e| PayloadError::StringPayloadParsingError { payload: s.to_string(), source: anyhow!("Couldn't parse EndpointData Payload") })
    }
}

impl Display for EndpointData {


    //Todo remove this fmt method, you can configure yaserde to remove the XML tag
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {

        let yaserde_cfg = yaserde::ser::Config{
            perform_indent: false,
            write_document_declaration: false,
            indent_string: None
        };
        
        if let Ok(serialized) = to_string_with_config(self, &yaserde_cfg) {
            return write!(f, "{}", serialized);
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
       from_str::<PrivateEndpointData>(s).map_err(|e| PayloadError::StringPayloadParsingError { payload: s.to_string(), source: anyhow!("Couldn't deserialize Private Endpoint Data: error: {}", e) } )
    }
}

impl Display for PrivateEndpointData {

    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {

        let yaserde_cfg = yaserde::ser::Config{
            perform_indent: false,
            write_document_declaration: false,
            indent_string: None
        };

        if let Ok(serialized) = to_string_with_config(self, &yaserde_cfg) {
            return write!(f, "{}", serialized);
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


#[cfg(test)]
mod tests {
    use std::str::FromStr;

    use crate::{msnp::notification::{command::not::factories::NotificationFactory, models::endpoint_data::{ClientType, EndpointData, PrivateEndpointData}}, shared::models::{msn_user::MsnUser, presence_status::PresenceStatus}};
    use crate::shared::models::email_address::EmailAddress;


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
        let command = "<EndpointData id=\"{00000000-0000-0000-0000-000000000000}\"><Capabilities>2789003324:48</Capabilities></EndpointData>";

        //Act
        let parsed = EndpointData::from_str(command).unwrap();

        //Assert
        assert_eq!(parsed.machine_guid.map(|e| e.0.to_string()), Some("00000000-0000-0000-0000-000000000000".to_string()));
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
}