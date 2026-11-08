use log::{debug, error, warn};
use crate::matrix::extensions::message_dedup::{apply_room_op, RoomOp};
use crate::matrix::extensions::msn_user_resolver::room_to_msn_user_lazy;
use crate::matrix::nudge_custom_event::create_buzz_message;
use crate::p2p::p2p_handler::handle_p2p_packet;
use crate::switchboard::models::local_switchboard_data::LocalSwitchboardData;
use crate::switchboard::switchboard_server::SwitchboardSenderMsg;
use crate::tachyon::client::tachyon_client::TachyonClient;
use crate::tachyon::mappers::user_id::MatrixIdCompatible;
use matrix_sdk::attachment::AttachmentConfig;
use matrix_sdk::ruma::events::room::message::RoomMessageEventContent;
use matrix_sdk::ruma::UserId;
use matrix_sdk::Room;
use msnp::msnp::switchboard::command::ack::AckServer;
use msnp::msnp::switchboard::command::command::SwitchboardServerCommand;
use msnp::msnp::switchboard::command::msg::{MsgAcknowledgment, MsgClient, MsgPayload};
use msnp::p2p::v2::p2p_transport_packet::P2PTransportPacket;
use msnp::shared::command::nak::NakServer;
use msnp::shared::models::msn_object::{MsnObject, MsnObjectType};
use msnp::shared::payload::msg::chunked_msg_payload::{ChunkedMsgPayload, MsgChunks};
use msnp::shared::payload::msg::datacast_msg::{Datacast, DatacastType};
use tokio::sync::mpsc::Sender;

/// The matrix-bound effects an SB MSG payload can decide (§3). Bottom-up
/// from the real pre-D4 arms: every variant has a live call site, and the
/// arms that produced nothing collapse into `Nothing` (§5, honest no-op).
pub(crate) enum PayloadEffect {
    /// A plain text message: rides the single dedup path — the frozen D3
    /// `RoomOp::SendText` executor.
    SendText(RoomMessageEventContent),
    /// The nudge: a `chat.tachyon.buzz` custom event to the room.
    SendBuzz,
    /// The datacast announced a voice clip; its bytes are requested back
    /// over P2P.
    RequestVoiceClip(MsnObject),
    /// The typing notification — own user only (the pre-D4 filter).
    TypingNotice,
    /// A P2P transport packet: lazy resolver + transport + packet handling
    /// in apply.
    P2P(P2PTransportPacket),
    /// An ink drawing: the "ink.gif" attachment send.
    SendGif(Vec<u8>),
    /// Honest no-op (§5): Raw/Chunked/Ink payloads, Wink/ActionMsg
    /// datacasts and unsupported MSNObject announcements produced nothing
    /// pre-D4 and produce nothing here.
    Nothing,
}

/// The ack decision — an enum (not the command itself) so the shell keeps
/// the pre-D4 per-command warn wording ("ACK dropped" / "NAK dropped")
/// exactly, with the tr_id injected at the render.
#[derive(Clone, Copy, PartialEq, Eq, Debug)]
pub(crate) enum MsgAck {
    Ack,
    Nak,
}

pub(super) async fn handle_msg(msg_command: MsgClient, command_sender: Sender<SwitchboardSenderMsg>, tachyon_client: TachyonClient, room: Room, local_switchboard_data: &mut LocalSwitchboardData) -> Result<(), anyhow::Error> {

    // Chunk reassembly stays shell-side state (lsd.chunks, §3.6): a chunk
    // either completes into a payload worth dispatching, or it parks.
    if let MsgPayload::Chunked(chunk) = msg_command.payload {
        if let Some(complete) = handle_chunked(chunk, local_switchboard_data).await? {
            dispatch_msg_payload(msg_command.tr_id, msg_command.ack_type, complete, &room, &command_sender, tachyon_client.clone()).await;
        }
    } else {
        dispatch_msg_payload(msg_command.tr_id, msg_command.ack_type, msg_command.payload, &room, &command_sender, tachyon_client.clone()).await;
    };

    Ok(())
}

