use async_trait::async_trait;
use crate::domain::auth::TachyonToken;
use crate::domain::error::{TachyonResult};
use crate::domain::models::Participant;
use crate::domain::ids::{ConversationId, MessageId, SessionId, UserId};




pub enum TachyonEvent {
    BridgeAnnounce(BridgeAnnounceContent),
    BridgeGoodbye(SessionId),
    BridgeAuth(BridgeAuthContent),
}

pub struct BridgeAnnounceContent {
    pub id: SessionId,
    pub sender: Box<dyn EventSender<BackendEvent>>
}

pub struct BridgeAuthContent {
    session_id: SessionId,
    kind: AuthKind
}

pub enum AuthKind {
    Restore(TachyonToken),
    InitiateInteractiveAuth(InitiateAuthContent),
    FinishInteractiveAuth(FinishInteractiveAuthContent),
    PasswordAuth(PasswordAuthContent)
}

pub struct InitiateAuthContent {
    server_name: String,
    user_id: UserId,
    response_sender: Box<dyn EventSender<TachyonResult<InitiateInteractiveAuthResponse>>>
}

pub struct InitiateInteractiveAuthResponse {
    auth_url: String,
    nonce: String
}

pub struct FinishInteractiveAuthContent {
    return_url: String
}

pub struct PasswordAuthContent {
    user_id: UserId,
    password: String
}


#[async_trait]
pub trait EventSender<T> : Send + Sync {
    async fn send(&self, message: T);
}


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