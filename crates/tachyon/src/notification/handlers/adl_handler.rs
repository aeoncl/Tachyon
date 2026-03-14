use std::time::Duration;
use log::debug;
use tokio::sync::mpsc::Sender;
use tokio::time::sleep;
use msnp::msnp::notification::command::adl::AdlClient;
use msnp::msnp::notification::command::command::NotificationServerCommand;
use msnp::msnp::notification::command::iln::IlnServer;
use msnp::msnp::notification::command::nln::NlnServer;
use msnp::msnp::notification::command::ubx::{ExtendedPresenceContent, UbxPayload, UbxServer};
use msnp::shared::models::display_name::DisplayName;
use msnp::shared::models::network_id_email::NetworkIdEmail;
use msnp::shared::models::presence_status::PresenceStatus;
use crate::matrix::extensions::msn_user_resolver::{FindRoomFromEmail, ToMsnUser};
use crate::tachyon::tachyon_client::TachyonClient;

pub async fn handle_adl(command: AdlClient, tachyon_client: TachyonClient, command_sender: Sender<NotificationServerCommand>) -> Result<(), anyhow::Error>  {
    debug!("ADL: {:?}", &command);

    let contacts = command.payload.get_contacts()?;

    {
    let mut contact_list = tachyon_client.get_contact_list().lock().unwrap();
        contact_list.add_contacts(contacts.clone(), command.payload.is_initial());
    }
    
    
    command_sender.send(NotificationServerCommand::OK(command.get_ok_response("ADL"))).await?;


    let matrix_client = tachyon_client.matrix_client();
    tokio::spawn( async move {

        sleep(Duration::from_millis(1000)).await;
        //Hardcoded presence to Online
        //TODO: implement real presence
        for contact in contacts {

            let display_name = if let Ok(Some(room)) = matrix_client.find_room_from_email(&contact.email_address) {
                if let Ok(msn_user) = room.to_msn_user_lazy().await {
                    msn_user.display_name
                } else {
                    None
                }

            } else {
                None
            };

            let network_id_email = NetworkIdEmail {
                network_id: contact.network_id.clone(),
                email: contact.email_address.clone(),
            };

            let _ = command_sender.send(NotificationServerCommand::NLN(NlnServer{
                presence_status: PresenceStatus::NLN,
                target_user: network_id_email.clone(),
                via: None,
                display_name: display_name.map(|name| DisplayName::new(name) ).unwrap_or_default(),
                client_capabilities: Default::default(),
                avatar: None,
                badge_url: None,
            })).await;

            let _ = command_sender.send(NotificationServerCommand::UBX(UbxServer {
                target_user: network_id_email,
                via: None,
                payload: UbxPayload::ExtendedPresence(ExtendedPresenceContent {
                    psm: "".to_string(),
                    current_media: "".to_string(),
                    endpoint_data: Default::default(),
                    private_endpoint_data: None,
                }),
            })).await;
        }
    });
    Ok(())
}
