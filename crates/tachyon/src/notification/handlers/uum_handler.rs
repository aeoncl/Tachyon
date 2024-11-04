use crate::notification::client_store::ClientData;
use matrix_sdk::ruma::events::room::message::RoomMessageEventContent;
use msnp::msnp::notification::command::command::NotificationServerCommand;
use msnp::msnp::notification::command::uum::{UumClient, UumPayload};
use tokio::sync::mpsc::Sender;
use msnp::shared::payload::msg::text_msg::FontStyle;
use crate::shared::identifiers::MatrixIdCompatible;

pub async fn handle_uum(command: UumClient, client_data: ClientData, command_sender: Sender<NotificationServerCommand>) -> Result<(), anyhow::Error>  {
    let ok_response = command.get_ok_response();

    match command.payload {
        UumPayload::TextMessage(content) => {
            let matrix_client = client_data.get_matrix_client();

            let room = matrix_client.get_dm_room(&command.destination.email_addr.to_owned_user_id());
            match room {
                None => {
                    //NO DM ROOM FOUND
                }
                Some(room) => {
                    //TODO SMILEY TO EMOJI
                    //TODO Store event id for dedup

                    let content = if content.is_styling_default() {
                        RoomMessageEventContent::text_plain(content.body)
                    } else {
                        let mut message = content.body.clone();

                        if !content.is_default_font_styles() {
                            if content.font_styles.matches(FontStyle::Bold) {
                                message = format!("<b>{}</b>", message)
                            }

                            if content.font_styles.matches(FontStyle::Italic) {
                                message = format!("<i>{}</i>", message)
                            }

                            if content.font_styles.matches(FontStyle::Underline) {
                                message = format!("<u>{}</u>", message)
                            }

                            if content.font_styles.matches(FontStyle::StrikeThrough) {
                                message = format!("<strike>{}</strike>", message)
                            }
                        }

                        let color_attr = if content.is_default_font_color() { String::new() } else { format!(" color=\"{}\"", content.font_color.serialize_rgb())};
                        let face_attr = if content.is_default_font() { String::new() } else { format!(" face=\"{}\"", content.font_family) };
                        message = format!("<font{}{}>{}</font>",  color_attr, face_attr, message);

                        RoomMessageEventContent::text_html(content.body, message)
                    };

                    let response = room.send(content).await?;
                    //self.add_to_events_sent(response.event_id.to_string());
                    command_sender.send(NotificationServerCommand::OK(ok_response)).await?;
                }
            }

            Ok(())
        },
        UumPayload::TypingUser(_) => {
            todo!()
        }
        UumPayload::Nudge(_) => {
            todo!()

        }
        UumPayload::Raw(_) => {
            todo!()
        }
    }
}