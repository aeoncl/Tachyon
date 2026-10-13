use crate::msnp::error::{CommandError, PayloadError};
use crate::msnp::notification::models::endpoint_data::{EndpointData, PrivateEndpointData};
use crate::msnp::raw_command_parser::RawCommand;
use crate::shared::models::network_id_email::NetworkIdEmail;
use crate::shared::traits::{IntoBytes, TryFromBytes, TryFromRawCommand};
use std::fmt::Display;
use std::str::{from_utf8, FromStr};
use yaserde::ser::to_string_with_config;
use yaserde_derive::{YaDeserialize, YaSerialize};


#[cfg(test)]
mod tests {
    use crate::msnp::notification::command::ubx::{ExtendedPresenceContent, UbxPayload, UbxServer};
    use crate::msnp::notification::models::endpoint_data::EndpointData;
    use crate::msnp::notification::models::endpoint_guid::EndpointGuid;
    use crate::shared::models::capabilities::ClientCapabilities;
    use crate::shared::models::email_address::EmailAddress;
    use crate::shared::models::network_id::NetworkId;
    use crate::shared::models::network_id_email::NetworkIdEmail;
    use crate::shared::models::uuid::Uuid;
    use crate::shared::traits::{IntoBytes, TryFromRawCommand};
    use std::str::FromStr;

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

    #[test]
    pub fn deser_ubx_extended_presence() {
        let raw = r#"<Data><PSM>Hello</PSM><CurrentMedia></CurrentMedia><EndpointData id="{00000000-0000-0000-0000-000000000000}"><Capabilities>0:0</Capabilities></EndpointData></Data>"#;

        let parsed = yaserde::de::from_str::<ExtendedPresenceContent>(raw).expect("to deserialize");

        assert_eq!(parsed.psm, "Hello");
        assert_eq!(parsed.current_media, "");
        assert_eq!(parsed.endpoint_data.machine_guid.as_ref().map(|g| g.0.to_string()), Some("00000000-0000-0000-0000-000000000000".to_string()));
        assert_eq!(parsed.endpoint_data.capabilities.to_string(), "0:0");
        assert!(parsed.private_endpoint_data.is_none());
    }

    #[test]
    fn server_round_trips_extended_presence() {
        let ubx = UbxServer {
            target_user: NetworkIdEmail::new(NetworkId::WindowsLive, EmailAddress::from_str("aeon@lukewarmmail.com").unwrap()),
            via: None,
            payload: UbxPayload::ExtendedPresence(ExtendedPresenceContent {
                psm: "Hello".to_string(),
                current_media: "".to_string(),
                endpoint_data: EndpointData {
                    machine_guid: Some(EndpointGuid(Uuid::nil())),
                    capabilities: ClientCapabilities::new(0, 0),
                },
                private_endpoint_data: None,
            }),
        };
        let bytes = ubx.into_bytes();

        let mut parser = crate::msnp::raw_command_parser::ServerCommandParser::new();
        let raw = parser.parse_message(&bytes).unwrap().pop().unwrap();
        let parsed = UbxServer::try_from_raw(raw).unwrap();

        assert_eq!(parsed.target_user.email.as_str(), "aeon@lukewarmmail.com");
        assert!(parsed.via.is_none());
        let UbxPayload::ExtendedPresence(content) = parsed.payload else {
            panic!("expected an extended presence payload");
        };
        assert_eq!(content.psm, "Hello");
    }

    #[test]
    fn server_deserializes_unknown_payloads_as_raw() {
        let mut parser = crate::msnp::raw_command_parser::ServerCommandParser::new();
        let raw = parser
            .parse_message(b"UBX 1:aeon@lukewarmmail.com 13\r\n<Data></Data>")
            .unwrap()
            .pop()
            .unwrap();

        let parsed = UbxServer::try_from_raw(raw).unwrap();

        assert_eq!(parsed.target_user.email.as_str(), "aeon@lukewarmmail.com");
        match parsed.payload {
            UbxPayload::Raw(payload) => assert_eq!(payload, "<Data></Data>"),
            _ => panic!("expected the raw payload fallback"),
        }
    }

    #[test]
    fn server_deserializes_the_via_route() {
        let mut parser = crate::msnp::raw_command_parser::ServerCommandParser::new();
        let raw = parser
            .parse_message(b"UBX 1:me@lukewarmmail.com;via=1:peer@lukewarmmail.com 0\r\n")
            .unwrap()
            .pop()
            .unwrap();

        let parsed = UbxServer::try_from_raw(raw).unwrap();

        assert_eq!(parsed.target_user.email.as_str(), "me@lukewarmmail.com");
        let via = parsed.via.expect("via route");
        assert_eq!(via.email.as_str(), "peer@lukewarmmail.com");
    }
}



pub struct UbxServer {
    pub target_user: NetworkIdEmail,
    pub via: Option<NetworkIdEmail>,
    pub payload: UbxPayload
}

impl TryFromRawCommand for UbxServer {
    type Err = CommandError;

    fn try_from_raw(raw: RawCommand) -> Result<Self, Self::Err> where Self: Sized {
        let mut split = raw.command_split;
        let _operand = split.pop_front();

        let raw_target_user = split.pop_front().ok_or(CommandError::MissingArgument(raw.command.clone(), "target_user".into(), 1))?;
        let (target_user, via) = match raw_target_user.split_once(";via=") {
            Some((target, via)) => (
                NetworkIdEmail::from_str(target)?,
                Some(NetworkIdEmail::from_str(via)?),
            ),
            None => (NetworkIdEmail::from_str(&raw_target_user)?, None),
        };

        let payload = UbxPayload::try_from_bytes(raw.payload)?;

        Ok(Self { target_user, via, payload })
    }

}

impl IntoBytes for UbxServer {

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
    ExtendedPresence(ExtendedPresenceContent),
    Raw(String),
}

impl TryFromBytes for UbxPayload {
    type Err = PayloadError;

    fn try_from_bytes(bytes: Vec<u8>) -> Result<Self, Self::Err> where Self: Sized {
        let payload = from_utf8(&bytes)?;

        match yaserde::de::from_str::<ExtendedPresenceContent>(payload) {
            Ok(content) if content.to_string() == payload => Ok(Self::ExtendedPresence(content)),
            _ => Ok(Self::Raw(payload.to_string())),
        }
    }

}

impl IntoBytes for UbxPayload {
    fn into_bytes(self) -> Vec<u8> {
        match self {
            UbxPayload::ExtendedPresence(content) => {
                content.to_string().into_bytes()
            }
            UbxPayload::Raw(payload) => payload.into_bytes(),
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
