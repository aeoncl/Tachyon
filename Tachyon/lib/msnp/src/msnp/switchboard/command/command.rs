use strum_macros::Display;
use crate::msnp::error::CommandError;
use crate::msnp::raw_command_parser::RawCommand;
use crate::msnp::switchboard::command::ack::AckServer;
use crate::msnp::switchboard::command::ans::AnsClient;
use crate::msnp::switchboard::command::cal::{CalClient, CalServer};
use crate::msnp::switchboard::command::Iro::IroServer;
use crate::msnp::switchboard::command::joi::JoiServer;
use crate::msnp::switchboard::command::msg::{MsgClient, MsgServer};
use crate::msnp::switchboard::command::usr::{UsrClient, UsrServerOk};
use crate::shared::command::ok::OkCommand;
use crate::shared::traits::SerializeMsnp;

#[derive(Display)]
pub enum SwitchboardClientCommand {
    ANS(AnsClient),
    USR(UsrClient),
    CAL(CalClient),
    MSG(MsgClient),
    OUT,
    RAW(RawCommand)
}

impl TryFrom<RawCommand> for SwitchboardClientCommand {
    type Error = CommandError;

    fn try_from(value: RawCommand) -> Result<Self, Self::Error> {

        let out = match value.get_operand() {
            "ANS" => SwitchboardClientCommand::ANS(AnsClient::try_from(value)?),
            "USR" => SwitchboardClientCommand::USR(UsrClient::try_from(value)?),
            "CAL" => SwitchboardClientCommand::CAL(CalClient::try_from(value)?),
            "MSG" => SwitchboardClientCommand::MSG(MsgClient::try_from(value)?),
            "OUT" => SwitchboardClientCommand::OUT,
            _ => SwitchboardClientCommand::RAW(value),
        };

        Ok(out)
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

impl SwitchboardServerCommand {
    pub fn serialize_msnp(self) -> Vec<u8> {
        match self {
            SwitchboardServerCommand::OK(command) => command.serialize_msnp(),
            SwitchboardServerCommand::USR(command) => command.serialize_msnp(),
            SwitchboardServerCommand::CAL(command) => command.serialize_msnp(),
            SwitchboardServerCommand::ACK(command) => command.serialize_msnp(),
            SwitchboardServerCommand::MSG(command) => command.serialize_msnp(),
            SwitchboardServerCommand::IRO(command) => command.serialize_msnp(),
            SwitchboardServerCommand::JOI(command) => command.serialize_msnp(),
            SwitchboardServerCommand::OUT => b"OUT\r\n".to_vec(),
            SwitchboardServerCommand::RAW(command) => command.serialize_msnp(),
        }


    }
}