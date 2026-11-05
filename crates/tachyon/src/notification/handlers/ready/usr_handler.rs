use msnp::msnp::notification::command::command::NotificationServerCommand;
use msnp::msnp::notification::command::usr::{AuthOperationTypeClient, OperationTypeServer, UsrClient, UsrServer};
use msnp::shared::models::email_address::EmailAddress;
use crate::notification::models::client_out::ClientOut;

/// Pure decision (§3.1, pattern A): the Ready-phase USR re-auth answer. The
/// Sha arm rides Command(..) with today's exact USR OK fields; the dispatcher
/// (ready/mod.rs) sends it through send_client_outs, preserving the `?` send
/// policy this handler used to hold itself.
pub(crate) fn decide_usr(command: UsrClient, email_addr: EmailAddress) -> ClientOut {
    match command.auth_type {

        AuthOperationTypeClient::Sso(_) => {
            // Stays per the fence: an M2-era shape. The legacy/SSO re-auth
            // handshake is not decided (§3.7, §10); no stub ships (§6 D).
            todo!()
            //return error;
        },
        AuthOperationTypeClient::Sha(_phase) => {
            ClientOut::Command(NotificationServerCommand::USR(UsrServer::new(command.tr_id, OperationTypeServer::Ok {
                email_addr,
                verified: true,
                unknown_arg: false,
            })))
        }

    }
}