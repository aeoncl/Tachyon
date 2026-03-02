use crate::notification::models::local_client_data::LocalClientData;
use msnp::msnp::notification::command::command::NotificationServerCommand;
use msnp::msnp::notification::command::xfr::XfrClient;
use msnp::msnp::notification::models::ip_address::IpAddress;
use std::str::FromStr;
use tokio::sync::mpsc::Sender;

pub async fn handle_xfr(command: XfrClient, local_store: &mut LocalClientData, command_sender: Sender<NotificationServerCommand>) -> Result<(), anyhow::Error>  {
    let xfr_response = command.get_response_for(IpAddress::from_str("127.0.0.1:1864")?, local_store.token.to_string());
    command_sender.send(NotificationServerCommand::XFR(xfr_response)).await?;
    Ok(())
}
