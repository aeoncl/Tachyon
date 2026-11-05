//! Matrix membership family (§8 D3): the closures in `handlers/mod.rs` stay
//! thin shells; this module owns the gather → decide → apply pipeline (§3).
//! gather reads the SDK (about-me filter, room → MSN user resolution) into
//! plain facts; decide is the pure membership/room-state → ABCH BaseMember
//! table; apply owns the single sink — the account's `SoapHolder` membership
//! list. The space filter stays shell policy, exactly where it was pre-D3.
use crate::matrix::extensions::msn_user_resolver::ToMsnUser;
use crate::tachyon::client::tachyon_client::TachyonClient;
use log::warn;
use matrix_sdk::deserialized_responses::RawSyncOrStrippedState;
use matrix_sdk::ruma::events::room::member::{MembershipState, RoomMemberEventContent};
use matrix_sdk::ruma::room::RoomType;
use matrix_sdk::{ruma::events::room::member::{StrippedRoomMemberEvent, SyncRoomMemberEvent}, Client, Room, RoomState};
use matrix_sdk::ruma::events::room::tombstone::OriginalSyncRoomTombstoneEvent;
use msnp::shared::models::msn_user::MsnUser;
use msnp::shared::models::role_list::RoleList;
use msnp::soap::abch::msnab_datatypes::{BaseMember, MemberState};

/// Gathered facts for one about-me membership event — plain data only (§10).
pub(crate) struct GatheredMembershipEvent {
    pub(crate) membership: MembershipState,
    /// The room itself as an MSN user — the member rows are about the ROOM.
    pub(crate) room_user: MsnUser,
}

/// The stripped-event counterpart: no membership on the event itself, so the
/// room state is the fact decide reads; the invite reason rides along for the
/// invited-room annotation.
pub(crate) struct GatheredStrippedMembershipEvent {
    pub(crate) state: RoomState,
    pub(crate) reason: Option<String>,
    pub(crate) room_user: MsnUser,
}

pub(super) async fn handle_memberships(
    event: SyncRoomMemberEvent,
    room: Room,
    tachyon_client: TachyonClient,
    client: Client,
) {
    let is_space = room.room_type().is_some_and(|room_type| matches!(room_type, RoomType::Space));
    if is_space {
        return;
    }

    let Some(gathered) = gather_membership_event(&event, &room).await else {
        return;
    };

    apply_memberships(decide_memberships(gathered), &tachyon_client);
}

/// Gather (§3): the about-me filter and the room → MSN-user resolution, into
/// plain facts. `None` = skip the event. §1.4: pre-D3 the resolver error rode
/// a `Result` that the handler `.unwrap()`ed — a room with an unresolvable id
/// panicked the Matrix handler task; now it warns and skips the event
/// (failure-path-only delta, D1/D2c precedent).
async fn gather_membership_event(event: &SyncRoomMemberEvent, room: &Room) -> Option<GatheredMembershipEvent> {
    let event_is_about_me = event.state_key() == room.own_user_id();
    if !event_is_about_me {
        return None;
    }

    match room.to_msn_user().await {
        Ok(room_user) => Some(GatheredMembershipEvent { membership: event.membership().clone(), room_user }),
        Err(error) => {
            warn!("Matrix: could not resolve room {} to an MSN user: {} — skipping membership event", room.room_id(), error);
            None
        }
    }
}

/// Decide (§3): the membership → BaseMember table, pre-D3 semantics verbatim.
pub(crate) fn decide_memberships(gathered: GatheredMembershipEvent) -> Vec<BaseMember> {
    let msn_user = &gathered.room_user;

    match gathered.membership {
        MembershipState::Ban => {
            vec![
                BaseMember::new_passport_member(msn_user, MemberState::Accepted, RoleList::Allow, true),
                BaseMember::new_passport_member(msn_user, MemberState::Accepted, RoleList::Pending, true),
                BaseMember::new_passport_member(msn_user, MemberState::Accepted, RoleList::Reverse, true),
            ]
        }

        MembershipState::Join => {
            vec![
                BaseMember::new_passport_member(msn_user, MemberState::Accepted, RoleList::Allow, false),
                BaseMember::new_passport_member(msn_user, MemberState::Accepted, RoleList::Reverse, false),
            ]
        }

        MembershipState::Leave => {
            vec![
                BaseMember::new_passport_member(msn_user, MemberState::Accepted, RoleList::Allow, true),
            ]
        }
        MembershipState::Invite | MembershipState::Knock => {
            //Invite & Knocks are handled in stripped state
            vec![]
        }
        _ => vec![],
    }
}

pub(super) async fn handle_memberships_stripped(
    event: StrippedRoomMemberEvent,
    room: Room,
    tachyon_client: TachyonClient,
    client: Client,
) {
    let is_space = room.room_type().is_some_and(|room_type| matches!(room_type, RoomType::Space));
    if is_space {
        return;
    }

    let Some(gathered) = gather_stripped_membership_event(&event, &room).await else {
        return;
    };

    apply_memberships(decide_memberships_stripped(gathered), &tachyon_client);
}

