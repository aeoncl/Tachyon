use tokio::sync::broadcast::{Receiver, Sender};
use msnp::msnp::notification::models::endpoint_data::PrivateEndpointData;
use msnp::shared::models::email_address::EmailAddress;
use msnp::shared::models::ticket_token::TicketToken;
use crate::tachyon::tachyon_client::TachyonClient;
use crate::notification::models::connection_phase::ConnectionPhase;

pub(crate) struct LocalClientData {
    pub(crate) phase: ConnectionPhase,
    pub(crate) email_addr: EmailAddress,
    pub(crate) token: TicketToken,
    pub(crate) tachyon_client: Option<TachyonClient>,
    pub(crate) matrix_client: Option<matrix_sdk::Client>,
    pub(crate) private_endpoint_data: PrivateEndpointData,
    pub(crate) needs_initial_presence: bool,
    pub(crate) client_kill_recv: Receiver<()>,
    pub(crate) client_kill_snd: Sender<()>
}

impl LocalClientData {
    pub(crate) fn new(client_kill_snd: Sender<()>, client_kill_recv: Receiver<()>) -> Self {
        Self {
            phase: ConnectionPhase::default(),
            email_addr: EmailAddress::default(),
            token: TicketToken::default(),
            tachyon_client: None,
            matrix_client: None,
            private_endpoint_data: Default::default(),
            needs_initial_presence: true,
            client_kill_recv,
            client_kill_snd,
        }
    }
}