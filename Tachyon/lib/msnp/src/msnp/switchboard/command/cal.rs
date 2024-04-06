use crate::msnp::error::CommandError;
use crate::msnp::raw_command_parser::RawCommand;
use crate::shared::command::command::{get_split_part, parse_tr_id};
use crate::shared::traits::SerializeMsnp;

// Invite someone to join the SB
// >>> CAL 58 aeontest@shl.local
// <<< CAL 58 RINGING 4324234

pub struct CalClient {
    tr_id: u128,
    email_addr: String
}

impl TryFrom<RawCommand> for CalClient {
    type Error = CommandError;

    fn try_from(command: RawCommand) -> Result<Self, Self::Error> {
        let split = command.get_command_split();
        let tr_id = parse_tr_id(&split)?;
        let email_addr = get_split_part(2, &split, command.get_command(), "email_addr")?;

        Ok(CalClient{ tr_id, email_addr: email_addr.to_string() })
    }
}


pub struct CalServer {
    tr_id: u128,
    session_id: u64
}

impl SerializeMsnp for CalServer {
    fn serialize_msnp(&self) -> Vec<u8> {
        format!("CAL {} {}\r\n", self.tr_id, self.session_id).into_bytes()
    }
}