async fn gather_stripped_membership_event(event: &StrippedRoomMemberEvent, room: &Room) -> Option<GatheredStrippedMembershipEvent> {
    let event_is_about_me = event.state_key == room.own_user_id();
    if !event_is_about_me {
        return None;
    }

    match room.to_msn_user().await {
        Ok(room_user) => Some(GatheredStrippedMembershipEvent {
            state: room.state(),
            reason: event.content.reason.clone(),
            room_user,
        }),
        Err(error) => {
            warn!("Matrix: could not resolve room {} to an MSN user: {} — skipping stripped membership event", room.room_id(), error);
            None
        }
    }
}

/// Decide (§3): the room-state → BaseMember table for stripped events —
/// invites and knocks are THIS family's half (the contact family's stripped
/// decide answers invites with nothing, on purpose).
pub(crate) fn decide_memberships_stripped(gathered: GatheredStrippedMembershipEvent) -> Vec<BaseMember> {
    let msn_user = &gathered.room_user;

    match gathered.state {
        RoomState::Invited => {
            // I'm invited
            vec![
                BaseMember::new_invite_passsport_member(msn_user, gathered.reason.clone(), false),
                BaseMember::new_passport_member(msn_user, MemberState::Accepted, RoleList::Reverse, false),
            ]
        }
        RoomState::Knocked => {
            // I Knocked
            vec![
                BaseMember::new_passport_member(msn_user, MemberState::Accepted, RoleList::Allow, false),
            ]
        }
        // memberships we don't care about.
        _ => vec![],
    }
}

/// The full-list rebuild for the Sharing SOAP path — gather + decide over
/// every room's own-member state event, same table as the live handlers.
pub async fn compute_all_memberships(client: Client) -> Vec<BaseMember> {
    let mut out = Vec::new();
    for room in client.rooms() {
        if let Ok(Some(event)) = room.get_state_event_static_for_key::<RoomMemberEventContent, _>(room.own_user_id()).await {
            match event {
                RawSyncOrStrippedState::Sync(sync) => {
                    let deserialized = sync.deserialize().unwrap();
                    if let Some(gathered) = gather_membership_event(&deserialized, &room).await {
                        out.append(&mut decide_memberships(gathered));
                    }
                }
                RawSyncOrStrippedState::Stripped(stripped) => {
                    let deserialized = stripped.deserialize().unwrap();
                    if let Some(gathered) = gather_stripped_membership_event(&deserialized, &room).await {
                        out.append(&mut decide_memberships_stripped(gathered));
                    }
                }
            }
        }
    }

    out
}

pub(super) async fn handle_tombstone(event: OriginalSyncRoomTombstoneEvent,
                               room: Room,
                               tachyon_client: TachyonClient,
                               client: Client) {

    // §1.4: pre-D3 this was `room.to_msn_user().await.unwrap()` — an
    // unresolvable room panicked the Matrix handler task. Now: warn + skip
    // the event (failure-path-only delta).
    let room_user = match room.to_msn_user().await {
        Ok(room_user) => room_user,
        Err(error) => {
            warn!("Matrix: could not resolve room {} to an MSN user: {} — skipping tombstone", room.room_id(), error);
            return;
        }
    };

    apply_memberships(decide_membership_tombstone(room_user), &tachyon_client);
}

/// Decide — honest passthrough (§5): every tombstone decides the same three
/// deletions (allow, pending, reverse — pre-D3 push order preserved); there
/// is no decision input beyond the gathered room user.
pub(crate) fn decide_membership_tombstone(room_user: MsnUser) -> Vec<BaseMember> {
    vec![
        BaseMember::new_passport_member(&room_user, MemberState::Accepted, RoleList::Allow, true),
        BaseMember::new_passport_member(&room_user, MemberState::Accepted, RoleList::Pending, true),
        BaseMember::new_passport_member(&room_user, MemberState::Accepted, RoleList::Reverse, true),
    ]
}

/// Apply (§3): the family's single sink — the account's SoapHolder
/// membership list. Lock + append semantics are pre-D3 verbatim (an empty
/// decision never touched the lock, and still doesn't).
pub(crate) fn apply_memberships(mut members: Vec<BaseMember>, tachyon_client: &TachyonClient) {
    if members.is_empty() {
        return;
    }

    let mut member_holder = tachyon_client.soap_holder().memberships.lock().unwrap();
    member_holder.append(&mut members);
}

#[cfg(test)]
mod tests {
    use std::str::FromStr;

