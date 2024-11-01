use std::str::FromStr;
use crate::msnp::error::CommandError;
use crate::msnp::raw_command_parser::RawCommand;
use crate::shared::command::ok::OkCommand;
use crate::shared::models::b64_string::Base64String;
use crate::shared::models::endpoint_id::EndpointId;
use crate::shared::traits::MSNPCommand;

// Answers an XFR command from the Notification Sever, joining a Switchboard
// >>> ANS 3 aeontest@shl.local;{F52973B6-C926-4BAD-9BA8-7C1E840E4AB0} base64token 4060759068338340280
// <<< ANS 3 OK
pub struct AnsClient {
    tr_id: u128,
    endpoint_id: EndpointId,
    token: Base64String,
    session_id: u64
}

impl MSNPCommand for AnsClient {
    type Err = CommandError;

    fn try_from_raw(raw: RawCommand) -> Result<Self, Self::Err> {
        let mut split = raw.command_split;
        let _operand = split.pop_front();

        let raw_tr_id = split.pop_front().ok_or(CommandError::MissingArgument(raw.command.clone(), "tr_id".into(), 1))?;
        let tr_id = u128::from_str(&raw_tr_id)?;

        let raw_endpoint_id = split.pop_front().ok_or(CommandError::MissingArgument(raw.command.clone(), "endpoint_id".into(), 1))?;
        let endpoint_id = EndpointId::from_str(&raw_endpoint_id)?;

        let raw_token = split.pop_front().ok_or(CommandError::MissingArgument(raw.command.clone(), "token".into(), 1))?;
        let token = Base64String::from_str(&raw_token)?;

        let raw_session_id = split.pop_front().ok_or(CommandError::MissingArgument(raw.command.clone(), "session_id".into(), 1))?;
        let session_id = u64::from_str(&raw_session_id)?;

        Ok(AnsClient{
            tr_id,
            endpoint_id,
            token,
            session_id
        })
    }

    fn into_bytes(self) -> Vec<u8> {
        todo!()
    }
}

impl AnsClient {

    pub fn get_ok_response(&self) -> OkCommand {
        OkCommand{ operand: "ANS".to_string(), tr_id: self.tr_id}
    }

}
