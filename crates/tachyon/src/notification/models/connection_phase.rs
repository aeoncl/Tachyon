
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