pub async fn handle_chunked(chunk: ChunkedMsgPayload, local_switchboard_data: &mut LocalSwitchboardData) -> Result<Option<MsgPayload>, anyhow::Error> {
    let message_id = chunk.message_id();
    match local_switchboard_data.chunks.remove(&message_id) {
        Some(mut chunks) => {
            chunks.append_chunk(chunk);
            if chunks.is_complete()? {
                return Ok(Some(chunks.merge_chunks()?));
            } else {
                local_switchboard_data.chunks.insert(message_id, chunks);
            }
        }
        None => {
            local_switchboard_data.chunks.insert(message_id, MsgChunks::from_first_chunk(chunk)?);
        }
    }

    Ok(None)
}

pub async fn dispatch_msg_payload(tr_id: u128, ack_type: MsgAcknowledgment, payload: MsgPayload, room: &Room, command_sender: &Sender<SwitchboardSenderMsg>, tachyon_client: TachyonClient) {

    let room_clone = room.clone();
    let command_sender_clone = command_sender.clone();

    if matches!(payload, MsgPayload::P2P(_)) {
        //P2P packets must be handled in the order they were received:
        //chunk reassembly and the transport handshake break if a later packet overtakes an earlier one.
        handle_msg_payload(tr_id, ack_type, payload, room_clone, command_sender_clone, tachyon_client).await;
    } else {
        tokio::spawn(handle_msg_payload(tr_id, ack_type, payload, room_clone, command_sender_clone, tachyon_client));
    }
}

/// The payload-task shell (pattern B, §3.1): decide the effect (pure arm
/// table) → apply it (the matrix IO) → decide the ack from the outcome
/// fact (§10's two-decide shape: the matrix effects run FIRST, the ack is
/// a pure function of their outcome) → render + send it. The Sender lives
/// in this task because §3.6's dispatch spawns it — a spawned task cannot
/// return ops to the connection loop. The error log and the warn-on-drop
/// policies are the pre-D4 ones, verbatim.
async fn handle_msg_payload(tr_id: u128, ack_type: MsgAcknowledgment, payload: MsgPayload, room_clone: Room, command_sender_clone: Sender<SwitchboardSenderMsg>, tachyon_client: TachyonClient) {
        if let MsgPayload::Datacast(datacast) = &payload {
            debug!("received DATACAST {:?}", datacast.get_type());
        }

        let effect = decide_payload_effect(payload, room_clone.own_user_id());

        let result = apply_payload_effect(effect, &room_clone, &tachyon_client).await;

        let succeeded = result.is_ok();
        if let Err(e) = result {
            error!("Could not send message {:?}", e);
        }

        if let Some(ack) = decide_msg_ack(ack_type, succeeded) {
            let label = match ack {
                MsgAck::Ack => "ACK",
                MsgAck::Nak => "NAK",
            };
            // Err = receiver dropped, client gone: the ack is undeliverable
            // and nothing else is produced here (§8 D2).
            if let Err(error) = command_sender_clone.send(SwitchboardSenderMsg::Single(render_msg_ack(tr_id, ack))).await {
                warn!("MSNP|SB: {} dropped, client gone: {}", label, error);
            }
        }
}

