use std::sync::{Arc, RwLock};
use log::error;
use crate::matrix::directs::direct_service::{DirectService, MappingDiff, RoomMapping};
use crate::notification::client_store::ClientData;
use matrix_sdk::ruma::events::room::member::{MembershipChange, MembershipState, RoomMemberEventContent, StrippedRoomMemberEvent, SyncRoomMemberEvent};
use matrix_sdk::ruma::{OwnedRoomId, OwnedUserId, RoomId};
use matrix_sdk::Client;
use matrix_sdk::ruma::events::OriginalSyncStateEvent;
use msnp::msnp::models::contact::Contact;
use msnp::msnp::models::contact_list::ContactList;
use msnp::msnp::notification::command::adl::ADLPayload;
use msnp::shared::models::msn_user::MsnUser;
use msnp::shared::models::role_list::RoleList;
use msnp::soap::abch::msnab_datatypes::{BaseMember, CircleInverseInfoType, ContactType, ContactTypeEnum, MemberState};
use crate::shared::identifiers::MatrixIdCompatible;

pub struct ContactServiceInner {
    contact_list: RwLock<ContactList>,

}

#[derive(Clone)]
pub struct ContactService {
    inner: Arc<ContactServiceInner>,
    direct_service: DirectService,
    own_user_id: OwnedUserId
}
impl ContactService {

    pub fn new(direct_service: DirectService, own_user_id: OwnedUserId) -> Self {
        Self {
            inner: Arc::new(
                ContactServiceInner {
                        contact_list: RwLock::new(ContactList::default())
                }
            ),
            direct_service,
            own_user_id
        }
    }


    pub fn add_contacts(&self, payload: ADLPayload) {

    }

    pub fn remove_contacts(&self, payload: ADLPayload) {

    }

    pub fn handle_direct_mapping_diff(&self, diff: MappingDiff) {
        match diff {
            MappingDiff::NewMapping(user_id, room_id) => {

            }
            MappingDiff::UpdatedMapping(user_id, room_id) => {

            }
            MappingDiff::RemovedMapping(user_id, room_id) => {

            }
        }
    }

    pub(crate) fn handle_stripped_room_member_event(&self, event: StrippedRoomMemberEvent, room_id: &RoomId) -> Option<MappingResult> {
        todo!()
    }

    pub fn handle_room_member_event(&self, event: SyncRoomMemberEvent, room_id: &RoomId) -> Option<MappingResult> {
        if let SyncRoomMemberEvent::Original(_event) = &event {

            match self.direct_service.get_mapping_for_room(room_id) {
                RoomMapping::Canonical(contact_id, room_id) => {
                    self.handle_canonical_dm_room_member_event(contact_id, room_id, event)
                }
                RoomMapping::Group => {
                    self.handle_group_room_member_event(event)
                }
            }

        } else {
            None
        }
    }

