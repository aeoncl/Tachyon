use std::collections::{HashMap, HashSet, VecDeque};
use std::convert::Infallible;
use std::mem;
use log::{debug, error, warn};
use matrix_sdk::{Client, Room};
use matrix_sdk::deserialized_responses::SyncTimelineEvent;
use matrix_sdk::ruma::events::{AnyStrippedStateEvent, AnySyncStateEvent, AnySyncTimelineEvent, OriginalSyncStateEvent};
use matrix_sdk::ruma::events::room::member::{MembershipChange, MembershipState, RoomMemberEventContent, StrippedRoomMemberEvent, SyncRoomMemberEvent};
use matrix_sdk::ruma::{OwnedEventId, OwnedUserId, RoomId, UserId};
use matrix_sdk::ruma::serde::Raw;
use matrix_sdk::sync::{Notification, RoomUpdate, RoomUpdates, SyncResponse};
use tokio::sync::mpsc::Sender;
use msnp::msnp::notification::command::command::NotificationServerCommand;
use msnp::msnp::notification::command::msg::MsgServer;
use msnp::msnp::notification::command::not::factories::NotificationFactory;
use msnp::msnp::notification::command::not::NotServer;
use msnp::shared::models::email_address::EmailAddress;
use msnp::shared::models::endpoint_id::EndpointId;
use msnp::shared::models::msn_user::MsnUser;
use msnp::shared::models::role_list::RoleList;
use msnp::shared::payload::msg::raw_msg_payload::factories::RawMsgPayloadFactory;
use msnp::soap::abch::msnab_datatypes::{Annotation, ArrayOfAnnotation, BaseMember, ContactType, ContactTypeEnum, MemberState, MemberType, RoleId};
use crate::matrix::directs::{get_invite_room_mapping_info, get_joined_room_mapping_info, get_left_room_mapping_info, RoomMappingInfo};
use crate::notification::client_store::ClientData;
use crate::shared::identifiers::MatrixIdCompatible;

