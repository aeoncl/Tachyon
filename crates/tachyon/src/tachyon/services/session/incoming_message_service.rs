use crate::switchboard::extensions::CustomStyles;
use crate::tachyon::services::session::user_service::UserService;
use crate::tachyon::state::session::switchboard_handle_repository::SwitchboardHandleRepository;
use matrix_sdk::ruma::RoomId;
use msnp::msnp::switchboard::command::command::SwitchboardServerCommand;
use msnp::msnp::switchboard::command::msg::{MsgPayload, MsgServer};
use msnp::shared::models::display_name::DisplayName;
use msnp::shared::models::email_address::EmailAddress;
use msnp::shared::payload::msg::text_plain_msg::TextPlainMessagePayload;
use std::sync::Arc;

pub struct IncomingTextMessage {
    text: String,
    bold: bool,
    italic: bool,
    underline: bool,
    strikethrough: bool,
    color: String,
    font: String,
    size: String,
}

impl IncomingTextMessage {
    pub fn new(
        text: String,
        bold: bool,
        italic: bool,
        underline: bool,
        strikethrough: bool,
        color: String,
        font: String,
        size: String,
    ) -> Self {
        Self {
            text,
            bold,
            italic,
            underline,
            strikethrough,
            color,
            font,
            size,
        }
    }

    pub fn new_with_default_style(text: &str) -> Self {
        Self::new(
            text.to_string(),
            false,
            false,
            false,
            false,
            "000000".to_string(),
            "Arial".to_string(),
            "12".to_string(),
        )
    }
}

pub trait IncomingMessagingService: Send + Sync {

    fn receive_message(
        &self,
        sender: &EmailAddress,
        room_id: &RoomId,
        message: IncomingTextMessage,
    );

    fn receive_notice(&self, sender: &EmailAddress, room_id: &RoomId, message: IncomingTextMessage);
}

#[derive(Clone)]
pub struct IncomingMessagingServiceImpl {
    switchboards: Arc<dyn SwitchboardHandleRepository>,
    user_service: Arc<dyn UserService>,
}

impl IncomingMessagingServiceImpl {
    pub fn new(
        switchboards: Arc<dyn SwitchboardHandleRepository>,
        user_service: Arc<dyn UserService>,
    ) -> Self {
        Self {
            switchboards,
            user_service,
        }
    }
}

impl IncomingMessagingService for IncomingMessagingServiceImpl {

    fn receive_message(
        &self,
        sender: &EmailAddress,
        room_id: &RoomId,
        message: IncomingTextMessage,
    ) {
        //Check if this shouldn't add a switchboard to the list
        let proxy_room_email = self.user_service.resolve_room_proxy_email(room_id).unwrap();
        let handle = self
            .switchboards
            .get_or_initialize(room_id, &proxy_room_email);

        let sender_clone = sender.clone();
        tokio::spawn(async move {
            let _ = handle
                .send_command(SwitchboardServerCommand::MSG(MsgServer {
                    sender: sender_clone.clone(),
                    display_name: DisplayName::new_from_ref(sender_clone.as_str()),
                    payload: MsgPayload::TextPlain(
                        TextPlainMessagePayload::new_with_default_style(&message.text),
                    ),
                }))
                .await;
        });
    }

    fn receive_notice(
        &self,
        sender: &EmailAddress,
        room_id: &RoomId,
        message: IncomingTextMessage,
    ) {
        let proxy_room_email = self.user_service.resolve_room_proxy_email(room_id).unwrap();

        let handle = self
            .switchboards
            .get_or_initialize(room_id, &proxy_room_email);

        let sender_clone = sender.clone();
        tokio::spawn(async move {
            let _ = handle
                .send_command(SwitchboardServerCommand::MSG(MsgServer {
                    sender: sender_clone.clone(),
                    display_name: DisplayName::new_from_ref(sender_clone.as_str()),
                    payload: MsgPayload::TextPlain(TextPlainMessagePayload::new_with_notice_style(
                        &message.text,
                    )),
                }))
                .await;
        });
    }
}