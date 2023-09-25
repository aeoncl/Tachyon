use super::content::{file_received_event_content::FileReceivedEventContent, file_transfer_accepted_event_content::FileTransferAcceptedEventContent, message_event_content::MessageEventContent, msn_object_requested_event_content::MSNObjectRequestedEventContent};

#[derive(Clone, Debug)]
pub enum P2PEvent {
    FileReceived(FileReceivedEventContent),
    Message(MessageEventContent),
    FileTransferAccepted(FileTransferAcceptedEventContent),
    MSNObjectRequested(MSNObjectRequestedEventContent)
}
