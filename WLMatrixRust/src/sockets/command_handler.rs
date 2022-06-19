use async_trait::async_trait;
use tokio::sync::broadcast::{Sender, self};
use super::msnp_command::MSNPCommand;
pub struct NotificationCommandHandler {
    protocol_version: i16,
    msn_addr: String,
    matrix_token: String,
    sender: Sender<String>,
    kill_sender: Option<Sender<String>>
}

#[async_trait]
pub trait CommandHandler : Send {

    async fn handle_command(&mut self, command: &MSNPCommand) -> String;

    fn get_matrix_token(&self) -> String;

    fn cleanup(&self);
}