pub async fn handle_memberships(client: Client, response: SyncResponse, mut client_data: ClientData, notif_sender: Sender<NotificationServerCommand>) -> Result<(), anyhow::Error> {
    let me = client.user_id().expect("UserID to be here");

    let mut contacts = Vec::new();
    let mut memberships = VecDeque::new();

    for (room_id, update) in response.rooms.join {
        debug!("SYNC|MEMBERSHIPS|JOIN: Handling room: {}: state count: {}", &room_id, update.state.len());

        let room = client.get_room(&room_id).expect("Room to be here");
        let mut dedup: HashSet<String> = HashSet::new();

        for state_event in &update.state {
            let event = state_event.deserialize();
            match event {
                Ok(AnySyncStateEvent::RoomMember(room_member_event)) => {
                    if dedup.get(room_member_event.event_id().as_str()).is_none() {
                        dedup.insert(room_member_event.event_id().to_string());
                    }
                    handle_joined_room_member_event(&room_member_event, &room, me, &client, &mut contacts, &mut memberships).await;
                },
                Ok(other) => {
                    error!("SYNC|MEMBERSHIPS|JOIN: Received non member event : {:?}", other);
                },
                Err(e) => {
                    error!("SYNC|MEMBERSHIPS|JOIN: Couldnt deserialize sync state event: {:?}", e);
                }
            }
        }

        for event in &update.timeline.events {
            match event.event.deserialize() {
                Ok(AnySyncTimelineEvent::State(AnySyncStateEvent::RoomMember(room_member_event))) => {
                    if dedup.get(room_member_event.event_id().as_str()).is_none() {
                        dedup.insert(room_member_event.event_id().to_string());
                        handle_joined_room_member_event(&room_member_event, &room, me, &client, &mut contacts, &mut memberships).await;
                    }
                },
                Ok(other) => {
                    error!("SYNC|MEMBERSHIPS|JOIN: Received non member event : {:?}", other);
                },
                Err(e) => {
                    error!("SYNC|MEMBERSHIPS|JOIN: Couldnt deserialize sync state event: {:?}", e);
                }
            }
        }

    }

    for (room_id, update) in response.rooms.invite {
        debug!("SYNC|MEMBERSHIPS|INVITE: Handling room: {}: state count: {}", &room_id, update.invite_state.events.len());

        for state_event in update.invite_state.events {

            match state_event.deserialize() {
                Ok(AnyStrippedStateEvent::RoomMember(stripped_rm_event)) => {
                    debug!("SYNC|MEMBERSHIPS|INVITE: Stripped RoomMemberEvent Received: {:?}", stripped_rm_event);
                    handle_invite_room_member_event(&stripped_rm_event, &room_id, me, &client, &mut contacts, &mut memberships).await;
                },
                Ok(other) => {
                    error!("SYNC|MEMBERSHIPS|INVITE: Received non member event : {:?}", other);

                }
                Err(e) => {
                    error!("SYNC|MEMBERSHIPS|INVITE: Couldnt deserialize invite room sync state event: {:?}", e);
                },

            }
        }
    }

    for (room_id, update) in response.rooms.leave {

        let room = client.get_room(&room_id).expect("Room to be here");


        debug!("SYNC|MEMBERSHIPS|LEAVE: Handling room: {}: state count: {}", &room_id, update.state.len());

        for state_event in update.state {

            match state_event.deserialize() {
                Ok(AnySyncStateEvent::RoomMember(room_member_event)) => {
                    handle_leave_room_member_event(&room_member_event, &room, me, &client, &mut contacts, &mut memberships).await;
                },
                Ok(other) => {
                    debug!("SYNC|MEMBERSHIPS|LEAVE: Received Non Member Event: {:?}", &other);
                }
                Err(e) => {
                    error!("SYNC|MEMBERSHIPS|LEAVE: Couldnt deserialize SyncStateEvent: {:?}", &e);
                },
            }

        }

    }


    if !contacts.is_empty() || !memberships.is_empty() {
        {
            let mut contacts_mtx = client_data.inner.soap_holder.contacts.lock().unwrap();
            let mut memberships_mtx = client_data.inner.soap_holder.memberships.lock().unwrap();

            if (contacts_mtx.is_empty()) {
                let _ = mem::replace(&mut *contacts_mtx, contacts);
            } else {
                contacts_mtx.append(&mut contacts);
            }

            if memberships_mtx.is_empty() {
                let _ = mem::replace(&mut *memberships_mtx, memberships);
            } else {
                memberships_mtx.append(&mut memberships);
            }
        }


        let me_msn_usr = MsnUser::without_endpoint_guid(EmailAddress::from_user_id(&me));

        //TODO make this less shit later
        notif_sender.send(NotificationServerCommand::NOT(NotServer {
            payload: NotificationFactory::get_abch_updated(&me_msn_usr.uuid, me_msn_usr.get_email_address().as_str()),
        })).await;

    }
    Ok(())
}