/// Decide (§3.1): the pure payload-arm table — which matrix-bound effect
/// this MSG payload decides, from plain facts only (the payload and the
/// room's own user id for the typing filter). No IO, no client: the pre-D4
/// inline arms' behavior, table-tested below. The one log-only side effect
/// (the unsupported-object debug line) is kept verbatim from the pre-D4
/// arm and is wire-neutral.
pub(crate) fn decide_payload_effect(payload: MsgPayload, own_user_id: &UserId) -> PayloadEffect {
    match payload {
        MsgPayload::TextPlain(text_plain) => {
            PayloadEffect::SendText(RoomMessageEventContent::text_plain(text_plain.body))
        }
        MsgPayload::Datacast(datacast) => {
            match datacast.get_type() {
                DatacastType::Nudge => PayloadEffect::SendBuzz,
                //The datacast only announces the object, its bytes have to be fetched over P2P.
                DatacastType::MsnObject => {
                    if let Datacast::MsnObject(msn_object) = datacast.data {
                        match msn_object.obj_type {
                            MsnObjectType::VoiceClip => PayloadEffect::RequestVoiceClip(msn_object),
                            // Every other object type keeps the pre-D4
                            // debug line and decides nothing (§5). Named,
                            // no wildcard (§10).
                            MsnObjectType::Avatar | MsnObjectType::CustomEmoticon
                            | MsnObjectType::DisplayPicture | MsnObjectType::SharedFile
                            | MsnObjectType::Background | MsnObjectType::History
                            | MsnObjectType::DynamicDisplayPicture | MsnObjectType::Wink
                            | MsnObjectType::MapFile | MsnObjectType::DynamicBackground
                            | MsnObjectType::PluginState | MsnObjectType::RoamingObject
                            | MsnObjectType::SignatureSound | MsnObjectType::UnknownYet
                            | MsnObjectType::Scene | MsnObjectType::WebcamDynamicDisplayPicture => {
                                debug!("Received a yet unsupported MSNObject datacast: {:?}", msn_object);
                                PayloadEffect::Nothing
                            }
                        }
                    } else {
                        // A type/data mismatch the pre-D4 if-let skipped.
                        PayloadEffect::Nothing
                    }
                }
                // Wink and ActionMsg announcements were ignored pre-D4.
                DatacastType::Wink | DatacastType::ActionMsg => PayloadEffect::Nothing,
            }
        }
        MsgPayload::Control(control) => {
            let typing_user_id = control.typing_user.to_owned_user_id();

            if *typing_user_id == *own_user_id {
                PayloadEffect::TypingNotice
            } else {
                PayloadEffect::Nothing
            }
        }
        MsgPayload::P2P(p2p) => PayloadEffect::P2P(p2p.payload),
        MsgPayload::Gif(gif) => PayloadEffect::SendGif(gif.gif_bytes),
        // Honest no-ops (§5): Raw carries profiles and other server-facing
        // payloads; a Chunked payload never reaches dispatch (reassembly
        // completes it first); Ink was ignored pre-D4. All produced
        // Ok(()) with no effect and still do.
        MsgPayload::Raw(_) => PayloadEffect::Nothing,
        MsgPayload::Chunked(_) => PayloadEffect::Nothing,
        MsgPayload::Ink(_) => PayloadEffect::Nothing,
    }
}

/// Decide (§3.1, §10 two-decide shape): the ack table over the outcome
/// fact — the matrix effects ran first, this is pure. Pre-D4 the same
/// table was two inline matches: NAK on failure for N/A/D, ACK on success
/// for A/D, silence for U and for a successful N. Every MsgAcknowledgment
/// variant is named — no wildcard (§10).
pub(crate) fn decide_msg_ack(ack_type: MsgAcknowledgment, succeeded: bool) -> Option<MsgAck> {
    match (ack_type, succeeded) {
        (MsgAcknowledgment::NoAck, false) => None,
        (MsgAcknowledgment::NoAck, true) => None,
        (MsgAcknowledgment::AckOnFailure, false) => Some(MsgAck::Nak),
        (MsgAcknowledgment::AckOnFailure, true) => None,
        (MsgAcknowledgment::AckA, false) => Some(MsgAck::Nak),
        (MsgAcknowledgment::AckA, true) => Some(MsgAck::Ack),
        (MsgAcknowledgment::AckD, false) => Some(MsgAck::Nak),
        (MsgAcknowledgment::AckD, true) => Some(MsgAck::Ack),
    }
}

/// Render: the ack wire command (byte-identical to the pre-D4 inline
/// AckServer/NakServer construction, tr_id injected here).
pub(crate) fn render_msg_ack(tr_id: u128, ack: MsgAck) -> SwitchboardServerCommand {
    match ack {
        MsgAck::Ack => SwitchboardServerCommand::ACK(AckServer::new(tr_id)),
        MsgAck::Nak => SwitchboardServerCommand::NAK(NakServer::new(tr_id)),
    }
}

