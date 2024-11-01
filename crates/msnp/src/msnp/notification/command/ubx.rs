use std::fmt::Display;
use yaserde::ser::to_string_with_config;
use yaserde_derive::{YaDeserialize, YaSerialize};
use crate::msnp::error::{CommandError, PayloadError};
use crate::msnp::notification::command::uum::{UumClient, UumPayload};
use crate::msnp::notification::command::uun::UunPayload;
use crate::msnp::notification::models::endpoint_data::{ClientType, EndpointData, PrivateEndpointData};
use crate::msnp::raw_command_parser::RawCommand;
use crate::shared::models::email_address::EmailAddress;
use crate::shared::models::endpoint_id::EndpointId;
use crate::shared::models::network_id::NetworkId;
use crate::shared::models::network_id_email::NetworkIdEmail;
use crate::shared::models::presence_status::PresenceStatus;
use crate::shared::traits::{MSNPCommand, MSNPPayload};


#[cfg(test)]
mod tests {
    use std::str::FromStr;
    use crate::msnp::notification::command::ubx::{ExtendedPresenceContent, UbxPayload, UbxServer};
    use crate::msnp::notification::models::endpoint_data::EndpointData;
    use crate::msnp::notification::models::endpoint_guid::EndpointGuid;
    use crate::shared::models::capabilities::ClientCapabilities;
    use crate::shared::models::email_address::EmailAddress;
    use crate::shared::models::network_id::NetworkId;
    use crate::shared::models::network_id_email::NetworkIdEmail;
    use crate::shared::models::uuid::Uuid;
    use crate::shared::traits::MSNPCommand;

    #[test]
    pub fn ubx_extended_presence_ser_test() {
        let ubx = UbxServer {
            target_user: NetworkIdEmail::new(NetworkId::WindowsLive, EmailAddress::from_str("aeon@lukewarmmail.com").unwrap()),
            via: None,
            payload: UbxPayload::ExtendedPresence(ExtendedPresenceContent {
                psm: "Hello".to_string(),
                current_media: "".to_string(),
                endpoint_data: EndpointData {
                    machine_guid: Some(EndpointGuid(Uuid::nil())),
                    capabilities: ClientCapabilities::new(0,0),
                },
                private_endpoint_data: None,
            }),
        };

        let bytes = ubx.into_bytes();

        let deser = String::from_utf8(bytes).unwrap();

        assert_eq!("UBX 1:aeon@lukewarmmail.com 163\r\n<Data><PSM>Hello</PSM><CurrentMedia></CurrentMedia><EndpointData id=\"{00000000-0000-0000-0000-000000000000}\"><Capabilities>0:0</Capabilities></EndpointData></Data>", deser);
    }
}



pub struct UbxServer {
    pub target_user: NetworkIdEmail,
    pub via: Option<NetworkIdEmail>,
    pub payload: UbxPayload
}

impl MSNPCommand for UbxServer {
    type Err = CommandError;

    fn try_from_raw(raw: RawCommand) -> Result<Self, Self::Err> where Self: Sized {
        todo!()
    }

    fn into_bytes(self) -> Vec<u8> {
        let mut payload = self.payload.into_bytes();

        let target_user = match self.via {
            None => {
                self.target_user.to_string()
            }
            Some(via) => {
                format!("{};via={}", self.target_user.to_string(), via.to_string())
            }
        };


        let mut cmd = format!("UBX {target_user} {payload_size}\r\n", target_user = target_user, payload_size = payload.len()).into_bytes();

        cmd.append(&mut payload);

        cmd

    }
}


pub enum UbxPayload {
    ExtendedPresence(ExtendedPresenceContent)
}

impl MSNPPayload for UbxPayload {
    type Err = PayloadError;

    fn try_from_bytes(bytes: Vec<u8>) -> Result<Self, Self::Err> where Self: Sized {
        todo!()
    }

    fn into_bytes(self) -> Vec<u8> {
        match self {
            UbxPayload::ExtendedPresence(content) => {
                content.to_string().into_bytes()
            }
        }
    }

}

#[derive(Debug, Clone, Default, YaSerialize, YaDeserialize)]
#[yaserde(rename="Data")]
pub struct ExtendedPresenceContent {
    #[yaserde(rename = "PSM")]
    pub psm: String,
    #[yaserde(rename = "CurrentMedia")]
    pub current_media: String,
    #[yaserde(rename = "EndpointData")]
    pub endpoint_data: EndpointData,
    #[yaserde(rename = "PrivateEndpointData")]
    pub private_endpoint_data: Option<PrivateEndpointData>
}

impl Display for ExtendedPresenceContent {
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
