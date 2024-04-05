
use strum_macros::Display;

use crate::msnp::{error::CommandError, raw_command_parser::RawCommand};
use crate::msnp::notification::command::cvr::CvrServer;
use crate::msnp::notification::command::msg::MsgServer;
use crate::msnp::notification::command::usr::UsrServer;
use crate::msnp::notification::command::ver::VerServer;
use crate::shared::traits::SerializeMsnp;

use super::{adl::{AdlClient, RmlClient}, blp::BlpClient, chg::ChgClient, cvr::CvrClient, prp::PrpClient, usr::UsrClient, uun::UunClient, uux::UuxClient, ver::VerClient};

#[derive(Display)]
pub enum NotificationClientCommand {
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
    OUT,
    RAW(RawCommand)
}

impl TryFrom<RawCommand> for NotificationClientCommand {
    type Error = CommandError;

    fn try_from(command: RawCommand) -> Result<Self, Self::Error> {
        match command.get_operand() {
            "VER" => {
                Ok(NotificationClientCommand::VER(VerClient::try_from(command)?))
            },
            "CVR" => {
                Ok(NotificationClientCommand::CVR(CvrClient::try_from(command)?))
            }
            _ => {
                Ok(NotificationClientCommand::RAW(command))
                //Err(CommandError::UnsupportedCommand { command: format!("{:?}", command) })
            }
        }
    }
}

#[derive(Display)]
pub enum NotificationServerCommand {
    VER(VerServer),
    CVR(CvrServer),
    MSG(MsgServer),
    //Timeout before the next PNG command from client
    QNG(u32),
    USR(UsrServer),
    OUT,
    RAW(RawCommand)
}

impl NotificationServerCommand {

    pub fn serialize_msnp(self) -> Vec<u8> {
        match self {
            NotificationServerCommand::OUT => {
                b"OUT\r\n".to_vec()
            }
            NotificationServerCommand::RAW(content) => {
                content.serialize_msnp()
            }
            NotificationServerCommand::VER(command) => {
                command.serialize_msnp()
            }
            NotificationServerCommand::CVR(command) => {
                command.serialize_msnp()
            }
            NotificationServerCommand::QNG(timeout) => {
                format!("QNG {}\r\n", timeout).into_bytes()
            },
            NotificationServerCommand::MSG(command) => {
                command.serialize_msnp()
            }
            NotificationServerCommand::USR(command) => {
                command.serialize_msnp()
            }
        }
    }
}