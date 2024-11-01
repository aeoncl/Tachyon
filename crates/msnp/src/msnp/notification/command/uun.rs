use std::{fmt::Display, str::{from_utf8, FromStr}};
use std::process::Command;

use anyhow::anyhow;
use num_derive::FromPrimitive;
use yaserde::{de::from_str, ser::to_string_with_config};
use yaserde_derive::{YaDeserialize, YaSerialize};

use crate::{msnp::{error::{CommandError, PayloadError}, notification::models::endpoint_guid::EndpointGuid, raw_command_parser::RawCommand}, shared::{command::ok::OkCommand, payload}};
use crate::shared::models::endpoint_id::EndpointId;
use crate::shared::traits::{MSNPCommand, MSNPCommandPart, MSNPPayload};

pub struct UunClient {
    tr_id: u128,
    destination: EndpointId,
    payload: UunPayload
}

impl UunClient {
    pub fn get_ok_response(&self) -> OkCommand {
        OkCommand {tr_id: self.tr_id, operand: "UUN".to_string()}
    }
}

impl MSNPCommand for UunClient {
    type Err = CommandError;

    fn try_from_raw(raw: RawCommand) -> Result<Self, Self::Err> {
        let mut split = raw.command_split;
        let _operand = split.pop_front();

        let raw_tr_id = split.pop_front().ok_or(CommandError::MissingArgument(raw.command.clone(), "tr_id".into(), 1))?;
        let tr_id = u128::from_str(&raw_tr_id)?;

        let raw_destination = split.pop_front().ok_or(CommandError::MissingArgument(raw.command.clone(), "destination".into(), 2))?;
        let destination = EndpointId::from_str(&raw_destination)?;

        let raw_notification_type = split.pop_front().ok_or(CommandError::MissingArgument(raw.command.clone(), "notification_type".into(), 3))?;
        let notification_type: UserNotificationType = num::FromPrimitive::from_u32(u32::from_str(&raw_notification_type)?)
                                                        .ok_or(CommandError::ArgumentParseError { argument: raw_notification_type.to_string(), command: raw.command, source: anyhow!("Couldn't parse int to UserNotificationType") })?;

        let payload = UunPayload::parse_uun_payload(notification_type, raw.payload)?;

        Ok(Self { tr_id, destination, payload })

    }

    fn into_bytes(self) -> Vec<u8> {
        todo!()
    }
}

pub enum UunPayload {
    DisconnectClient,
    DisconnectAllClients,
    ConversationWindowClosed { email_addr: String },
    DismissUserInvite{email_addr: String, unknown: u32},
    Resynchronize(UunSoapStatePayload),
    Unknown(Vec<u8>)
}

impl MSNPPayload for UunPayload {
    type Err = PayloadError;

    fn try_from_bytes(bytes: Vec<u8>) -> Result<Self, Self::Err> {
        todo!()
    }

    fn into_bytes(self) -> Vec<u8> {
        match self {
            UunPayload::DisconnectClient => b"goawyplzthxbye".to_vec(),
            UunPayload::DisconnectAllClients => b"gtfo".to_vec(),
            UunPayload::ConversationWindowClosed { email_addr } => todo!(),
            UunPayload::DismissUserInvite { email_addr, unknown } => format!("{} {}", email_addr, unknown).as_bytes().to_vec(),
            UunPayload::Resynchronize(payload) => payload.to_string().as_bytes().to_vec(),
            UunPayload::Unknown(payload) => payload.to_owned(),
        }
    }
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

impl From<&UunPayload> for UserNotificationType {
    fn from(value: &UunPayload) -> Self {
        match value {
            UunPayload::DisconnectClient => UserNotificationType::DisconnectClient,
            UunPayload::DisconnectAllClients => UserNotificationType::DisconnectAllClients,
            UunPayload::ConversationWindowClosed { email_addr } => UserNotificationType::ClosedConversation,
            UunPayload::DismissUserInvite { email_addr, unknown } => UserNotificationType::DismissUserInvite,
            UunPayload::Resynchronize(_) => UserNotificationType::Resynchronize,
            UunPayload::Unknown(_) => todo!(),
        }
    }
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


pub type UbnPayload = UunPayload;
pub struct UbnServer {
    destination: EndpointId,
    payload: UbnPayload
}

impl MSNPCommand for UbnServer {
    type Err = CommandError;

    fn try_from_raw(raw: RawCommand) -> Result<Self, Self::Err> {
        todo!()
    }

    fn into_bytes(self) -> Vec<u8> {
        let payload_type  = UserNotificationType::from(&self.payload);

        let mut payload = self.payload.into_bytes();
        let mut command = format!("UBN {dest} {payload_type} {payload_size}\r\n", dest = self.destination, payload_type = payload_type as u32, payload_size = payload.len()).into_bytes();

        command.append(&mut payload);
        command
    }
}