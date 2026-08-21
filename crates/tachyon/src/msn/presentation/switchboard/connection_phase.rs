pub(crate) enum ConnectionPhase {
    Authenticating,
    Initializing,
    Ready
}

impl Default for ConnectionPhase {
    fn default() -> Self {
        ConnectionPhase::Authenticating
    }
}
