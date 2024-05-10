use std::fmt::Display;
use std::str::FromStr;

use anyhow::anyhow;
use byteorder::ByteOrder;
use num_derive::FromPrimitive;
use num_traits::FromPrimitive;

use crate::msnp::error::{CommandError, PayloadError};
use crate::msnp::raw_command_parser::RawCommand;
use crate::shared::command::ok::OkCommand;
use crate::shared::models::endpoint_id::EndpointId;
use crate::shared::payload::msg::datacast_msg::{DatacastMessageContent, DatacastType};
use crate::shared::payload::msg::raw_msg_payload::RawMsgPayload;
use crate::shared::payload::msg::text_msg::TextMessageContent;
use crate::shared::payload::msg::typing_user_msg::TypingUserMessageContent;
use crate::shared::traits::{IntoBytes, MSGPayload, MSNPCommand, MSNPPayload};

#[cfg(test)]
mod tests {
    use std::str::FromStr;

    use crate::msnp::raw_command_parser::RawCommandParser;
    use crate::shared::models::email_address::EmailAddress;
    use crate::shared::models::endpoint_id::EndpointId;
    use crate::shared::payload::msg::text_msg::{FontStyle, TextMessageContent};
    use crate::shared::traits::MSNPCommand;

    use super::{NetworkId, UumClient, UumPayload};

    #[test]
    fn uum_client_text_message_deser() {
        let mut command_parser = RawCommandParser::new();
        let raw = "UUM 12 bob@yahoo.com 32 1 144\r\nMIME-Version: 1.0\r\nContent-Type: text/plain; charset=utf-8\r\nX-MMS-IM-Format: FN=Microsoft%20Sans%20Serif; EF=B; CO=0; CS=0; PF=22\r\n\r\nHello Bob !";
        let raw_command = command_parser.parse_message(raw.as_bytes()).unwrap().pop().unwrap();
        let uum_client = UumClient::try_from_raw(raw_command).unwrap();

        assert_eq!(12, uum_client.tr_id);
        assert_eq!("bob@yahoo.com", uum_client.destination.email_addr.as_str());
        assert!(uum_client.destination.endpoint_guid.is_none());
        assert_eq!(NetworkId::Yahoo, uum_client.network_id);

        assert!(matches!( uum_client.payload, UumPayload::TextMessage(_)));

        if let UumPayload::TextMessage(content) = uum_client.payload {
            //  assert_eq!(0, content.font_color);
            assert_eq!("Microsoft Sans Serif", &content.font_family);
            assert_eq!("Hello Bob !", &content.body);
            assert_eq!(false, content.right_to_left);
            assert!(content.font_styles.matches(FontStyle::Bold));
            assert!(!content.font_styles.matches(FontStyle::Italic));
        }
    }

    #[test]
    pub fn uum_client_text_message_deser_with_color() {
        let mut command_parser = RawCommandParser::new();
        let raw = "UUM 27 aeoncl@shlasouf.local 1 1 160\r\nMIME-Version: 1.0\r\nContent-Type: text/plain; charset=UTF-8\r\nX-MMS-IM-Format: FN=Segoe%20UI%20Semibold; EF=IU; CO=ff00ff; CS=0; PF=22\r\nDest-Agent: client\r\n\r\nTEST";
        let raw_command = command_parser.parse_message(raw.as_bytes()).unwrap().pop().unwrap();
        let uum_client = UumClient::try_from_raw(raw_command).unwrap();

        assert_eq!(27, uum_client.tr_id);
        assert_eq!("aeoncl@shlasouf.local", uum_client.destination.email_addr.as_str());
        assert!(uum_client.destination.endpoint_guid.is_none());
        assert_eq!(NetworkId::WindowsLive, uum_client.network_id);

        if let UumPayload::TextMessage(content) = uum_client.payload {
            // assert_eq!(0, content.font_color);
            assert_eq!("Segoe UI Semibold", &content.font_family);
            assert_eq!("TEST", &content.body);
            assert_eq!(false, content.right_to_left);
            assert!(content.font_styles.matches(FontStyle::Italic));
            assert!(content.font_styles.matches(FontStyle::Underline));
        }
    }

    #[test]
    pub fn uum_client_text_message_ser() {
        let uum = UumClient {
            tr_id: 1,
            destination: EndpointId::new(EmailAddress::from_str("aeon@lukewarmmail.com").unwrap(), None),
            network_id: NetworkId::WindowsLive,
            payload: UumPayload::TextMessage(TextMessageContent::new_with_default_style("Hello")),
        };

        let bytes = uum.into_bytes();
        let mut command_parser = RawCommandParser::new();
        let raw_command = command_parser.parse_message(&bytes).unwrap().pop().unwrap();

        let uum_deser = UumClient::try_from_raw(raw_command).unwrap();
        assert_eq!(NetworkId::WindowsLive, uum_deser.network_id);
        assert_eq!("aeon@lukewarmmail.com", uum_deser.destination.email_addr.as_str());
        assert!(matches!( uum_deser.payload, UumPayload::TextMessage(_)));

        if let UumPayload::TextMessage(content) = uum_deser.payload {
            assert_eq!("Hello", &content.body);
            assert!(content.is_styling_default());
        }
    }
}


