use crate::notification::models::local_client_data::LocalClientData;
use crate::tachyon::client::tachyon_client::TachyonClient;
use crate::tachyon::client::user_service::UserService;
use crate::tachyon::identifiers::is_sha1::IsSha1;
use matrix_sdk::Client;
use msnp::msnp::notification::command::chg::ChgClient;
use msnp::msnp::notification::command::command::NotificationServerCommand;
use msnp::msnp::notification::command::iln::IlnServer;
use msnp::msnp::notification::command::ubx::{ExtendedPresenceContent, UbxPayload, UbxServer};
use msnp::shared::models::display_name::DisplayName;
use msnp::shared::models::network_id_email::NetworkIdEmail;
use msnp::shared::models::presence_status::PresenceStatus;
use tokio::sync::mpsc::Sender;

pub async fn handle_chg(command: ChgClient, local_store: &mut LocalClientData, client_data: TachyonClient, user_service: Box<dyn UserService>, command_sender: Sender<NotificationServerCommand>) -> Result<(), anyhow::Error>  {
    command_sender.send(NotificationServerCommand::CHG(command.clone())).await?;

    let client_data = client_data.clone();


    if local_store.needs_initial_presence {
        local_store.needs_initial_presence = false;

        tokio::spawn( async move {

            let contacts = {
                client_data.get_contact_list().lock().unwrap().get_forward_list()
            };

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

                let _ = command_sender.send(NotificationServerCommand::ILN(IlnServer{
                    tr_id: command.tr_id,
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
        
        
    }



    //notif_sender.send(NotificationServerCommand::SDG())


    Ok(())
}
