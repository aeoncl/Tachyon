use crate::matrix::extensions::msn_user_resolver::ToMsnUser;
use crate::notification::models::soap_holder::AddressBookContact;
use crate::tachyon::tachyon_client::TachyonClient;
use matrix_sdk::deserialized_responses::RawSyncOrStrippedState;
use matrix_sdk::ruma::events::room::member::{MembershipState, RoomMemberEventContent, StrippedRoomMemberEvent, SyncRoomMemberEvent};
use matrix_sdk::ruma::room::RoomType;
use matrix_sdk::{Client, Room, RoomState};
use msnp::soap::abch::msnab_datatypes::{ContactType, ContactTypeEnum};

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

    let mut contacts = compute_contacts(&event, &room).await.unwrap();

    if !contacts.is_empty() {
        let mut contact_holder = tachyon_client.soap_holder().contacts.lock().unwrap();
        contact_holder.append(&mut contacts);
    }

}

async fn compute_contacts(event: &SyncRoomMemberEvent, room: &Room) -> Result<Vec<AddressBookContact>, anyhow::Error> {
    let mut out = Vec::new();
    let event_is_about_me = event.state_key() == room.own_user_id();

    if event_is_about_me {

        let room_msn_user = room.to_msn_user().await?;

        match event.membership() {
            MembershipState::Ban => {
                //I Got Ban
                let contact = ContactType::new(&room_msn_user, ContactTypeEnum::LivePending, false);
                out.push( AddressBookContact::Contact(contact));
            }
            MembershipState::Leave => {
                //I Bailed
                let contact = ContactType::new(&room_msn_user, ContactTypeEnum::Live, true);
                out.push( AddressBookContact::Contact(contact));
            }
            MembershipState::Join => {
                //I'm joined
                let contact = ContactType::new(&room_msn_user, ContactTypeEnum::Live, false);
                out.push( AddressBookContact::Contact(contact));
            }
            _ => {
                // memberships we don't care about.
            }
        };

    }

    Ok(out)
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


    let mut contacts = compute_contacts_from_stripped_event(&event, &room).await.unwrap();

    if !contacts.is_empty() {
        let mut contact_holder = tachyon_client.soap_holder().contacts.lock().unwrap();
        contact_holder.append(&mut contacts);
    }

}

async fn compute_contacts_from_stripped_event(event: &StrippedRoomMemberEvent, room: &Room) ->  Result<Vec<AddressBookContact>, anyhow::Error> {
    let mut out = Vec::new();

    let event_is_about_me = event.state_key == room.own_user_id();

    if event_is_about_me {

        let msn_user = room.to_msn_user().await?;

        match room.state() {
            RoomState::Invited => {
                // I'm invited
                //We do nothing here, invitations are handled in memberships
            }
            RoomState::Knocked => {
                // I Knocked
                let contact = ContactType::new(&msn_user, ContactTypeEnum::LivePending, false);
                out.push( AddressBookContact::Contact(contact));
            }
            _ => {
                // memberships we don't care about.
            }
        }
    }

    Ok(out)

}

pub async fn compute_all_contacts(client: Client) -> Vec<AddressBookContact> {
    let mut out = Vec::new();

    for room in client.rooms() {
        if let Ok(Some(event)) = room.get_state_event_static_for_key::<RoomMemberEventContent, _>(room.own_user_id()).await {
            match event {
                RawSyncOrStrippedState::Sync(sync) => {
                    let deserialized = sync.deserialize().unwrap();
                    let mut memberships = compute_contacts(&deserialized, &room).await.unwrap();
                    out.append(&mut memberships);

                }
                RawSyncOrStrippedState::Stripped(stripped) => {
                    let deserialized = stripped.deserialize().unwrap();
                    let mut stripped_memberships = compute_contacts_from_stripped_event(&deserialized, &room).await.unwrap();
                    out.append(&mut stripped_memberships);
                }
            }
        }
    }

    out
}