async fn handle_joined_room_member_event(event: &SyncRoomMemberEvent, room: &Room, me: &UserId, client: &Client, contacts: &mut Vec<ContactType>, memberships: &mut VecDeque<BaseMember>) -> Result<(), anyhow::Error> {

            match event {
                SyncRoomMemberEvent::Original(og_rm_event) => {

                    debug!("SYNC|MEMBERSHIPS|JOIN: Original SyncRoomMemberEvent Received: {:?}", og_rm_event);

                    let mapping = get_joined_room_mapping_info(&room, me, &og_rm_event, &client).await?;

                    match mapping {
                        RoomMappingInfo::Direct(direct_target) => {
                            debug!("SYNC|MEMBERSHIPS|JOIN: Mapping is Direct({})", &direct_target);

                            let target_msn_user = MsnUser::with_email_addr(EmailAddress::from_user_id(&direct_target));
                            let target_msn_addr = target_msn_user.get_email_address().as_str();

                            if &og_rm_event.state_key == &direct_target {
                                let display_name = og_rm_event.content.displayname.as_ref().map(|d| d.as_str()).unwrap_or(target_msn_user.get_email_address().as_str());

                                match og_rm_event.membership_change() {
                                    MembershipChange::Joined | MembershipChange::InvitationAccepted | MembershipChange::KnockAccepted => {
                                        debug!("SYNC|MEMBERSHIPS|JOIN: Sent Invite was accepted by contact: {}", &target_msn_addr);
                                        //He accepted my invitation, ADD TO REVERSE LIST, CHANGE CONTACT TO NOT PENDING
                                        let invited_contact = ContactType::new(&target_msn_user.uuid, target_msn_addr, &display_name, ContactTypeEnum::Live, false);
                                        let invited_reverse_member = BaseMember::new_passport_member(&target_msn_user.uuid, target_msn_addr, MemberState::Accepted, RoleList::Reverse, false);

                                        contacts.push(invited_contact);
                                        memberships.push_back(invited_reverse_member);
                                    }
                                    MembershipChange::Left | MembershipChange::Banned | MembershipChange::Kicked | MembershipChange::KickedAndBanned => {
                                        debug!("SYNC|MEMBERSHIPS|JOIN: Contact Left the room: {}", &target_msn_addr);
                                        //He left the room, Remove from Reverse List, Set to contact pending
                                        let left_contact = ContactType::new(&target_msn_user.uuid, target_msn_addr, &display_name, ContactTypeEnum::Live, true);
                                        let left_contact_pending = ContactType::new(&target_msn_user.uuid, target_msn_addr, &display_name, ContactTypeEnum::LivePending, false);
                                        let left_reverse_member = BaseMember::new_passport_member(&target_msn_user.uuid, target_msn_addr, MemberState::Accepted, RoleList::Reverse, true);
                                        contacts.push(left_contact);
                                        contacts.push(left_contact_pending);
                                        memberships.push_back(left_reverse_member);
                                    }
                                    MembershipChange::Invited => {
                                        debug!("SYNC|MEMBERSHIPS|JOIN: Invited contact to join room: {}", &target_msn_addr);
                                        //I Invited him, Add to allow list, add to contact pending.
                                        let invited_contact = ContactType::new(&target_msn_user.uuid, target_msn_addr, &display_name, ContactTypeEnum::LivePending, false);
                                        let invited_allow_member = BaseMember::new_passport_member(&target_msn_user.uuid, target_msn_addr, MemberState::Accepted, RoleList::Allow, false);
                                        contacts.push(invited_contact);
                                        memberships.push_back(invited_allow_member);
                                    }
                                    _ => {}
                                }
                            } else if &og_rm_event.state_key == me {
                                match og_rm_event.membership_change() {
                                    MembershipChange::Left => {
                                        log::info!("SYNC|MEMBERSHIPS|JOIN: I Left, delete: {}", &target_msn_user.get_email_address());
                                        //I Left the room, remove member from PENDING_LIST, ALLOW_LIST, REVERSE_LIST. Remove Contact from Contact List
                                        let current_contact = ContactType::new(&target_msn_user.uuid, target_msn_addr, target_msn_addr, ContactTypeEnum::Live, true);
                                        let current_allow_member = BaseMember::new_passport_member(&target_msn_user.uuid, target_msn_addr, MemberState::Accepted, RoleList::Allow, true);
                                        let current_reverse_member = BaseMember::new_passport_member(&target_msn_user.uuid, target_msn_addr, MemberState::Accepted, RoleList::Reverse, true);
                                        let current_pending_member = BaseMember::new_passport_member(&target_msn_user.uuid, target_msn_addr, MemberState::Accepted, RoleList::Pending, true);

                                        contacts.push(current_contact);
                                        memberships.push_back(current_allow_member);
                                        memberships.push_back(current_reverse_member);
                                        memberships.push_back(current_pending_member);
                                    }
                                    MembershipChange::InvitationAccepted | MembershipChange::Joined => {
                                        log::info!("SYNC|MEMBERSHIPS|JOIN: I Accepted an invite from: {}", &target_msn_user.get_email_address());
                                        let inviter_contact = ContactType::new(&target_msn_user.uuid, target_msn_addr, target_msn_addr, ContactTypeEnum::Live, false);
                                        let inviter_allow_member = BaseMember::new_passport_member(&target_msn_user.uuid, target_msn_addr, MemberState::Accepted, RoleList::Allow, false);
                                        let inviter_pending_member = BaseMember::new_passport_member(&target_msn_user.uuid, target_msn_addr, MemberState::Accepted, RoleList::Pending, true);
                                        contacts.push(inviter_contact);
                                        memberships.push_back(inviter_allow_member);
                                        memberships.push_back(inviter_pending_member);
                                    }
                                    _ => {}
                                }
                            } else {
                                warn!("SYNC|MEMBERSHIPS|JOIN: 1o1 dm room event state_key was unexpected: state_key: {}, event_id: {}, room_id: {}", &og_rm_event.state_key, &og_rm_event.state_key, &room.room_id() );
                            }
                        }
                        RoomMappingInfo::Group => {
                            debug!("SYNC|MEMBERSHIPS|JOIN: Mapping is Group");
                        }
                    }
                }
                _ => {
                    debug!("SYNC|MEMBERSHIPS|JOIN: Non Original SyncRoomMemberEvent Received: {:?}", event);

                }
            }

    Ok(())
}

