//! Matrix contact family (§8 D3): the closures in `handlers/mod.rs` stay thin
//! shells; this module owns the gather → decide → apply pipeline (§3).
//! gather reads the SDK (about-me filter, room → MSN user resolution) into
//! plain facts; decide is the pure membership/room-state → ABCH contact-row
//! table; apply owns the single sink — the account's `SoapHolder` contact
//! list. The space filter and the room println stay shell policy, exactly
//! where they were pre-D3.
use crate::matrix::extensions::msn_user_resolver::{room_to_msn_user, RoomHashCache};
use crate::notification::models::soap_holder::AddressBookContact;
use crate::tachyon::client::tachyon_client::TachyonClient;
use log::warn;
use matrix_sdk::deserialized_responses::RawSyncOrStrippedState;
use matrix_sdk::ruma::events::room::member::{MembershipState, RoomMemberEventContent, StrippedRoomMemberEvent, SyncRoomMemberEvent};
use matrix_sdk::ruma::room::RoomType;
use matrix_sdk::{Client, Room, RoomState};
use matrix_sdk::ruma::events::room::tombstone::OriginalSyncRoomTombstoneEvent;
use msnp::shared::models::msn_user::MsnUser;
use msnp::soap::abch::msnab_datatypes::{ContactType, ContactTypeEnum};

/// Gathered facts for one about-me membership event — plain data only (§10).
pub(crate) struct GatheredContactEvent {
    pub(crate) membership: MembershipState,
    /// The room itself as an MSN user — the contact row is about the ROOM.
    pub(crate) room_user: MsnUser,
}

/// The stripped-event counterpart: a stripped event carries no membership of
/// its own, so the room state is the fact decide reads.
pub(crate) struct GatheredStrippedContactEvent {
    pub(crate) state: RoomState,
    pub(crate) room_user: MsnUser,
}

pub(super) async fn handle_contacts(
    event: SyncRoomMemberEvent,
    room: Room,
    tachyon_client: TachyonClient,
    client: Client,
) {
    let is_space = room.room_type().is_some_and(|room_type| matches!(room_type, RoomType::Space));
    if is_space {
        return;
    }

    println!("Handling contact event for room: {}", room.room_id().to_string());

    let Some(gathered) = gather_contact_event(&event, &room, tachyon_client.room_hashes()).await else {
        return;
    };

    apply_contacts(decide_contacts(gathered), &tachyon_client);
}

/// Gather (§3): the about-me filter and the room → MSN-user resolution, into
/// plain facts. `None` = skip the event. §1.4: pre-D3 the resolver error rode
/// a `Result` that the handler `.unwrap()`ed — a room with an unresolvable id
/// panicked the Matrix handler task; now it warns and skips the event
/// (failure-path-only delta, D1/D2c precedent).
async fn gather_contact_event(event: &SyncRoomMemberEvent, room: &Room, hashes: &RoomHashCache) -> Option<GatheredContactEvent> {
    let event_is_about_me = event.state_key() == room.own_user_id();
    if !event_is_about_me {
        return None;
    }

    match room_to_msn_user(room, hashes).await {
        Ok(room_user) => Some(GatheredContactEvent { membership: event.membership().clone(), room_user }),
        Err(error) => {
            warn!("Matrix: could not resolve room {} to an MSN user: {} — skipping contact event", room.room_id(), error);
            None
        }
    }
}

/// Decide (§3): the membership → ABCH contact-row table, pre-D3 semantics
/// verbatim. Every row is about the room itself, as an MSN user.
pub(crate) fn decide_contacts(gathered: GatheredContactEvent) -> Vec<AddressBookContact> {
    let room_user = &gathered.room_user;

    match gathered.membership {
        MembershipState::Ban => {
            //I Got Ban
            vec![AddressBookContact::Contact(ContactType::new(room_user, ContactTypeEnum::LivePending, false))]
        }
        MembershipState::Leave => {
            //I Bailed
            vec![AddressBookContact::Contact(ContactType::new(room_user, ContactTypeEnum::Live, true))]
        }
        MembershipState::Join => {
            //I'm joined
            vec![AddressBookContact::Contact(ContactType::new(room_user, ContactTypeEnum::Live, false))]
        }
        // memberships we don't care about.
        _ => vec![],
    }
}

