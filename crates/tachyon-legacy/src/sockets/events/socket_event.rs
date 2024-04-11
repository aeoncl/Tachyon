
#[derive(Debug, Clone)]
pub enum SocketEvent {
    Single(String),
    Multiple(Vec<String>)
}

impl From<String> for SocketEvent {
    fn from(value: String) -> Self {
        SocketEvent::Single(value)
    }
}

impl From<Vec<String>> for SocketEvent {
    fn from(value: Vec<String>) -> Self {
        SocketEvent::Multiple(value)
    }
}