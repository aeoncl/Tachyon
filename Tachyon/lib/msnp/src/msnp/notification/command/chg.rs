use std:: str::{from_utf8, FromStr};
use std::fmt::Display;
use crate::shared::traits::SerializeMsnp;
use crate::{msnp::{error::{CommandError, PayloadError}, raw_command_parser::{self, RawCommand}}, shared::{command::command::{get_split_part, parse_tr_id}, models::{capabilities::ClientCapabilities, msn_object::MsnObject, presence_status::PresenceStatus}}};


pub struct Chg {

    tr_id: u128,
    presence_status: PresenceStatus,
    client_capabilities : ClientCapabilities,
    avatar: Option<MsnObject>

}

pub type ChgClient = Chg;

pub type ChgServer = Chg;

impl TryFrom<RawCommand> for Chg {
    type Error = CommandError;

    fn try_from(command: RawCommand) -> Result<Self, Self::Error> {
        let split = command.get_command_split();
        let tr_id: u128 = parse_tr_id(&split)?;
        let raw_presence_status = get_split_part(2, &split, command.get_command(), "presence_status")?;
        let presence_status = PresenceStatus::from_str(raw_presence_status).map_err(|e| CommandError::ArgumentParseError { argument: raw_presence_status.to_string(), command: command.get_command().to_string(), source: e.into() })?;
        let raw_capabilities = get_split_part(3, &split, command.get_command(), "client_capabilities")?;
        let client_capabilities = ClientCapabilities::from_str(raw_capabilities)?;

        let raw_avatar_msn_obj = get_split_part(4, &split, command.get_command(), "avatar_msn_obj")?;

        let avatar = if raw_avatar_msn_obj != "0" { Some(MsnObject::from_str(from_utf8(command.get_payload()).map_err(PayloadError::Utf8Error)?)?) } else { None };
        
        Ok(Chg { tr_id, presence_status, client_capabilities , avatar })
    }
}

impl Display for Chg {

    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "CHG {tr_id} {presence_status} {client_capabilities} ", tr_id = self.tr_id, presence_status = self.presence_status, client_capabilities = self.client_capabilities)?;
        
        match self.avatar.as_ref() {
            Some(avatar) => {
                write!(f, "{}", avatar)?;
            },
            None => {
                write!(f, "0")?;
            }
        }
        write!(f, "\r\n")?;
        Ok(())
    }
}

impl SerializeMsnp for Chg {

    fn serialize_msnp(&self) -> Vec<u8> {
        self.to_string().into_bytes()
    }
}
