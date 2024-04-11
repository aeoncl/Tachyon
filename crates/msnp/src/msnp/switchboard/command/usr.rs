use std::str::FromStr;
use crate::msnp::error::CommandError;
use crate::msnp::raw_command_parser::RawCommand;
use crate::shared::models::endpoint_id::EndpointId;
use crate::shared::traits::MSNPCommand;

// Initiate a new SB.
// >>> USR 55 aeontest@shl.local;{F52973B6-C926-4BAD-9BA8-7C1E840E4AB0} token
// <<< USR 55 aeontest@shl.local aeontest@shl.local OK
pub struct UsrClient {

    tr_id: u128,
    endpoint_id: EndpointId,
    token: String

}

impl MSNPCommand for UsrClient {
    type Err = CommandError;

    fn try_from_raw(raw: RawCommand) -> Result<Self, Self::Err> {
        let mut split = raw.command_split;
        let _operand = split.pop_front();

        let raw_tr_id = split.pop_front().ok_or(CommandError::MissingArgument(raw.command.clone(), "tr_id".into(), 1))?;
        let tr_id = u128::from_str(&raw_tr_id)?;

        let raw_endpoint_id = split.pop_front().ok_or(CommandError::MissingArgument(raw.command.clone(), "endpoint_id".into(), 1))?;
        let endpoint_id = EndpointId::from_str(&raw_endpoint_id)?;

        let token = split.pop_front().ok_or(CommandError::MissingArgument(raw.command.clone(), "token".into(), 1))?;

        Ok(UsrClient {
            tr_id,
            endpoint_id,
            token,
        })
    }

    fn to_bytes(self) -> Vec<u8> {
        todo!()
    }
}


pub struct UsrServerOk {
    tr_id: u128,
    email_addr: String,
    display_name: String,
}

impl MSNPCommand for UsrServerOk {
    type Err = CommandError;

    fn try_from_raw(raw: RawCommand) -> Result<Self, Self::Err> {
        todo!()
    }

    fn to_bytes(self) -> Vec<u8> {
        format!("USR {} {} {} OK\r\n", self.tr_id, self.email_addr, self.display_name).into_bytes()
    }
}