use tokio::sync::broadcast::{Receiver, Sender};
use msnp::msnp::notification::models::endpoint_data::PrivateEndpointData;
use msnp::shared::models::email_address::EmailAddress;
use msnp::shared::models::client_version::ClientVersion;
use msnp::shared::models::ticket_token::TicketToken;
use crate::tachyon::client::tachyon_client::TachyonClient;
use crate::notification::models::connection_phase::ConnectionPhase;
use crate::tachyon::global_state::ClientDropGuard;

pub(crate) struct LocalClientData {
    pub(crate) phase: ConnectionPhase,
    pub(crate) email_addr: EmailAddress,
    pub(crate) token: TicketToken,
    /// From `CVR`, so `None` until the client has introduced itself.
    pub(crate) client_version: Option<ClientVersion>,
    pub(crate) tachyon_client: Option<TachyonClient>,
    pub(crate) matrix_client: Option<matrix_sdk::Client>,
    pub(crate) private_endpoint_data: PrivateEndpointData,
    pub(crate) needs_initial_presence: bool,
    pub(crate) client_shutdown_recv: Receiver<()>,
    pub(crate) client_shutdown_snd: Sender<()>,
    pub(crate) client_drop_guard: Option<ClientDropGuard>
}

impl LocalClientData {
    pub(crate) fn new(client_shutdown_snd: Sender<()>, client_shutdown_recv: Receiver<()>) -> Self {
        Self {
            phase: ConnectionPhase::default(),
            email_addr: EmailAddress::default(),
            token: TicketToken::default(),
            client_version: None,
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