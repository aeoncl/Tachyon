use msnp::msnp::notification::models::dialect::Dialect;
use msnp::msnp::notification::models::endpoint_data::PrivateEndpointData;
use msnp::shared::models::email_address::EmailAddress;
use msnp::shared::models::ticket_token::TicketToken;
use crate::tachyon::client::tachyon_client::TachyonClient;
use crate::notification::models::connection_phase::ConnectionPhase;

/// Decision inputs for one notification connection. The shell-owned fields
/// (shutdown channels, drop guard) live in `ConnectionShell`
/// (notification_server.rs).
pub(crate) struct SessionState {
    pub(crate) phase: ConnectionPhase,
    pub(crate) email_addr: EmailAddress,
    pub(crate) token: TicketToken,
    pub(crate) tachyon_client: Option<TachyonClient>,
    pub(crate) matrix_client: Option<matrix_sdk::Client>,
    pub(crate) private_endpoint_data: PrivateEndpointData,
    pub(crate) needs_initial_presence: bool,
    /// Negotiated once at VER and stored only here (§3.3). Defaults to the
    /// single profiled dialect so pre-negotiation flows (a CVR before any
    /// VER, pinned by the D0 fixture) behave exactly as before.
    pub(crate) dialect: Dialect,
}

impl SessionState {
    pub(crate) fn new() -> Self {
        Self {
            phase: ConnectionPhase::default(),
            email_addr: EmailAddress::default(),
            token: TicketToken::default(),
            tachyon_client: None,
            matrix_client: None,
            private_endpoint_data: Default::default(),
            needs_initial_presence: true,
            dialect: Dialect::default(),
        }
    }
}

