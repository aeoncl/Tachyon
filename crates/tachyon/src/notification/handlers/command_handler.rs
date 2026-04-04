use msnp::msnp::notification::command::command::{NotificationClientCommand, NotificationServerCommand};
use tokio::sync::mpsc::Sender;
use anyhow::anyhow;
use crate::notification::handlers::{auth, negotiation};
use crate::notification::handlers::ready::handle_ready;
use crate::notification::models::connection_phase::ConnectionPhase;
use crate::notification::models::local_client_data::LocalClientData;
use crate::tachyon::tachyon_state::TachyonState;

pub(crate) async fn handle_command(command: NotificationClientCommand, command_sender: Sender<NotificationServerCommand>, tachyon_state: &TachyonState, local_client_data: &mut LocalClientData) -> Result<(), anyhow::Error> {

    let _command_result = match &local_client_data.phase {
        ConnectionPhase::Negotiating => {
            negotiation::handle_negotiation(command, command_sender, local_client_data).await
        },
        ConnectionPhase::Authenticating  => {
            auth::handle_auth(command, command_sender, &tachyon_state, local_client_data).await
        },
        ConnectionPhase::Ready => {
            let matrix_client = local_client_data.matrix_client.as_ref().ok_or(anyhow!("Matrix Client should be here by now"))?.clone();
            let tachyon_client = local_client_data.tachyon_client.as_ref().ok_or(anyhow!("Tachyon Client should be here by now"))?.clone();
            handle_ready(command, command_sender, tachyon_client, matrix_client, local_client_data).await
        }
    };

    Ok(())

}