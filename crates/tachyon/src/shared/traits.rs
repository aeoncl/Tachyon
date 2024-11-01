use matrix_sdk::ruma::presence::PresenceState;
use msnp::shared::models::email_address::EmailAddress;
use msnp::shared::models::presence_status::PresenceStatus;
use msnp::shared::models::uuid::Uuid;

pub trait ToUuid {
    fn to_uuid(&self) -> Uuid;
}

impl ToUuid for EmailAddress {
    fn to_uuid(&self) -> Uuid {
        Uuid::from_seed(self.as_str())
    }
}

impl ToUuid for &EmailAddress {
    fn to_uuid(&self) -> Uuid {
        Uuid::from_seed(self.as_str())
    }
}

pub trait PresenceStateCompatible {
    fn from_presence_state(presence_state: PresenceState) -> PresenceStatus;
    fn into_presence_state(self) -> PresenceState;
}

impl PresenceStateCompatible for PresenceStatus {
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