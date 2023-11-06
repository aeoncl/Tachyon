use async_trait::async_trait;

use crate::models::notification::error::MSNPServerError;

use super::msnp_command::MSNPCommand;

#[async_trait]
pub trait CommandHandler : Send {

    async fn handle_command(&mut self, command: &MSNPCommand) -> Result<String, MSNPServerError>;

    fn get_matrix_token(&self) -> String;
}