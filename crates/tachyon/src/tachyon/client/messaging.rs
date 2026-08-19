use log::error;
use crate::p2p::client::session::{ReceiveFileContent, SessionType};
use crate::tachyon::client::tachyon_client::TachyonClient;
use matrix_sdk::ruma::events::room::MediaSource;
use matrix_sdk::ruma::RoomId;
use ruma::OwnedUserId;
use msnp::msnp::error::PayloadError;
use msnp::msnp::notification::command::not::TextMessageBody;
use msnp::shared::models::font_color::FontColor;
use msnp::shared::models::font_name::FontName;
use msnp::shared::models::font_style::{FontStyle, FontStyles};
use msnp::shared::models::msn_user::MsnUser;
use msnp::shared::payload::msg::text_plain_msg::TextPlainMessagePayload;

pub struct TextStyle {
    pub font_name: String,
    pub bold: bool,
    pub italic: bool,
    pub strikethrough: bool,
    pub underline: bool,
    pub hex_color: String,
}

impl Default for TextStyle {
    fn default() -> Self {
        TextStyle {
            font_name: "Segoe UI Emojis".to_string(),
            bold: false,
            italic: false,
            strikethrough: false,
            underline: false,
            hex_color: "#000000".to_string(),
        }
    }
}

pub enum TachyonSender {
    ProxyRoomUser,
    User(OwnedUserId)
}

impl TachyonClient {

    pub async fn receive_text_message(&self, room_id: &RoomId, sender: &TachyonSender, content: &str, message_style: TextStyle) -> Result<(), anyhow::Error>{

        let room_user = self.resolve_room_to_msn_user(room_id).await.unwrap();
        let switchboard = self.switchboards().get_or_initialize(room_id, &room_user);

        let message_sender = match sender {
            TachyonSender::ProxyRoomUser => {
                room_user.clone()
            }
            TachyonSender::User(sender_id) => {
                self.get_msn_user(room_id, sender_id).await?
            }
        };

        let (font_name, font_color, font_styles) = map_text_style_to_msn(message_style);
        let message = TextPlainMessagePayload::new(font_name, font_color, font_styles, false, content);

        switchboard.receive_msg(message_sender.get_email_address(), message_sender.compute_display_name(), message).await
    }

    pub async fn receive_file(&self, room_id: &RoomId, inviter: &MsnUser, sender: &MsnUser, file_size: usize, filename: String, media_source: MediaSource) {



        let transport = self.get_or_create_transport(room_id, inviter);
        let (session_id, session) = self.create_session_with_random_id(transport, SessionType::ReceiveFile(ReceiveFileContent {
            sender: inviter.endpoint_id.clone(),
            sender_display_name: sender.compute_display_name().to_string(),
            receiver: self.own_user().endpoint_id,
            media_source,
            file_size,
            filename,
        }));

        session.receive_invite().await;

    }
}

fn map_text_style_to_msn(text_style: TextStyle) -> (FontName, FontColor, FontStyles) {
    let font_name = FontName::new(text_style.font_name);
    let font_color = {
        let parsed = match text_style.hex_color.strip_prefix("#") {
            None => {
                FontColor::parse_from_rgb(&text_style.hex_color)
            }
            Some(no_prefix) => {
                FontColor::parse_from_rgb(no_prefix)
            }
        };

        match parsed {
            Ok(parsed) => parsed,
            Err(err) => {
                error!("Could not parse font_color: {}", &text_style.hex_color);
                FontColor::default()
            }
        }
    };

    let font_styles = {
        let mut font_styles = FontStyles::default();
        if text_style.bold {
            font_styles.bold();
        }

        if text_style.italic {
            font_styles.italic();
        }

        if text_style.strikethrough {
            font_styles.strikethrough();
        }

        if text_style.underline {
            font_styles.underline();
        }

        font_styles
    };

    (font_name, font_color, font_styles)
}