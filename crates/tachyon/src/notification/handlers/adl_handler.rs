use log::debug;
use tokio::sync::mpsc::Sender;
use msnp::msnp::notification::command::adl::AdlClient;
use msnp::msnp::notification::command::command::NotificationServerCommand;
use crate::notification::client_store::ClientData;

pub async fn handle_adl(command: AdlClient, client_data: ClientData, command_sender: Sender<NotificationServerCommand>) -> Result<(), anyhow::Error>  {
    debug!("ADL: {:?}", &command);

    let contacts = command.payload.get_contacts()?;

    {
    let mut contact_list = client_data.get_contact_list().lock().unwrap();
        contact_list.add_contacts(contacts, command.payload.is_initial());
    }
    
    
    command_sender.send(NotificationServerCommand::OK(command.get_ok_response("ADL"))).await?;

    Ok(())
}
