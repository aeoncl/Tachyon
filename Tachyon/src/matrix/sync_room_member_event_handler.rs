use std::sync::Arc;
use log::{info, warn};
use matrix_sdk::{Client, Room, RoomMemberships, RoomState};
use matrix_sdk::room::RoomMember;
use matrix_sdk::ruma::__private_macros::mxc_uri;
use matrix_sdk::ruma::events::{GlobalAccountDataEvent, GlobalAccountDataEventType, OriginalSyncStateEvent};
use matrix_sdk::ruma::events::direct::DirectEventContent;
use matrix_sdk::ruma::events::room::member::{MembershipChange, MembershipState, RoomMemberEventContent, SyncRoomMemberEvent};
use matrix_sdk::ruma::{owned_mxc_uri, OwnedMxcUri, OwnedUserId};
use tokio::sync::broadcast::Sender;
use crate::generated::msnab_datatypes::types::{ContactTypeEnum, MemberState, RoleId};
use crate::generated::msnab_sharingservice::factories::{ContactFactory, MemberFactory};
use crate::matrix::direct_target_resolver::resolve_direct_target;
use crate::matrix::loop_bootstrap::DedupGrimoire;
use crate::models::abch::events::{AddressBookEvent, AddressBookEventFactory};
use crate::models::msn_object::MSNObject;
use crate::models::msn_user::MSNUser;
use crate::models::notification::msn_client::MSNClient;
use crate::repositories::msn_user_repository::MSNUserRepository;
use crate::utils::identifiers::{compute_sha1, matrix_mxc_id_to_annoying_matrix_mxc_id};
use crate::utils::string::decode_base64;


fn maybe_msn_obj_to_maybe_mxc_id(maybe: Option<MSNObject>) -> Option<OwnedMxcUri> {
    if maybe.is_none() {
        return None;
    }

    let location = &maybe.unwrap().location;
    if location.as_str() == "0" {
        return None;
    }

    let me_mxc = matrix_mxc_id_to_annoying_matrix_mxc_id(&decode_base64(location).unwrap());
    return Some(me_mxc);
}

pub async fn handle_sync_room_member_event(ev: SyncRoomMemberEvent, room: Room, client: Client, mut msn_client: MSNClient, ab_sender: Sender<AddressBookEvent>, user_repo: MSNUserRepository, mut dedup: DedupGrimoire) {

    let my_user_id = client.user_id().unwrap();
    let mut me = msn_client.get_user();

    let mut notify_ab = false;
    if let SyncRoomMemberEvent::Original(ev) = &ev {
        info!("ABDEBUG: Original Event !!");

        if ev.sender == my_user_id.to_owned() {
            info!("ABDEBUG: I Changed!");
                if let MembershipChange::ProfileChanged { displayname_change, avatar_url_change } = ev.membership_change() {
                    if let Some(displayname_change) = displayname_change {
                        let new_display_name = displayname_change.new.unwrap_or("").to_string();
                        if dedup.get_display_name() != new_display_name {
                            dedup.set_display_name(new_display_name.clone());
                            notify_ab = true;
                        }
                    }

                    if let Some(avatar_url_change) = avatar_url_change{
                        match avatar_url_change.new {
                            Some(new_avatar_url) => {
                               if dedup.get_display_picture() != new_avatar_url.to_string() {
                                   dedup.set_display_picture(new_avatar_url.to_string());
                                   notify_ab = true;
                               }
                            },
                            None => {
                                if !dedup.get_display_picture().is_empty() {
                                    dedup.set_display_picture(String::new());
                                    notify_ab = true;
                                }
                            }
                        }
                    }
            }
            if notify_ab {
                ab_sender.send(AddressBookEvent::ExpressionProfileUpdateEvent);
            }
        }

        if room.is_direct().await.unwrap_or(false) || ev.content.is_direct.unwrap_or(false) {
            info!("Room is direct !!");
            notify_ab = notify_ab || handle_directs(ev, &room, &client, &client.access_token().unwrap(), &me.get_msn_addr(), ab_sender).await;
        } else {
            info!("Room is not a direct !!");
            //TODO
        }
    } else if let SyncRoomMemberEvent::Redacted(ev) = &ev {
        info!("ABDEBUG: Redacted event !!");
    }

    if notify_ab {
        msn_client.on_notify_ab_update();
    }
}

