use std::str::FromStr;

use strum_macros::{Display, EnumString};

use crate::msnp::{cvr::{CvrClient, CvrServer}, error::CommandError, raw_command_parser::{self, RawCommand, RawCommandParser}, ver::{self, VerClient, VerServer}};
use super::protocol::MSNP18;

#[derive(Display)]
pub enum NotificationCommand {
    VER(VerClient),
    CVR(CvrClient),
    USR(),
    PNG(),
    ADL(),
    RML(),
    UUX(),
    BLP(),
    CHG(),
    PRP(),
    UUN(),
    XFR()
}

pub enum SwitchboardCommand {
    MSG(),
    USR()
}

pub struct NotificationCommandParser {

    raw_parser : RawCommandParser<MSNP18>

}

impl NotificationCommandParser {

    pub fn new() -> Self {
        Self {
            raw_parser: RawCommandParser::new(MSNP18::new())
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