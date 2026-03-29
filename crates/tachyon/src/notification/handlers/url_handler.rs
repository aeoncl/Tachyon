use tokio::sync::mpsc::Sender;
use msnp::msnp::notification::command::command::NotificationServerCommand;
use msnp::msnp::notification::command::url::{UrlClient, UrlServer, UrlType};
use crate::notification::models::local_client_data::LocalClientData;
use crate::tachyon::tachyon_client::TachyonClient;

pub async fn handle_url(command: UrlClient, client_data: &mut LocalClientData, tachyon_client: TachyonClient, sender: Sender<NotificationServerCommand>) -> Result<(), anyhow::Error>{

    //TODO: handle url types

    sender.send(NotificationServerCommand::URL(
        UrlServer::new(command.tr_id, "https://hotmail.live.com/tachyon".to_string(), "https://login.live.com/RST2.srf".to_string(),0)
    )).await;

    Ok(())
}