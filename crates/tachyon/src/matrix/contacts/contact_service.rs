use std::cmp::Ordering;
use std::collections::HashMap;
use crate::matrix::contacts::contact_service::ContactDiff::{AddContact, ClearContact, RemoveContact};
use crate::matrix::contacts::contact_service::MembershipDiff::{AddInviteMembership, AddMembership, ClearMemberships, RemoveMembership};
use crate::matrix::directs::direct_service::{DirectService, MappingDiff, RoomMapping};
use crate::notification::client_store::ClientData;
use crate::shared::identifiers::MatrixIdCompatible;
use log::{debug, error};
use matrix_sdk::ruma::events::room::member::{MembershipChange, MembershipState, RoomMemberEventContent, StrippedRoomMemberEvent, SyncRoomMemberEvent};
use matrix_sdk::ruma::events::OriginalSyncStateEvent;
use matrix_sdk::ruma::{OwnedRoomId, OwnedUserId, RoomId};
use matrix_sdk::Client;
use msnp::msnp::models::contact::Contact;
use msnp::msnp::models::contact_list::ContactList;
use msnp::msnp::notification::command::adl::ADLPayload;
use msnp::shared::models::email_address::EmailAddress;
use msnp::shared::models::msn_user::MsnUser;
use msnp::shared::models::role_list::RoleList;
use msnp::soap::abch::msnab_datatypes::{Annotation, ArrayOfAnnotation, BaseMember, CircleInverseInfoType, ContactType, ContactTypeEnum, MemberState};
use std::sync::{Arc, Mutex, RwLock};

pub struct ContactServiceInner {
    contact_list: RwLock<ContactList>,
    pub pending_contacts: Mutex<HashMap<String, ContactDiff>>,
    pub pending_members: Mutex<Vec<MembershipDiff>>,
    pub pending_circles: Mutex<Vec<CircleDiff>>,
}

#[derive(Clone)]
pub struct ContactService {
    pub inner: Arc<ContactServiceInner>,
    direct_service: DirectService,
    own_user_id: OwnedUserId
}

const LOG_PREFIX: &str = "ContactService |";

impl ContactService {

