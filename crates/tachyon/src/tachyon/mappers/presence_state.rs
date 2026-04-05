use matrix_sdk::ruma::presence::PresenceState;
use msnp::shared::models::presence_status::PresenceStatus;

pub trait PresenceStateMapper {
    fn from_presence_state(presence_state: PresenceState) -> PresenceStatus;
    fn into_presence_state(self) -> PresenceState;
}

impl PresenceStateMapper for PresenceStatus {
    fn from_presence_state(presence_state: PresenceState) -> PresenceStatus {
        match presence_state {
            PresenceState::Online => {
                PresenceStatus::NLN
            },
            PresenceState::Unavailable => {
                PresenceStatus::AWY
            },
            PresenceState::Offline => {
                PresenceStatus::default()
            }
            _ => {
                PresenceStatus::default()
            }
        }    }

    fn into_presence_state(self) -> PresenceState {
        match self {
            PresenceStatus::NLN => {
                PresenceState::Online
            },
            PresenceStatus::HDN | PresenceStatus::FLN => {
                PresenceState::Offline
            },
            _ => {
                PresenceState::Unavailable
            }
        }    }
}