pub(super) async fn handle_contacts_stripped(
    event: StrippedRoomMemberEvent,
    room: Room,
    tachyon_client: TachyonClient,
    client: Client,
) {
    let is_space = room.room_type().is_some_and(|room_type| matches!(room_type, RoomType::Space));
    if is_space {
        return;
    }

    let Some(gathered) = gather_stripped_contact_event(&event, &room, tachyon_client.room_hashes()).await else {
        return;
    };

    apply_contacts(decide_contacts_stripped(gathered), &tachyon_client);
}

async fn gather_stripped_contact_event(event: &StrippedRoomMemberEvent, room: &Room, hashes: &RoomHashCache) -> Option<GatheredStrippedContactEvent> {
    let event_is_about_me = event.state_key == room.own_user_id();
    if !event_is_about_me {
        return None;
    }

    match room_to_msn_user(room, hashes).await {
        Ok(room_user) => Some(GatheredStrippedContactEvent { state: room.state(), room_user }),
        Err(error) => {
            warn!("Matrix: could not resolve room {} to an MSN user: {} — skipping stripped contact event", room.room_id(), error);
            None
        }
    }
}

/// Decide (§3): the room-state → contact-row table for stripped events.
pub(crate) fn decide_contacts_stripped(gathered: GatheredStrippedContactEvent) -> Vec<AddressBookContact> {
    let room_user = &gathered.room_user;

    match gathered.state {
        RoomState::Invited => {
            // I'm invited
            //We do nothing here, invitations are handled in memberships
            vec![]
        }
        RoomState::Knocked => {
            // I Knocked
            vec![AddressBookContact::Contact(ContactType::new(room_user, ContactTypeEnum::LivePending, false))]
        }
        // memberships we don't care about.
        _ => vec![],
    }
}

/// The full-list rebuild for the ABCH SOAP path — gather + decide over every
/// room's own-member state event, same table as the live handlers. The hash
/// cache rides along from the calling account's owned state (§3.5).
pub async fn compute_all_contacts(client: Client, hashes: &RoomHashCache) -> Vec<AddressBookContact> {
    let mut out = Vec::new();
    for room in client.rooms() {
        if let Ok(Some(event)) = room.get_state_event_static_for_key::<RoomMemberEventContent, _>(room.own_user_id()).await {
            match event {
                RawSyncOrStrippedState::Sync(sync) => {
                    let deserialized = sync.deserialize().unwrap();
                    if let Some(gathered) = gather_contact_event(&deserialized, &room, hashes).await {
                        out.append(&mut decide_contacts(gathered));
                    }
                }
                RawSyncOrStrippedState::Stripped(stripped) => {
                    let deserialized = stripped.deserialize().unwrap();
                    if let Some(gathered) = gather_stripped_contact_event(&deserialized, &room, hashes).await {
                        out.append(&mut decide_contacts_stripped(gathered));
                    }
                }
            }
        }
    }

    out
}

pub(super) async fn handle_tombstone(    event: OriginalSyncRoomTombstoneEvent,
                               room: Room,
                               tachyon_client: TachyonClient,
                               client: Client) {

    // §1.4: pre-D3 this was `room.to_msn_user().await.unwrap()` — an
    // unresolvable room panicked the Matrix handler task. Now: warn + skip
    // the event (failure-path-only delta).
    let room_user = match room_to_msn_user(&room, tachyon_client.room_hashes()).await {
        Ok(room_user) => room_user,
        Err(error) => {
            warn!("Matrix: could not resolve room {} to an MSN user: {} — skipping tombstone", room.room_id(), error);
            return;
        }
    };

    apply_contacts(decide_contact_tombstone(room_user), &tachyon_client);
}

/// Decide — honest passthrough (§5): every tombstone decides the same single
/// deleted-live row for the replaced room; there is no decision input beyond
/// the gathered room user.
pub(crate) fn decide_contact_tombstone(room_user: MsnUser) -> Vec<AddressBookContact> {
    vec![AddressBookContact::Contact(ContactType::new(&room_user, ContactTypeEnum::Live, true))]
}

/// Apply (§3): the family's single sink — the account's SoapHolder contact
/// list. Lock + append semantics are pre-D3 verbatim (an empty decision
/// never touched the lock, and still doesn't).
pub(crate) fn apply_contacts(mut contacts: Vec<AddressBookContact>, tachyon_client: &TachyonClient) {
    if contacts.is_empty() {
        return;
    }

    let mut contact_holder = tachyon_client.soap_holder().contacts.lock().unwrap();
    contact_holder.append(&mut contacts);
}

#[cfg(test)]
mod tests {
    use std::str::FromStr;

