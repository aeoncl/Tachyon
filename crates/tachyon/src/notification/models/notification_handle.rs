use std::net::Ipv4Addr;
use std::time::Duration;
use matrix_sdk::ruma::RoomId;
use tokio::sync::mpsc;
use msnp::msnp::notification::command::command::NotificationServerCommand;
use tokio::sync::mpsc::error::SendTimeoutError;
use msnp::msnp::notification::command::rng::RngServer;
use msnp::msnp::notification::models::ip_address::IpAddress;
use msnp::msnp::switchboard::models::session_id::SessionId;
use msnp::shared::models::msn_user::MsnUser;

#[derive(Clone)]
pub struct NotificationHandle {
    notification_sender: mpsc::Sender<NotificationServerCommand>,
}

impl NotificationHandle {
    pub fn new(notification_sender: mpsc::Sender<NotificationServerCommand>) -> Self {
        NotificationHandle {
            notification_sender,
        }
    }

    pub async fn send(
        &self,
        notification: NotificationServerCommand,
    ) -> Result<(), SendTimeoutError<NotificationServerCommand>> {
        self.notification_sender
            .send_timeout(notification, Duration::from_secs(5))
            .await
    }
}