    fn handle_canonical_dm_room_member_event(&self, contact_id: OwnedUserId, room_id: OwnedRoomId, event: SyncRoomMemberEvent) -> Option<MappingResult> {

       let event_target ={
           let own_user_id = self.own_user_id.clone();

           if event.state_key() == &contact_id {
               CanonicalRoomEventTarget::DirectTarget
           } else if event.state_key() == &own_user_id {
               CanonicalRoomEventTarget::Me(own_user_id)
           } else {
               CanonicalRoomEventTarget::Thirdwheel
           }
       };

        let contact_user = MsnUser::from_user_id(&contact_id);

        match event_target {
            CanonicalRoomEventTarget::Me(user_id) => {
                match event.membership() {
                    MembershipState::Join => {
                        Some(MappingResult {
                                contacts: vec![
                                    ContactType::new(&contact_user, ContactTypeEnum::Live, false)
                                ],
                                memberships: vec![
                                    BaseMember::new_passport_member(&contact_user, MemberState::Accepted, RoleList::Allow, false),
                                    BaseMember::new_passport_member(&contact_user, MemberState::Accepted, RoleList::Pending, true)
                                ],
                            circle_reverse_info: vec![],
                        })
                    }
                    MembershipState::Leave | MembershipState::Ban  => {
                        Some(MappingResult {
                                contacts: vec![ContactType::new(&contact_user, ContactTypeEnum::Live, true)],
                                memberships: vec![
                                    BaseMember::new_passport_member(&contact_user, MemberState::Accepted, RoleList::Allow, true),
                                    BaseMember::new_passport_member(&contact_user, MemberState::Accepted, RoleList::Reverse, true),
                                    BaseMember::new_passport_member(&contact_user, MemberState::Accepted, RoleList::Pending, true),
                                ],
                            circle_reverse_info: vec![],
                        })
                    }
                    _ => {
                        None
                    }
                }
            }
            CanonicalRoomEventTarget::DirectTarget => {
                match event.membership() {
                    MembershipState::Join => {
                        Some(
                            MappingResult{
                                contacts: vec![
                                    ContactType::new(&contact_user, ContactTypeEnum::LivePending, true),
                                    ContactType::new(&contact_user, ContactTypeEnum::Live, false)
                                ],
                                memberships: vec![
                                    BaseMember::new_passport_member(&contact_user, MemberState::Accepted, RoleList::Reverse, false)
                                ],
                                circle_reverse_info: vec![],
                            }
                        )
                    }
                    MembershipState::Ban | MembershipState::Leave  => {
                        Some(
                            MappingResult{
                                contacts: vec![
                                    ContactType::new(&contact_user, ContactTypeEnum::Live, true),
                                    ContactType::new(&contact_user, ContactTypeEnum::LivePending, false)
                                ],
                                memberships: vec![
                                    BaseMember::new_passport_member(&contact_user, MemberState::Accepted, RoleList::Reverse, true)
                                ],
                                circle_reverse_info: vec![],
                            }
                        )
                    }
                    MembershipState::Invite => {
                        Some(
                            MappingResult{
                                contacts: vec![
                                    ContactType::new(&contact_user, ContactTypeEnum::LivePending, false),
                                ],
                                memberships: vec![
                                    BaseMember::new_passport_member(&contact_user, MemberState::Accepted, RoleList::Allow, false)
                                ],
                                circle_reverse_info: vec![],
                            }
                        )
                    },
                    MembershipState::Knock => {
                        None
                    }
                    _ => {
                        None
                    }
                }

            }
            CanonicalRoomEventTarget::Thirdwheel => {
                error!("Thirdwheel was found in canonical dm room, it should not be possible.");
                //Todo handle this err
                None
            }
        }

    }


    fn handle_group_room_member_event(&self, event: SyncRoomMemberEvent) -> Option<MappingResult> {
        let event_target ={
            if event.state_key() == &self.own_user_id {
                GroupRoomEventTarget::Me
            } else {
                GroupRoomEventTarget::Thirdwheel(event.state_key().to_owned())
            }
        };

        match event_target {
            GroupRoomEventTarget::Me => {
                match event.membership() {
                    MembershipState::Ban => {}
                    MembershipState::Invite => {}
                    MembershipState::Join => {}
                    MembershipState::Knock => {}
                    MembershipState::Leave => {}
                    MembershipState::_Custom(_) => {}
                    _ => {}
                }

            }
            GroupRoomEventTarget::Thirdwheel(user_id) => {

                match event.membership() {
                    MembershipState::Ban => {}
                    MembershipState::Invite => {}
                    MembershipState::Join => {}
                    MembershipState::Knock => {}
                    MembershipState::Leave => {}
                    MembershipState::_Custom(_) => {}
                    _ => {}
                }

            }
        }

        None
    }





}


pub struct MappingResult {
    pub contacts: Vec<ContactType>,
    pub memberships: Vec<BaseMember>,
    pub circle_reverse_info: Vec<CircleInverseInfoType>
}

impl MappingResult {
    pub fn new() -> Self {
        Self {
            contacts: Vec::new(),
            memberships: Vec::new(),
            circle_reverse_info: Vec::new()
        }
    }
}

enum GroupRoomEventTarget{
    Me,
    Thirdwheel(OwnedUserId)
}

enum CanonicalRoomEventTarget{
    Me(OwnedUserId),
    DirectTarget,
    Thirdwheel
}