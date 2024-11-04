use msnp::msnp::notification::command::command::NotificationServerCommand;
use tokio::sync::mpsc::Sender;

pub async fn handle_png(command_sender: Sender<NotificationServerCommand>) -> Result<(), anyhow::Error>  {
    command_sender.send(NotificationServerCommand::QNG(60)).await?;
    Ok(())
}