async fn handle_invite_room_member_event(event: &StrippedRoomMemberEvent, room_id: &RoomId, me: &UserId, client: &Client, contacts: &mut Vec<ContactType>, memberships: &mut VecDeque<BaseMember>) -> Result<(), anyhow::Error> {
    match event.content.membership {
        MembershipState::Invite => {

            if event.state_key == me {

                let target_user = event.sender.as_ref();
                let mapping = get_invite_room_mapping_info(&room_id, target_user, &event, &client).await?;

                match mapping {
                    RoomMappingInfo::Direct(direct_target) => {
                        debug!("SYNC|MEMBERSHIPS|INVITE: Mapping is Direct({})", &direct_target);

                        //I've been invited ! ADD TO PENDING LIST WITH INVITE MSG, ADD TO REVERSE LIST

                        let target_msn_user = MsnUser::with_email_addr(EmailAddress::from_user_id(&direct_target));
                        let target_msn_addr = target_msn_user.get_email_address().as_str();

                        log::info!("SYNC|MEMBERSHIPS|INVITE: I received Direct invite from: {}", &target_msn_addr);

                        let mut current_pending_member = BaseMember::new_passport_member(&target_msn_user.uuid, &target_msn_addr, MemberState::Accepted, RoleList::Pending, false);
                        current_pending_member.display_name = Some(target_msn_addr.to_string());
                        let annotation = Annotation::new_invite(event.content.reason.as_ref().map(|e| e.as_str()).unwrap_or(""));
                        let mut annotations = Vec::new();
                        annotations.push(annotation);
                        current_pending_member.annotations = Some(ArrayOfAnnotation { annotation: annotations });


                        let current_reverse_member = BaseMember::new_passport_member(&target_msn_user.uuid, &target_msn_addr, MemberState::Accepted, RoleList::Reverse, false);
                        memberships.push_back(current_pending_member);
                        memberships.push_back(current_reverse_member);

                    }
                    RoomMappingInfo::Group => {
                        debug!("SYNC|MEMBERSHIPS|INVITE: Mapping is Group");
                        log::info!("SYNC|MEMBERSHIPS|INVITE: I received a Group invite from: {}", &target_user);
                    }
                }

            }
        }
        _ => {}
    }

    Ok(())

}

async fn handle_leave_room_member_event(event: &SyncRoomMemberEvent, room: &Room, me: &UserId, client: &Client, contacts: &mut Vec<ContactType>, memberships: &mut VecDeque<BaseMember>) -> Result<(), anyhow::Error> {
    match event {
        SyncRoomMemberEvent::Original(og_rm_event) => {
            debug!("SYNC|MEMBERSHIPS|LEAVE: Original SyncRoomMemberEvent Received: {:?}", og_rm_event);

            //TODO circle before this.
            // let mapping = get_left_room_mapping_info(&room, me, &client).await?;
            // match mapping {
            //     RoomMappingInfo::Direct(direct_target) => {
            //
            //     }
            //     RoomMappingInfo::Group => {
            //
            //     }
            // }


            }
        _ => {
            debug!("SYNC|MEMBERSHIPS|LEAVE: Non Original SyncRoomMemberEvent Received: {:?}", event);
        }
    }

    Ok(())


}


