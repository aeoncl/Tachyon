use crate::matrix::extensions::msn_user_resolver::ToMsnUser;
use crate::tachyon::client::tachyon_client::TachyonClient;
use matrix_sdk::deserialized_responses::RawSyncOrStrippedState;
use matrix_sdk::ruma::events::room::member::{MembershipState, RoomMemberEventContent};
use matrix_sdk::ruma::room::RoomType;
use matrix_sdk::{ruma::events::room::member::{StrippedRoomMemberEvent, SyncRoomMemberEvent}, Client, Room, RoomState};
use matrix_sdk::ruma::events::room::tombstone::{OriginalSyncRoomTombstoneEvent, RoomTombstoneEvent, SyncRoomTombstoneEvent};
use msnp::shared::models::role_list::RoleList;
use msnp::soap::abch::msnab_datatypes::{BaseMember, MemberState};

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

    let mut members = compute_memberships(&event, &room).await.unwrap();

    if !members.is_empty() {
        let mut member_holder = tachyon_client.soap_holder().memberships.lock().unwrap();
        member_holder.append(&mut members);
    }

}

async fn compute_memberships(event: &SyncRoomMemberEvent, room: &Room) -> Result<Vec<BaseMember>, anyhow::Error> {
    let mut out = Vec::new();

    let event_is_about_me = event.state_key() == room.own_user_id();

    if event_is_about_me {

        let msn_user = room.to_msn_user().await?;

        match event.membership() {
            MembershipState::Ban => {
                let delete_allow_member = BaseMember::new_passport_member(&msn_user, MemberState::Accepted, RoleList::Allow, true);
                out.push(delete_allow_member);
                let delete_pending_member = BaseMember::new_passport_member(&msn_user, MemberState::Accepted, RoleList::Pending, true);
                out.push(delete_pending_member);
                let delete_reverse_member = BaseMember::new_passport_member(&msn_user, MemberState::Accepted, RoleList::Reverse, true);
                out.push(delete_reverse_member);
            }

            MembershipState::Join => {
                let allow_member = BaseMember::new_passport_member(&msn_user, MemberState::Accepted, RoleList::Allow, false);
                out.push(allow_member);
                let reverse_member = BaseMember::new_passport_member(&msn_user, MemberState::Accepted, RoleList::Reverse, false);
                out.push(reverse_member);

            }
            MembershipState::Leave => {
                let delete_allow_member = BaseMember::new_passport_member(&msn_user, MemberState::Accepted, RoleList::Allow, true);
                out.push(delete_allow_member);

            }
            MembershipState::Invite | MembershipState::Knock => {
                //Invite & Knocks are handled in stripped state
            }
            _ => {}
        }
    }


    Ok(out)
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
    
    let mut members = compute_memberships_from_stripped_event(&event, &room).await.unwrap();

    if !members.is_empty() {
        let mut member_holder = tachyon_client.soap_holder().memberships.lock().unwrap();
        member_holder.append(&mut members);
    }

}


async fn compute_memberships_from_stripped_event(
    event: &StrippedRoomMemberEvent,
    room: &Room,
) -> Result<Vec<BaseMember>, anyhow::Error> {

    let mut out = Vec::new();

    let event_is_about_me = event.state_key == room.own_user_id();

    if event_is_about_me {

        let msn_user = room.to_msn_user().await?;

        match room.state() {
            RoomState::Invited => {
                // I'm invited
                let invite_member = BaseMember::new_invite_passsport_member(&msn_user, event.content.reason.clone() ,false);
                out.push(invite_member);
                let reverse_member = BaseMember::new_passport_member(&msn_user, MemberState::Accepted, RoleList::Reverse, false);
                out.push(reverse_member);
            }
            RoomState::Knocked => {
                // I Knocked
                let allow_member = BaseMember::new_passport_member(&msn_user, MemberState::Accepted, RoleList::Allow, false);
                out.push(allow_member);
            }
            _ => {
                // memberships we don't care about.
            }
        }
    }

    Ok(out)
}

pub async fn compute_all_memberships(client: Client) -> Vec<BaseMember> {
    let mut out = Vec::new();

    for room in client.rooms() {
        if let Ok(Some(event)) = room.get_state_event_static_for_key::<RoomMemberEventContent, _>(room.own_user_id()).await {
            match event {
                RawSyncOrStrippedState::Sync(sync) => {
                    let deserialized = sync.deserialize().unwrap();
                    let mut memberships = compute_memberships(&deserialized, &room).await.unwrap();
                    out.append(&mut memberships);

                }
                RawSyncOrStrippedState::Stripped(stripped) => {
                    let deserialized = stripped.deserialize().unwrap();
                    let mut stripped_memberships = compute_memberships_from_stripped_event(&deserialized, &room).await.unwrap();
                    out.append(&mut stripped_memberships);
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

    let room_msn_user = room.to_msn_user().await.unwrap();


    let delete_allow_member = BaseMember::new_passport_member(&room_msn_user, MemberState::Accepted, RoleList::Allow, true);
    let delete_pending_member = BaseMember::new_passport_member(&room_msn_user, MemberState::Accepted, RoleList::Pending, true);
    let delete_reverse_member = BaseMember::new_passport_member(&room_msn_user, MemberState::Accepted, RoleList::Reverse, true);

    let mut member_holder = tachyon_client.soap_holder().memberships.lock().unwrap();
    member_holder.push(delete_allow_member);
    member_holder.push(delete_pending_member);
    member_holder.push(delete_reverse_member);

}