use std::collections::{BTreeMap, HashMap, HashSet, VecDeque};
use std::convert::Infallible;
use std::mem;
use dashmap::mapref::one::RefMut;
use log::{debug, error, warn};
use matrix_sdk::{Client, Room, RoomMemberships};
use matrix_sdk::deserialized_responses::SyncTimelineEvent;
use matrix_sdk::room::RoomMember;
use matrix_sdk::ruma::events::{AnyGlobalAccountDataEvent, AnyStrippedStateEvent, AnySyncStateEvent, AnySyncTimelineEvent, OriginalSyncStateEvent};
use matrix_sdk::ruma::events::room::member::{MembershipChange, MembershipDetails, MembershipState, RoomMemberEventContent, StrippedRoomMemberEvent, SyncRoomMemberEvent};
use matrix_sdk::ruma::{OwnedEventId, OwnedRoomId, OwnedUserId, RoomId, UserId};
use matrix_sdk::ruma::events::GlobalAccountDataEventType::IgnoredUserList;
use matrix_sdk::ruma::events::ignored_user_list::IgnoredUserListEvent;
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
use msnp::shared::models::uuid::Uuid;
use msnp::shared::payload::msg::raw_msg_payload::factories::RawMsgPayloadFactory;
use msnp::soap::abch::ab_service::ab_find_contacts_paged::response::CircleData;
use msnp::soap::abch::msnab_datatypes::{Annotation, ArrayOfAnnotation, BaseMember, ContactType, ContactTypeEnum, MemberState, MemberType, CircleRelationshipRole, RelationshipState, RoleId, NetworkInfoType, CircleInverseInfoType};
use crate::matrix::directs::{force_update_rooms_with_fresh_m_direct, get_invite_room_mapping_info, get_joined_room_mapping_info, get_left_room_mapping_info, RoomMappingInfo};
use crate::notification::client_store::{ClientData, Contact};
use crate::shared::identifiers::MatrixIdCompatible;


