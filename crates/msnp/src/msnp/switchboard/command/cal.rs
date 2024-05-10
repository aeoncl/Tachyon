use std::str::FromStr;
use crate::msnp::error::CommandError;
use crate::msnp::raw_command_parser::RawCommand;
use crate::shared::traits::MSNPCommand;

// Invite someone to join the SB
// >>> CAL 58 aeontest@shl.local
// <<< CAL 58 RINGING 4324234

pub struct CalClient {
    tr_id: u128,
    email_addr: String
}

impl MSNPCommand for CalClient {
    type Err = CommandError;

    fn try_from_raw(raw: RawCommand) -> Result<Self, Self::Err> {
        let mut split = raw.command_split;
        let _operand = split.pop_front();

        let raw_tr_id = split.pop_front().ok_or(CommandError::MissingArgument(raw.command.clone(), "tr_id".into(), 1))?;
        let tr_id = u128::from_str(&raw_tr_id)?;

        let email_addr = split.pop_front().ok_or(CommandError::MissingArgument(raw.command.clone(), "email".into(), 1))?;

        Ok(CalClient{ tr_id, email_addr })

    }

    fn into_bytes(self) -> Vec<u8> {
        todo!()
    }
}

pub struct CalServer {
    tr_id: u128,
    session_id: u64
}

impl MSNPCommand for CalServer {
    type Err = CommandError;

    fn try_from_raw(raw: RawCommand) -> Result<Self, Self::Err> {
        todo!()
    }

    fn into_bytes(self) -> Vec<u8> {
        format!("CAL {} {}\r\n", self.tr_id, self.session_id).into_bytes()
    }
}
