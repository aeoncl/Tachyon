use crate::domain::models::Participant;
use crate::domain::ids::{ConversationId, MessageId};




pub enum BackendEvent {

    MessageEvent(MessageEventContent),
    RedactedMessageEvent(MessageEventContent)

}

pub enum MessageKind {
    Text(TextContent),
    Image,
    VoiceMessage,
    File,
    Buzz
}

pub struct TextContent {
    plaintext: String,
    formatted: Option<TextFormat>
}

pub enum TextFormat {
    HTML(String),

}

pub struct MessageEventContent {
    id: MessageId,
    conversation: ConversationId,
    sender: Participant,
    kind: MessageKind
}

pub enum BridgeEvent {

}