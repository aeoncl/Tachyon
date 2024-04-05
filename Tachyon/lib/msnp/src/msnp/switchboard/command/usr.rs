use std::str::FromStr;
use crate::msnp::error::CommandError;
use crate::msnp::raw_command_parser::RawCommand;
use crate::msnp::switchboard::command::ans::AnsClient;
use crate::shared::command::command::{get_split_part, parse_tr_id};
use crate::shared::models::endpoint_id::EndpointId;
use crate::shared::traits::SerializeMsnp;

// Initiate a new SB.
// >>> USR 55 aeontest@shl.local;{F52973B6-C926-4BAD-9BA8-7C1E840E4AB0} token
// <<< USR 55 aeontest@shl.local aeontest@shl.local OK
pub struct UsrClient {

    tr_id: u128,
    endpoint_id: EndpointId,
    token: String

}

impl TryFrom<RawCommand> for UsrClient {
    type Error = CommandError;

    fn try_from(command: RawCommand) -> Result<Self, Self::Error> {
        let split = command.get_command_split();
        let tr_id = parse_tr_id(&split)?;
        let raw_endpoint_id = get_split_part(2, &split, command.get_command(), "endpoint_id")?;
        let endpoint_id = EndpointId::from_str(raw_endpoint_id)?;
        let token = get_split_part(3, &split, command.get_command(), "token")?.to_string();

        Ok(UsrClient {
            tr_id,
            endpoint_id,
            token,
        })

    }
}


pub struct UsrServerOk {
    tr_id: u128,
    email_addr: String,
    display_name: String,
}

impl SerializeMsnp for UsrServerOk {
    fn serialize_msnp(&self) -> Vec<u8> {
        format!("USR {} {} {} OK\r\n", self.tr_id, self.email_addr, self.display_name).into_bytes()
    }
}