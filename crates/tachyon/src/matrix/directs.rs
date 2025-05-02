use matrix_sdk::deserialized_responses::RawAnySyncOrStrippedState;
use matrix_sdk::ruma::events::direct::DirectEventContent;
use matrix_sdk::ruma::events::{AnySyncStateEvent, GlobalAccountDataEventType, StateEventType};
use matrix_sdk::ruma::{MilliSecondsSinceUnixEpoch, OwnedRoomId, OwnedUserId, UserId};
use matrix_sdk::{Client, Error, Room, RoomMemberships};


struct RoomWithTimestamp {
    room: Room,
    timestamp: MilliSecondsSinceUnixEpoch
}

pub trait OneOnOneDmClient {
    async fn get_canonical_dm_room(&self, user_id: &UserId) -> Result<Option<OwnedRoomId>, matrix_sdk::Error>;

    async fn force_update_rooms_with_fresh_m_direct(&self) -> Result<(), matrix_sdk::Error>;
}

impl OneOnOneDmClient for Client {
    async fn get_canonical_dm_room(&self, user_id: &UserId) -> Result<Option<OwnedRoomId>, Error> {
        let mut rooms = self.joined_rooms();
        let mut dm_rooms = Vec::new();

        for room in rooms.drain(..) {

            if let Some(direct_target) =  room.get_1o1_direct_target().await? {
                if &direct_target == user_id {
                    let timestamp = room.creation_timestamp().await?;

                    dm_rooms.push(
                        RoomWithTimestamp {
                            room,
                            timestamp,
                        }
                    );
                }
            }
        }

        dm_rooms.sort_by( |a, b|  {
            a.timestamp.cmp(&b.timestamp)
        });

        Ok(dm_rooms.first().map( |room| room.room.room_id().to_owned() ))
    }

    async fn force_update_rooms_with_fresh_m_direct(&self) -> Result<(), Error> {
        if let Some(raw_content) = self.account().fetch_account_data(GlobalAccountDataEventType::Direct).await? {
            let mut e = raw_content.deserialize_as::<DirectEventContent>()?;
            for (mut user_id, rooms) in e.0 {
                for room_id in rooms {
                    let room = self.get_room(&room_id);
                    if let Some(room) = room {
                        room.direct_targets().insert(user_id.clone());
                        room.set_is_direct(true).await?
                    }
                }
            }
        }

        return Ok(())
    }
}

pub trait OneOnOneDmRoom {

    async fn is_room_1o1_direct(&self) -> Result<bool, matrix_sdk::Error>;

    async fn get_1o1_direct_target(&self) -> Result<Option<OwnedUserId>, matrix_sdk::Error>;

    async fn creation_timestamp(&self) -> Result<MilliSecondsSinceUnixEpoch, matrix_sdk::Error>;
}

impl OneOnOneDmRoom for Room {
    async fn is_room_1o1_direct(&self) -> Result<bool, Error> {
        Ok(self.get_1o1_direct_target().await?.is_some())
    }

    async fn get_1o1_direct_target(&self) -> Result<Option<OwnedUserId>, Error> {

        let client = self.client();

        let me_user_id = client.user_id().ok_or(Error::AuthenticationRequired)?;

        if !self.is_direct().await? {
            return Ok(None);
        }

        let members = self.members(RoomMemberships::ACTIVE).await?;

        if members.len() > 2 {
            return Ok(None);
        }

        let direct_targets = self.direct_targets();

        let not_me_direct_targets = direct_targets.iter().filter(|target| target.as_user_id().unwrap() != me_user_id).collect::<Vec<_>>();
        if not_me_direct_targets.is_empty() || not_me_direct_targets.len() > 1 {
            return Ok(None);
        }

        let direct_target = not_me_direct_targets.first().unwrap().as_user_id().unwrap();

        let target = members.iter().find(|member| { member.user_id() != me_user_id && member.user_id() == direct_target }).map(|member| member.user_id().to_owned());

        Ok(target)
    }

    async fn creation_timestamp(&self) -> Result<MilliSecondsSinceUnixEpoch, Error> {
        let room_create_event = self.get_state_event(StateEventType::RoomCreate, "").await?.expect("RoomCreateEvent to be present");

        if let RawAnySyncOrStrippedState::Sync(raw) = room_create_event {
            if let Ok(AnySyncStateEvent::RoomCreate(room_create_event)) = raw.deserialize_as() {
                return Ok(room_create_event.origin_server_ts());
            }
        }

        return Err(Error::InsufficientData);

    }
}