/// Apply (§3): the effect IO, each arm keeping its pre-D4 policy — with
/// the two §1.4 hazards fixed in place (D4), named at their sites. The
/// error type widens to anyhow because the P2P arm's lazy resolver fails
/// in anyhow (its pre-D4 `.unwrap()` panicked instead); the matrix-bound
/// arms convert unchanged, and the "Could not send message" error log now
/// prints the anyhow-wrapped cause — a failure-path log-only delta.
async fn apply_payload_effect(effect: PayloadEffect, room: &Room, tachyon_client: &TachyonClient) -> Result<(), anyhow::Error> {
    match effect {
        PayloadEffect::SendText(content) => {
            // §1.1 site 1 routes through the single RoomOp::SendText
            // executor (D3) — same owned-dedup semantics as the pre-D3
            // dedup-send trait. Frozen D3 call, verbatim.
            apply_room_op(tachyon_client, RoomOp::SendText { room: room.clone(), content }).await?;
            Ok(())
        }
        PayloadEffect::SendBuzz => {
            let buzz = create_buzz_message(tachyon_client.own_user().compute_display_name());
            // §1.4 (D4), fixed in place: this was `.unwrap()` — a failed
            // nudge send panicked the payload task and the client never
            // received ANY ack. Now: warn + skip the nudge; the Err feeds
            // the ack table (NAK on N/A/D). Failure-path-only delta.
            match room.send(buzz).await {
                Ok(_response) => Ok(()),
                Err(error) => {
                    warn!("MSNP|SB: could not deliver the nudge to {}: {} — skipping", room.room_id(), error);
                    Err(error)?
                }
            }
        }
        PayloadEffect::RequestVoiceClip(msn_object) => {
            // Error-log policy preserved verbatim (pre-D4): a failed
            // request logs and the arm still decides success for the ack.
            if let Err(e) = tachyon_client.request_msn_object(room, msn_object).await {
                error!("Could not request the voice clip announced by the client: {:?}", e);
            }
            Ok(())
        }
        PayloadEffect::TypingNotice => {
            room.typing_notice(true).await?;
            Ok(())
        }
        PayloadEffect::P2P(packet) => {
            // §1.4 (D4), fixed in place: this was `.unwrap()` on the lazy
            // resolver — a room it rejects panicked the payload task (no
            // ack was ever produced). Now: warn + skip the packet; the Err
            // feeds the ack table. Failure-path-only delta.
            let msn_user = match room_to_msn_user_lazy(room, tachyon_client.room_hashes()).await {
                Ok(msn_user) => msn_user,
                Err(error) => {
                    warn!("MSNP|SB: could not resolve room {} to an MSN user: {} — skipping the P2P packet", room.room_id(), error);
                    return Err(error);
                }
            };

            let transport = tachyon_client.get_or_create_transport(room.room_id(), &msn_user);

            handle_p2p_packet(room.room_id(), transport, packet, tachyon_client.clone()).await;
            Ok(())
        }
        PayloadEffect::SendGif(gif_bytes) => {
            room.send_attachment("ink.gif", &mime::IMAGE_GIF, gif_bytes, AttachmentConfig::new()).await.map(|_| ())?;
            Ok(())
        }
        PayloadEffect::Nothing => Ok(()),
    }
}

#[cfg(test)]
mod tests {
    use std::str::FromStr;

    use matrix_sdk::ruma::events::room::message::MessageType;
    use matrix_sdk::ruma::OwnedUserId;
    use msnp::msnp::switchboard::command::msg::{MsgAcknowledgment, MsgPayload};
    use msnp::p2p::v2::p2p_transport_packet::P2PTransportPacket;
    use msnp::shared::models::email_address::EmailAddress;
    use msnp::shared::models::endpoint_id::EndpointId;
    use msnp::shared::models::msn_object::{FriendlyName, MSNObjectFactory, MsnObjectType};
    use msnp::shared::payload::msg::chunked_msg_payload::{ChunkedMsgPayload, ChunkMetadata, MsgChunks};
    use msnp::shared::payload::msg::control_msg::ControlMessagePayload;
    use msnp::shared::payload::msg::datacast_msg::{Datacast, DatacastMessagePayload};
    use msnp::shared::payload::msg::gif_msg::GifMsgPayload;
    use msnp::shared::payload::msg::p2p_msg_payload::P2PMessagePayload;
    use msnp::shared::payload::msg::raw_msg_payload::RawMsgPayload;
    use msnp::shared::payload::msg::text_plain_msg::TextPlainMessagePayload;
    use msnp::shared::traits::{IntoBytes, TryFromBytes};

    use super::{decide_msg_ack, decide_payload_effect, handle_chunked, render_msg_ack, MsgAck, PayloadEffect};
    use crate::switchboard::models::local_switchboard_data::LocalSwitchboardData;
    use crate::tachyon::mappers::user_id::MatrixIdCompatible;

    fn own_user_id() -> OwnedUserId {
        EmailAddress::from_str("aeon@test.com").unwrap().to_owned_user_id()
    }

    fn peer_endpoint() -> EndpointId {
        EndpointId::from_email_addr(EmailAddress::from_str("peer@test.com").unwrap())
    }

