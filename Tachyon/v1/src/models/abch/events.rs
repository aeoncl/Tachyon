use crate::generated::msnab_datatypes::types::{BaseMember, ContactType, RoleId};

#[derive(Clone, Debug)]
pub enum AddressBookEvent {
    ContactEvent(ContactEventContent),
    MembershipEvent(MembershipEventContent),
    ExpressionProfileUpdateEvent,
    CircleEvent
}

#[derive(Clone, Debug)]
pub struct MembershipEventContent {
    pub token: String,
    pub member: BaseMember,
    pub list: RoleId
}

#[derive(Clone, Debug)]
pub struct ContactEventContent {
    pub token: String,
    pub contact: ContactType
}

pub struct AddressBookEventFactory;

impl AddressBookEventFactory {

    pub fn get_membership_event(token: String, member: BaseMember, list: RoleId) -> AddressBookEvent {
        return AddressBookEvent::MembershipEvent(MembershipEventContent{ token, member, list });
    }

    pub fn get_contact_event(token: String, contact: ContactType) -> AddressBookEvent {
        return AddressBookEvent::ContactEvent(ContactEventContent { token, contact });
    }

}
