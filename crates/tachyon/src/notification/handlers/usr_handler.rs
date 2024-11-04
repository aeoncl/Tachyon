use tokio::sync::mpsc::Sender;
use msnp::msnp::notification::command::command::NotificationServerCommand;
use msnp::msnp::notification::command::usr::{AuthOperationTypeClient, OperationTypeServer, UsrClient, UsrServer};
use msnp::shared::models::email_address::EmailAddress;

pub async fn handle_usr(command: UsrClient, email_addr: EmailAddress, command_sender: Sender<NotificationServerCommand>) -> Result<(), anyhow::Error>  {
    match command.auth_type {

        AuthOperationTypeClient::Sso(_) => {
            todo!()
            //return error;
        },
        AuthOperationTypeClient::Sha(phase) => {

            let usr_response = UsrServer::new(command.tr_id, OperationTypeServer::Ok {
                email_addr,
                verified: true,
                unknown_arg: false,
            });
            command_sender.send(NotificationServerCommand::USR(usr_response)).await?;
        }

    }
    Ok(())
}