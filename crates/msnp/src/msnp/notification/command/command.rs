
use strum_macros::Display;

use crate::msnp::{error::CommandError, raw_command_parser::RawCommand};
use crate::msnp::notification::command::blp::BlpServer;
use crate::msnp::notification::command::chg::ChgServer;
use crate::msnp::notification::command::cvr::CvrServer;
use crate::msnp::notification::command::iln::IlnServer;
use crate::msnp::notification::command::msg::MsgServer;
use crate::msnp::notification::command::nfy::NfyServer;
use crate::msnp::notification::command::nln::NlnServer;
use crate::msnp::notification::command::not::NotServer;
use crate::msnp::notification::command::put::{PutClient, PutServer};
use crate::msnp::notification::command::sdg::{SdgClient, SdgServer};
use crate::msnp::notification::command::ubx::UbxServer;
use crate::msnp::notification::command::usr::UsrServer;
use crate::msnp::notification::command::uum::UumClient;
use crate::msnp::notification::command::uux::UuxServer;
use crate::msnp::notification::command::ver::VerServer;
use crate::shared::command::ok::OkCommand;
use crate::shared::traits::MSNPCommand;

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
    UUM(UumClient),
    SDG(SdgClient),
    PUT(PutClient),
    XFR(),
    OUT,
    RAW(RawCommand)
}

impl MSNPCommand for NotificationClientCommand {
    type Err = CommandError;

    fn try_from_raw(raw: RawCommand) -> Result<Self, Self::Err> {
        let out = match raw.get_operand() {
            "VER" => NotificationClientCommand::VER(VerClient::try_from_raw(raw)?),
            "CVR" => NotificationClientCommand::CVR(CvrClient::try_from_raw(raw)?),
            "USR" => NotificationClientCommand::USR(UsrClient::try_from_raw(raw)?),
            "PNG" => NotificationClientCommand::PNG,
            "ADL" => NotificationClientCommand::ADL(AdlClient::try_from_raw(raw)?),
            "RML" => NotificationClientCommand::RML(RmlClient::try_from_raw(raw)?),
            "UUX" => NotificationClientCommand::UUX(UuxClient::try_from_raw(raw)?),
            "BLP" => NotificationClientCommand::BLP(BlpClient::try_from_raw(raw)?),
            "CHG" => NotificationClientCommand::CHG(ChgClient::try_from_raw(raw)?),
            "PRP" => NotificationClientCommand::PRP(PrpClient::try_from_raw(raw)?),
            "UUN" => NotificationClientCommand::UUN(UunClient::try_from_raw(raw)?),
            "UUM" => NotificationClientCommand::UUM(UumClient::try_from_raw(raw)?),
            "SDG" => NotificationClientCommand::SDG(SdgClient::try_from_raw(raw)?),
            "PUT" => NotificationClientCommand::PUT(PutClient::try_from_raw(raw)?),
            "XFR" => NotificationClientCommand::XFR(),
            "OUT" => NotificationClientCommand::OUT,
            _ => NotificationClientCommand::RAW(raw)
            //Err(CommandError::UnsupportedCommand { command: format!("{:?}", command) })
        };

        Ok(out)
    }

    fn into_bytes(self) -> Vec<u8> {
        todo!()
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
    Uux(UuxServer),
    UBX(UbxServer),
    Ok(OkCommand),
    CHG(ChgServer),
    NFY(NfyServer),
    BLP(BlpServer),
    NOT(NotServer),
    ILN(IlnServer),
    NLN(NlnServer),
    PUT(PutServer),
    SDG(SdgServer),
    OUT,
    RAW(RawCommand)
}

impl MSNPCommand for NotificationServerCommand {
    type Err = CommandError;

    fn try_from_raw(raw: RawCommand) -> Result<Self, Self::Err> {
        todo!()
    }

    fn into_bytes(self) -> Vec<u8> {
        match self {
            NotificationServerCommand::VER(command) => command.into_bytes(),
            NotificationServerCommand::CVR(command) => command.into_bytes(),
            NotificationServerCommand::MSG(command) => command.into_bytes(),
            NotificationServerCommand::QNG(timeout) => format!("QNG {}\r\n", timeout).into_bytes(),
            NotificationServerCommand::USR(command) => command.into_bytes(),
            NotificationServerCommand::Ok(command) => command.into_bytes(),
            NotificationServerCommand::Uux(command) => command.into_bytes(),
            NotificationServerCommand::CHG(command) => command.into_bytes(),
            NotificationServerCommand::BLP(command) => command.into_bytes(),
            NotificationServerCommand::OUT => b"OUT\r\n".to_vec(),
            NotificationServerCommand::RAW(content) => content.into_bytes(),
            NotificationServerCommand::NOT(content) => {content.into_bytes()},
            NotificationServerCommand::ILN(content) => { content.into_bytes() },
            NotificationServerCommand::UBX(content) => { content.into_bytes()}
            NotificationServerCommand::NFY(content) => {content.into_bytes()}
            NotificationServerCommand::PUT(content) => {content.into_bytes()}
            NotificationServerCommand::NLN(content) => { content.into_bytes() }
            NotificationServerCommand::SDG(content) => { content.into_bytes() }
        }
    }
}

