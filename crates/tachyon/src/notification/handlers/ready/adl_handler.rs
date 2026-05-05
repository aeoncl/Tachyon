use crate::matrix::extensions::msn_user_resolver::ToMsnUser;
use crate::tachyon::client::tachyon_session_data::TachyonSessionData;
use crate::tachyon::services::session::user_service::UserService;
use crate::tachyon::identifiers::is_sha1::IsSha1;
use log::debug;
use matrix_sdk::Client;
use msnp::msnp::notification::command::adl::AdlClient;
use msnp::msnp::notification::command::command::NotificationServerCommand;
use msnp::msnp::notification::command::iln::IlnServer;
use msnp::msnp::notification::command::nln::NlnServer;
use msnp::msnp::notification::command::ubx::{ExtendedPresenceContent, UbxPayload, UbxServer};
use msnp::shared::models::display_name::DisplayName;
use msnp::shared::models::network_id_email::NetworkIdEmail;
use msnp::shared::models::presence_status::PresenceStatus;
use std::time::Duration;
use tokio::sync::mpsc::Sender;
use tokio::time::sleep;

pub async fn handle_adl(command: AdlClient, tachyon_client: TachyonSessionData, user_service: Box<dyn UserService>, command_sender: Sender<NotificationServerCommand>) -> Result<(), anyhow::Error>  {
    debug!("ADL: {:?}", &command);

    let contacts = command.payload.get_contacts()?;

    {
    let mut contact_list = tachyon_client.get_contact_list().lock().unwrap();
        contact_list.add_contacts(contacts.clone(), command.payload.is_initial());
    }

    command_sender.send(NotificationServerCommand::OK(command.get_ok_response("ADL"))).await?;

    tokio::spawn( async move {

        sleep(Duration::from_millis(1000)).await;
        //Hardcoded presence to Online
        //TODO: implement real presence
        for contact in contacts {

            if !contact.email_address.is_sha1_imprecise() {
                continue;
            }



            let display_name = if let Some(proxy_user) = user_service.resolve_room_proxy_user_from_email(&contact.email_address).await {
                proxy_user.display_name.clone()
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
