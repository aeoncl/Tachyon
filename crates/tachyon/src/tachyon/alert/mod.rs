use anyhow::anyhow;
use msnp::shared::models::ticket_token::TicketToken;
use tokio::sync::oneshot;
use tokio::sync::oneshot::error::TryRecvError;

pub mod alert_repository;

pub type AlertError = anyhow::Error;

pub enum AlertSuccess {
    Unit,
    TicketToken(TicketToken),
}

impl From<TicketToken> for AlertSuccess {
    fn from(value: TicketToken) -> Self {
        Self::TicketToken(value)
    }
}

impl From<()> for AlertSuccess {
    fn from(_: ()) -> Self {
        Self::Unit
    }
}

pub trait AlertNotify {
    fn notify_success(self, result: AlertSuccess) -> Result<(), AlertError>;
    fn notify_failure(self, error: AlertError) -> Result<(), AlertError>;
}

pub enum Alert {
    CrossSign(CrossSignAlertContent),
    WebLogin(WebLoginAlertContent),
}

pub struct CrossSignAlertContent {
    sender: oneshot::Sender<Result<(), AlertError>>,
    creation_time: std::time::Instant,
    expiration_time: std::time::Instant,
}

pub struct WebLoginAlertContent {
    sender: oneshot::Sender<Result<TicketToken, AlertError>>,
}

pub enum AlertReceiver {
    Unit(oneshot::Receiver<Result<(), AlertError>>),
    TicketToken(oneshot::Receiver<Result<TicketToken, AlertError>>),
}

impl AlertReceiver {
    pub fn try_recv(&mut self) -> Result<Option<AlertSuccess>, AlertError> {
        match self {
            AlertReceiver::Unit(rx) => {
                match rx.try_recv() {
                    Ok(Ok(content)) => {
                        Ok(Some(content.into()))
                    }
                    Ok(Err(e)) => Err(e),
                    Err(TryRecvError::Empty) => {
                        Ok(None)
                    }
                    Err(e) => Err(anyhow!("Failed to receive alert: {}", e)),
                }
            }
            AlertReceiver::TicketToken(rx) => {
                match rx.try_recv() {
                    Ok(Ok(content)) => {
                        Ok(Some(content.into()))
                    }
                    Ok(Err(e)) => Err(e),
                    Err(TryRecvError::Empty) => {
                        Ok(None)
                    }
                    Err(e) => Err(anyhow!("Failed to receive alert: {}", e)),
                }
            }
        }
    }

    /// Await the result
    pub async fn recv(self) -> Result<AlertSuccess, AlertError> {
        match self {
            AlertReceiver::Unit(rx) => {
                let alert_result = rx.await.map_err(|e| anyhow!("Failed to receive alert: {}", e))?;
                alert_result.map(|e| e.into())
            }
            AlertReceiver::TicketToken(rx) => {
                let alert_result = rx.await.map_err(|e| anyhow!("Failed to receive alert: {}", e))?;
                alert_result.map(|e| e.into())
            }
        }
    }
}

impl AlertNotify for CrossSignAlertContent {
    fn notify_success(self, result: AlertSuccess) -> Result<(), AlertError> {
        if !matches!(result, AlertSuccess::Unit) {
            return Err(anyhow::anyhow!("Invalid alert success type, expected Unit"));
        }
        self.sender.send(Ok(())).map_err(|_| anyhow::anyhow!("Failed to send alert"))
    }

    fn notify_failure(self, error: AlertError) -> Result<(), AlertError> {
        self.sender.send(Err(error)).map_err(|_| anyhow::anyhow!("Failed to send alert error"))
    }
}

impl AlertNotify for WebLoginAlertContent {
    fn notify_success(self, result: AlertSuccess) -> Result<(), AlertError> {
        let token = match result {
            AlertSuccess::TicketToken(token) => token,
            _ => return Err(anyhow::anyhow!("Invalid alert success type, expected TicketToken")),
        };
        self.sender.send(Ok(token)).map_err(|_| anyhow::anyhow!("Failed to send alert"))
    }

    fn notify_failure(self, error: AlertError) -> Result<(), AlertError> {
        self.sender.send(Err(error)).map_err(|_| anyhow::anyhow!("Failed to send alert error"))
    }
}

impl AlertNotify for Alert {
    fn notify_success(self, result: AlertSuccess) -> Result<(), AlertError> {
        match self {
            Alert::CrossSign(content) => content.notify_success(result),
            Alert::WebLogin(content) => content.notify_success(result),
        }
    }

    fn notify_failure(self, error: AlertError) -> Result<(), AlertError> {
        match self {
            Alert::CrossSign(content) => content.notify_failure(error),
            Alert::WebLogin(content) => content.notify_failure(error),
        }
    }
}

impl Alert {
    pub fn new_weblogin() -> (Self, AlertReceiver) {
        let (sender, receiver) = oneshot::channel();
        (
            Alert::WebLogin(WebLoginAlertContent { sender }),
            AlertReceiver::TicketToken(receiver),
        )
    }

    pub fn new_crosssign(expiration_duration: std::time::Duration) -> (Self, AlertReceiver) {
        let creation_time = std::time::Instant::now();
        let expiration_time = creation_time + expiration_duration;

        let (sender, receiver) = oneshot::channel();
        (
            Alert::CrossSign(CrossSignAlertContent { sender, creation_time, expiration_time }),
            AlertReceiver::Unit(receiver),
        )
    }
}