    use matrix_sdk::ruma::events::room::member::MembershipState;
    use matrix_sdk::RoomState;
    use msnp::shared::models::email_address::EmailAddress;
    use msnp::shared::models::endpoint_id::EndpointId;
    use msnp::shared::models::msn_user::MsnUser;
    use msnp::shared::models::role_list::RoleList;
    use msnp::soap::abch::msnab_datatypes::{BaseMember, MemberState};

    use super::{decide_memberships, decide_memberships_stripped, decide_membership_tombstone, GatheredMembershipEvent, GatheredStrippedMembershipEvent};

    fn room_user() -> MsnUser {
        let mut user = MsnUser::new(EndpointId::new(EmailAddress::from_str("roomhash@test.local").unwrap(), None));
        user.display_name = Some("The Room".to_string());
        user
    }

    /// MemberState carries no PartialEq — assert on stable names.
    fn state_name(state: &MemberState) -> &'static str {
        match state {
            MemberState::Accepted => "Accepted",
            MemberState::Pending => "Pending",
            MemberState::Declined => "Declined",
            MemberState::Removed => "Removed",
            MemberState::Tentative => "Tentative",
        }
    }

    fn assert_member(member: &BaseMember, role: RoleList, deleted: bool) {
        assert_eq!(member.role_list, role);
        assert_eq!(member.deleted, Some(deleted));
        assert_eq!(state_name(&member.state), "Accepted");
        assert_eq!(member.passport_name.as_deref(), Some("roomhash@test.local"));
        assert_eq!(member.display_name.as_deref(), Some("The Room"));
    }

    #[test]
    fn a_ban_decides_deletions_across_allow_pending_and_reverse() {
        let members = decide_memberships(GatheredMembershipEvent { membership: MembershipState::Ban, room_user: room_user() });

        assert_eq!(members.len(), 3);
        assert_member(&members[0], RoleList::Allow, true);
        assert_member(&members[1], RoleList::Pending, true);
        assert_member(&members[2], RoleList::Reverse, true);
    }

    #[test]
    fn a_join_decides_the_allow_and_reverse_additions() {
        let members = decide_memberships(GatheredMembershipEvent { membership: MembershipState::Join, room_user: room_user() });

        assert_eq!(members.len(), 2);
        assert_member(&members[0], RoleList::Allow, false);
        assert_member(&members[1], RoleList::Reverse, false);
    }

    #[test]
    fn a_leave_decides_only_the_allow_deletion() {
        let members = decide_memberships(GatheredMembershipEvent { membership: MembershipState::Leave, room_user: room_user() });

        assert_eq!(members.len(), 1);
        assert_member(&members[0], RoleList::Allow, true);
    }

    #[test]
    fn invite_and_knock_wait_for_the_stripped_event() {
        // §5 honesty: the empty vecs are the pre-D3 comment made behavior —
        // "Invite & Knocks are handled in stripped state". The stripped
        // decide below is the half that actually decides them.
        for membership in [MembershipState::Invite, MembershipState::Knock] {
            let members = decide_memberships(GatheredMembershipEvent { membership, room_user: room_user() });
            assert!(members.is_empty());
        }
    }

    #[test]
    fn an_invited_room_decides_the_invite_member_with_reason_then_reverse() {
        let members = decide_memberships_stripped(GatheredStrippedMembershipEvent {
            state: RoomState::Invited,
            reason: Some("join my room".to_string()),
            room_user: room_user(),
        });

        assert_eq!(members.len(), 2);

        // The invite row carries the reason as the MSN.IM.InviteMessage
        // annotation and sits in the Pending role.
        assert_member(&members[0], RoleList::Pending, false);
        let annotations = members[0].annotations.as_ref().expect("the invite row carries its annotation");
        assert_eq!(annotations.annotation.len(), 1);
        assert_eq!(annotations.annotation[0].name, "MSN.IM.InviteMessage");
        assert_eq!(annotations.annotation[0].value.as_deref(), Some("join my room"));

        assert_member(&members[1], RoleList::Reverse, false);
        assert!(members[1].annotations.is_none());
    }

    #[test]
    fn a_knocked_room_decides_the_allow_addition() {
        let members = decide_memberships_stripped(GatheredStrippedMembershipEvent {
            state: RoomState::Knocked,
            reason: None,
            room_user: room_user(),
        });

        assert_eq!(members.len(), 1);
        assert_member(&members[0], RoleList::Allow, false);
    }

    #[test]
    fn every_tombstone_decides_the_three_membership_deletions() {
        // §5 honesty: passthrough — the tombstone decide has no input beyond
        // the room user and always decides the same allow/pending/reverse
        // deletions, in the pre-D3 push order.
        let members = decide_membership_tombstone(room_user());

        assert_eq!(members.len(), 3);
        assert_member(&members[0], RoleList::Allow, true);
        assert_member(&members[1], RoleList::Pending, true);
        assert_member(&members[2], RoleList::Reverse, true);
    }
}
