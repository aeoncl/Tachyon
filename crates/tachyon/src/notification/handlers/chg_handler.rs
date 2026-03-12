use crate::notification::models::client_data::ClientData;
use crate::notification::models::local_client_data::LocalClientData;
use msnp::msnp::notification::command::chg::ChgClient;
use msnp::msnp::notification::command::command::NotificationServerCommand;
use tokio::sync::mpsc::Sender;
use msnp::msnp::notification::command::iln::IlnServer;
use msnp::msnp::notification::command::nln::NlnServer;
use msnp::msnp::notification::command::ubx::{ExtendedPresenceContent, UbxPayload, UbxServer};
use msnp::shared::models::network_id_email::NetworkIdEmail;
use msnp::shared::models::presence_status::PresenceStatus;

pub async fn handle_chg(command: ChgClient, local_store: &mut LocalClientData, client_data: ClientData, command_sender: Sender<NotificationServerCommand>) -> Result<(), anyhow::Error>  {
    command_sender.send(NotificationServerCommand::CHG(command.clone())).await?;

    let notif_sender = command_sender.clone();
    let client_data = client_data.clone();


    if local_store.needs_initial_presence {
        local_store.needs_initial_presence = false;

        tokio::spawn( async move {


            let contacts = {
                client_data.get_contact_list().lock().unwrap().get_forward_list()
            };

            for contact in contacts {

                let network_id_email = NetworkIdEmail {
                    network_id: contact.network_id.clone(),
                    email: contact.email_address.clone(),
                };

                let _ = command_sender.send(NotificationServerCommand::ILN(IlnServer{
                    tr_id: command.tr_id,
                    presence_status: PresenceStatus::NLN,
                    target_user: network_id_email.clone(),
                    via: None,
                    display_name: "".to_string(),
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
