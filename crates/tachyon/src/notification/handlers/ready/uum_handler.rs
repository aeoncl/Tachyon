use std::str::FromStr;
use matrix_sdk::Client;
use matrix_sdk::ruma::events::room::message::RoomMessageEventContent;
use msnp::msnp::notification::command::command::NotificationServerCommand;
use msnp::msnp::notification::command::uum::{UumClient, UumPayload};
use msnp::shared::models::font_style::FontStyle;
use msnp::shared::models::email_address::EmailAddress;
use msnp::shared::traits::IntoBytes;
use crate::matrix::extensions::message_dedup::{apply_room_op, RoomOp};
use crate::matrix::extensions::msn_user_resolver::find_room_from_email;
use crate::notification::models::client_out::ClientOut;
use crate::tachyon::client::tachyon_client::TachyonClient;

/// Pattern A shell (§3.1), async — the Sender is gone; the dispatcher sends
/// whatever this returns. D3 (§8): the TachyonClient parameter is back (the
/// owned dedup/hash state lives on it, §3.5), and the text send now routes
/// through the ONE `RoomOp::SendText` executor. That retires the §1.1 echo
/// gap — "Observed gap (no runtime repro taken); retired in D3, not
/// hotfixed": the UUM path gains own-echo suppression and per-room
/// send-lock serialization, a DELIBERATE behavior delta. The executor
/// records the sent event id, so the pre-D3 "TODO Store event id for dedup"
/// is retired with it. Everything else about the D2b-ii shape is FROZEN:
/// the styling translation is untouched, the OK ack is produced only after
/// the send succeeded, an unparseable destination or a missing DM room
/// silently produces nothing, and the `todo!()` arms stay `todo!()`.
pub async fn handle_uum(command: UumClient, tachyon_client: TachyonClient, matrix_client: Client) -> Result<Vec<ClientOut>, anyhow::Error>  {
    let ok_ack = decide_uum(&command);

    match command.payload {
        UumPayload::TextPlain(content) => {

            let dest_email = EmailAddress::from_str(&command.destination);
            if let Ok(dest_email) = dest_email {

                let room = find_room_from_email(&matrix_client, tachyon_client.room_hashes(), &dest_email)?;
                match room {
                    None => {
                        //NO DM ROOM FOUND
                    }
                    Some(room) => {
                        //TODO SMILEY TO EMOJI

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

                        // The single dedup path (§3.5): send + record the
                        // event id on the account's owned state. `?` keeps
                        // the pre-D3 policy — the OK ack is produced only
                        // after the send succeeded.
                        apply_room_op(&tachyon_client, RoomOp::SendText { room, content }).await?;
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
    use std::str::FromStr;

    use msnp::msnp::models::contact_list::ContactList;
    use msnp::msnp::notification::command::uum::{UumClient, UumPayload};
    use msnp::shared::models::email_address::EmailAddress;
    use msnp::shared::models::endpoint_id::EndpointId;
    use msnp::shared::models::msn_user::MsnUser;
    use msnp::shared::models::network_id::NetworkId;
    use msnp::shared::models::ticket_token::TicketToken;
    use msnp::shared::payload::msg::text_plain_msg::TextPlainMessagePayload;
    use msnp::shared::traits::IntoBytes;

    use super::{decide_uum, handle_uum};
    use crate::notification::models::client_out::ClientOut;
    use crate::tachyon::client::tachyon_client::TachyonClient;
    use crate::tachyon::config::tachyon_config::TachyonConfig;

    fn uum(tr_id: u128, destination: &str) -> UumClient {
        UumClient {
            tr_id,
            destination: destination.to_string(),
            network_id: NetworkId::WindowsLive,
            payload: UumPayload::TextPlain(TextPlainMessagePayload::new_with_default_style("Hello")),
        }
    }

    fn tachyon_client() -> TachyonClient {
        let (snd, _rcv) = tokio::sync::mpsc::channel(10);
        let (kill_snd, kill_recv) = tokio::sync::broadcast::channel::<()>(1);
        let msn_user = MsnUser::new(EndpointId::new(EmailAddress::from_str("aeon@test.com").unwrap(), None));
        TachyonClient::new(TachyonConfig::default(), msn_user, TicketToken::default(), snd, kill_snd, kill_recv, ContactList::default())
    }

    #[test]
    fn a_delivered_uum_is_acked_with_the_uum_operand_and_its_own_tr_id() {
        let table = vec![
            (15, "peer@test.com", "UUM 15 OK\r\n"),
            (16, "tel:+199****9999", "UUM 16 OK\r\n"),
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

        let outs = handle_uum(uum(17, "not an email"), tachyon_client(), matrix_client).await.unwrap();

        assert!(outs.is_empty());
    }
}
