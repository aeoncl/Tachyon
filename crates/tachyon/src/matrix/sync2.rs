use core::sync;

use futures::StreamExt;
use crate::notification::client_store::ClientData;
use matrix_sdk::{Client, Error, SlidingSyncListBuilder};
use matrix_sdk::ruma::api::client::sync::sync_events::v5::request::{ListFilters, RoomSubscription};
use matrix_sdk::ruma::events::direct::DirectEventContent;
use matrix_sdk::ruma::events::{GlobalAccountDataEventType, StateEventType};
use matrix_sdk::ruma::{assign, OwnedRoomId, RoomId, UInt};
use matrix_sdk_ui::sync_service::{self, SyncService};
use matrix_sdk_ui::timeline::RoomExt;
use msnp::msnp::notification::command::command::NotificationServerCommand;
use tokio::sync::mpsc::Sender;
use msnp::msnp::notification::command::iln::IlnServer;
use msnp::msnp::notification::command::not::NotServer;
use crate::matrix::events::room_mappings::{RoomMappingsEvent, RoomMappingsEventContent};

#[derive(Clone)]
pub struct TachyonContext {
    notif_sender: Sender<NotificationServerCommand>,
    client_data: ClientData
}

const DM_REQUIRED_STATE: &[(StateEventType, &str)] = &[
    (StateEventType::RoomName, ""),
    (StateEventType::RoomEncryption, ""),
    (StateEventType::RoomMember, "*"),
    (StateEventType::RoomTopic, ""),
    (StateEventType::RoomCanonicalAlias, ""),
    (StateEventType::RoomPowerLevels, ""),
    (StateEventType::CallMember, "*"),
    (StateEventType::RoomJoinRules, ""),
    // Those two events are required to properly compute room previews.
    (StateEventType::RoomCreate, ""),
    (StateEventType::RoomHistoryVisibility, ""),
    // Required to correctly calculate the room display name.
    (StateEventType::MemberHints, ""),
];

async fn get_mandatory_rooms_for_initial_sync<'a>(room_mappings: &'a Option<Result<RoomMappingsEventContent, serde_json::Error>>, directs: &'a Option<Result<DirectEventContent, serde_json::Error>>) -> Result<Vec<&'a RoomId>, Error> {

    if let Some(room_mappings) = room_mappings {
        match room_mappings {
            Ok(room_mappings) => {
                let room_ids = room_mappings.get_room_ids();
                return Ok(room_ids);
            }
            Err(e) => {
                log::error!("Malformed room mappings received, ignoring. {}", e);
            }
        }
    }

    if let Some(directs) = directs {
        match directs {
            Ok(directs) => {
                return Ok(directs.values().flatten().map(|room_id| room_id.as_ref()).collect());            }
            Err(e) => {
                log::error!("Malformed directs received, ignoring. {}", e);
            }
        }
    }

    Ok(Vec::new())
}

pub async fn sliding_sync(tr_id: u128, client_data: &ClientData) -> Result<(Vec<IlnServer>, Vec<NotServer>), anyhow::Error>{

    let client = client_data.get_matrix_client();

    let sync_service = client_data.get_sync_service();

    let test1 = client.account()
        .fetch_account_data(GlobalAccountDataEventType::from("com.tachyon.room.mappings")).await?
        .map(|raw| raw.deserialize_as::<RoomMappingsEventContent>());

    let test = client.account().fetch_account_data(GlobalAccountDataEventType::Direct).await?.map(|raw| raw.deserialize_as::<DirectEventContent>());

    let rooms_to_watch = get_mandatory_rooms_for_initial_sync(&test1, &test).await?;

    let subscription = assign!(RoomSubscription::default(), {
        required_state: DM_REQUIRED_STATE.iter().map(|(key, val)| (key.to_owned(), val.to_string())).collect(),
        timeline_limit: UInt::new_wrapping(10),
        include_heroes: Some(true),
            });

    let room_refs: Vec<&RoomId> = rooms_to_watch.iter().map(|r| r.as_ref()).collect();

    sync_service.room_list_service().sliding_sync().subscribe_to_rooms(&room_refs, Some(subscription), false);

    sync_service.start().await;
    
    Ok((Vec::new(), Vec::new()))

}



