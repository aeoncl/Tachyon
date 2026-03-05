use crate::msnp::error::CommandError;
use crate::msnp::raw_command_parser::RawCommand;
use crate::shared::models::endpoint_id::EndpointId;
use crate::shared::traits::MSNPCommand;
use std::str::FromStr;
use crate::shared::models::email_address::EmailAddress;

// Initiate a new SB.
// >>> USR 55 aeontest@shl.local;{F52973B6-C926-4BAD-9BA8-7C1E840E4AB0} token
// <<< USR 55 aeontest@shl.local aeontest@shl.local OK
pub struct UsrClient {
    pub tr_id: u128,
    pub endpoint_id: EndpointId,
    pub token: String
}

impl UsrClient {

    pub fn get_ok_response_for(&self, display_name: String) -> UsrServer {
        UsrServer{
            tr_id: self.tr_id,
            email_addr: self.endpoint_id.email_addr.clone(),
            display_name,
        }
    }
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

    fn into_bytes(self) -> Vec<u8> {
        todo!()
    }
}


pub struct UsrServer {
    pub tr_id: u128,
    pub email_addr: EmailAddress,
    pub display_name: String,
}

impl MSNPCommand for UsrServer {
    type Err = CommandError;

    fn try_from_raw(_raw: RawCommand) -> Result<Self, Self::Err> {
        todo!()
    }

    fn into_bytes(self) -> Vec<u8> {
        format!("USR {} {} {} OK\r\n", self.tr_id, self.email_addr, self.display_name).into_bytes()
    }
}