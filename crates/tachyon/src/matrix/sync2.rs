use crate::notification::client_store::ClientData;
use matrix_sdk::Client;
use matrix_sdk::ruma::events::GlobalAccountDataEventType;
use matrix_sdk_ui::sync_service::SyncService;
use matrix_sdk_ui::timeline::RoomExt;
use msnp::msnp::notification::command::command::NotificationServerCommand;
use tokio::sync::mpsc::Sender;
use msnp::msnp::notification::command::iln::IlnServer;
use msnp::msnp::notification::command::not::NotServer;
use crate::matrix::events::room_mappings::{RoomMappingsEvent, RoomMappingsEventContent};

#[derive(Clone)]
struct TachyonContext {
    notif_sender: Sender<NotificationServerCommand>,
    client_data: ClientData
}


pub async fn sliding_sync(tr_id: u128, client_data: &ClientData) -> Result<(Vec<IlnServer>, Vec<NotServer>), anyhow::Error>{

    let client = client_data.get_matrix_client();

    //let event = client.account().fetch_account_data(GlobalAccountDataEventType::from("com.tachyon.room.mappings")).await.unwrap().unwrap().deserialize_as::<RoomMappingsEventContent>().unwrap();
    
    
    let sync_service = SyncService::builder(client.clone()).build().await?;

    
    start_room_update_handlers_task(client.clone());

    sync_service.start().await;

    Ok((Vec::new(), Vec::new()))

}




pub fn start_room_update_handlers_task(matrix_client: Client) {
    tokio::spawn(async move {
        loop {
            let updates = matrix_client.subscribe_to_all_room_updates().recv().await;
            match updates {
                Ok(room_update) => {

                   let (id, room) = room_update.join.first_key_value().unwrap();
                    matrix_client.get_room(id).unwrap();


                }
                Err(_) => {
                    todo!("handle room updates recv error")
                }
            }

        }

    });
}
