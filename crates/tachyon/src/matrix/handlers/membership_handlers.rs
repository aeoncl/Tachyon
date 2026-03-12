use matrix_sdk::{event_handler::Ctx, ruma::events::room::member::{StrippedRoomMemberEvent, SyncRoomMemberEvent}, Client, Room, RoomState};
use matrix_sdk::ruma::events::room::member::MembershipState;
use msnp::shared::models::role_list::RoleList;
use msnp::soap::abch::msnab_datatypes::{BaseMember, ContactType, ContactTypeEnum, MemberState};
use crate::matrix::extensions::msn_user_resolver::ToMsnUser;
use crate::matrix::handlers::context::TachyonContext;
use crate::notification::models::soap_holder::AddressBookContact;

pub async fn handle_memberships(
    event: SyncRoomMemberEvent,
    room: Room,
    context: Ctx<TachyonContext>,
    client: Client,
) {

    let event_is_about_me =
        event.state_key() == client.user_id().expect("UserId to be known while syncing");
    
    if event_is_about_me {

        let msn_user = room.to_msn_user().await.unwrap();
        let mut member_holder = context.client_data.soap_holder().memberships.lock().unwrap();
        
        match event.membership() {
                MembershipState::Ban => {
                    let delete_allow_member = BaseMember::new_passport_member(&msn_user, MemberState::Accepted, RoleList::Allow, true);
                    member_holder.push_back(delete_allow_member);
                    let delete_pending_member = BaseMember::new_passport_member(&msn_user, MemberState::Accepted, RoleList::Pending, true);
                    member_holder.push_back(delete_pending_member);
                    let delete_reverse_member = BaseMember::new_passport_member(&msn_user, MemberState::Accepted, RoleList::Reverse, true);
                    member_holder.push_back(delete_reverse_member);

                }
                MembershipState::Join => {
                    let allow_member = BaseMember::new_passport_member(&msn_user, MemberState::Accepted, RoleList::Allow, false);
                    member_holder.push_back(allow_member);
                    let reverse_member = BaseMember::new_passport_member(&msn_user, MemberState::Accepted, RoleList::Reverse, false);
                    member_holder.push_back(reverse_member);

                }
                MembershipState::Leave => {
                    let delete_allow_member = BaseMember::new_passport_member(&msn_user, MemberState::Accepted, RoleList::Allow, true);
                    member_holder.push_back(delete_allow_member);

                }
                MembershipState::Invite | MembershipState::Knock => {
                    //Invite & Knocks are handled in stripped state
                }
                _ => {}
            }
    }
}

pub async fn handle_memberships_stripped(
    event: StrippedRoomMemberEvent,
    room: Room,
    context: Ctx<TachyonContext>,
    client: Client,
) {

    let event_is_about_me =
        event.state_key == client.user_id().expect("UserId to be known while syncing");

    if event_is_about_me {

        let msn_user = room.to_msn_user().await.unwrap();
        let mut member_holder = context.client_data.soap_holder().memberships.lock().unwrap();


        match room.state() {
            RoomState::Invited => {
                // I'm invited
                let invite_member = BaseMember::new_invite_passsport_member(&msn_user, event.content.reason.clone() ,false);
                member_holder.push_back(invite_member);
                let reverse_member = BaseMember::new_passport_member(&msn_user, MemberState::Accepted, RoleList::Reverse, false);
                member_holder.push_back(reverse_member);
            }
            RoomState::Knocked => {
                // I Knocked
                let allow_member = BaseMember::new_passport_member(&msn_user, MemberState::Accepted, RoleList::Allow, false);
                member_holder.push_back(allow_member);
            }
            _ => {
                // memberships we don't care about.
            }
        }
    }

}
