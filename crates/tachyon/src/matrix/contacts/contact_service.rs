use std::sync::{Arc, Mutex, RwLock};
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
use msnp::shared::models::email_address::EmailAddress;
use msnp::shared::models::msn_user::MsnUser;
use msnp::shared::models::role_list::RoleList;
use msnp::soap::abch::msnab_datatypes::{Annotation, ArrayOfAnnotation, BaseMember, CircleInverseInfoType, ContactType, ContactTypeEnum, MemberState};
use crate::matrix::contacts::contact_service::ContactDiff::{AddContact, ClearContact, RemoveContact};
use crate::matrix::contacts::contact_service::MembershipDiff::{AddInviteMembership, AddMembership, ClearMemberships, RemoveMembership};
use crate::shared::identifiers::MatrixIdCompatible;

pub struct ContactServiceInner {
    contact_list: RwLock<ContactList>,
    pub pending_contacts: Mutex<Vec<ContactDiff>>,
    pub pending_members: Mutex<Vec<MembershipDiff>>,
    pub pending_circles: Mutex<Vec<CircleDiff>>,
}

#[derive(Clone)]
pub struct ContactService {
    pub inner: Arc<ContactServiceInner>,
    direct_service: DirectService,
    own_user_id: OwnedUserId
}
impl ContactService {

    pub fn new(direct_service: DirectService, own_user_id: OwnedUserId) -> Self {
        Self {
            inner: Arc::new(
                ContactServiceInner {
                        contact_list: RwLock::new(ContactList::default()),
                    pending_contacts: Mutex::new(vec![]),
                    pending_members: Mutex::new(vec![]),
                    pending_circles: Mutex::new(vec![]),
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

    pub fn handle_stripped_room_member_event(&self, event: StrippedRoomMemberEvent, room_id: &RoomId) -> Vec<AddressBookDiff> {


        match self.direct_service.get_mapping_for_room(room_id) {

            RoomMapping::Canonical(contact_id, room_id) => {

                if &event.sender != &self.own_user_id || &event.sender != &contact_id {
                    return vec![]
                }

                let event_target ={
                    let own_user_id = self.own_user_id.clone();

                    if &event.state_key == &contact_id {
                        CanonicalRoomEventTarget::DirectTarget
                    } else if &event.state_key == &own_user_id {
                        CanonicalRoomEventTarget::Me(own_user_id)
                    } else {
                        CanonicalRoomEventTarget::Thirdwheel
                    }
                };


                match event_target {
                    CanonicalRoomEventTarget::Me(user_id) => {

                        match event.content.membership {
                            MembershipState::Invite => {
                                vec![AddInviteMembership { user_id: contact_id.clone(), message: event.content.reason }.into(), AddMembership { user_id: contact_id.clone(), list_type: RoleList::Reverse}.into()]
                            }
                            _ => {
                                vec![]
                            }
                        }



                    }
                    CanonicalRoomEventTarget::DirectTarget => {
                        vec![]
                    }
                    CanonicalRoomEventTarget::Thirdwheel => {
                        error!("Thirdwheel was found in canonical stripped dm room, it should not be possible.");
                        vec![]
                    }
                }



            }

            RoomMapping::Group => {
                vec![]
            }
        }


    }

    pub fn handle_room_member_event(&self, event: SyncRoomMemberEvent, room_id: &RoomId) {
        let address_book_diffs = {
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
                vec![]
            }
        };

        self.persist_diffs(address_book_diffs)
    }

    fn persist_diffs(&self, diffs: Vec<AddressBookDiff>) {
        let mut pending_contacts = self.inner.pending_contacts.lock().unwrap();
        let mut pending_members  = self.inner.pending_members.lock().unwrap();
        let mut pending_circles = self.inner.pending_circles.lock().unwrap();

        for diff in diffs {
            match diff {
                AddressBookDiff::Contact(diff) => {
                    pending_contacts.push(diff);
                }
                AddressBookDiff::Membership(diff) => {
                    pending_members.push(diff);
                }
                AddressBookDiff::Circle(diff) => {
                    pending_circles.push(diff);
                }
            }
        }
    }

    fn handle_canonical_dm_room_member_event(&self, contact_id: OwnedUserId, room_id: OwnedRoomId, event: SyncRoomMemberEvent) -> Vec<AddressBookDiff> {

        if event.sender() != &self.own_user_id || event.sender() != &contact_id {
            return vec![]
        }

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
            CanonicalRoomEventTarget::Me(own_user_id) => {

                match event.membership() {
                    MembershipState::Join => {
                        vec![AddContact { user_id: contact_id.clone(), pending: true }.into(), AddMembership { user_id: contact_id.clone(), list_type: RoleList::Allow }.into()]
                    },
                    MembershipState::Leave | MembershipState::Ban  =>  {
                        vec![ClearContact { user_id: contact_id.clone() }.into(), ClearMemberships { user_id: contact_id.clone() }.into()]
                    },
                    _ => vec![]
                }
            }
            CanonicalRoomEventTarget::DirectTarget => {
                match event.membership() {
                    MembershipState::Join => {
                        vec![AddContact { user_id: contact_id.clone(), pending: false }.into(), AddMembership { user_id: contact_id.clone(), list_type: RoleList::Reverse}.into()]
                    }
                    MembershipState::Ban | MembershipState::Leave  => {
                        vec![AddContact { user_id: contact_id.clone(), pending: true }.into(), RemoveMembership { user_id: contact_id.clone(), list_type: RoleList::Reverse}.into()]
                    }
                    MembershipState::Invite => {
                        vec![AddContact { user_id: contact_id.clone(), pending: true }.into(), AddMembership { user_id: contact_id.clone(), list_type: RoleList::Allow}.into()]
                    },
                    MembershipState::Knock => {
                        vec![]
                    }
                    _ => {
                        vec![]
                    }
                }

            }
            CanonicalRoomEventTarget::Thirdwheel => {
                error!("Thirdwheel was found in canonical dm room, it should not be possible.");
                //Todo handle this err
                vec![]
            }
        }

    }

    fn handle_group_room_member_event(&self, event: SyncRoomMemberEvent) -> Vec<AddressBookDiff> {
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
                    MembershipState::Ban => {

                    }
                    MembershipState::Invite => {

                    }
                    MembershipState::Join => {

                    }
                    MembershipState::Knock => {

                    }
                    MembershipState::Leave => {

                    }
                    MembershipState::_Custom(_) => {}
                    _ => {}
                }

            }
            GroupRoomEventTarget::Thirdwheel(user_id) => {

                match event.membership() {
                    MembershipState::Ban => {

                    }
                    MembershipState::Invite => {

                    }
                    MembershipState::Join => {

                    }
                    MembershipState::Knock => {

                    }
                    MembershipState::Leave => {

                    }
                    MembershipState::_Custom(_) => {}
                    _ => {}
                }

            }
        }