async fn handle_directs(ev: &OriginalSyncStateEvent<RoomMemberEventContent>, room: &Room, client: &Client, mtx_token: &String, me_msn_addr: &String, ab_sender: Sender<AddressBookEvent>) -> bool {
    let joined_members = room.members(RoomMemberships::JOIN).await.unwrap_or(Vec::new());

    let mut notify_ab = false;
    if joined_members.len() <= 2 {
        //1O1 DM Room
        info!("Room is One on One Direct !!");
        notify_ab = notify_ab || handle_1v1_dm2(ev, room, client, mtx_token, me_msn_addr, joined_members, ab_sender).await;
    } else {
        info!("Room is Group DM!!");
        //Group DMs
    }

    return notify_ab;
}

async fn handle_1v1_dm2(ev: &OriginalSyncStateEvent<RoomMemberEventContent>, room: &Room, client: &Client, mtx_token: &String, msn_addr: &String, joined_members: Vec<RoomMember>, ab_sender: Sender<AddressBookEvent>) -> bool {
    let mut notify_ab = false;
    let matrix_token = client.access_token().unwrap();
    let me = client.user_id().unwrap().to_owned();

    //TODO Fix this unwrap
    let target = resolve_direct_target(&room.direct_targets(), &room, &me, &client).await.unwrap();
    let target_usr = MSNUser::from_matrix_id(target.clone());
    let target_uuid = target_usr.get_uuid();
    let target_msn_addr = target_usr.get_msn_addr();

    log::info!("AB DEBUG - State_key: {}, sender: {}, membership: {}", &ev.state_key, &ev.sender, &ev.content.membership);

    match room.state() {
        RoomState::Joined => {
            if &ev.state_key == &target {
                let display_name = ev.content.displayname.as_ref().unwrap_or(&target_msn_addr).to_owned();
                if ev.content.membership == MembershipState::Invite && &ev.sender == &me {
                    //I Invited him, Add to allow list, add to contact pending.
                    log::info!("AB - Send invitation to: {}", &target_msn_addr);
                    let invited_contact = ContactFactory::get_contact(&target_uuid, &target_msn_addr, &display_name, ContactTypeEnum::LivePending, false);
                    let invited_allow_member = MemberFactory::get_passport_member(&target_uuid, &target_msn_addr, MemberState::Accepted, RoleId::Allow, false);
                    ab_sender.send(AddressBookEventFactory::get_contact_event(matrix_token.clone(), invited_contact));
                    ab_sender.send(AddressBookEventFactory::get_membership_event(matrix_token.clone(), invited_allow_member, RoleId::Allow));
                    notify_ab = true;
                } else if ev.content.membership == MembershipState::Join {
                    //He accepted my invitation, ADD TO REVERSE LIST, CHANGE CONTACT TO NOT PENDING
                    log::info!("AB - Invitation Accepted from: {}", &target_msn_addr);
                    let invited_contact = ContactFactory::get_contact(&target_uuid, &target_msn_addr, &display_name, ContactTypeEnum::Live, false);
                    let invited_reverse_member = MemberFactory::get_passport_member(&target_uuid, &target_msn_addr, MemberState::Accepted, RoleId::Reverse, false);
                    ab_sender.send(AddressBookEventFactory::get_contact_event(matrix_token.clone(), invited_contact));
                    ab_sender.send(AddressBookEventFactory::get_membership_event(matrix_token.clone(), invited_reverse_member, RoleId::Reverse));
                    notify_ab = true;
                } else if ev.content.membership == MembershipState::Leave || ev.content.membership == MembershipState::Ban {
                    log::info!("AB - Contact left: {}", &target_msn_addr);
                    //He left the room, Remove from Reverse List, Set to contact pending
                    let left_contact = ContactFactory::get_contact(&target_uuid, &target_msn_addr, &display_name, ContactTypeEnum::LivePending, false);
                    let left_reverse_member = MemberFactory::get_passport_member(&target_uuid, &target_msn_addr, MemberState::Accepted, RoleId::Reverse, true);
                    ab_sender.send(AddressBookEventFactory::get_contact_event(matrix_token.clone(), left_contact));
                    ab_sender.send(AddressBookEventFactory::get_membership_event(matrix_token.clone(), left_reverse_member, RoleId::Reverse));
                    notify_ab = true;
                }
            } else if &ev.state_key == &me {
                if &ev.content.membership == &MembershipState::Invite && &ev.sender == &target {
                    //This is strange, i should get a join membership when i accept an invite, but i get an invite
                    //I Accepted his invitation, REMOVE FROM PENDING LIST, ADD TO ALLOW LIST, ADD TO CONTACT LIST
                    log::info!("AB - I Accepted an invite from: {}", &target_msn_addr);

                    let inviter_contact = ContactFactory::get_contact(&target_uuid, &target_msn_addr, &target_msn_addr, ContactTypeEnum::Live, false);
                    let inviter_allow_member = MemberFactory::get_passport_member(&target_uuid, &target_msn_addr, MemberState::Accepted, RoleId::Allow, false);
                    let inviter_pending_member = MemberFactory::get_passport_member(&target_uuid, &target_msn_addr, MemberState::Accepted, RoleId::Pending, true);
                    ab_sender.send(AddressBookEventFactory::get_contact_event(matrix_token.clone(), inviter_contact));
                    ab_sender.send(AddressBookEventFactory::get_membership_event(matrix_token.clone(), inviter_allow_member, RoleId::Allow));
                    ab_sender.send(AddressBookEventFactory::get_membership_event(matrix_token.clone(), inviter_pending_member, RoleId::Pending));
                    notify_ab = true;
                }
            }
        }
        RoomState::Left => {
            if should_i_really_delete_contact(&client, target.clone()).await {
                log::info!("AB - I Deleted: {}", &target_msn_addr);
                //I Left the room, remove member from PENDING_LIST, ALLOW_LIST, REVERSE_LIST. Remove Contact from Contact List
                let current_contact = ContactFactory::get_contact(&target_uuid, &target_msn_addr, &target_msn_addr, ContactTypeEnum::Live, true);
                let current_allow_member = MemberFactory::get_passport_member(&target_uuid, &target_msn_addr, MemberState::Accepted, RoleId::Allow, true);
                let current_reverse_member = MemberFactory::get_passport_member(&target_uuid, &target_msn_addr, MemberState::Accepted, RoleId::Reverse, true);
                let current_pending_member = MemberFactory::get_passport_member(&target_uuid, &target_msn_addr, MemberState::Accepted, RoleId::Pending, true);

                ab_sender.send(AddressBookEventFactory::get_contact_event(matrix_token.clone(), current_contact));
                ab_sender.send(AddressBookEventFactory::get_membership_event(matrix_token.clone(), current_allow_member, RoleId::Allow));
                ab_sender.send(AddressBookEventFactory::get_membership_event(matrix_token.clone(), current_reverse_member, RoleId::Reverse));
                ab_sender.send(AddressBookEventFactory::get_membership_event(matrix_token.clone(), current_pending_member, RoleId::Pending));
                notify_ab = true;
            }
        }
        _ => {}
    }

    return notify_ab;
}

async fn handle_group_dm(ev: &SyncRoomMemberEvent, room: &Room, client: &Client, mtx_token: &String, msn_addr: &String) {}

async fn should_i_really_delete_contact(client: &Client, contact: OwnedUserId) -> bool {
    let directs = client.store().get_account_data_event(GlobalAccountDataEventType::Direct).await.unwrap().unwrap(); //fix this

    let directs_parsed: GlobalAccountDataEvent<DirectEventContent> = directs.deserialize_as().unwrap();

    let content = directs_parsed.content.0;

    for current in content {
        if current.0 == contact {
            let dm_rooms = current.1;
            for dm_room in dm_rooms {
                //For each dm room for a contact
                //if let Some(member_event) = client.store().get_member_event(&dm_room, &contact).await.unwrap() {
                if let Some(joined_room) = client.get_room(&dm_room) {
                    //If we are still in the room
                    // if member_event.content.membership == MembershipState::Invite || member_event.content.membership == MembershipState::Join {
                    //If the contact is still in the room, don't delete it from the contact list.
                    return false;

                    //};
                }

                // }
            }
            break;
        }
    }
    return true;
}
