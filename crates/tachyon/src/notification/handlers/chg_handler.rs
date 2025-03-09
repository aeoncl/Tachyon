use log::error;
use tokio::sync::mpsc::Sender;
use msnp::msnp::notification::command::chg::ChgClient;
use msnp::msnp::notification::command::command::NotificationServerCommand;
use msnp::msnp::notification::command::uux::UuxClient;
use crate::matrix::sync2::sliding_sync;
use crate::matrix::sync::initial_sync;
use crate::notification::client_store::ClientData;
use crate::notification::models::local_client_data::LocalClientData;

pub async fn handle_chg(command: ChgClient, local_store: &mut LocalClientData, client_data: ClientData, command_sender: Sender<NotificationServerCommand>) -> Result<(), anyhow::Error>  {
    if local_store.needs_initial_presence {
        local_store.needs_initial_presence = false;

        let notif_sender = command_sender.clone();
        let client_data = client_data.clone();

        tokio::spawn(async move {
            let initial_sync_result = sliding_sync(command.tr_id, &client_data).await;
            if let Err(err) = initial_sync_result.as_ref() {
                error!("An error occured during initial sync: {}", err);
                //TODO return a real error instead of outing the client
                let _result = notif_sender.send(NotificationServerCommand::OUT).await;
            }

            let (mut iln, mut notifications) = initial_sync_result.expect("to be here");

            for current in iln.drain(..) {
                let _result = notif_sender.send(NotificationServerCommand::ILN(current)).await;
            }

            for current in notifications.drain(..) {
                let _result = notif_sender.send(NotificationServerCommand::NOT(current)).await;
            }
        });
    }


    command_sender.send(NotificationServerCommand::CHG(command.clone())).await?;

    //notif_sender.send(NotificationServerCommand::SDG())


    Ok(())
}