    fn test_lsd() -> LocalSwitchboardData {
        let (_, kill_recv) = tokio::sync::broadcast::channel::<()>(1);
        LocalSwitchboardData::new(kill_recv)
    }

    /// The chunker eats raw payloads: render the styled text message into
    /// its raw MIME form (the codec's own IntoBytes/TryFromBytes pair), so
    /// the merged chunks parse back into TextPlain like on the wire.
    fn big_text_raw(body: &str) -> RawMsgPayload {
        RawMsgPayload::try_from_bytes(TextPlainMessagePayload::new_with_default_style(body).into_bytes())
            .expect("a rendered text payload parses back into a raw payload")
    }

    /// The ack/NAK outcome table (§10 two-decide shape): every
    /// MsgAcknowledgment variant × Ok/Err, pinned to the pre-D4 inline
    /// matches — U never acks, N acks failures only, A/D always ack.
    #[test]
    fn the_ack_table_covers_every_ack_type_and_outcome() {
        use MsgAcknowledgment::{AckA, AckD, AckOnFailure, NoAck};

        let table: Vec<(&str, MsgAcknowledgment, bool, Option<MsgAck>)> = vec![
            ("U/err", NoAck, false, None),
            ("U/ok", NoAck, true, None),
            ("N/err", AckOnFailure, false, Some(MsgAck::Nak)),
            ("N/ok", AckOnFailure, true, None),
            ("A/err", AckA, false, Some(MsgAck::Nak)),
            ("A/ok", AckA, true, Some(MsgAck::Ack)),
            ("D/err", AckD, false, Some(MsgAck::Nak)),
            ("D/ok", AckD, true, Some(MsgAck::Ack)),
        ];

        for (row, ack_type, succeeded, expected) in table {
            assert_eq!(decide_msg_ack(ack_type, succeeded), expected, "ack table row {}", row);
        }
    }

    /// Wire pin (§2): the ack render bytes — the exact pre-D4
    /// AckServer/NakServer output.
    #[test]
    fn the_ack_render_pins_the_ack_and_nak_bytes() {
        assert_eq!(String::from_utf8(render_msg_ack(231, MsgAck::Ack).into_bytes()).unwrap(), "ACK 231\r\n");
        assert_eq!(String::from_utf8(render_msg_ack(231, MsgAck::Nak).into_bytes()).unwrap(), "NAK 231\r\n");
    }

    /// Payload-arm table: a plain text MSG decides the dedup-executor send
    /// with the body and the default style preserved.
    #[test]
    fn a_text_payload_decides_the_dedup_executor_send() {
        let effect = decide_payload_effect(
            MsgPayload::TextPlain(TextPlainMessagePayload::new_with_default_style("hello there")),
            &own_user_id(),
        );

        let PayloadEffect::SendText(content) = effect else {
            panic!("a text payload decides the SendText effect");
        };
        let MessageType::Text(text) = &content.msgtype else {
            panic!("the decided content is a text message");
        };
        assert_eq!(text.body, "hello there");
    }

    /// Payload-arm table: the nudge datacast decides the buzz send.
    #[test]
    fn a_nudge_datacast_decides_the_buzz_send() {
        let effect = decide_payload_effect(
            MsgPayload::Datacast(DatacastMessagePayload::new_nudge()),
            &own_user_id(),
        );
        assert!(matches!(effect, PayloadEffect::SendBuzz));
    }

    /// Payload-arm table: an announced voice clip decides the P2P object
    /// request, carrying the announced object.
    #[test]
    fn a_voice_clip_datacast_decides_the_object_request() {
        let clip = MSNObjectFactory::get_voice_message(b"", "peer@test.com".to_string(), FriendlyName::default());
        let effect = decide_payload_effect(
            MsgPayload::Datacast(DatacastMessagePayload::new_msn_object(clip)),
            &own_user_id(),
        );

        let PayloadEffect::RequestVoiceClip(msn_object) = effect else {
            panic!("a voice-clip announcement decides the object request");
        };
        assert_eq!(msn_object.obj_type, MsnObjectType::VoiceClip);
    }

