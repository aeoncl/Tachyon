use std::str::FromStr;
use strum_macros::{Display, EnumString};
use crate::msnp::error::CommandError;
use crate::msnp::raw_command_parser::RawCommand;
use crate::msnp::switchboard::models::session_id::SessionId;
use crate::shared::models::email_address::EmailAddress;
use crate::shared::traits::{IntoBytes, TryFromRawCommand};

// Invite someone to join the SB
// >>> CAL 58 aeontest@shl.local
// <<< CAL 58 RINGING 4324234

pub struct CalClient {
    pub tr_id: u128,
    pub email_addr: EmailAddress
}

impl TryFromRawCommand for CalClient {
    type Err = CommandError;

    fn try_from_raw(raw: RawCommand) -> Result<Self, Self::Err> {
        let mut split = raw.command_split;
        let _operand = split.pop_front();

        let raw_tr_id = split.pop_front().ok_or(CommandError::MissingArgument(raw.command.clone(), "tr_id".into(), 1))?;
        let tr_id = u128::from_str(&raw_tr_id)?;

        let raw_email_addr = split.pop_front().ok_or(CommandError::MissingArgument(raw.command.clone(), "email".into(), 1))?;
        let email_addr = EmailAddress::from_str(&raw_email_addr)?;
        Ok(CalClient{ tr_id, email_addr })

    }
}

#[derive(Display, EnumString)]
pub enum CalServerFunction {
    RINGING,
}

pub struct CalServer {
    pub tr_id: u128,
    pub function: CalServerFunction,
    pub session_id: SessionId
}

impl TryFromRawCommand for CalServer {
    type Err = CommandError;

    fn try_from_raw(_raw: RawCommand) -> Result<Self, Self::Err> {
        todo!()
    }

}

impl IntoBytes for CalServer {

    fn into_bytes(self) -> Vec<u8> {
        format!("CAL {} {} {}\r\n", self.tr_id, self.function, self.session_id).into_bytes()
    }
}
