use std::str::FromStr;
use std::fmt::Display;

use crate::{msnp::{error::CommandError, raw_command_parser::RawCommand}, shared::models::{capabilities::ClientCapabilities, msn_object::MsnObject, presence_status::PresenceStatus}};
use crate::shared::traits::MSNPCommand;

pub struct Chg {

    tr_id: u128,
    presence_status: PresenceStatus,
    client_capabilities : ClientCapabilities,
    avatar: Option<MsnObject>

}

pub type ChgClient = Chg;

pub type ChgServer = Chg;

impl MSNPCommand for Chg {
    type Err = CommandError;

    fn try_from_raw(raw: RawCommand) -> Result<Self, Self::Err> {
        let mut split = raw.command_split;
        let _operand = split.pop_front();

        let raw_tr_id = split.pop_front().ok_or(CommandError::MissingArgument(raw.command.clone(), "tr_id".into(), 1))?;
        let tr_id = u128::from_str(&raw_tr_id)?;

        let raw_presence_status = split.pop_front().ok_or(CommandError::MissingArgument(raw.command.clone(), "presence_status".into(), 2))?;
        let presence_status = PresenceStatus::from_str(&raw_presence_status)?;

        let raw_capabilities = split.pop_front().ok_or(CommandError::MissingArgument(raw.command.clone(), "client_capabilities".into(), 3))?;
        let client_capabilities = ClientCapabilities::from_str(&raw_capabilities)?;

        let raw_avatar_msn_obj = split.pop_front().ok_or(CommandError::MissingArgument(raw.command.clone(), "avatar_msn_obj".into(), 4))?;
        let avatar_decoded = urlencoding::decode(&raw_avatar_msn_obj)?;

        let avatar = if raw_avatar_msn_obj != "0" { Some(MsnObject::from_str(&avatar_decoded)?) } else { None };

        Ok(Chg { tr_id, presence_status, client_capabilities , avatar })

    }

    fn to_bytes(self) -> Vec<u8> {
        self.to_string().into_bytes()
    }
}

impl Display for Chg {

    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "CHG {tr_id} {presence_status} {client_capabilities} ", tr_id = self.tr_id, presence_status = self.presence_status, client_capabilities = self.client_capabilities)?;
        
        match self.avatar.as_ref() {
            Some(avatar) => {
                let avatar_as_str = avatar.to_string();
                write!(f, "{}", &urlencoding::encode(&avatar_as_str))?;
            },
            None => {
                write!(f, "0")?;
            }
        }
        write!(f, "\r\n")?;
        Ok(())
    }
}