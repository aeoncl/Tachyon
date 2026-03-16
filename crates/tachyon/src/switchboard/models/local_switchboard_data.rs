use std::collections::HashMap;
use crate::switchboard::models::connection_phase::ConnectionPhase;
use crate::tachyon::tachyon_client::TachyonClient;
use matrix_sdk::ruma::OwnedRoomId;
use matrix_sdk::Room;
use msnp::msnp::notification::models::endpoint_guid::EndpointGuid;
use msnp::msnp::switchboard::models::session_id::SessionId;
use msnp::shared::models::email_address::EmailAddress;
use msnp::shared::models::ticket_token::TicketToken;
use tokio::sync::broadcast::Receiver;
use msnp::shared::payload::msg::chunked_msg_payload::{ChunkedMsgPayload, MsgChunks};

pub struct LocalSwitchboardData {
    pub(crate) phase: ConnectionPhase,
    pub(crate) email_addr: EmailAddress,
    pub(crate) endpoint_guid: Option<EndpointGuid>,
    pub(crate) token: TicketToken,
    pub(crate) tachyon_client: Option<TachyonClient>,
    pub(crate) client_kill_recv: Receiver<()>,
    pub(crate) room: Option<Room>,
    pub(crate) room_id: Option<OwnedRoomId>,
    pub(crate) session_id: SessionId,
    pub(crate) chunks: HashMap<String, MsgChunks>
}

impl LocalSwitchboardData {
    pub fn new(client_kill_recv: Receiver<()>) -> Self {
        Self {
            phase: ConnectionPhase::default(),
            email_addr: EmailAddress::default(),
            endpoint_guid: None,
            token: TicketToken::default(),
            tachyon_client: None,
            client_kill_recv,
            room: None,
            room_id: None,
            session_id: SessionId::empty(),
            chunks: Default::default(),
        }
    }
}