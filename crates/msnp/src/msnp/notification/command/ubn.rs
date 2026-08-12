use crate::msnp::notification::command::uun::{UserNotificationType, UunPayload};
use crate::msnp::raw_command_parser::RawCommand;
use crate::shared::models::endpoint_id::EndpointId;
use crate::shared::traits::{IntoBytes, TryFromRawCommand};

pub type UbnPayload = UunPayload;
pub type UserNotificationTypeServer = UserNotificationType;


pub struct UbnServer {
    pub source: EndpointId,
    pub payload: UbnPayload
}

impl UbnServer {
    pub fn new(source: EndpointId, payload: UbnPayload) -> Self {
        Self {
            source,
            payload,
        }
    }
}

impl TryFromRawCommand for UbnServer {
    type Err = anyhow::Error;

    fn try_from_raw(value: RawCommand) -> Result<Self, Self::Err> {
        todo!()
    }
}

impl IntoBytes for UbnServer {
    fn into_bytes(self) -> Vec<u8> {

        let notification_type = UserNotificationTypeServer::from(&self.payload) as u32;

        let mut payload = self.payload.into_bytes();

        let mut cmd = format!("UBN {source} {notification_type} {payload_len}\r\n", source = self.source, notification_type = notification_type, payload_len = payload.len()).into_bytes();
        cmd.append(&mut payload);
        cmd
    }
}