    /// Payload-arm table (§5 honest no-ops): wink announcements, action
    /// messages and non-voice-clip object announcements decided Ok(())
    /// with no effect pre-D4 — they decide Nothing.
    #[test]
    fn unsupported_datacasts_decide_nothing() {
        let picture = MSNObjectFactory::get_display_picture_no_bytes(
            10,
            &EmailAddress::from_str("peer@test.com").unwrap(),
            "pic.tmp".to_string(),
            FriendlyName::default(),
        );

        let table = vec![
            MsgPayload::Datacast(DatacastMessagePayload { data: Datacast::Wink(picture.clone()) }),
            MsgPayload::Datacast(DatacastMessagePayload { data: Datacast::ActionMsg("waves".to_string()) }),
            MsgPayload::Datacast(DatacastMessagePayload::new_msn_object(picture)),
        ];

        for payload in table {
            assert!(matches!(decide_payload_effect(payload, &own_user_id()), PayloadEffect::Nothing));
        }
    }

    /// Payload-arm table: the typing control decides a notice ONLY for the
    /// room's own user — the pre-D4 filter, now a pure decision.
    #[test]
    fn typing_control_decides_a_notice_only_for_the_own_user() {
        let own = ControlMessagePayload::new(EmailAddress::from_str("aeon@test.com").unwrap());
        assert!(matches!(
            decide_payload_effect(MsgPayload::Control(own), &own_user_id()),
            PayloadEffect::TypingNotice
        ));

        let other = ControlMessagePayload::new(EmailAddress::from_str("peer@test.com").unwrap());
        assert!(matches!(
            decide_payload_effect(MsgPayload::Control(other), &own_user_id()),
            PayloadEffect::Nothing
        ));
    }

    /// Payload-arm table: a P2P MSG decides the packet transport,
    /// preserving the packet (the apply side keeps §3.6's inline
    /// ordering).
    #[test]
    fn a_p2p_payload_decides_the_packet_transport() {
        let packet = P2PMessagePayload::new(peer_endpoint(), peer_endpoint(), P2PTransportPacket::new(7, None), None);
        let effect = decide_payload_effect(MsgPayload::P2P(packet), &own_user_id());

        let PayloadEffect::P2P(transport_packet) = effect else {
            panic!("a P2P payload decides the packet transport");
        };
        assert_eq!(transport_packet.sequence_number, 7);
    }

    /// Payload-arm table: an ink gif decides the attachment send with the
    /// gathered bytes.
    #[test]
    fn a_gif_payload_decides_the_attachment_send() {
        let effect = decide_payload_effect(MsgPayload::Gif(GifMsgPayload::new(vec![1, 2, 3])), &own_user_id());

        let PayloadEffect::SendGif(bytes) = effect else {
            panic!("a gif payload decides the attachment send");
        };
        assert_eq!(bytes, vec![1, 2, 3]);
    }

    /// Payload-arm table (§5 honest no-ops): Raw and Chunked payloads
    /// produced Ok(()) with no effect pre-D4 and decide Nothing. (Ink is
    /// the third no-op arm but its payload type is not constructible
    /// outside the codec crate — the arm is covered by the exhaustive
    /// match, said honestly.)
    #[test]
    fn raw_and_chunked_payloads_decide_nothing() {
        assert!(matches!(
            decide_payload_effect(MsgPayload::Raw(RawMsgPayload::default()), &own_user_id()),
            PayloadEffect::Nothing
        ));

        let chunked = ChunkedMsgPayload::new(
            RawMsgPayload::default(),
            ChunkMetadata::First { message_id: "id".to_string(), chunks: 2 },
        );
        assert!(matches!(
            decide_payload_effect(MsgPayload::Chunked(chunked), &own_user_id()),
            PayloadEffect::Nothing
        ));
    }

    /// Chunk reassembly (shell-side state, §3.6): chunks park in lsd until
    /// the last one lands, then merge back into the original payload —
    /// the completion rule the SB wire depends on.
    #[tokio::test]
    async fn chunks_park_until_the_last_one_completes_the_message() {
        let mut lsd = test_lsd();
        // The codec's TextPlain parse trims trailing whitespace from the
        // reassembled body (frozen msnp behavior), so the pin body ends on
        // a non-space character.
        let body = format!("{}finis", "The quick brown fox jumps over the lazy dog. ".repeat(80));
        let raw = big_text_raw(&body);

        let mut chunks = MsgChunks::split_into_chunks(raw, 1311);
        let total = chunks.chunks_mut().len();
        assert!(total >= 2, "the test body must split into chunks");

        let mut drained = chunks.chunks_mut().drain(..).collect::<Vec<_>>();
        let last = drained.pop().expect("at least one chunk");

        for chunk in drained {
            assert!(handle_chunked(chunk, &mut lsd).await.expect("a chunk parks cleanly").is_none());
        }
        assert_eq!(lsd.chunks.len(), 1, "one message is parked mid-reassembly");

        let merged = handle_chunked(last, &mut lsd).await.expect("the final chunk merges").expect("the final chunk completes the message");
        assert!(lsd.chunks.is_empty(), "the completed message leaves no parked state");

        let MsgPayload::TextPlain(text) = merged else {
            panic!("the merged payload is the original text message");
        };
        assert_eq!(text.body, body);
    }

