use tokio::sync::broadcast::{Receiver, Sender};
use msnp::msnp::notification::models::endpoint_data::PrivateEndpointData;
use msnp::shared::models::email_address::EmailAddress;
use msnp::shared::models::ticket_token::TicketToken;
use crate::tachyon::client::tachyon_client::TachyonClient;
use crate::app_state::ClientDropGuard;

pub struct LocalClientData {
    pub email_addr: EmailAddress,
    pub token: TicketToken,
    pub tachyon_client: Option<TachyonClient>,
    pub matrix_client: Option<matrix_sdk::Client>,
    pub private_endpoint_data: PrivateEndpointData,
    pub needs_initial_presence: bool,
    pub client_shutdown_recv: Receiver<()>,
    pub client_shutdown_snd: Sender<()>,
    pub client_drop_guard: Option<ClientDropGuard>
}

impl LocalClientData {
    pub fn new(client_shutdown_snd: Sender<()>, client_shutdown_recv: Receiver<()>) -> Self {
        Self {
            email_addr: EmailAddress::default(),
            token: TicketToken::default(),
            tachyon_client: None,
            matrix_client: None,
            private_endpoint_data: Default::default(),
            needs_initial_presence: true,
            client_shutdown_recv,
            client_shutdown_snd,
            client_drop_guard: None
        }
    }
}