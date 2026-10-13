
#[derive(Debug, PartialEq)]
pub(crate) enum ConnectionPhase {
    Negotiating,
    Authenticating,
    AwaitingLogin,
    Ready
}

impl Default for ConnectionPhase {
    fn default() -> Self {
        ConnectionPhase::Negotiating
    }
}
