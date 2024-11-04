use crate::notification::client_store::ClientData;
use crate::notification::models::local_client_data::LocalClientData;
use msnp::msnp::notification::command::command::NotificationServerCommand;
use msnp::msnp::notification::command::put::PutClient;
use tokio::sync::mpsc::Sender;
use msnp::msnp::notification::command::nfy::{NfyOperation, NfyServer};

pub async fn handle_put(command: PutClient, local_store: &mut LocalClientData, client_data: ClientData, command_sender: Sender<NotificationServerCommand>) -> Result<(), anyhow::Error> {
    let ok = command.get_ok_command();
    command_sender.send(NotificationServerCommand::PUT(ok)).await?;

    let mut payload = command.payload;
    payload.envelope.swap_sides();
    payload.envelope.flags = None;

    command_sender.send(NotificationServerCommand::NFY(NfyServer {
        operation: NfyOperation::Put,
        payload
    })).await?;

    Ok(())
}