    use matrix_sdk::RoomState;
    use matrix_sdk::ruma::events::room::member::MembershipState;
    use msnp::shared::models::email_address::EmailAddress;
    use msnp::shared::models::endpoint_id::EndpointId;
    use msnp::shared::models::msn_user::MsnUser;
    use msnp::soap::abch::msnab_datatypes::{ContactInfoType, ContactType};

    use super::{decide_contact_tombstone, decide_contacts, decide_contacts_stripped, GatheredContactEvent, GatheredStrippedContactEvent};
    use crate::notification::models::soap_holder::AddressBookContact;

    fn room_user() -> MsnUser {
        let mut user = MsnUser::new(EndpointId::new(EmailAddress::from_str("roomhash@test.local").unwrap(), None));
        user.display_name = Some("The Room".to_string());
        user
    }

    /// ContactTypeEnum carries no PartialEq — assert on stable names.
    fn contact_type_name(info: &ContactInfoType) -> &'static str {
        match info.contact_type {
            Some(msnp::soap::abch::msnab_datatypes::ContactTypeEnum::Regular) => "Regular",
            Some(msnp::soap::abch::msnab_datatypes::ContactTypeEnum::Me) => "Me",
            Some(msnp::soap::abch::msnab_datatypes::ContactTypeEnum::Live) => "Live",
            Some(msnp::soap::abch::msnab_datatypes::ContactTypeEnum::LivePending) => "LivePending",
            Some(msnp::soap::abch::msnab_datatypes::ContactTypeEnum::Circle) => "Circle",
            None => "None",
        }
    }

    fn single_contact(rows: Vec<AddressBookContact>) -> ContactType {
        assert_eq!(rows.len(), 1, "expected exactly one decided row");
        let AddressBookContact::Contact(contact) = rows.into_iter().next().unwrap() else {
            panic!("the contact family decides contact rows, never circles");
        };
        contact
    }

    #[test]
    fn ban_leave_and_join_on_my_own_membership_decide_the_contact_rows() {
        // The pre-D3 table, verbatim: a ban leaves a pending row, bailing
        // deletes the live row, joining creates it.
        let table = vec![
            (MembershipState::Ban, "LivePending", false),
            (MembershipState::Leave, "Live", true),
            (MembershipState::Join, "Live", false),
        ];

        for (membership, expected_type, expected_deleted) in table {
            let contact = single_contact(decide_contacts(GatheredContactEvent { membership, room_user: room_user() }));

            let info = contact.contact_info.as_ref().expect("a decided contact carries its info");
            assert_eq!(contact_type_name(info), expected_type);
            assert_eq!(contact.f_deleted, Some(expected_deleted));
            assert_eq!(info.passport_name.as_deref(), Some("roomhash@test.local"));
            assert_eq!(info.display_name.as_deref(), Some("The Room"));
        }
    }

    #[test]
    fn other_sync_memberships_decide_no_contact_row() {
        // Pre-D3: "memberships we don't care about" — invite/knock wait for
        // the stripped event, everything else decides nothing.
        for membership in [MembershipState::Invite, MembershipState::Knock] {
            let rows = decide_contacts(GatheredContactEvent { membership, room_user: room_user() });
            assert!(rows.is_empty());
        }
    }

    #[test]
    fn a_knocked_room_decides_the_pending_contact_row() {
        let contact = single_contact(decide_contacts_stripped(GatheredStrippedContactEvent {
            state: RoomState::Knocked,
            room_user: room_user(),
        }));

        let info = contact.contact_info.as_ref().expect("a decided contact carries its info");
        assert_eq!(contact_type_name(info), "LivePending");
        assert_eq!(contact.f_deleted, Some(false));
    }

    #[test]
    fn an_invited_room_decides_no_contact_row_because_memberships_own_invites() {
        // §5 honesty: this empty result is the pre-D3 comment made behavior —
        // "We do nothing here, invitations are handled in memberships". The
        // membership family's stripped decide is the half that decides.
        let rows = decide_contacts_stripped(GatheredStrippedContactEvent {
            state: RoomState::Invited,
            room_user: room_user(),
        });
        assert!(rows.is_empty());
    }

    #[test]
    fn every_tombstone_decides_the_same_deleted_live_contact() {
        // §5 honesty: passthrough — the tombstone decide has no input beyond
        // the room user and always decides the deleted live row.
        let contact = single_contact(decide_contact_tombstone(room_user()));

        let info = contact.contact_info.as_ref().expect("a decided contact carries its info");
        assert_eq!(contact_type_name(info), "Live");
        assert_eq!(contact.f_deleted, Some(true));
    }
}
