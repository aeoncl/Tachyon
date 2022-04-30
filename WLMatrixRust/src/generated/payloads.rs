use std::{io::Read, convert::Infallible, fmt::{Display, Error}, str::FromStr};

use matrix_sdk::ruma::presence::PresenceState;
use substring::Substring;
use yaserde::{de::{self, from_str}, ser::to_string};
use yaserde_derive::{YaSerialize, YaDeserialize};
use std::fmt::Write;
use crate::models::errors::Errors;


#[derive(Default, YaSerialize, YaDeserialize)]

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

impl FromStr for PrivateEndpointData {
    type Err = Errors;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
      if let Ok(deserialized) = from_str::<PrivateEndpointData>(s) {
        return Ok(deserialized);
      } else {
          return Err(Errors::PayloadDeserializeError);
      }
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
        use std::str::FromStr;

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




#[derive(Debug, Clone, YaSerialize, YaDeserialize)]
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
        return PresenceStatus::NLN;
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
            _ => {
                PresenceStatus::FLN
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




#[cfg(test)]
mod tests {
    use std::str::FromStr;

    use yaserde::{de::from_str, ser::to_string};

    use crate::generated::payloads::{PrivateEndpointData, ClientType, PresenceStatus};


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
    fn test_serialize_private_endpoint_data() {
           //Arrange
           let command = PrivateEndpointData{ machine_guid: None, ep_name: String::from("M1CROW8Vl"), idle: true, client_type: ClientType::Website, state: PresenceStatus::AWY };

           //Act
           let parsed = command.to_string();
   
           //Assert
           assert_eq!(parsed, String::from("<PrivateEndpointData><EpName>M1CROW8Vl</EpName><Idle>true</Idle><ClientType>2</ClientType><State>AWY</State></PrivateEndpointData>"));
   
       }
}