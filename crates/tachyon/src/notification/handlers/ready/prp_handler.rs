use crate::notification::models::local_client_data::LocalClientData;
use crate::tachyon::tachyon_client::TachyonClient;
use msnp::msnp::notification::command::command::NotificationServerCommand;
use msnp::msnp::notification::command::prp::{PrpClient, PrpOperation};
use tokio::sync::mpsc::Sender;

pub async fn handle_prp(command: PrpClient, local_store: &mut LocalClientData, client_data: TachyonClient, command_sender: Sender<NotificationServerCommand>) -> Result<(), anyhow::Error>  {

    match &command.operation {



        PrpOperation::ModifyName { display_name } => {
            let to_set = if display_name.is_empty() {
                None
            } else {
                Some(display_name.as_str())
            };

            client_data.own_user_mut().display_name = to_set.map(|n| n.to_owned());
            command_sender.send(NotificationServerCommand::PRP(command)).await?;
        }
    }

    Ok(())

}
