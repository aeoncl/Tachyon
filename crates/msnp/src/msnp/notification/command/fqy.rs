use std::str::FromStr;
use crate::msnp::error::{CommandError, PayloadError};
use crate::msnp::notification::command::adl::AdlClient;
use crate::msnp::notification::command::put::PutClient;
use crate::msnp::notification::models::adl_payload::ADLPayload;
use crate::msnp::raw_command_parser::RawCommand;
use crate::shared::command::ok::OkCommand;
use crate::shared::traits::{IntoBytes, TryFromBytes, TryFromRawCommand};

/**



What steps will reproduce the problem? 1. The email address is anyyahoouser@yahoo.com.TR 2. Send a FQY command whether this user a yahoo user:

<ml><d n="yahoo.com.TR"><c n="anyyahoouser" /></d></ml> 3. You will receive a response, see actual attribute: <ml><d n="yahoo.com.TR"><c n="anyyahoouser" t="32" actual="anyyahoouser@yahoo.com" />

4. "t" is type (optional), if this attribute exists, this is a real yahoo user. "actual" (optional) is real yahoo! messenger address to add.

4.1 Add actual address if the field exists, otherwise add <d> and <c> 4.2 Add also original address as other mail (but not messenger) as ContactEmailTypeType.ContactEmailOther.


*/
pub struct FqyClient {
    pub tr_id: u128,
    pub payload: ADLPayload
}

impl TryFromRawCommand for FqyClient {

    type Err = CommandError;

    fn try_from_raw(command: RawCommand) -> Result<Self, Self::Err> {
        let mut split = command.command_split;
        let _operand = split.pop_front();

        let raw_tr_id = split.pop_front().ok_or(CommandError::MissingArgument(command.command.clone(), "tr_id".into(), 1))?;

        let tr_id = u128::from_str(&raw_tr_id)?;

        let payload_size = command.expected_payload_size;

        if payload_size == 0 {
            Err(PayloadError::MissingPayload { command: command.command })?;
        }

        let payload = ADLPayload::try_from_bytes(command.payload)?;

        Ok(Self{
            tr_id,
            payload,
        })
    }
}

impl IntoBytes for FqyClient {

    fn into_bytes(self) -> Vec<u8> {
        let mut payload = self.payload.into_bytes();

        let mut out = format!("FQY {} {}\r\n", self.tr_id, payload.len()).into_bytes();

        out.append(&mut payload);

        out
    }
}

pub type FqyServer = FqyClient;