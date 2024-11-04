use crate::notification::client_store::ClientData;

pub(crate) enum ConnectionPhase {
    Negotiating,
    Authenticating,
    Ready
}

impl Default for ConnectionPhase {
    fn default() -> Self {
        ConnectionPhase::Negotiating
    }
}
