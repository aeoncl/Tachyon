use strum_macros::Display;
use crate::msnp::raw_command_parser::RawCommand;
use crate::msnp::switchboard::command::ack::AckServer;
use crate::msnp::switchboard::command::ans::AnsClient;
use crate::msnp::switchboard::command::cal::{CalClient, CalServer};
use crate::msnp::switchboard::command::Iro::IroServer;
use crate::msnp::switchboard::command::joi::JoiServer;
use crate::msnp::switchboard::command::msg::{MsgClient, MsgServer};
use crate::msnp::switchboard::command::usr::{UsrClient, UsrServerOk};
use crate::shared::command::ok::OkCommand;

#[derive(Display)]
pub enum SwitchboardClientCommand {
    ANS(AnsClient),
    USR(UsrClient),
    CAL(CalClient),
    MSG(MsgClient),
    OUT,
    RAW(RawCommand)
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