pub struct UumClient {
    pub tr_id: u128,
    pub destination: EndpointId,
    pub network_id: NetworkId,
    pub payload: UumPayload
}
#[derive(FromPrimitive, Eq, PartialEq, Debug)]
pub enum NetworkId {
    WindowsLive = 0x01,
    OfficeCommunicator = 0x02,
    Telephone = 0x04,
    //used by Vodafone
    MobileNetworkInterop = 0x08,
    //Jaguire, Japanese mobile interop
    Smtp = 0x10,
    Yahoo = 0x20
}

#[derive(FromPrimitive, Clone, Eq, PartialEq, Debug)]
pub enum MessageType {
    TextMessage = 1,
    TypingUser = 2,
    Nudge = 3,
    UnknownYet = 4
}

pub enum UumPayload {
    TextMessage(TextMessageContent),
    TypingUser(TypingUserMessageContent),
    Nudge(DatacastMessageContent),
    Raw(RawMsgPayload),
}

impl UumPayload {

    fn parse_uum_payload(payload_type: MessageType, payload: Vec<u8>) -> Result<Self, PayloadError> {
        match payload_type {
            MessageType::TextMessage => {
                let raw = RawMsgPayload::try_from_bytes(payload)?;
                let content = TextMessageContent::try_from_raw(raw)?;
                Ok(UumPayload::TextMessage(content))
            },
            MessageType::TypingUser => {
                let raw = RawMsgPayload::try_from_bytes(payload)?;
                let content = TypingUserMessageContent::try_from_raw(raw)?;
                Ok(UumPayload::TypingUser(content))
            },
            MessageType::Nudge => {
                let raw = RawMsgPayload::try_from_bytes(payload)?;
                let content = DatacastMessageContent::try_from_raw(raw)?;
                if content.get_type() != DatacastType::Nudge {
                    return Err(PayloadError::AnyError(anyhow!("Wrong datacast type for UUM Nudge message: expected 1, got {}", content.get_type() as u32)))
                }
                Ok(UumPayload::Nudge(content))
            },
            MessageType::UnknownYet => {
                let raw = RawMsgPayload::try_from_bytes(payload)?;
                Ok(UumPayload::Raw(raw))
            },
        }
    }
}

impl IntoBytes for UumPayload {
    fn into_bytes(self) -> Vec<u8> {
        match self {
            UumPayload::TextMessage(content) => {
                content.into_bytes()
            }
            UumPayload::TypingUser(content) => {
                content.into_bytes()
            }
            UumPayload::Nudge(content) => {
                content.into_bytes()
            }
            UumPayload::Raw(content) => {
                content.into_bytes()
            }
        }
    }
}

impl From<&UumPayload> for MessageType {
    fn from(value: &UumPayload) -> Self {
        match value {
            UumPayload::TextMessage(_) => {
                MessageType::TextMessage
            }
            UumPayload::TypingUser(_) => {
                MessageType::TypingUser
            }
            UumPayload::Nudge(_) => {
                MessageType::Nudge
            }
            UumPayload::Raw(_) => {
                MessageType::UnknownYet
            }
        }
    }
}

impl MSNPCommand for UumClient {
    type Err = CommandError;

    fn try_from_raw(raw: RawCommand) -> Result<Self, Self::Err> {
        let mut split = raw.command_split;
        let _operand = split.pop_front();

        let raw_tr_id = split.pop_front().ok_or(CommandError::MissingArgument(raw.command.clone(), "tr_id".into(), 1))?;
        let tr_id = u128::from_str(&raw_tr_id)?;

        let raw_destination = split.pop_front().ok_or(CommandError::MissingArgument(raw.command.clone(), "destination".into(), 2))?;
        let destination = EndpointId::from_str(&raw_destination)?;

        let raw_network_id = split.pop_front().ok_or(CommandError::MissingArgument(raw.command.clone(), "network-id".into(), 3))?;
        let network_id = NetworkId::from_u32(u32::from_str(&raw_network_id)?).ok_or(CommandError::ArgumentParseError {
            argument: "network-id".into(),
            command: raw.command.clone(),
            source: anyhow!("Unknown network-id: {}", raw_network_id),
        })?;

        let raw_message_type = split.pop_front().ok_or(CommandError::MissingArgument(raw.command.clone(), "message-type".into(), 4))?;

        let message_type = MessageType::from_u32(u32::from_str(&raw_message_type)?).ok_or(CommandError::ArgumentParseError { argument: raw_message_type.to_string(), command: raw.command.clone(), source: anyhow!("Couldn't parse int to UserNotificationType") })?;

        let payload = UumPayload::parse_uum_payload(message_type, raw.payload)?;

        Ok(Self {
            tr_id,
            destination,
            network_id,
            payload,
        })
    }

    fn into_bytes(self) -> Vec<u8> {
        let message_type = MessageType::from(&self.payload);

        let mut payload = self.payload.into_bytes();
        let mut command = format!("UUM {tr_id} {dest} {network_id} {message_type} {payload_size}\r\n", tr_id = self.tr_id, dest = self.destination, network_id = self.network_id as u8, message_type = message_type as u8, payload_size = payload.len()).into_bytes();

        let mut out = Vec::with_capacity(command.len() + payload.len());
        out.append(&mut command);
        out.append(&mut payload);

        out
    }
}

impl UumClient {
    pub fn get_ok_response(&self) -> OkCommand {
        OkCommand {tr_id: self.tr_id, operand: "UUM".to_string()}
    }
}
