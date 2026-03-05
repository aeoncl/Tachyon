use crate::matrix::handlers::context::TachyonContext;
use matrix_sdk::event_handler::Ctx;
use matrix_sdk::{Client, Room};
use ruma::events::room::member::{MembershipChange, RoomMemberEvent};
use ruma::events::room::name::RoomNameEvent;
use ruma::user_id;

pub async fn handle_direct_member_profile_changed(
    event: RoomMemberEvent,
    room: Room,
    context: Ctx<TachyonContext>,
    client: Client,
) {
    if room.is_valid_one_to_one_direct() {
        if let Some(original) = event.as_original() {

            if room.is_valid_one_to_one_direct() {
                if let Some(direct_target) = room.get_single_direct_target() {
                    if event.state_key() == direct_target.as_ref() {
                        if let MembershipChange::ProfileChanged { avatar_url_change, displayname_change } = original.membership_change() {
                            //TODO Send NLN Command with new name and avatar
                        }
                    }
                }
            }
        }
    }
}

//TODO handle when a 101 dm room has more than 1 member, it should not show the user name anymore but the room.display_name
//TODO default display_name computed algorithm should be enough, no need to handle when the user is o1o room.
pub async fn handle_room_name(
    event: RoomNameEvent,
    room: Room,
    context: Ctx<TachyonContext>,
    client: Client,
) {

        if !room.is_valid_one_to_one_direct() {
            if let Some(original) = event.as_original() {
                //TODO Send NLN Command with new Name
                original.content.name;
            }
        }
}
