use std::collections::VecDeque;
use std::convert::Infallible;
use std::mem;
use log::warn;
use matrix_sdk::Client;
use matrix_sdk::ruma::events::{AnyStrippedStateEvent, AnySyncStateEvent, OriginalSyncStateEvent};
use matrix_sdk::ruma::events::room::member::{MembershipChange, MembershipState, RoomMemberEventContent, StrippedRoomMemberEvent, SyncRoomMemberEvent};
use matrix_sdk::ruma::{OwnedUserId, RoomId};
use matrix_sdk::sync::{RoomUpdate, RoomUpdates, SyncResponse};
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
use crate::matrix::directs::{get_room_mapping_info, RoomMappingInfo};
use crate::notification::client_store::ClientData;
use crate::shared::identifiers::MatrixIdCompatible;

pub async fn handle_memberships(client: Client, response: SyncResponse, mut client_data: ClientData, notif_sender: Sender<NotificationServerCommand>) -> Result<(), anyhow::Error> {
    let me = client.user_id().expect("UserID to be here");

    let mut contacts = Vec::new();
    let mut memberships = VecDeque::new();

    let membership_events = get_member_events(&response.rooms);

    for current in membership_events {

        let room_id = current.room_id;
        let room = client.get_room(&room_id).expect("Room to be here");
        let mapping = get_room_mapping_info(&room, me, &client).await?;

        match mapping {
            RoomMappingInfo::Direct(direct_target) => {

                let target_msn_user = MsnUser::with_email_addr(EmailAddress::from_user_id(&direct_target));
                let target_msn_addr = target_msn_user.get_email_address().as_str();

                match current.event {
                    MembershipEventType::Stripped(stripped_rm_event) => {
                        match stripped_rm_event.content.membership {
                            MembershipState::Invite => {
                                if stripped_rm_event.state_key == me {
                                    //I've been invited ! ADD TO PENDING LIST WITH INVITE MSG, ADD TO REVERSE LIST
                                    log::info!("AB - I received an invite from: {}", &target_msn_addr);

                                    let mut current_pending_member = BaseMember::new_passport_member(&target_msn_user.uuid, &target_msn_addr, MemberState::Accepted, RoleList::Pending, false);
                                    current_pending_member.display_name = Some(target_msn_addr.to_string());
                                    let annotation = Annotation::new_invite(stripped_rm_event.content.reason.as_ref().map(|e| e.as_str()).unwrap_or(""));
                                    let mut annotations = Vec::new();
                                    annotations.push(annotation);
                                    current_pending_member.annotations = Some(ArrayOfAnnotation { annotation: annotations });


                                    let current_reverse_member = BaseMember::new_passport_member(&target_msn_user.uuid, &target_msn_addr, MemberState::Accepted, RoleList::Reverse, false);
                                    memberships.push_back(current_pending_member);
                                    memberships.push_back(current_reverse_member);
                                }

                            }
                            _ => {}

                        }

                    }
                    MembershipEventType::Original(og_rm_event) => {

                        if &og_rm_event.state_key == &direct_target {
                            let display_name = og_rm_event.content.displayname.as_ref().map(|d| d.as_str()).unwrap_or(target_msn_user.get_email_address().as_str());

                            match og_rm_event.membership_change() {
                                MembershipChange::Joined | MembershipChange::InvitationAccepted | MembershipChange::KnockAccepted => {
                                    //He accepted my invitation, ADD TO REVERSE LIST, CHANGE CONTACT TO NOT PENDING
                                    let invited_contact = ContactType::new(&target_msn_user.uuid, target_msn_addr, &display_name, ContactTypeEnum::Live, false);
                                    let invited_reverse_member = BaseMember::new_passport_member(&target_msn_user.uuid, target_msn_addr, MemberState::Accepted, RoleList::Reverse, false);

                                    contacts.push(invited_contact);
                                    memberships.push_back(invited_reverse_member);
                                }
                                MembershipChange::Left | MembershipChange::Banned | MembershipChange::Kicked | MembershipChange::KickedAndBanned => {
                                    //He left the room, Remove from Reverse List, Set to contact pending
                                    let left_contact = ContactType::new(&target_msn_user.uuid, target_msn_addr, &display_name, ContactTypeEnum::LivePending, false);
                                    let left_reverse_member = BaseMember::new_passport_member(&target_msn_user.uuid, target_msn_addr, MemberState::Accepted, RoleList::Reverse, true);
                                    contacts.push(left_contact);
                                    memberships.push_back(left_reverse_member);
                                }
                                MembershipChange::Invited => {
                                    //I Invited him, Add to allow list, add to contact pending.
                                    let invited_contact = ContactType::new(&target_msn_user.uuid, target_msn_addr, &display_name, ContactTypeEnum::LivePending, false);
                                    let invited_allow_member = BaseMember::new_passport_member(&target_msn_user.uuid, target_msn_addr, MemberState::Accepted, RoleList::Allow, false);
                                    contacts.push(invited_contact);
                                    memberships.push_back(invited_allow_member);
                                }
                                _ => {}
                            }
                        } else if &og_rm_event.state_key == me{
                            match og_rm_event.membership_change() {
                                MembershipChange::Left => {
                                    log::info!("AB - I Left: Delete: {}", &target_msn_user.get_email_address());
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
                                    log::info!("AB - I Accepted an invite from: {}", &target_msn_user.get_email_address());
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
                            warn!("1o1 dm room event state_key was unexpected: state_key: {}, event_id: {}, room_id: {}", &og_rm_event.state_key, &og_rm_event.state_key, &room_id );

                        }
                    }
                }


            }
            RoomMappingInfo::Group => {

            }
        }

    }

    if !contacts.is_empty() || !memberships.is_empty() {

        {
            let mut contacts_mtx = client_data.inner.soap_holder.contacts.lock().unwrap();
            let mut memberships_mtx = client_data.inner.soap_holder.memberships.lock().unwrap();

            if(contacts_mtx.is_empty()) {
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
        notif_sender.send(NotificationServerCommand::NOT(NotServer{
            payload: NotificationFactory::get_abch_updated(&me_msn_usr.uuid, me_msn_usr.get_email_address().as_str()),
        })).await;

    }
    Ok(())
}

pub enum MembershipEventType {
    Stripped(StrippedRoomMemberEvent),
    Original(OriginalSyncStateEvent<RoomMemberEventContent>),
}

pub struct MembershipEvent<'a> {
    event: MembershipEventType,
    room_type: MembershipState,
    room_id: &'a RoomId,
}

pub fn get_member_events(room_updates: &RoomUpdates) -> Vec<MembershipEvent> {
    let mut out = Vec::new();

    for (room_id, update) in &room_updates.join {
        for state_event in &update.state {
            if let Ok(AnySyncStateEvent::RoomMember(room_member_event)) = state_event.deserialize() {
                match room_member_event {
                    SyncRoomMemberEvent::Original(og_rm_event) => {
                        out.push(MembershipEvent {
                            event: MembershipEventType::Original(og_rm_event),
                            room_type: MembershipState::Join,
                            room_id: &room_id,
                        });
                    }
                    _ => {}
                }
            }
        }
    }

    for (room_id, update) in &room_updates.leave {
        for state_event in &update.state {
            if let Ok(AnySyncStateEvent::RoomMember(room_member_event)) = state_event.deserialize() {
                match room_member_event {
                    SyncRoomMemberEvent::Original(og_rm_event) => {
                        out.push(MembershipEvent {
                            event: MembershipEventType::Original(og_rm_event),
                            room_type: MembershipState::Leave,
                            room_id: &room_id,
                        });
                    }
                    _ => {}
                }
            }
        }
    }

    for (room_id, updates) in &room_updates.invite {
        for state_event in &updates.invite_state.events {
            if let Ok(AnyStrippedStateEvent::RoomMember(room_member_event)) = state_event.deserialize() {
                out.push(MembershipEvent {
                    event: MembershipEventType::Stripped(room_member_event),
                    room_type: MembershipState::Invite,
                    room_id: &room_id,
                });
            }
        }
    }

    out
}

pub async fn handle_contact_mapping() {}

pub async fn handle_circle_mapping() {}