use msnp::msnp::notification::models::endpoint_data::PrivateEndpointData;
use msnp::shared::models::email_address::EmailAddress;
use msnp::shared::models::ticket_token::TicketToken;
use crate::notification::client_store::ClientData;
use crate::notification::models::connection_phase::ConnectionPhase;

pub(crate) struct LocalClientData {
    pub(crate) phase: ConnectionPhase,
    pub(crate) email_addr: EmailAddress,
    pub(crate) token: TicketToken,
    pub(crate) client_data: Option<ClientData>,
    pub(crate) private_endpoint_data: PrivateEndpointData,
    pub(crate) needs_initial_presence: bool
}

impl Default for LocalClientData {
    fn default() -> Self {
        Self {
            phase: ConnectionPhase::default(),
            email_addr: EmailAddress::default(),
            token: TicketToken(String::new()),
            client_data: None,
            private_endpoint_data: Default::default(),
            needs_initial_presence: true,
        }
    }
}