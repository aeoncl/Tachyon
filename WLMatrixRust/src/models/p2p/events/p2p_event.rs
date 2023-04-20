use super::content::{file_received_event_content::FileReceivedEventContent, message_event_content::MessageEventContent, file_transfer_accepted_event_content::FileTransferAcceptedEventContent};

#[derive(Clone, Debug)]
pub enum P2PEvent {
    FileReceived(FileReceivedEventContent),
    Message(MessageEventContent),
    FileTransferAccepted(FileTransferAcceptedEventContent)
}
