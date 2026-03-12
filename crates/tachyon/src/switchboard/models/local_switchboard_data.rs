use matrix_sdk::Room;
use tokio::sync::broadcast::Receiver;
use msnp::msnp::notification::models::endpoint_guid::EndpointGuid;
use msnp::shared::models::email_address::EmailAddress;
use msnp::shared::models::endpoint_id::EndpointId;
use msnp::shared::models::ticket_token::TicketToken;
use crate::notification::models::client_data::ClientData;
use crate::switchboard::models::connection_phase::ConnectionPhase;

pub struct LocalSwitchboardData {
    pub(crate) phase: ConnectionPhase,
    pub(crate) email_addr: EmailAddress,
    pub(crate) endpoint_guid: Option<EndpointGuid>,
    pub(crate) token: TicketToken,
    pub(crate) client_data: Option<ClientData>,
    pub(crate) client_kill_recv: Receiver<()>,
    pub(crate) room: Option<Room>,
    pub(crate) session_id: u16
}

impl LocalSwitchboardData {
    pub fn new(client_kill_recv: Receiver<()>) -> Self {
        Self {
            phase: ConnectionPhase::default(),
            email_addr: EmailAddress::default(),
            endpoint_guid: None,
            token: TicketToken::default(),
            client_data: None,
            client_kill_recv,
            room: None,
            session_id: 0,
        }
    }
}