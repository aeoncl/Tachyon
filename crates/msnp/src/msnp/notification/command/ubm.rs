use crate::msnp::error::CommandError;
use crate::msnp::notification::command::uum::{MessageType, UumPayload};
use crate::msnp::raw_command_parser::RawCommand;
use crate::shared::models::email_address::EmailAddress;
use crate::shared::models::network_id::NetworkId;
use crate::shared::traits::{IntoBytes, TryFromRawCommand};
use anyhow::anyhow;
use num_traits::FromPrimitive;
use std::str::FromStr;

pub type UbmPayload = UumPayload;

/**

Message received from Yahoo contact:

UBM eagleearth_ap@yahoo.com 32
eagle-earth@live.com 1 1 125
MIME-Version: 1.0
Content-Type: text/plain; charset=UTF-8
X-MMS-IM-Format: FN=MS%20Shell%20Dlg; EF=; CO=0; CS=0; PF=0


aa


Typing message from Yahoo contact:

UBM eagleearth_ap@yahoo.com 32
eagle-earth@live.com 1 2 94
MIME-Version: 1.0
Content-Type: text/x-msmsgscontrol
TypingUser: eagleearth_ap@yahoo.com

*/

pub struct UbmServer {
    //Used to create the AdressableEntity (the conversation window)
    pub contact_sender: String,
    pub contact_network_id: NetworkId,
    //Used to create the sender of the message
    pub message_sender: EmailAddress,
    pub message_sender_network_id: NetworkId,
    pub payload: UbmPayload
}

impl UbmServer {
    pub fn new(sender: String, network_id: NetworkId, message_sender: EmailAddress, message_sender_network_id: NetworkId, payload: UumPayload) -> Self {
        Self {
            contact_sender: sender,
            contact_network_id: network_id,
            message_sender,
            message_sender_network_id,
            payload
        }
    }
}

impl IntoBytes for UbmServer {

    fn into_bytes(self) -> Vec<u8> {
        let message_type = MessageType::from(&self.payload);

        let mut payload = self.payload.into_bytes();
        let mut command = format!("UBM {contact_sender} {contact_network_id} {message_sender} {message_sender_network_id} {message_type} {payload_size}\r\n", contact_sender = self.contact_sender, contact_network_id = self.contact_network_id as u8, message_sender = self.message_sender, message_sender_network_id = self.message_sender_network_id as u8, message_type = message_type as u8, payload_size = payload.len()).into_bytes();

        let mut out = Vec::with_capacity(command.len() + payload.len());
        out.append(&mut command);
        out.append(&mut payload);

        out
    }
}

impl TryFromRawCommand for UbmServer {
    type Err = CommandError;

    fn try_from_raw(raw: RawCommand) -> Result<Self, Self::Err> {
        let mut split = raw.command_split;
        let _operand = split.pop_front();

        let contact_sender = split.pop_front().ok_or(CommandError::MissingArgument(raw.command.clone(), "sender".into(), 1))?;

        let raw_contact_network_id = split.pop_front().ok_or(CommandError::MissingArgument(raw.command.clone(), "network-id".into(), 2))?;
        let contact_network_id = NetworkId::from_u32(u32::from_str(&raw_contact_network_id)?).ok_or(CommandError::ArgumentParseError {
            argument: "contact-network-id".into(),
            command: raw.command.clone(),
            source: anyhow!("Unknown network-id: {}", raw_contact_network_id),
        })?;

        let raw_message_sender = split.pop_front().ok_or(CommandError::MissingArgument(raw.command.clone(), "email_addr".into(), 3))?;
        let message_sender = EmailAddress::from_str(&raw_message_sender)?;

        let raw_message_sender_network_id = split.pop_front().ok_or(CommandError::MissingArgument(raw.command.clone(), "unknown".into(), 4))?;
        let message_sender_network_id = NetworkId::from_u32(u32::from_str(&raw_message_sender_network_id)?).ok_or(CommandError::ArgumentParseError {
            argument: "message-sender-network-id".into(),
            command: raw.command.clone(),
            source: anyhow!("Unknown network-id: {}", raw_contact_network_id),
        })?;
        let raw_message_type = split.pop_front().ok_or(CommandError::MissingArgument(raw.command.clone(), "message-type".into(), 5))?;
        let message_type = MessageType::from_u32(u32::from_str(&raw_message_type)?).ok_or(CommandError::ArgumentParseError { argument: raw_message_type.to_string(), command: raw.command.clone(), source: anyhow!("Couldn't parse int to UserNotificationType") })?;

        let payload = UbmPayload::parse_payload(message_type, raw.payload)?;

        Ok(Self {
            contact_sender,
            contact_network_id,
            message_sender,
            message_sender_network_id,
            payload,
        })
    }

}

#[cfg(test)]
mod tests {
    use crate::msnp::raw_command_parser::RawCommandParser;
    use crate::shared::models::font_style::FontStyle;
    use crate::shared::models::network_id::NetworkId;
    use crate::shared::traits::TryFromRawCommand;

    use super::{UbmServer, UumPayload};

    #[test]
    fn ubm_client_text_message_deser() {
        let mut command_parser = RawCommandParser::new();

        let raw = "UBM eagleearth_ap@yahoo.com 32 eagle-earth@live.com 1 1 125\r\nMIME-Version: 1.0\r\nContent-Type: text/plain; charset=UTF-8\r\nX-MMS-IM-Format: FN=MS%20Shell%20Dlg; EF=; CO=0; CS=0; PF=0\r\n\r\naa\r\n";
        let raw_command = command_parser.parse_message(raw.as_bytes()).unwrap().pop().unwrap();
        let ubm_server = UbmServer::try_from_raw(raw_command).unwrap();

        assert_eq!("eagleearth_ap@yahoo.com", ubm_server.contact_sender.as_str());
        assert_eq!(NetworkId::Yahoo, ubm_server.contact_network_id);
        assert_eq!("eagle-earth@live.com", ubm_server.message_sender.as_str());
        assert_eq!(NetworkId::WindowsLive, ubm_server.message_sender_network_id);

        assert!(matches!( ubm_server.payload, UumPayload::TextPlain(_)));

        if let UumPayload::TextPlain(content) = ubm_server.payload {
            //  assert_eq!(0, content.font_color);
            assert_eq!("MS Shell Dlg", content.font_family.value());
            assert_eq!("aa", &content.body);
            assert_eq!(false, content.right_to_left);
        }
    }

}