use std::{time::Duration, sync::Arc};

use chashmap::ReadGuard;
use matrix_sdk::{deserialized_responses::SyncResponse, config::SyncSettings, Client, ruma::{OwnedUserId, events::room::{member::{MembershipState, RoomMemberEventContent, RoomMemberEvent, SyncRoomMemberEvent}, message::SyncRoomMessageEvent}}, RoomMember, room::Room};
use tokio::{join, sync::broadcast::Sender};

use crate::{CLIENT_DATA_REPO, MATRIX_CLIENT_REPO, repositories::{matrix_client_repository::MatrixClientRepository, client_data_repository::ClientDataRepository, repository::Repository}, generated::{msnab_sharingservice::factories::{ContactFactory, MemberFactory}, msnab_datatypes::types::{MemberState, RoleId}}, models::uuid::UUID};

use super::identifiers::matrix_id_to_msn_addr;


pub async fn start_matrix_loop(token: String, sender: Sender<String>) {

    let mut settings = SyncSettings::new().timeout(Duration::from_secs(10));
    let matrix_client_repo : Arc<MatrixClientRepository> = MATRIX_CLIENT_REPO.clone();
    let matrix_client = matrix_client_repo.find(&token).unwrap().clone();
    matrix_client.register_event_handler( |ev: SyncRoomMemberEvent, room: Room, client: Client| async move {
        let me = client.user_id().await.unwrap();

        if let SyncRoomMemberEvent::Original(ev) = ev {
            
            let joined_members = room.joined_members().await.unwrap_or(Vec::new());
            if room.is_direct() && joined_members.len() > 0 && joined_members.len() <= 2 {

                if let Room::Joined(room) = room {
                    for target in room.direct_targets() {
                        if ev.sender == target {
                            // C'est mon poto en reverse
                            let uuid = UUID::from_string(&target.to_string());
                            let msn_addr = matrix_id_to_msn_addr(&target.to_string());
                            let current_contact = ContactFactory::get_contact(&uuid, &msn_addr);
                            let current_allow_member = MemberFactory::get_passport_member(&uuid, &msn_addr, MemberState::Accepted, RoleId::Allow, false);
                        } else if ev.sender == me && ev.content.membership == MembershipState::Invite {
                               // C'est mon poto pending seulement dans l'allowlist
                               let uuid = UUID::from_string(&target.to_string());
                               let msn_addr = matrix_id_to_msn_addr(&target.to_string());
                               let current_contact = ContactFactory::get_contact(&uuid, &msn_addr);
                               let current_allow_member = MemberFactory::get_passport_member(&uuid, &msn_addr, MemberState::Accepted, RoleId::Allow, false);
                        }
                    }
                } else if let Room::Invited(room) = room {
                
                    for target in room.direct_targets() {
                        //if let Ok(invite_target) = ev.state_key.try_into()::<OwnedUserId>;
                    
                        //if ev.sender() == target && ev.content.membership == MembershipState::Invite && e ==  {
                            // C'est mon poto
                        //}
                    }
                
                }
            }

        }
       
    }).await;

    loop {

        {
            let client_data_repo : Arc<ClientDataRepository> = CLIENT_DATA_REPO.clone();
            let client_data = client_data_repo.find(&token).unwrap();
            settings = settings.set_presence(client_data.presence_status.clone().into());
        }


     
        let result = matrix_client.sync_once(settings.clone()).await.unwrap();

        settings = settings.token(result.next_batch);

        //let presence_update = presence_update_task(result.clone(), sender.clone(), matrix_client.clone());
        //let contact_update = contact_update_task(result.clone(), sender.clone(), matrix_client.clone());

        //let _test = join!(presence_update, contact_update);
    }
}

// pub async fn presence_update_task(response : SyncResponse, sender: Sender<String>, matrix_client: ReadGuard<'_, String, Client>) {
    
// }

// pub async fn contact_update_task(response : SyncResponse, sender: Sender<String>, matrix_client: ReadGuard<'_, String, Client>) {
    
//     let me : OwnedUserId = matrix_client.user_id().await.unwrap();

    
// matrix_client.register_event_handler(handler)
//     for (room_id, _room) in &response.rooms.join {
    

//        let current = matrix_client.get_joined_room(room_id).unwrap();
//        if current.is_direct() { 
//            //Direct room
//            let members = current.joined_members().await.unwrap_or(Vec::new());

//            if members.len() > 0 && members.len() <= 2 {
//             for direct_target in current.direct_targets() {
//                 let direct_target_member = current.get_member(&direct_target).await.unwrap().unwrap();
//                 _room.state.events;
//                 match direct_target_member.membership() {
//                     MembershipState::Join => {
//                       
//                     MembershipState::Ban | MembershipState::Leave => {

//                     },
//                     MembershipState::Leave => {

//                     }
//                     _ => {

//                     }
//                 }
               

//             }

//         }
//        }
//     }

//     // make data availlable for FindContactsPaged & FindMembership
// }