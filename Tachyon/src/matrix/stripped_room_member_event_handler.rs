use matrix_sdk::{Client, Room, RoomState};
use matrix_sdk::ruma::events::room::member::{MembershipState, StrippedRoomMemberEvent};
use tokio::sync::broadcast::Sender;
use crate::AB_LOCATOR;
use crate::generated::msnab_datatypes::types::{ArrayOfAnnotation, MemberState, RoleId};
use crate::generated::msnab_sharingservice::factory::{AnnotationFactory, MemberFactory};
use crate::models::abch::events::{AddressBookEvent, AddressBookEventFactory};
use crate::models::notification::msn_client::MSNClient;
use crate::repositories::msn_user_repository::MSNUserRepository;

pub(crate) async fn handle_stripped_room_member_event(ev: StrippedRoomMemberEvent, room: Room, client: Client, msn_client: MSNClient, usr_repo: MSNUserRepository, ab_sender: Sender<AddressBookEvent>) -> bool {
    let mut notify_ab = false;
    let matrix_token = client.access_token().expect("Matrix token to be present");


    let me_matrix_id = msn_client.get_user().get_matrix_id();

    log::info!("AB DEBUG - STRIPPED: state_key: {}, sender: {}, membership: {}", &ev.state_key, &ev.sender, &ev.content.membership);

    if ev.content.is_direct.unwrap_or(false) || room.is_direct().await.unwrap()  {
        log::info!("AB DEBUG - STRIPPED: DIRECT");
        //This is a direct

        match room.state() {
            RoomState::Joined => {
                //StrippedRoomEvents only for Invites
            }
            RoomState::Invited => {
                if &ev.content.membership == &MembershipState::Invite && &ev.state_key == &me_matrix_id {
                    //I've been invited ! ADD TO PENDING LIST WITH INVITE MSG, ADD TO REVERSE LIST

                    let target_usr = usr_repo.get_msnuser(room.room_id(), &ev.sender, false).await.unwrap();
                    let target_uuid = target_usr.get_uuid();
                    let target_msn_addr = target_usr.get_msn_addr();
                    let target_display_name = target_usr.get_display_name();
                    log::info!("AB - I received an invite from: {}", &target_msn_addr);

                    let mut current_pending_member = MemberFactory::get_passport_member(&target_uuid, &target_msn_addr, MemberState::Accepted, RoleId::Pending, false);
                    current_pending_member.display_name = Some(target_display_name);
                    let annotation = AnnotationFactory::get_invite(ev.content.reason.unwrap_or(String::new()));
                    let mut annotations = Vec::new();
                    annotations.push(annotation);
                    current_pending_member.annotations = Some(ArrayOfAnnotation { annotation: annotations });


                    let current_reverse_member = MemberFactory::get_passport_member(&target_uuid, &target_msn_addr, MemberState::Accepted, RoleId::Reverse, false);

                    ab_sender.send(AddressBookEventFactory::get_membership_event(matrix_token.clone(), current_pending_member, RoleId::Pending));
                    ab_sender.send(AddressBookEventFactory::get_membership_event(matrix_token.clone(), current_reverse_member, RoleId::Reverse));
                    notify_ab = true;
                }
            }
            RoomState::Left => {
                //StrippedRoomEvents only for Invites
                log::info!("AB DEBUG 1o1DM - STRIPPED: LEFT ROOM");
            }
        }
    } else {
        //TODO handle non direct invites
        log::info!("AB DEBUG - STRIPPED: NON DIRECT")
    }
    return notify_ab;
}