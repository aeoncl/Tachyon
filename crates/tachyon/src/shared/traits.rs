use msnp::shared::models::email_address::EmailAddress;
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