        vec![]
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

pub enum AddressBookDiff {
    Contact(ContactDiff),
    Membership(MembershipDiff),
    Circle(CircleDiff)
}

impl Into<AddressBookDiff> for ContactDiff {
    fn into(self) -> AddressBookDiff {
        AddressBookDiff::Contact(self)
    }
}

impl Into<AddressBookDiff> for MembershipDiff {
    fn into(self) -> AddressBookDiff {
        AddressBookDiff::Membership(self)
    }
}

impl Into<AddressBookDiff> for CircleDiff {
    fn into(self) -> AddressBookDiff {
        AddressBookDiff::Circle(self)
    }
}

pub enum CircleDiff {

}

pub enum MembershipDiff {
    AddMembership {
        user_id: OwnedUserId,
        list_type: RoleList
    },
    AddInviteMembership {
        user_id: OwnedUserId,
        message: Option<String>
    },
    RemoveMembership {
        user_id: OwnedUserId,
        list_type: RoleList
    },
    ClearMemberships {
        user_id: OwnedUserId,
    }
}
pub enum ContactDiff {
    AddContact {
        user_id: OwnedUserId,
        pending: bool,
    },
    RemoveContact {
        user_id: OwnedUserId,
        pending: bool,
    },
    ClearContact{
        user_id: OwnedUserId,
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