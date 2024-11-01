use strum_macros::Display;
use crate::msnp::error::CommandError;
use crate::msnp::raw_command_parser::RawCommand;
use crate::msnp::switchboard::command::ack::AckServer;
use crate::msnp::switchboard::command::ans::AnsClient;
use crate::msnp::switchboard::command::cal::{CalClient, CalServer};
use crate::msnp::switchboard::command::iro::IroServer;
use crate::msnp::switchboard::command::joi::JoiServer;
use crate::msnp::switchboard::command::msg::{MsgClient, MsgServer};
use crate::msnp::switchboard::command::usr::{UsrClient, UsrServerOk};
use crate::shared::command::ok::OkCommand;
use crate::shared::traits::{MSNPCommand};

#[derive(Display)]
pub enum SwitchboardClientCommand {
    ANS(AnsClient),
    USR(UsrClient),
    CAL(CalClient),
    MSG(MsgClient),
    OUT,
    RAW(RawCommand)
}

impl MSNPCommand for SwitchboardClientCommand {
    type Err = CommandError;

    fn try_from_raw(raw: RawCommand) -> Result<Self, Self::Err> {
        let out = match raw.get_operand() {
            "ANS" => SwitchboardClientCommand::ANS(AnsClient::try_from_raw(raw)?),
            "USR" => SwitchboardClientCommand::USR(UsrClient::try_from_raw(raw)?),
            "CAL" => SwitchboardClientCommand::CAL(CalClient::try_from_raw(raw)?),
            "MSG" => SwitchboardClientCommand::MSG(MsgClient::try_from_raw(raw)?),
            "OUT" => SwitchboardClientCommand::OUT,
            _ => SwitchboardClientCommand::RAW(raw),
        };
        Ok(out)    }

    fn into_bytes(self) -> Vec<u8> {
        todo!()
    }
}

#[derive(Display)]
pub enum SwitchboardServerCommand {
    OK(OkCommand),
    USR(UsrServerOk),
    CAL(CalServer),
    ACK(AckServer),
    MSG(MsgServer),
    IRO(IroServer),
    JOI(JoiServer),
    OUT,
    RAW(RawCommand)
}

impl MSNPCommand for SwitchboardServerCommand {
    type Err = CommandError;

    fn try_from_raw(raw: RawCommand) -> Result<Self, Self::Err> {
        todo!()
    }

    fn into_bytes(self) -> Vec<u8> {
        match self {
            SwitchboardServerCommand::OK(command) => command.into_bytes(),
            SwitchboardServerCommand::USR(command) => command.into_bytes(),
            SwitchboardServerCommand::CAL(command) => command.into_bytes(),
            SwitchboardServerCommand::ACK(command) => command.into_bytes(),
            SwitchboardServerCommand::MSG(command) => command.into_bytes(),
            SwitchboardServerCommand::IRO(command) => command.into_bytes(),
            SwitchboardServerCommand::JOI(command) => command.into_bytes(),
            SwitchboardServerCommand::OUT => b"OUT\r\n".to_vec(),
            SwitchboardServerCommand::RAW(command) => command.into_bytes(),
        }    }
}