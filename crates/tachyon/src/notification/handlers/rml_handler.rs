use log::debug;
use tokio::sync::mpsc::Sender;
use msnp::msnp::notification::command::adl::{AdlClient, RmlClient};
use msnp::msnp::notification::command::command::NotificationServerCommand;
use crate::notification::client_store::ClientData;

pub async fn handle_rml(command: RmlClient, client_data: ClientData, command_sender: Sender<NotificationServerCommand>) -> Result<(), anyhow::Error>  {
    debug!("RML: {:?}", &command);

    client_data.inner.contact_list.lock().unwrap().remove_contacts(command.payload.get_contacts()?);
    command_sender.send(NotificationServerCommand::OK(command.get_ok_response("RML"))).await?;

    Ok(())
}
