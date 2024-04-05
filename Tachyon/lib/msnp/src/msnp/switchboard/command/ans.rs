use std::str::FromStr;
use crate::msnp::error::CommandError;
use crate::msnp::raw_command_parser::RawCommand;
use crate::msnp::switchboard::command::cal::CalClient;
use crate::msnp::switchboard::models::b64_string::Base64String;
use crate::shared::command::command::{get_split_part, parse_tr_id};
use crate::shared::command::ok::OkCommand;
use crate::shared::models::endpoint_id::EndpointId;

// Answers an XFR command from the Notification Sever, joining a Switchboard
// >>> ANS 3 aeontest@shl.local;{F52973B6-C926-4BAD-9BA8-7C1E840E4AB0} base64token 4060759068338340280
// <<< ANS 3 OK
pub struct AnsClient {
    tr_id: u128,
    endpoint_id: EndpointId,
    token: Base64String,
    session_id: u64
}

impl TryFrom<RawCommand> for AnsClient {
    type Error = CommandError;

    fn try_from(command: RawCommand) -> Result<Self, Self::Error> {
        let split = command.get_command_split();
        let tr_id = parse_tr_id(&split)?;
        let raw_endpoint_id = get_split_part(2, &split, command.get_command(), "endpoint_id")?;
        let endpoint_id = EndpointId::from_str(raw_endpoint_id)?;
        let token = Base64String::from_str(get_split_part(3, &split, command.get_command(), "token")?)?;
        let raw_session_id = get_split_part(4, &split, command.get_command(), "endpoint_id")?;
        let session_id =  u64::from_str(raw_session_id).map_err(|e| CommandError::ArgumentParseError {
            argument: "session_id".to_string(),
            command: command.get_command().to_string(),
            source: e.into(),
        })?;
        
        Ok(AnsClient{
            tr_id,
            endpoint_id,
            token,
            session_id
        })

    }
}


impl AnsClient {

    pub fn get_ok_response(&self) -> OkCommand {
        OkCommand{ operand: "ANS".to_string(), tr_id: self.tr_id}
    }

}
