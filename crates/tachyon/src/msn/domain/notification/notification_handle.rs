use msnp::msnp::notification::command::command::NotificationServerCommand;
use std::time::Duration;
use tokio::sync::mpsc;
use tokio::sync::mpsc::error::SendTimeoutError;

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