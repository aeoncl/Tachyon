use std::sync::Arc;
use uuid::Uuid;

#[derive(Clone, PartialEq, Eq, Hash, Debug)]
pub struct DeviceId(Arc<str>);

//Represent a LoggedIn User
#[derive(Clone, PartialEq, Eq, Hash, Debug)]
pub struct LoginId(Arc<str>);

#[derive(Clone, PartialEq, Eq, Hash, Debug)]
pub struct UserId(Arc<str>);

#[derive(Clone, PartialEq, Eq, Hash, Debug)]
pub struct ConversationId(Arc<str>);

#[derive(Clone, PartialEq, Eq, Hash, Debug)]
pub struct MediaId(Arc<str>);

//Represents an active session, lost on shutdown.
#[derive(Clone, Copy, PartialEq, Eq, Hash, Debug)]
pub struct SessionId(Uuid);

#[derive(Clone, PartialEq, Eq, Hash, Debug)]
pub struct MessageId(Arc<str>);
