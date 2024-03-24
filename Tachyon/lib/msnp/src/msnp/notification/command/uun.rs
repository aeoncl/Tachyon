use std::{fmt::Display, str::{from_utf8, FromStr}};

use yaserde::{de::from_str, ser::to_string_with_config};
use yaserde_derive::{YaDeserialize, YaSerialize};
use anyhow::anyhow;
use crate::{msnp::{error::{CommandError, PayloadError}, notification::models::endpoint_guid::EndpointGuid, raw_command_parser::RawCommand}, shared::{command::{command::{get_split_part, parse_tr_id}, ok::OkCommand}, payload}};

pub struct UunClient {
    tr_id: u128,
    destination: Endpoint,
    payload_size: usize,
    payload: UunPayload

}

impl UunClient {
    pub fn get_ok_response(&self) -> OkCommand {
        OkCommand {tr_id: self.tr_id, operand: "UUN".to_string()}
    }
}

impl TryFrom<RawCommand> for UunClient {
    type Error = CommandError;

    fn try_from(command: RawCommand) -> Result<Self, Self::Error> {
        let split = command.get_command_split();
        let tr_id: u128 = parse_tr_id(&split)?;
        let raw_destination = get_split_part(2, &split, command.get_command(), "destination")?;
        let destination = Endpoint::from_str(raw_destination)?;
        let raw_notification_type = get_split_part(3, &split, command.get_command(), "payload_type")?;
        let notification_type: UserNotificationType = num::FromPrimitive::from_u32(u32::from_str(raw_notification_type).map_err(|e| CommandError::ArgumentParseError { argument: raw_notification_type.to_string(), command: command.get_command().to_string(), source: e.into() })?).ok_or(CommandError::ArgumentParseError { argument: raw_notification_type.to_string(), command: command.get_command().to_string(), source: anyhow!("Couldn't parse int to UserNotificationType") })?;
        
        let payload_size = command.expected_payload_size;

        let payload = UunPayload::parse_uun_payload(notification_type, command.payload)?;

        Ok(Self { tr_id, destination, payload_size, payload })
    }
}

pub struct Endpoint {
    email_addr: String,
    endpoint_guid: Option<EndpointGuid>
}

impl FromStr for Endpoint {
    type Err = CommandError;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        let split: Vec<&str> = s.split(';').collect();
        let email_addr = split[0].to_string();

        let endpoint_guid = if split.len() >= 2 {
            Some(EndpointGuid::from_str(split[1])?)
        } else {
            None
        };

        Ok(Endpoint{email_addr, endpoint_guid})
    }
}


pub enum UunPayload {
    DisconnectClient,
    DisconnectAllClients,
    ConversationWindowClosed { email_addr: String },
    DismissUserInvite{msn_addr: String, unknown: u32},
    Resynchronize(UunSoapStatePayload),
    Unknown(Vec<u8>)
}

impl UunPayload {
    fn parse_uun_payload(payload_type: UserNotificationType, payload: Vec<u8>) -> Result<Self, PayloadError>{
        Ok(match payload_type {
            UserNotificationType::DisconnectClient => {
                Self::DisconnectClient
            },
            UserNotificationType::Resynchronize => {
                let payload_str = from_utf8(&payload)?;
                let payload = from_str::<UunSoapStatePayload>(payload_str).map_err(|e| PayloadError::StringPayloadParsingError { payload: payload_str.to_string(), source: anyhow!(e) })?;
                Self::Resynchronize(payload)
            },
            UserNotificationType::DisconnectAllClients => {
                Self::DisconnectAllClients
            }
            _ => {
                Self::Unknown(payload)
            }
        })
    }
}

#[derive(Default, YaSerialize, YaDeserialize)]
#[yaserde(rename = "State")]
pub struct UunSoapStatePayload {
    services: Vec<UunService>
}

#[derive(Default, YaSerialize, YaDeserialize)]
#[yaserde(rename = "Service")]
pub struct UunService {
    #[yaserde(rename = "type", attribute)]
    service_type: String,

    #[yaserde(rename = "reason", attribute)]
    reason: u32
}

use num_derive::FromPrimitive;

#[derive(Clone, Debug, FromPrimitive)]
pub enum UserNotificationType {
    XmlData = 1,
    SipInvite = 2,
    P2PData = 3,
    DisconnectClient = 4,
    ClosedConversation = 5,
    Resynchronize = 6,
    DismissUserInvite = 7,
    DisconnectAllClients = 8,
    RTCActivity = 11,
    TunneledSip = 12
}

//TODO
pub enum UunServiceType {
    AdressBook,
    Membership,
    Unknown(String)
}


impl Display for UunSoapStatePayload {

    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {

        let yaserde_cfg = yaserde::ser::Config{
            perform_indent: false,
            write_document_declaration: false,
            indent_string: None
        };

        if let Ok(serialized) = to_string_with_config(self, &yaserde_cfg) {
            write!(f, "{}", serialized)
        } else {
            Err(std::fmt::Error)
        }
     
    }
}