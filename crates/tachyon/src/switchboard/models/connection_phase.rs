pub(crate) enum ConnectionPhase {
    Authenticating,
    Ready
}

impl Default for ConnectionPhase {
    fn default() -> Self {
        ConnectionPhase::Authenticating
    }
}
