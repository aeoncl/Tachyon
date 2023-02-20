use async_trait::async_trait;

use crate::models::notification::error::MsnpError;

use super::msnp_command::MSNPCommand;

#[async_trait]
pub trait CommandHandler : Send {

    async fn handle_command(&mut self, command: &MSNPCommand) -> Result<String, MsnpError>;

    fn get_matrix_token(&self) -> String;
}