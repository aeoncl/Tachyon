use std::net::Ipv4Addr;
use msnp::msnp::notification::command::command::NotificationServerCommand;
use std::time::Duration;
use tokio::sync::mpsc;
use tokio::sync::mpsc::error::SendTimeoutError;
use msnp::msnp::notification::command::rng::RngServer;
use msnp::msnp::notification::models::ip_address::IpAddress;
use msnp::msnp::switchboard::models::session_id::SessionId;
use msnp::shared::models::msn_user::MsnUser;
use msnp::shared::models::ticket_token::TicketToken;
use crate::switchboard::models::switchboard_token::SwitchboardToken;

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

    pub async fn request_switchboard(&self, ip: IpAddress, token: TicketToken, inviter: MsnUser) -> Result<(), SendTimeoutError<NotificationServerCommand>> {
        
        let cmd = NotificationServerCommand::RNG(
            RngServer::new(
                SessionId::random(),
                ip,
                token,
                inviter.get_email_address().clone(),
                inviter.compute_display_name().to_string()
            )
        );

        self.send(cmd).await

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