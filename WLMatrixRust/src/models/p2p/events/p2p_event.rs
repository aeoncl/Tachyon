use super::content::{file_received_event_content::FileReceivedEventContent, message_event_content::MessageEventContent};

#[derive(Clone, Debug)]
pub enum P2PEvent {
    FileReceived(FileReceivedEventContent),
    Message(MessageEventContent),
}
