use std::str::FromStr;
use matrix_sdk::Client;
use matrix_sdk::ruma::events::room::message::RoomMessageEventContent;
use msnp::msnp::notification::command::command::NotificationServerCommand;
use msnp::msnp::notification::command::uum::{UumClient, UumPayload};
use msnp::shared::models::font_style::FontStyle;
use msnp::shared::models::email_address::EmailAddress;
use msnp::shared::traits::IntoBytes;
use crate::matrix::extensions::msn_user_resolver::FindRoomFromEmail;
use crate::notification::models::client_out::ClientOut;

/// Pattern A shell (§3.1), async — the Sender is gone; the dispatcher sends
/// whatever this returns. The matrix-side flow (room lookup, styling
/// translation, `room.send`) stays in the shell with behavior IDENTICAL —
/// D3 owns the matrix reshape, and the dedup/smiley TODOs stay TODOs per
/// the fence. Ordering and per-site policy preserved exactly: the OK ack is
/// produced only after `room.send` succeeded; an unparseable destination or
/// a missing DM room silently produces nothing; the `todo!()` arms stay
/// `todo!()`. The never-read TachyonClient parameter died with the Sender.
pub async fn handle_uum(command: UumClient, matrix_client: Client) -> Result<Vec<ClientOut>, anyhow::Error>  {
    let ok_ack = decide_uum(&command);

    match command.payload {
        UumPayload::TextPlain(content) => {

            let dest_email = EmailAddress::from_str(&command.destination);
            if let Ok(dest_email) = dest_email {

                let room = matrix_client.find_room_from_email(&dest_email)?;
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

                        let _response = room.send(content).await?;
                        //self.add_to_events_sent(response.event_id.to_string());
                        return Ok(vec![ok_ack]);
                    }
                }

            }
            Ok(vec![])
        },
        UumPayload::TypingUser(_) => {
            todo!()
        }
        UumPayload::Nudge(_) => {
            todo!()

        }
        UumPayload::Raw(raw) => {
            println!("UUM: {}", String::from_utf8(raw.into_bytes()).unwrap());
            todo!()
        }
    }
}

/// Pure decision (§3.1): the UUM acknowledgement. Passthrough — no real
/// decision: every delivered UUM earns the same OK echo; whether one is
/// produced at all is the shell's delivery ordering above, not a decide
/// input.
pub(crate) fn decide_uum(command: &UumClient) -> ClientOut {
    ClientOut::Command(NotificationServerCommand::OK(command.get_ok_response()))
}

#[cfg(test)]
mod tests {
    use msnp::msnp::notification::command::uum::{UumClient, UumPayload};
    use msnp::shared::models::network_id::NetworkId;
    use msnp::shared::payload::msg::text_plain_msg::TextPlainMessagePayload;
    use msnp::shared::traits::IntoBytes;

    use super::{decide_uum, handle_uum};
    use crate::notification::models::client_out::ClientOut;

    fn uum(tr_id: u128, destination: &str) -> UumClient {
        UumClient {
            tr_id,
            destination: destination.to_string(),
            network_id: NetworkId::WindowsLive,
            payload: UumPayload::TextPlain(TextPlainMessagePayload::new_with_default_style("Hello")),
        }
    }

    #[test]
    fn a_delivered_uum_is_acked_with_the_uum_operand_and_its_own_tr_id() {
        let table = vec![
            (15, "peer@test.com", "UUM 15 OK\r\n"),
            (16, "tel:+19999999999", "UUM 16 OK\r\n"),
        ];

        for (tr_id, destination, expected_bytes) in table {
            let ClientOut::Command(out) = decide_uum(&uum(tr_id, destination)) else {
                panic!("uum decides a bare wire command");
            };
            assert_eq!(String::from_utf8(out.into_bytes()).unwrap(), expected_bytes);
        }
    }

    #[tokio::test]
    async fn an_unparseable_destination_silently_produces_no_ack_at_all() {
        // built, never connected to: an unparseable destination must be
        // rejected before any matrix lookup happens
        let matrix_client = matrix_sdk::Client::builder().homeserver_url("http://127.0.0.1:1").build().await.unwrap();

        let outs = handle_uum(uum(17, "not an email"), matrix_client).await.unwrap();

        assert!(outs.is_empty());
    }
}
