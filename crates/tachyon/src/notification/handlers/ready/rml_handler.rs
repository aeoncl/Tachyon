use crate::tachyon::tachyon_client::TachyonClient;
use log::debug;
use msnp::msnp::notification::command::adl::RmlClient;
use msnp::msnp::notification::command::command::NotificationServerCommand;

use tokio::sync::mpsc::Sender;

pub async fn handle_rml(
    command: RmlClient,
    client_data: TachyonClient,
    command_sender: Sender<NotificationServerCommand>,
) -> Result<(), anyhow::Error> {
    debug!("RML: {:?}", &command);

    client_data
        .get_contact_list()
        .lock()
        .unwrap()
        .remove_contacts(command.payload.get_contacts()?);
    command_sender
        .send(NotificationServerCommand::OK(
            command.get_ok_response("RML"),
        ))
        .await?;

    Ok(())
}
