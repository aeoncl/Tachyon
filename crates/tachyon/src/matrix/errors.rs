use std::time::Duration;
use thiserror::Error;

/// What the session manager should DO when something goes wrong.
///
/// This decouples internal error details from your "client-facing" behavior.
#[derive(Debug, Clone)]
pub enum SessionAction {
    /// Keep TCP connection; just continue the loop.
    Continue,

    /// Keep TCP connection; sleep before retrying backend poll.
    Backoff(Duration),

    /// Disconnect the TCP client (and end the session).
    Disconnect { reason: DisconnectReason },
}

#[derive(Debug, Clone)]
pub enum DisconnectReason {
    BackendUnresponsive,
    Unauthorized,
    FatalBackendError,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum BackendFailureKind {
    Timeout,
    Network,
    RateLimited,
    Unauthorized,
    Server5xx,
    BadRequest,
    Decode,
    Unknown,
}

#[derive(Debug, Error)]
#[error("backend failure: {kind:?}")]
pub struct BackendFailure {
    pub kind: BackendFailureKind,

    /// Keep the original error for logs/debugging.
    #[source]
    pub source: anyhow::Error,

    /// Optional hint (e.g. Retry-After).
    pub retry_after: Option<Duration>,
}

impl BackendFailure {
    pub fn new(kind: BackendFailureKind, source: anyhow::Error) -> Self {
        Self { kind, source, retry_after: None }
    }

    pub fn with_retry_after(mut self, d: Duration) -> Self {
        self.retry_after = Some(d);
        self
    }

    /// Default policy for what to do after a *single* failure.
    /// (The watchdog below handles "unresponsive for too long".)
    pub fn immediate_action(&self) -> SessionAction {
        match self.kind {
            BackendFailureKind::RateLimited => {
                SessionAction::Backoff(self.retry_after.unwrap_or(Duration::from_secs(2)))
            }
            BackendFailureKind::Timeout
            | BackendFailureKind::Network
            | BackendFailureKind::Server5xx => SessionAction::Backoff(Duration::from_secs(1)),
            BackendFailureKind::Unauthorized => SessionAction::Disconnect {
                reason: DisconnectReason::Unauthorized,
            },
            BackendFailureKind::BadRequest
            | BackendFailureKind::Decode
            | BackendFailureKind::Unknown => SessionAction::Disconnect {
                reason: DisconnectReason::FatalBackendError,
            },
        }
    }
}