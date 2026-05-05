use crate::notification::models::local_client_data::LocalClientData;
use crate::tachyon::tachyon_client::TachyonClient;
use msnp::msnp::notification::command::command::NotificationServerCommand;
use msnp::msnp::notification::command::nfy::{NfyOperation, NfyServer};
use msnp::msnp::notification::command::put::PutClient;
use tokio::sync::mpsc::Sender;

pub async fn handle_put(
    command: PutClient,
    _local_store: &mut LocalClientData,
    _client_data: TachyonClient,
    command_sender: Sender<NotificationServerCommand>,
) -> Result<(), anyhow::Error> {
    let ok = command.get_ok_command();
    command_sender
        .send(NotificationServerCommand::PUT(ok))
        .await?;

    let mut payload = command.payload;
    payload.envelope.swap_sides();
    payload.envelope.flags = None;

    command_sender
        .send(NotificationServerCommand::NFY(NfyServer {
            operation: NfyOperation::Put,
            payload,
        }))
        .await?;

    Ok(())
}
