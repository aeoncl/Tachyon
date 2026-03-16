use crate::matrix::extensions::msn_user_resolver::ToMsnUser;
use crate::matrix::handlers::context::TachyonContext;
use crate::notification::models::soap_holder::AddressBookContact;
use matrix_sdk::event_handler::Ctx;
use matrix_sdk::ruma::events::room::member::{
    MembershipState, StrippedRoomMemberEvent, SyncRoomMemberEvent,
};
use matrix_sdk::{Client, Room, RoomState};
use msnp::soap::abch::msnab_datatypes::{ContactType, ContactTypeEnum};
use crate::matrix::extensions::direct::DirectRoom;

pub async fn handle_contacts(
    event: SyncRoomMemberEvent,
    room: Room,
    context: Ctx<TachyonContext>,
    client: Client,
) {

    println!("Handling contact event for room: {}", room.room_id().to_string());


    let event_is_about_me =
        event.state_key() == client.user_id().expect("UserId to be known while syncing");

    if event_is_about_me {
        let room_msn_user = room.to_msn_user().await.unwrap();

        {
            let mut contact_holder = context.client_data.soap_holder().contacts.lock().unwrap();


            match event.membership() {
                MembershipState::Ban => {
                    //I Got Ban
                    let contact = ContactType::new(&room_msn_user, ContactTypeEnum::LivePending, false);
                    contact_holder.push( AddressBookContact::Contact(contact));
                }
                MembershipState::Leave => {
                    //I Bailed
                    let contact = ContactType::new(&room_msn_user, ContactTypeEnum::Live, true);
                    contact_holder.push( AddressBookContact::Contact(contact));
                }
                MembershipState::Join => {
                    //I'm joined
                    let contact = ContactType::new(&room_msn_user, ContactTypeEnum::Live, false);
                    contact_holder.push( AddressBookContact::Contact(contact));
                }
                _ => {
                    // memberships we don't care about.
                }
            };

            println!("ContentHolder: {}", contact_holder.len());
        }
    }
}

pub async fn handle_contacts_stripped(
    event: StrippedRoomMemberEvent,
    room: Room,
    context: Ctx<TachyonContext>,
    client: Client,
) {
    let event_is_about_me =
        event.state_key == client.user_id().expect("UserId to be known while syncing");

    if event_is_about_me {

        let msn_user = room.to_msn_user().await.unwrap();
        let mut contact_holder = context.client_data.soap_holder().contacts.lock().unwrap();

        match room.state() {
            RoomState::Invited => {
                // I'm invited
                //We do nothing here, invitations are handled in memberships
            }
            RoomState::Knocked => {
                // I Knocked
                let contact = ContactType::new(&msn_user, ContactTypeEnum::LivePending, false);
                contact_holder.push( AddressBookContact::Contact(contact));
            }
            _ => {
                // memberships we don't care about.
            }
        }
    }
}
