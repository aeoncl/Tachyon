
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
        let out = match command.get_operand() {
            "VER" => NotificationClientCommand::VER(VerClient::try_from(command)?),
            "CVR" => NotificationClientCommand::CVR(CvrClient::try_from(command)?),
            "USR" => NotificationClientCommand::USR(UsrClient::try_from(command)?),
            "PNG" => NotificationClientCommand::PNG,
            "ADL" => NotificationClientCommand::ADL(AdlClient::try_from(command)?),
            "RML" => NotificationClientCommand::RML(RmlClient::try_from(command)?),
            "UUX" => NotificationClientCommand::UUX(UuxClient::try_from(command)?),
            "BLP" => NotificationClientCommand::BLP(BlpClient::try_from(command)?),
            "CHG" => NotificationClientCommand::CHG(ChgClient::try_from(command)?),
            "PRP" => NotificationClientCommand::PRP(PrpClient::try_from(command)?),
            "UUN" => NotificationClientCommand::UUN(UunClient::try_from(command)?),
            "XFR" => NotificationClientCommand::XFR(),
            "OUT" => NotificationClientCommand::OUT,
            _ => NotificationClientCommand::RAW(command)
                //Err(CommandError::UnsupportedCommand { command: format!("{:?}", command) })
        };

        Ok(out)
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
            NotificationServerCommand::VER(command) => command.serialize_msnp(),
            NotificationServerCommand::CVR(command) => command.serialize_msnp(),
            NotificationServerCommand::MSG(command) => command.serialize_msnp(),
            NotificationServerCommand::QNG(timeout) => format!("QNG {}\r\n", timeout).into_bytes(),
            NotificationServerCommand::USR(command) => command.serialize_msnp(),
            NotificationServerCommand::OUT => b"OUT\r\n".to_vec(),
            NotificationServerCommand::RAW(content) => content.serialize_msnp()
        }
    }
}