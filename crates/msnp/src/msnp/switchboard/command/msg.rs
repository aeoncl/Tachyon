//      0   1  2 3
// >>> MSG 231 U 91\r\npayload
// <<< ACK 231           on success
// <<< NAK 231          on failure
// The 2nd parameter is the type of ack the clients wants.
// N: ack only when the message was not received
// A + D: always send an ack
// U: never ack

use std::str::FromStr;

use strum_macros::{Display, EnumString};

use crate::msnp::error::{CommandError, PayloadError};
use crate::msnp::raw_command_parser::RawCommand;
use crate::shared::models::display_name::DisplayName;
use crate::shared::models::email_address::EmailAddress;
use crate::shared::payload::msg::chunked_msg_payload::ChunkedMsgPayload;
use crate::shared::payload::msg::datacast_msg::DatacastMessagePayload;
use crate::shared::payload::msg::msn_msgr_p2p_msg_payload::MsnMsgrP2PMessagePayload;
use crate::shared::payload::msg::raw_msg_payload::{MsgContentType, RawMsgPayload};
use crate::shared::payload::msg::text_plain_msg::TextPlainMessagePayload;
use crate::shared::traits::{IntoBytes, TryFromBytes, TryFromRawCommand, TryFromRawMsgPayload};

pub struct MsgClient {
    pub tr_id: u128,
    pub ack_type: MsgAcknowledgment,
    pub payload: MsgPayload
}

impl TryFromRawCommand for MsgClient {
    type Err = CommandError;

    fn try_from_raw(raw: RawCommand) -> Result<Self, Self::Err> {
        let mut split = raw.command_split;
        let _operand = split.pop_front();

        let raw_tr_id = split.pop_front().ok_or(CommandError::MissingArgument(raw.command.clone(), "tr_id".into(), 1))?;
        let tr_id = u128::from_str(&raw_tr_id)?;

        let raw_ack_type = split.pop_front().ok_or(CommandError::MissingArgument(raw.command.clone(), "ack_type".into(), 1))?;
        let ack_type = MsgAcknowledgment::from_str(&raw_ack_type)?;

        let payload = MsgPayload::try_from_bytes(raw.payload)?;

        Ok(MsgClient{
            tr_id,
            ack_type,
            payload,
        })
    }
}

#[derive(Display, EnumString)]
pub enum MsgAcknowledgment {
    #[strum(serialize = "U")]
    NoAck,
    #[strum(serialize = "N")]
    AckOnFailure,
    #[strum(serialize = "A")]
    AckA,
    #[strum(serialize = "D")]
    AckD
}

pub struct MsgServer {
    pub sender: EmailAddress,
    pub display_name: DisplayName,
    pub payload: MsgPayload
}

impl TryFromRawCommand for MsgServer {
    type Err = ();

    fn try_from_raw(_raw: RawCommand) -> Result<Self, Self::Err> {
        todo!()
    }
}

impl IntoBytes for MsgServer {
    fn into_bytes(self) -> Vec<u8> {
        let mut payload = self.payload.into_bytes();
        let mut cmd = format!("MSG {} {} {}\r\n", self.sender, self.display_name, payload.len()).into_bytes();
        cmd.append(&mut payload);

        cmd
    }
}

pub enum MsgPayload {
    Raw(RawMsgPayload),
    Chunked(ChunkedMsgPayload),
    TextPlain(TextPlainMessagePayload),
    Datacast(DatacastMessagePayload),
    Control,
    P2P(MsnMsgrP2PMessagePayload)
}

impl TryFromRawMsgPayload for MsgPayload {
    type Err = PayloadError;

    fn try_from_raw(raw_msg_payload: RawMsgPayload) -> Result<Self, Self::Err>
    where
        Self: Sized
    {
        if raw_msg_payload.is_chunked() {
            return Ok(MsgPayload::Chunked(ChunkedMsgPayload::try_from_raw(raw_msg_payload)?));
        }

        match raw_msg_payload.get_content_type()? {
            MsgContentType::TextPlain => {
                let text_plain_payload = TextPlainMessagePayload::try_from_raw(raw_msg_payload)?;
                Ok(MsgPayload::TextPlain(text_plain_payload))
            }
            MsgContentType::Profile => { Ok(MsgPayload::Raw(raw_msg_payload))}
            MsgContentType::InitialMailDataNotification => {Ok(MsgPayload::Raw(raw_msg_payload))}
            MsgContentType::SystemMessage => {Ok(MsgPayload::Raw(raw_msg_payload))}
            MsgContentType::ServiceMessage => {Ok(MsgPayload::Raw(raw_msg_payload))}
            MsgContentType::Control => {Ok(MsgPayload::Raw(raw_msg_payload))}
            MsgContentType::Datacast => {Ok(MsgPayload::Datacast(DatacastMessagePayload::try_from_raw(raw_msg_payload)?))}
            MsgContentType::P2P => {Ok(MsgPayload::Raw(raw_msg_payload))}
            MsgContentType::None => {Ok(MsgPayload::Raw(raw_msg_payload))}
        }
    }
}

impl IntoBytes for MsgPayload {

    fn into_bytes(self) -> Vec<u8> {
        match self {
            MsgPayload::Raw(payload) => { payload.into_bytes() }
            MsgPayload::TextPlain(payload) => { payload.into_bytes() }
            MsgPayload::Datacast(payload) => payload.into_bytes(),
            MsgPayload::Control => {todo!()}
            MsgPayload::P2P(payload) => {todo!()}
            MsgPayload::Chunked(payload) => payload.into_bytes(),
        }
    }
}

impl TryFromBytes for MsgPayload {
    type Err = PayloadError;

    fn try_from_bytes(bytes: Vec<u8>) -> Result<Self, Self::Err> {
        let raw_msg_payload = RawMsgPayload::try_from_bytes(bytes)?;
        Self::try_from_raw(raw_msg_payload)
    }

}