pub async fn handle_memberships(client: Client, response: SyncResponse) -> Result<(Vec<Contact>, VecDeque<BaseMember>, HashMap<String, Vec<ContactType>>), anyhow::Error> {
    debug!("---Handle Memberships---");
    let me = client.user_id().expect("UserID to be here");

    let mut contacts = Vec::new();
    let mut memberships = VecDeque::new();
    let mut circle_members: HashMap<String, Vec<ContactType>> = HashMap::new();

    // for account_data_event in response.account_data {
    //     match account_data_event.deserialize() {
    //         Ok(AnyGlobalAccountDataEvent::Direct(direct_event)) => {
    //
    //             debug!("NEW ACCOUNT DATA EVENT");
    //
    //             let fw : HashSet<OwnedUserId> = client_data.inner.contact_list.lock().unwrap().get_forward_list().iter().map(|u| u.get_email_address().to_owned_user_id()).collect();
    //
    //             let ignore_list = client.store().get_account_data_event(IgnoredUserList).await.map(|e| {
    //                 match e {
    //                     None => { BTreeMap::new() }
    //                     Some(raw_ev) => {
    //                         let ev = raw_ev.deserialize_as::<IgnoredUserListEvent>().expect("to be a valid event");
    //                         ev.content.ignored_users
    //                     }
    //                 }
    //             }).unwrap_or(BTreeMap::new());
    //
    //             for (user_id, dm_rooms) in direct_event.content.0 {
    //                 if !fw.contains(&user_id) && !ignore_list.contains_key(&user_id) {
    //                     //new contact to add hehe
    //
    //                     let target_msn_user = MsnUser::with_email_addr(EmailAddress::from_user_id(&user_id));
    //                     let target_msn_addr = target_msn_user.get_email_address().as_str();
    //
    //                     debug!("NEW CONTACT: {}", target_msn_addr);
    //                     let inviter_contact = ContactType::new(&target_msn_user.uuid, target_msn_addr, target_msn_addr, ContactTypeEnum::Live, false);
    //                     let inviter_allow_member = BaseMember::new_passport_member(&target_msn_user.uuid, target_msn_addr, MemberState::Accepted, RoleList::Allow, false);
    //                     let inviter_pending_member = BaseMember::new_passport_member(&target_msn_user.uuid, target_msn_addr, MemberState::Accepted, RoleList::Pending, true);
    //                     contacts.push(inviter_contact);
    //                     memberships.push_back(inviter_allow_member);
    //                     memberships.push_back(inviter_pending_member);
    //                 }
    //             }
    //
    //         }
    //         Err(_) => {}
    //         _ => {}
    //     }
    //
    //
    // }

    for (room_id, update) in response.rooms.join {
        debug!("SYNC|MEMBERSHIPS|JOIN: Handling room: {}: state count: {}", &room_id, update.state.len());

        let room = client.get_room(&room_id).expect("Room to be here");
        let mut dedup: HashSet<String> = HashSet::new();

        for event in update.timeline.events.iter().rev() {
            match event.event.deserialize() {
                Ok(AnySyncTimelineEvent::State(AnySyncStateEvent::RoomMember(room_member_event))) => {
                    if dedup.get(room_member_event.event_id().as_str()).is_none() {
                        dedup.insert(room_member_event.event_id().to_string());
                        handle_joined_room_member_event(&room_member_event, &room, me, &client, &mut contacts, &mut memberships, &mut circle_members).await?;
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

        if (update.timeline.limited) {

            for state_event in &update.state {
                let event = state_event.deserialize();
                match event {
                    Ok(AnySyncStateEvent::RoomMember(room_member_event)) => {
                        if dedup.get(room_member_event.event_id().as_str()).is_none() {
                            dedup.insert(room_member_event.event_id().to_string());
                        }
                        handle_joined_room_member_event(&room_member_event, &room, me, &client, &mut contacts, &mut memberships, &mut circle_members).await?;
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
    }

    for (room_id, update) in response.rooms.invite {
        debug!("SYNC|MEMBERSHIPS|INVITE: Handling room: {}: state count: {}", &room_id, update.invite_state.events.len());

        for state_event in update.invite_state.events {

            match state_event.deserialize() {
                Ok(AnyStrippedStateEvent::RoomMember(stripped_rm_event)) => {
                    debug!("SYNC|MEMBERSHIPS|INVITE: Stripped RoomMemberEvent Received: {:?}", stripped_rm_event);
                    handle_invite_room_member_event(&stripped_rm_event, &room_id, me, &client, &mut contacts, &mut memberships).await?;
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
                    handle_leave_room_member_event(&room_member_event, &room, me, &client, &mut contacts, &mut memberships).await?;
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

    Ok((contacts, memberships, circle_members))
}

pub async fn handle_joined_room_member_event(event: &SyncRoomMemberEvent, room: &Room, me: &UserId, client: &Client, contacts: &mut Vec<Contact>, memberships: &mut VecDeque<BaseMember>, circle_members: &mut HashMap<String, Vec<ContactType>>) -> Result<(), anyhow::Error> {

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

                                        contacts.push(Contact::Contact(invited_contact));
                                        memberships.push_back(invited_reverse_member);
                                    }
                                    MembershipChange::Left | MembershipChange::Banned | MembershipChange::Kicked | MembershipChange::KickedAndBanned => {
                                        debug!("SYNC|MEMBERSHIPS|JOIN: Contact Left the room: {}", &target_msn_addr);
                                        //He left the room, Remove from Reverse List, Set to contact pending
                                        let left_contact = ContactType::new(&target_msn_user.uuid, target_msn_addr, &display_name, ContactTypeEnum::Live, true);
                                        let left_contact_pending = ContactType::new(&target_msn_user.uuid, target_msn_addr, &display_name, ContactTypeEnum::LivePending, false);
                                        let left_reverse_member = BaseMember::new_passport_member(&target_msn_user.uuid, target_msn_addr, MemberState::Accepted, RoleList::Reverse, true);
                                        contacts.push(Contact::Contact(left_contact));
                                        contacts.push(Contact::Contact(left_contact_pending));
                                        memberships.push_back(left_reverse_member);
                                    }
                                    MembershipChange::Invited => {
                                        debug!("SYNC|MEMBERSHIPS|JOIN: Invited contact to join room: {}", &target_msn_addr);
                                        //I Invited him, Add to allow list, add to contact pending.
                                        let invited_contact = ContactType::new(&target_msn_user.uuid, target_msn_addr, &display_name, ContactTypeEnum::LivePending, false);
                                        let invited_allow_member = BaseMember::new_passport_member(&target_msn_user.uuid, target_msn_addr, MemberState::Accepted, RoleList::Allow, false);
                                        contacts.push(Contact::Contact(invited_contact));
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

                                        contacts.push(Contact::Contact(current_contact));
                                        memberships.push_back(current_allow_member);
                                        memberships.push_back(current_reverse_member);
                                        memberships.push_back(current_pending_member);
                                    }
                                    MembershipChange::InvitationAccepted | MembershipChange::Joined => {
                                        log::info!("SYNC|MEMBERSHIPS|JOIN: I Accepted an invite from: {} Do Nothing", &target_msn_user.get_email_address());
                                        let inviter_contact = ContactType::new(&target_msn_user.uuid, target_msn_addr, target_msn_addr, ContactTypeEnum::Live, false);
                                        let inviter_allow_member = BaseMember::new_passport_member(&target_msn_user.uuid, target_msn_addr, MemberState::Accepted, RoleList::Allow, false);
                                        let inviter_pending_member = BaseMember::new_passport_member(&target_msn_user.uuid, target_msn_addr, MemberState::Accepted, RoleList::Pending, true);
                                        contacts.push(Contact::Contact(inviter_contact));
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
                            let room_id = room.room_id();
                            let circle_uuid = Uuid::from_seed(room_id.as_str());

                            if &og_rm_event.state_key == me {
                                // Me action
                                match og_rm_event.membership_change() {
                                    MembershipChange::Left => {
                                        //I Left the Circle
                                        let mut circle = ContactType::new_circle(room_id.as_str(), &room.computed_display_name().await.unwrap().to_string(), true, RelationshipState::Accepted, CircleRelationshipRole::None);
                                        let inverse_info = CircleInverseInfoType::new(circle.contact_id.clone().expect("to be here"), room.computed_display_name().await.expect("").to_string(), true, CircleRelationshipRole::Member, RelationshipState::Accepted);
                                        contacts.push(Contact::Circle(CircleData {
                                            contact: circle,
                                            inverse_info,
                                        }));

                                    }
                                    MembershipChange::InvitationAccepted | MembershipChange::Joined => {
                                        //I accepted an invite to a circle

                                        let mut circle = ContactType::new_circle(room_id.as_str(), &room.computed_display_name().await.unwrap().to_string(), false, RelationshipState::Accepted, CircleRelationshipRole::None);
                                        let inverse_info = CircleInverseInfoType::new(circle.contact_id.clone().expect("to be here"), room.computed_display_name().await.expect("").to_string(), false, CircleRelationshipRole::Member, RelationshipState::Accepted);

                                        contacts.push(Contact::Circle(CircleData {
                                            contact: circle,
                                            inverse_info,
                                        }));

                                        let circle_members = {
                                            match circle_members.get_mut(&circle_uuid.to_string()) {
                                                None => {
                                                    circle_members.insert(circle_uuid.to_string(), Vec::new());
                                                    circle_members.get_mut(&circle_uuid.to_string()).expect("to be here")
                                                }
                                                Some(circle_members) => {
                                                    circle_members
                                                }
                                            }
                                        };

                                        let mut members = room.members_no_sync(RoomMemberships::ACTIVE).await?;

                                        //This method is the same as in GetContactsPaged TODO cleanup
                                        for current in members.drain(..){

                                            match current.membership() {
                                                MembershipState::Join => {
                                                    let msn_user = MsnUser::with_email_addr(EmailAddress::from_user_id(current.user_id()));
                                                    circle_members.push(ContactType::new_circle_member_contact(&msn_user.uuid, msn_user.get_email_address().as_str(), current.display_name().unwrap_or(msn_user.get_email_address().as_str()), ContactTypeEnum::Live, RelationshipState::Accepted , CircleRelationshipRole::Member , false));
                                                }
                                                _ => {
                                                    let msn_user = MsnUser::with_email_addr(EmailAddress::from_user_id(current.user_id()));
                                                    circle_members.push(ContactType::new_circle_member_contact(&msn_user.uuid, msn_user.get_email_address().as_str(), current.display_name().unwrap_or(msn_user.get_email_address().as_str()), ContactTypeEnum::LivePending, RelationshipState::WaitingResponse , CircleRelationshipRole::Member,false));
                                                }
                                            }
                                        }

                                    }
                                    _ => {
                                        //TODO maybe
                                    }
                                }

                            } else {
                                // Some circle member action

                                let display_name = room.get_member_no_sync(&og_rm_event.state_key).await?.map(|rm| rm.display_name().map(|name| name.to_string())).unwrap_or(None);
                                let target_user = MsnUser::with_email_addr(EmailAddress::from_user_id(&og_rm_event.state_key));
                                let target_msn_addr = target_user.get_email_address().as_str();
                                let display_name = display_name.unwrap_or(target_msn_addr.to_string());

                                let circle_members = {
                                    match circle_members.get_mut(&circle_uuid.to_string()) {
                                        None => {
                                            circle_members.insert(circle_uuid.to_string(), Vec::new());
                                            circle_members.get_mut(&circle_uuid.to_string()).expect("to be here")
                                        }
                                        Some(circle_members) => {
                                            circle_members
                                        }
                                    }
                                };


                                match og_rm_event.membership_change() {
                                    MembershipChange::Joined | MembershipChange::InvitationAccepted | MembershipChange::KnockAccepted => {
                                        debug!("SYNC|MEMBERSHIPS|JOIN: Sent Invite was accepted by contact in circle: {}", &target_msn_addr);
                                        circle_members.push(ContactType::new_circle_member_contact(&target_user.uuid, target_msn_addr, &display_name, ContactTypeEnum::Live, RelationshipState::Accepted , CircleRelationshipRole::Member , false));
                                    },
                                    MembershipChange::Left | MembershipChange::Banned | MembershipChange::Kicked | MembershipChange::KickedAndBanned => {
                                        debug!("SYNC|MEMBERSHIPS|JOIN: Contact Left the circle: {}", &target_msn_addr);
                                        circle_members.push(ContactType::new_circle_member_contact(&target_user.uuid, target_msn_addr, &display_name, ContactTypeEnum::Live, RelationshipState::Accepted , CircleRelationshipRole::Member , true));
                                    },
                                    MembershipChange::Invited => {
                                        debug!("SYNC|MEMBERSHIPS|JOIN: Invited contact to join circle: {}", &target_msn_addr);
                                        circle_members.push(ContactType::new_circle_member_contact(&target_user.uuid, target_msn_addr, &display_name, ContactTypeEnum::Live, RelationshipState::WaitingResponse , CircleRelationshipRole::Member , false));
                                    }

                                    _ => {
                                        //TODO maybe
                                    }
                                }
                                }

                        }
                    }
                }
                _ => {
                    //TODO Handle redactions :(
                    debug!("SYNC|MEMBERSHIPS|JOIN: Non Original SyncRoomMemberEvent Received: {:?}", event);

                }
            }

    Ok(())
}

async fn handle_invite_room_member_event(event: &StrippedRoomMemberEvent, room_id: &RoomId, me: &UserId, client: &Client, contacts: &mut Vec<Contact>, memberships: &mut VecDeque<BaseMember>) -> Result<(), anyhow::Error> {
    match event.content.membership {
        MembershipState::Invite => {

            if event.state_key == me {

                let target_user = event.sender.as_ref();
                let target_msn_user = MsnUser::with_email_addr(EmailAddress::from_user_id(target_user));
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
                        let room = client.get_room(room_id).unwrap();
                        let mut circle = ContactType::new_circle(room_id.as_str(), &room.computed_display_name().await.unwrap().to_string(), false, RelationshipState::WaitingResponse, CircleRelationshipRole::StatePendingOutbound);
                        let contact_info = circle.contact_info.as_mut().unwrap();
                        let network_info = contact_info.network_info_list.as_mut().unwrap();
                        let network_info_first = network_info.network_info.get_mut(0).unwrap();

                        network_info_first.inviter_message = event.content.reason.clone();
                        network_info_first.inviter_email = Some(target_msn_user.get_email_address().to_string());
                        network_info_first.inviter_cid = target_msn_user.uuid.to_decimal_cid() as u64;
                        network_info_first.inviter_name = Some(target_msn_user.get_email_address().to_string());
                        //network_info_first.inviter_name = Some(room.get_member(&target_user).await?.unwrap().display_name().unwrap_or(target_msn_user.get_email_address().as_str()).to_string());

                        let inverse_info = CircleInverseInfoType::new(circle.contact_id.clone().expect("to be here"), room.computed_display_name().await.expect("").to_string(), false, CircleRelationshipRole::StatePendingOutbound, RelationshipState::WaitingResponse);
                        contacts.push(Contact::Circle(CircleData {
                            contact: circle,
                            inverse_info,
                        }));
                    }
                }

            }
        }
        _ => {}
    }

    Ok(())

}

async fn handle_leave_room_member_event(event: &SyncRoomMemberEvent, room: &Room, me: &UserId, client: &Client, contacts: &mut Vec<Contact>, memberships: &mut VecDeque<BaseMember>) -> Result<(), anyhow::Error> {
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


