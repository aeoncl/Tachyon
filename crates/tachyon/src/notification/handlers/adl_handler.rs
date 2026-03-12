use log::debug;
use tokio::sync::mpsc::Sender;
use msnp::msnp::notification::command::adl::AdlClient;
use msnp::msnp::notification::command::command::NotificationServerCommand;
use msnp::msnp::notification::command::iln::IlnServer;
use msnp::msnp::notification::command::nln::NlnServer;
use msnp::shared::models::network_id_email::NetworkIdEmail;
use msnp::shared::models::presence_status::PresenceStatus;
use crate::tachyon::tachyon_client::TachyonClient;

pub async fn handle_adl(command: AdlClient, client_data: TachyonClient, command_sender: Sender<NotificationServerCommand>) -> Result<(), anyhow::Error>  {
    debug!("ADL: {:?}", &command);

    let contacts = command.payload.get_contacts()?;

    {
    let mut contact_list = client_data.get_contact_list().lock().unwrap();
        contact_list.add_contacts(contacts.clone(), command.payload.is_initial());
    }
    
    
    command_sender.send(NotificationServerCommand::OK(command.get_ok_response("ADL"))).await?;

    if !command.payload.is_initial() {

        for contact in contacts {
            command_sender.send(NotificationServerCommand::NLN(NlnServer{
                presence_status: PresenceStatus::NLN,
                target_user: NetworkIdEmail {
                    network_id: contact.network_id,
                    email: contact.email_address,
                },
                via: None,
                display_name: "".to_string(),
                client_capabilities: Default::default(),
                avatar: None,
                badge_url: None,
            })).await?;
        }

    }

    
    Ok(())
}
