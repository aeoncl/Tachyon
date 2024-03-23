
use strum_macros::Display;

use crate::msnp::{error::CommandError, raw_command_parser::{RawCommand, RawCommandParser}};

use super::{adl::{AdlClient, RmlClient}, cvr::CvrClient, usr::UsrClient, uux::UuxClient, ver::VerClient};

#[derive(Display)]
pub enum NotificationCommand {
    VER(VerClient),
    CVR(CvrClient),
    USR(UsrClient),
    PNG,
    ADL(AdlClient),
    RML(RmlClient),
    UUX(UuxClient),
    BLP(),
    CHG(),
    PRP(),
    UUN(),
    XFR()
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

    pub fn parse_message(&mut self, message: &str) -> Result<Vec<Result<NotificationCommand, CommandError>>, CommandError> {
        let raw_commands = self.raw_parser.parse_message(message)?;
        let mut out = Vec::with_capacity(raw_commands.len());

        for raw_command in raw_commands {
            out.push(Self::parse_raw_command(raw_command));
        }

        Ok(out)

    }

    fn parse_raw_command(command: RawCommand) -> Result<NotificationCommand, CommandError> {
        match command.operand.as_str() {
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