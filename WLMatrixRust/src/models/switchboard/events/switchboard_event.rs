use super::content::{file_upload_event_content::FileUploadEventContent, initial_roster_event_content::InitialRosterEventContent, message_event_content::MessageEventContent, messages_event_content::MessagesEventContent, typing_user_event_content::TypingUserEventContent, user_joined_event_content::UserJoinedEventContent};

#[derive(Clone, Debug)]

pub enum SwitchboardEvent {
    UserJoinedEvent(UserJoinedEventContent),
    InitialRosterEvent(InitialRosterEventContent),
    MessageEvent(MessageEventContent),
    MessagesEvent(MessagesEventContent),
    TypingUserEvent(TypingUserEventContent),
    FileUploadEvent(FileUploadEventContent)
}

impl From<TypingUserEventContent> for SwitchboardEvent {
    fn from(v: TypingUserEventContent) -> Self {
        Self::TypingUserEvent(v)
    }
}


impl From<InitialRosterEventContent> for SwitchboardEvent {
    fn from(v: InitialRosterEventContent) -> Self {
        Self::InitialRosterEvent(v)
    }
}

impl From<UserJoinedEventContent> for SwitchboardEvent {
    fn from(v: UserJoinedEventContent) -> Self {
        Self::UserJoinedEvent(v)
    }
}

impl From<MessageEventContent> for SwitchboardEvent {
    fn from(v: MessageEventContent) -> Self {
        Self::MessageEvent(v)
    }
}

impl From<FileUploadEventContent> for SwitchboardEvent {
    fn from(v: FileUploadEventContent) -> Self {
        Self::FileUploadEvent(v)
    }
}

impl From<MessagesEventContent> for SwitchboardEvent {
    fn from(v: MessagesEventContent) -> Self {
        Self::MessagesEvent(v)
    }
}



