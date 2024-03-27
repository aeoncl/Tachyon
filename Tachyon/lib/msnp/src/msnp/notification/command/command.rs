
use strum_macros::Display;

use crate::{msnp::{error::CommandError, raw_command_parser::{RawCommand, RawCommandParser}}, shared::command::command::SerializeMsnp};

use super::{adl::{AdlClient, RmlClient}, blp::BlpClient, chg::ChgClient, cvr::CvrClient, prp::PrpClient, usr::UsrClient, uun::UunClient, uux::UuxClient, ver::VerClient};

#[derive(Display)]
pub enum NotificationCommand {
    VER(VerClient),
    CVR(CvrClient),
    USR(UsrClient),
    PNG,
    ADL(AdlClient),
    RML(RmlClient),
    UUX(UuxClient),
    BLP(BlpClient),
    CHG(ChgClient),
    PRP(PrpClient),
    UUN(UunClient),
    XFR(),
    RAW(Vec<u8>)
}

impl NotificationCommand {
    
    fn serialize_msnp(self) -> Vec<u8> {

        if let NotificationCommand::RAW(data) = self {
            return data;
        }

        todo!()
    }
}



pub struct NotificationCommandParser {
    raw_parser : RawCommandParser
}

impl NotificationCommandParser {

    pub fn new() -> Self {
        Self {
            raw_parser: RawCommandParser::new()
        }
    }

    pub fn parse_message(&mut self, message: &[u8]) -> Result<Vec<Result<NotificationCommand, CommandError>>, CommandError> {
        let raw_commands = self.raw_parser.parse_message(message)?;
        let mut out = Vec::with_capacity(raw_commands.len());

        for raw_command in raw_commands {
            out.push(Self::parse_raw_command(raw_command));
        }

        Ok(out)

    }

    fn parse_raw_command(command: RawCommand) -> Result<NotificationCommand, CommandError> {
        match command.get_operand() {
            "VER" => {
                Ok(NotificationCommand::VER(VerClient::try_from(command)?))
            },
            "CVR" => {
                Ok(NotificationCommand::CVR(CvrClient::try_from(command)?))
            }
            _ => {
                Err(CommandError::UnsupportedCommand { command: format!("{:?}", command) })
            }
        } 
    }

}