    /// Chunk reassembly: chunks of ONE message may arrive out of order —
    /// merge_chunks finds them by chunk number, so a first/third/second
    /// order still completes (the pre-D4 behavior, pinned).
    #[tokio::test]
    async fn out_of_order_chunks_still_merge_by_their_chunk_numbers() {
        let mut lsd = test_lsd();
        // Ends on a non-space character: the codec trims trailing
        // whitespace when it parses the merged body (see the sibling test).
        let body = format!("{}finis", "Lorem ipsum dolor sit amet, consectetur adipiscing elit. ".repeat(60));
        let raw = big_text_raw(&body);

        let mut chunks = MsgChunks::split_into_chunks(raw, 1311);
        let mut drained = chunks.chunks_mut().drain(..).collect::<Vec<_>>();
        assert!(drained.len() >= 3, "the test body must split into at least three chunks");

        let c1 = drained.remove(0);
        let c2 = drained.remove(0);
        let c3 = drained.remove(0);

        assert!(handle_chunked(c1, &mut lsd).await.unwrap().is_none());
        assert!(handle_chunked(c3, &mut lsd).await.unwrap().is_none(), "two of three chunks are not complete");
        let merged = handle_chunked(c2, &mut lsd).await.unwrap().expect("the third chunk completes the message");

        let MsgPayload::TextPlain(text) = merged else {
            panic!("the merged payload is the original text message");
        };
        assert_eq!(text.body, body);
    }

    /// Chunk reassembly: a message whose FIRST received chunk is a middle
    /// chunk is rejected (from_first_chunk) — the pre-D4 error the shell
    /// `?`s to the dispatcher's log.
    #[tokio::test]
    async fn a_message_starting_with_a_middle_chunk_errors_instead_of_parking() {
        let mut lsd = test_lsd();
        let body = "The quick brown fox jumps over the lazy dog. ".repeat(80);
        let raw = big_text_raw(&body);

        let mut chunks = MsgChunks::split_into_chunks(raw, 1311);
        let drained = chunks.chunks_mut().drain(..).collect::<Vec<_>>();
        let middle = drained.into_iter().nth(1).expect("a middle chunk");

        assert!(handle_chunked(middle, &mut lsd).await.is_err());
        assert!(lsd.chunks.is_empty(), "a rejected first chunk parks nothing");
    }

    /// Chunk reassembly: two interleaved messages keep separate parked
    /// state (keyed by message id) and complete independently.
    #[tokio::test]
    async fn chunks_from_two_messages_reassemble_independently() {
        let mut lsd = test_lsd();

        let raw_a = big_text_raw(&"A message body. ".repeat(120));
        let raw_b = big_text_raw(&"B message body. ".repeat(120));

        let mut chunks_a = MsgChunks::split_into_chunks(raw_a, 1311);
        let mut chunks_b = MsgChunks::split_into_chunks(raw_b, 1311);
        let mut a: Vec<ChunkedMsgPayload> = chunks_a.chunks_mut().drain(..).collect();
        let mut b: Vec<ChunkedMsgPayload> = chunks_b.chunks_mut().drain(..).collect();
        assert!(a.len() >= 2 && b.len() >= 2);

        let a0 = a.remove(0);
        let b0 = b.remove(0);
        assert!(handle_chunked(a0, &mut lsd).await.unwrap().is_none());
        assert!(handle_chunked(b0, &mut lsd).await.unwrap().is_none());
        assert_eq!(lsd.chunks.len(), 2, "both messages park side by side");

        for chunk in a {
            let _ = handle_chunked(chunk, &mut lsd).await.unwrap();
        }
        assert_eq!(lsd.chunks.len(), 1, "message A completed and left; B is still parked");
    }
}