    pub fn new(direct_service: DirectService, own_user_id: OwnedUserId) -> Self {
        Self {
            inner: Arc::new(
                ContactServiceInner {
                        contact_list: RwLock::new(ContactList::default()),
                    pending_contacts: Mutex::new(HashMap::new()),
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

    pub fn handle_stripped_room_member_event(&self, event: StrippedRoomMemberEvent, room_id: &RoomId) {
        debug!("{} Handle Stripped Room Member Event: {:?} for room id: {}", LOG_PREFIX, &event, &room_id);

        let address_book_diffs = match self.direct_service.get_mapping_for_room(room_id) {

            RoomMapping::Canonical(contact_id, room_id) => {
                debug!("{} Room was canonical", LOG_PREFIX);
                self.handle_canonical_dm_stripped_room_member_event(contact_id, room_id, event)

            }

            RoomMapping::Group => {
                debug!("{} Room was group", LOG_PREFIX);
                vec![]
            }
        };

        self.persist_diffs(address_book_diffs)
    }

    pub fn handle_canonical_dm_stripped_room_member_event(&self, contact_id: OwnedUserId, room_id: OwnedRoomId,  event: StrippedRoomMemberEvent) -> Vec<AddressBookDiff> {
        if &event.sender != &self.own_user_id  && &event.sender != &contact_id {
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

    pub fn handle_room_member_event(&self, event: SyncRoomMemberEvent, room_id: &RoomId) {
        debug!("{} Handle Room Member Event: {:?} for room id: {}", LOG_PREFIX, &event, &room_id);
        let address_book_diffs = {
            if let SyncRoomMemberEvent::Original(_event) = &event {
                match self.direct_service.get_mapping_for_room(room_id) {
                    RoomMapping::Canonical(contact_id, room_id) => {
                        debug!("{} Room was canonical", LOG_PREFIX);
                        self.handle_canonical_dm_room_member_event(contact_id, room_id, event)
                    }
                    RoomMapping::Group => {
                        debug!("{} Room was group", LOG_PREFIX);
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

                    match pending_contacts.get(&diff.get_key()) {
                        None => {
                            pending_contacts.insert(diff.get_key(), diff);
                        },
                        Some(found) => {
                            if diff.get_weigth() > found.get_weigth() {
                                pending_contacts.insert(diff.get_key(), diff);
                            }
                        }
                    }

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

        if event.sender() != &self.own_user_id && event.sender() != &contact_id {
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
                debug!("{} Event target was me", LOG_PREFIX);


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

                debug!("{} Event target was direct target", LOG_PREFIX);

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

trait GetKey {
    fn get_key(&self) -> String;
}

impl GetKey for CircleDiff {
    fn get_key(&self) -> String {
        match self {
            _=>{
                "".to_string()
            }
        }
    }
}

impl GetKey for MembershipDiff {
    fn get_key(&self) -> String {
        match self {
            MembershipDiff::AddMembership { user_id, list_type } => {
                user_id.to_string()
            }
            MembershipDiff::AddInviteMembership { user_id, message } => {
                user_id.to_string()

            }
            MembershipDiff::RemoveMembership { user_id, list_type } => {
                user_id.to_string()

            }
            MembershipDiff::ClearMemberships { user_id } => {
                user_id.to_string()
            }
        }
    }
}

impl GetKey for AddressBookDiff {
    fn get_key(&self) -> String {
        match self {
            AddressBookDiff::Contact(diff) => {
                diff.get_key()
            }
            AddressBookDiff::Membership(diff) => {
                diff.get_key()
            }
            AddressBookDiff::Circle(diff) => {
                diff.get_key()
            }
        }
    }
}

impl GetKey for ContactDiff {
    fn get_key(&self) -> String {
        match self {
            ContactDiff::AddContact { user_id, pending } => {
                user_id.to_string()
            }
            ContactDiff::RemoveContact { user_id, pending } => {
                user_id.to_string()
            }
            ContactDiff::ClearContact { user_id } => {
                user_id.to_string()
            }
        }
    }
}

impl ContactDiff {
    pub fn get_weigth(&self) -> u8 {
        match self {
            ContactDiff::AddContact { user_id, pending } => {
                let mut weigth = 3;
                if !pending {
                    weigth += 100;
                }
                weigth
            }
            ContactDiff::RemoveContact { user_id, pending } => {
                1
            }
            ContactDiff::ClearContact { .. } => {
                2
            }
        }
    }
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

#[cfg(test)]
mod tests {
    use crate::matrix::contacts::contact_service::{ContactDiff, ContactService};
    use crate::matrix::directs::direct_service::DirectService;
    use matrix_sdk::ruma::events::direct::DirectEventContent;
    use matrix_sdk::ruma::{owned_room_id, owned_user_id, room_id, OwnedRoomId, OwnedUserId};
    use matrix_sdk::test_utils::logged_in_client_with_server;
    use matrix_sdk::Client;
    use std::collections::HashMap;
    use std::str::FromStr;
    use matrix_sdk::ruma::events::AnySyncStateEvent;
    use matrix_sdk::ruma::events::room::member::{RoomMemberEvent, SyncRoomMemberEvent};
    use matrix_sdk::ruma::serde::Raw;
    use wiremock::MockServer;

    async fn setup() -> (Client, MockServer, DirectService, ContactService, OwnedUserId) {
        let (client, server) = logged_in_client_with_server().await;
        let me_user_id = client.user_id().unwrap().to_owned();
        let direct_service = DirectService::new(HashMap::new(), DirectEventContent::default(), client.clone());
        let contact_service = ContactService::new(direct_service.clone(), me_user_id.clone());

        (client, server, direct_service, contact_service, me_user_id)
    }

    #[tokio::test]
    async fn when_i_join_a_canonical_dm_create_pending_contact() {

        let (client, server, direct_service, contact_service, own_user_id) = setup().await;

        let room_id = owned_room_id!("!room_id:example.org");
        let contact_id = owned_user_id!("@contact:example.org");

        direct_service.inner.direct_mappings.write().insert(contact_id.clone(), room_id.clone());

        let event = Raw::<AnySyncStateEvent>::from_json_string(
            format!(r#"
                {{
                  "content": {{
                    "membership": "join"
                  }},
                  "event_id": "$123456:tachyon.fake",
                  "origin_server_ts": 1432735824653,
                  "room_id": "{}",
                  "sender": "{}",
                  "state_key": "{}",
                  "type": "m.room.member",
                  "unsigned": {{
                    "age": 1234,
                    "membership": "leave"
                  }}
                }}
                "#, &room_id, &own_user_id, &own_user_id).to_string()).unwrap().deserialize_as::<SyncRoomMemberEvent>().unwrap();;


        println!("{:?}", &event);

        contact_service.handle_room_member_event(event, &room_id);

        let mut contacts = contact_service.inner.pending_contacts.lock().unwrap();

        assert_eq!(contacts.len(), 1);

        let diff = contacts.get(&contact_id.to_string()).unwrap();
        assert!(matches!(diff, ContactDiff::AddContact{
            user_id: contact_id,
            pending: true,
        }));
    }

    #[tokio::test]
    async fn when_i_join_a_canonical_dm_and_target_joins_create_contact() {

        let (client, server, direct_service, contact_service, own_user_id) = setup().await;

        let room_id = owned_room_id!("!room_id:example.org");
        let contact_id = owned_user_id!("@contact:example.org");

        direct_service.inner.direct_mappings.write().insert(contact_id.clone(), room_id.clone());

        {
            let mut contacts = contact_service.inner.pending_contacts.lock().unwrap();
            contacts.insert(contact_id.to_string(), ContactDiff::AddContact { user_id: contact_id.clone(), pending: true });
        }


        let event = Raw::<AnySyncStateEvent>::from_json_string(
            format!(r#"
                {{
                  "content": {{
                    "membership": "join"
                  }},
                  "event_id": "$123456:tachyon.fake",
                  "origin_server_ts": 1432735824653,
                  "room_id": "{}",
                  "sender": "{}",
                  "state_key": "{}",
                  "type": "m.room.member",
                  "unsigned": {{
                    "age": 1234,
                    "membership": "leave"
                  }}
                }}
                "#, &room_id, &contact_id, &contact_id).to_string()).unwrap().deserialize_as::<SyncRoomMemberEvent>().unwrap();;


        println!("{:?}", &event);



        contact_service.handle_room_member_event(event, &room_id);

        let mut contacts = contact_service.inner.pending_contacts.lock().unwrap();

        assert_eq!(contacts.len(), 1);

        let diff = contacts.get(&contact_id.to_string()).unwrap();
        assert!(matches!(diff, ContactDiff::AddContact{
            user_id: contact_id,
            pending: false,
        }));
    }

    #[tokio::test]
    async fn when_i_join_a_canonical_dm_and_target_joins_create_contact_reversed() {

        let (client, server, direct_service, contact_service, own_user_id) = setup().await;

        let room_id = owned_room_id!("!room_id:example.org");
        let contact_id = owned_user_id!("@contact:example.org");

        direct_service.inner.direct_mappings.write().insert(contact_id.clone(), room_id.clone());

        {
            let mut contacts = contact_service.inner.pending_contacts.lock().unwrap();
            contacts.insert(contact_id.to_string(), ContactDiff::AddContact { user_id: contact_id.clone(), pending: false });
        }


        let event = Raw::<AnySyncStateEvent>::from_json_string(
            format!(r#"
                {{
                  "content": {{
                    "membership": "join"
                  }},
                  "event_id": "$123456:tachyon.fake",
                  "origin_server_ts": 1432735824653,
                  "room_id": "{}",
                  "sender": "{}",
                  "state_key": "{}",
                  "type": "m.room.member",
                  "unsigned": {{
                    "age": 1234,
                    "membership": "leave"
                  }}
                }}
                "#, &room_id, &own_user_id, &own_user_id).to_string()).unwrap().deserialize_as::<SyncRoomMemberEvent>().unwrap();;


        println!("{:?}", &event);



        contact_service.handle_room_member_event(event, &room_id);

        let mut contacts = contact_service.inner.pending_contacts.lock().unwrap();

        assert_eq!(contacts.len(), 1);

        let diff = contacts.get(&contact_id.to_string()).unwrap();
        assert!(matches!(diff, ContactDiff::AddContact{
            user_id: contact_id,
            pending: false,
        }));
    }


}