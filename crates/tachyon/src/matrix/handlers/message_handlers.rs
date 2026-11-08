//! Matrix message family (§8 D3): the closures in `handlers/mod.rs` are thin
//! shells; this module owns the gather → decide → deliver pipeline (§3).
//! gather reads the SDK (sender resolution, voice-clip transcode) into plain
//! facts; decide is the pure msgtype → MSN-payload decision table; the
//! MSN-bound delivery rides the dialect seam — `deliver()` (§3.4, D4) owns
//! the transport dispatch, with each pre-D3 warn policy preserved verbatim.
use crate::matrix::extensions::direct::DirectRoom;
use crate::matrix::extensions::msn_user_resolver::{member_to_msn_user, room_to_msn_user_lazy};
use crate::switchboard::extensions::CustomStyles;
use crate::tachyon::client::tachyon_client::TachyonClient;
use crate::tachyon::mappers::user_id::MatrixIdCompatible;
use log::{info, warn};
use matrix_sdk::ruma::events::room::message::{MessageType, OriginalSyncRoomMessageEvent};
use matrix_sdk::ruma::events::room::MediaSource;
use matrix_sdk::ruma::events::typing::SyncTypingEvent;
use matrix_sdk::{Client, Room};
use msnp::msnp::notification::models::dialect::Dialect;
use msnp::msnp::switchboard::command::command::SwitchboardServerCommand;
use msnp::msnp::switchboard::command::msg::{MsgPayload, MsgServer};
use msnp::shared::models::display_name::DisplayName;
use msnp::shared::models::endpoint_id::EndpointId;
use msnp::shared::models::msn_object::MsnObject;
use msnp::shared::models::msn_user::MsnUser;
use msnp::shared::payload::msg::control_msg::ControlMessagePayload;
use msnp::shared::payload::msg::datacast_msg::DatacastMessagePayload;
use msnp::shared::payload::msg::text_plain_msg::TextPlainMessagePayload;

/// The MSN-bound effects a room event can decide (§3). Bottom-up from the
/// real call sites: `SwitchboardServerCommand` is already plain data, so the
/// decided command rides along as-is and `label` keeps the pre-D3 per-arm
/// warn wording verbatim inside the delivery seam. This family decides no
/// Matrix-bound op — the to-room text sends live on the MSN front (§1.1),
/// not here.
// One or two ops exist per event and die in apply() — boxing the transfer
// variant would be indirection with no measured benefit (§9 item 5).
#[allow(clippy::large_enum_variant)]
pub(crate) enum MessageOp {
    /// `switchboard.receive_command(command)`; on Err apply warns
    /// `MSNP|SB: {label} dropped, client gone: {}`.
    Sb { label: &'static str, command: SwitchboardServerCommand },
    /// `tachyon_client.receive_file(..)` — the file-transfer invitation for
    /// File/Image/Video events and the audio-fallback arm.
    ReceiveFile { inviter: MsnUser, sender: MsnUser, size: usize, filename: String, source: MediaSource },
}

/// Gathered facts for one room message event — plain data only, every SDK/IO
/// read already resolved (§10: no `&Room`/`&Client`/async inside decide).
pub(crate) struct GatheredMessage {
    pub(crate) content: GatheredContent,
    /// The resolved message sender: the peer member, the direct-target
    /// stand-in (the room user) or the own user with endpoint id attached.
    pub(crate) sender: MsnUser,
    /// The room itself as an MSN user — the voice-clip MSG sender and the
    /// receive_file inviter.
    pub(crate) room_user: MsnUser,
}

/// The per-msgtype payload facts decide consumes.
// Per-event facts consumed immediately by decide — the audio variant's
// MsnObject is not worth a Box of indirection (§9 item 5).
#[allow(clippy::large_enum_variant)]
pub(crate) enum GatheredContent {
    /// Audio goes out as a WLM voice clip when it fits in one, and as a plain
    /// file otherwise: `voice_clip` is the gather-run transcode result
    /// (`None` = failed, cause already logged there) — the Ok/Err fork is
    /// decide's, made from a plain fact instead of mid-decision IO (§10).
    Audio { voice_clip: Option<MsnObject>, size: usize, filename: String, source: MediaSource },
    File { size: usize, filename: String, source: MediaSource },
    Image { size: usize, filename: String, source: MediaSource },
    Video { size: usize, filename: String, source: MediaSource },
    Notice(String),
    Text(String),
    /// The `chat.tachyon.buzz` custom event.
    Buzz,
    /// Emote, Location, ServerNotice, VerificationRequest, unknown custom
    /// events and any future msgtype: ignored pre-D3, ignored here — an
    /// honest passthrough, not a decision (§5).
    Ignored,
}

pub async fn handle_message(
    event: OriginalSyncRoomMessageEvent,
    room: Room,
    tachyon_client: TachyonClient,
    client: Client,
) {

    // The single dedup path (§3.5): the inbound echo check reads the
    // account's OWNED state — single-use remove semantics, exactly as the
    // pre-D3 global had, minus the cross-account suppression (H7).
    if tachyon_client.dedup().is_event_deduped(room.room_id(), event.event_id.as_ref()).await {
        return;
    }

    // §1.4 (D1/D2c precedent): pre-D3 this was `.unwrap()` — a room the
    // resolver rejects panicked the Matrix handler task. Now: warn + skip
    // the event; a failure-path-only delta, the success path is unchanged.
    let room_user = match room_to_msn_user_lazy(&room, tachyon_client.room_hashes()).await {
        Ok(room_user) => room_user,
        Err(error) => {
            warn!("Matrix: could not resolve room {} to an MSN user: {} — skipping message event", room.room_id(), error);
            return;
        }
    };
    // RNG-invite timing anchor (D3 shape, preserved exactly): the handle
    // is established BEFORE the gather, so the RNG invite fires at the
    // pre-D3 point even for events the gather later skips. deliver's
    // classic arm does the transport lookup itself (§3.4: the transport
    // is the arm's business); this call owns the side effect and its
    // timing, and the plain lookup below finds exactly this handle.
    tachyon_client.switchboards().get_or_initialize(room.room_id(), &room_user);

    // Sequential read → decide → deliver in the handler task (§3.6) — no
    // spawn per effect.
    let Some(gathered) = gather_message(event, &room, room_user, &tachyon_client).await else {
        return;
    };

    deliver(tachyon_client.dialect(), &tachyon_client, &room, decide_message(gathered)).await;
}

/// Gather (§3): the SDK/IO reads this flow needs, into plain facts — per-flow
/// by design, only what decide_message reads (§10: no mega-context). `None` =
/// skip the event; the failure warn is logged here, once.
async fn gather_message(
    event: OriginalSyncRoomMessageEvent,
    room: &Room,
    room_user: MsnUser,
    tachyon_client: &TachyonClient,
) -> Option<GatheredMessage> {
    let sender = if event.sender != room.own_user_id() {
        match room.get_single_direct_target() {
            None => {
                // §1.4: pre-D3 this was an `.unwrap().unwrap()...unwrap()`
                // chain — a missing member or resolver failure panicked the
                // Matrix handler task. Now: warn + skip the event.
                let member = match room.get_member_no_sync(event.sender.as_ref()).await {
                    Ok(Some(member)) => member,
                    Ok(None) => {
                        warn!("Matrix: message sender {} is not a member of {} — skipping event", event.sender, room.room_id());
                        return None;
                    }
                    Err(error) => {
                        warn!("Matrix: member lookup for message sender {} failed: {} — skipping event", event.sender, error);
                        return None;
                    }
                };
                // member → MsnUser is infallible (display name only) — the
                // pre-D3 trait Result on RoomMember could never fail.
                member_to_msn_user(&member)
            }
            Some(_direct_target) => {
                room_user.clone()
            }
        }

    } else {
        let mut own_user = tachyon_client.own_user();
        own_user.endpoint_id = EndpointId::from_email_addr(own_user.get_email_address().clone());
        own_user
    };

    let content = match &event.content.msgtype {
        MessageType::Audio(audio) => {
            // The transcode is IO, so its result is a gathered fact and the
            // voice-clip/file fork is decide's (§10: no IO inside decide —
            // pre-D3 this ran inside the msgtype match).
            let voice_clip = match tachyon_client.prepare_voice_clip(&room_user, audio).await {
                Ok(msn_object) => Some(msn_object),
                Err(e) => {
                    info!("Could not send audio message as a voice clip, falling back to a file transfer: {}", e);
                    None
                }
            };
            let size = audio.info.as_ref().map( |i| i.size.map(|u| usize::try_from(u).unwrap_or(0))).flatten().unwrap_or(0);
            let filename = audio.filename.as_ref().unwrap_or(&audio.body).to_owned();
            //TODO fix filename
            GatheredContent::Audio { voice_clip, size, filename, source: audio.source.clone() }
        }
        MessageType::Emote(_) => GatheredContent::Ignored,
        MessageType::File(file) => {
            let size = file.info.as_ref().map( |i| i.size.map(|u| usize::try_from(u).unwrap_or(0))).flatten().unwrap_or(0);
            let filename = file.filename.as_ref().unwrap_or(&file.body).to_owned();
            //TODO fix filename
            GatheredContent::File { size, filename, source: file.source.clone() }
        }
        MessageType::Image(image) => {
            let size = image.info.as_ref().map( |i| i.size.map(|u| usize::try_from(u).unwrap_or(0))).flatten().unwrap_or(0);
            let filename = image.filename.as_ref().unwrap_or(&image.body).to_owned();
            //TODO fix filename
            GatheredContent::Image { size, filename, source: image.source.clone() }
        }
        MessageType::Location(_) => GatheredContent::Ignored,
        MessageType::Notice(message) => GatheredContent::Notice(message.body.clone()),
        MessageType::ServerNotice(_) => GatheredContent::Ignored,
        MessageType::Text(message) => GatheredContent::Text(message.body.clone()),
        MessageType::Video(video) => {
            let size = video.info.as_ref().map( |i| i.size.map(|u| usize::try_from(u).unwrap_or(0))).flatten().unwrap_or(0);
            let filename = video.filename.as_ref().unwrap_or(&video.body).to_owned();
            //TODO fix filename
            GatheredContent::Video { size, filename, source: video.source.clone() }
        }
        MessageType::VerificationRequest(_) => GatheredContent::Ignored,
        MessageType::_Custom(_) => {
            match event.content.msgtype.msgtype() {
                "chat.tachyon.buzz" => GatheredContent::Buzz,
                &_ => GatheredContent::Ignored,
            }
        }
        _ => GatheredContent::Ignored,
    };

    Some(GatheredMessage { content, sender, room_user })
}

/// Decide (§3): the pure msgtype → MSN-payload decision table — pre-D3 this
/// mapping lived inline in the handler, fused with the sends. No IO, no SDK
/// access: facts in, ops out (§10).
pub(crate) fn decide_message(gathered: GatheredMessage) -> Vec<MessageOp> {
    let sender = &gathered.sender;
    let room_user = &gathered.room_user;

    match gathered.content {
        GatheredContent::Audio { voice_clip: Some(msn_object), .. } => {
            // Pre-D3 order preserved exactly: the notice goes FIRST, then the
            // voice-clip datacast — and apply stops on the first dropped
            // send, which is the pre-D3 early return on a dead client.
            vec![
                MessageOp::Sb {
                    label: "voice-clip notice",
                    command: SwitchboardServerCommand::MSG(MsgServer {
                        sender: sender.get_email_address().clone(),
                        display_name: DisplayName::new_from_ref(sender.compute_display_name()),
                        payload: MsgPayload::TextPlain(TextPlainMessagePayload {
                            font_family: Default::default(),
                            right_to_left: false,
                            font_styles: Default::default(),
                            font_color: Default::default(),
                            body: "has sent you an audio message".to_string(),
                        }),
                    }),
                },
                MessageOp::Sb {
                    label: "voice clip",
                    // Pre-D3 quirk preserved verbatim: the clip MSG's sender
                    // is the ROOM user's address while the display name is
                    // the message sender's.
                    command: SwitchboardServerCommand::MSG(MsgServer {
                        sender: room_user.get_email_address().clone(),
                        display_name: DisplayName::new_from_ref(sender.compute_display_name()),
                        payload: MsgPayload::Datacast(DatacastMessagePayload::new_msn_object(msn_object)),
                    }),
                },
            ]
        }
        GatheredContent::Audio { voice_clip: None, size, filename, source } => {
            vec![
                MessageOp::Sb {
                    label: "file-transfer notice",
                    command: SwitchboardServerCommand::MSG(MsgServer {
                        sender: sender.get_email_address().clone(),
                        display_name: DisplayName::new_from_ref(sender.compute_display_name()),
                        payload: MsgPayload::TextPlain(TextPlainMessagePayload {
                            font_family: Default::default(),
                            right_to_left: false,
                            font_styles: Default::default(),
                            font_color: Default::default(),
                            body: "has sent you a file".to_string(),
                        }),
                    }),
                },
                MessageOp::ReceiveFile { inviter: room_user.clone(), sender: sender.clone(), size, filename, source },
            ]
        }
        GatheredContent::File { size, filename, source }
        | GatheredContent::Image { size, filename, source }
        | GatheredContent::Video { size, filename, source } => {
            // Pre-D3 these arms sent NO notice — just the file-transfer
            // invite (only the audio fallback announces itself).
            vec![
                MessageOp::ReceiveFile { inviter: room_user.clone(), sender: sender.clone(), size, filename, source },
            ]
        }
        GatheredContent::Notice(body) => {
            vec![
                MessageOp::Sb {
                    label: "notice",
                    command: SwitchboardServerCommand::MSG(MsgServer {
                        sender: sender.get_email_address().clone(),
                        display_name: DisplayName::new_from_ref(sender.compute_display_name()),
                        payload: MsgPayload::TextPlain(TextPlainMessagePayload::new_with_notice_style(&body)),
                    }),
                },
            ]
        }
        GatheredContent::Text(body) => {
            vec![
                MessageOp::Sb {
                    label: "message",
                    command: SwitchboardServerCommand::MSG(MsgServer {
                        sender: sender.get_email_address().clone(),
                        display_name: DisplayName::new_from_ref(sender.compute_display_name()),
                        payload: MsgPayload::TextPlain(TextPlainMessagePayload::new_with_default_style(&body)),
                    }),
                },
            ]
        }
        GatheredContent::Buzz => {
            vec![
                MessageOp::Sb {
                    label: "nudge",
                    command: SwitchboardServerCommand::MSG(MsgServer {
                        sender: sender.get_email_address().clone(),
                        display_name: DisplayName::new_from_ref(sender.compute_display_name()),
                        payload: MsgPayload::Datacast(DatacastMessagePayload::new_nudge()),
                    }),
                },
            ]
        }
        // Honest passthrough (§5): Emote/Location/ServerNotice/
        // VerificationRequest/unknown customs produced nothing pre-D3 —
        // the empty vec IS the preserved behavior, not a new decision.
        GatheredContent::Ignored => vec![],
    }
}

/// The delivery seam (§3.4, D4): a FUNCTION, not a trait — the one place
/// the MSN-bound output forks on dialect. The shared body (the dedup
/// check, room resolution, the gather/decide translation) sits above in
/// the handler shells; deliver owns transport dispatch only and never
/// duplicates the D3 executor/dedup/resolver.
///
/// Seam site (delivery): the anticipated difference here is topology —
/// MSNP21 has no switchboard, so its arm (§7 M1) delivers the same
/// decided ops as SDG/UUM over the NS writer and answers XFR with a
/// refusal instead of the SB referral. That arm is anticipated, NOT
/// shipped — no stub, no dormant variant (§6 D); the day a second
/// `Dialect` variant exists, the compiler prints this site as part of
/// the contributor's work list.
///
/// Drop policy: a failed send warns and STOPS the batch — the pre-D4
/// apply_message policy, whose warn wording is preserved verbatim. The
/// typing flow's pre-D4 "warn and CONTINUE" converges to the same wire
/// output here: its ops all ride one handle, and every receive_command
/// error (a dead write task, a poisoned lock) is sticky, so the remaining
/// sends would have failed identically — a failure-path log-count delta
/// only, named in the D4 C2 commit body. The typing defect canary (a
/// ReceiveFile op in a typing batch) is subsumed: delivery is now shared,
/// and decide_typing still never produces one.
pub(crate) async fn deliver(dialect: Dialect, client: &TachyonClient, room: &Room, out: Vec<MessageOp>) {
    match dialect {
        // The classic arm: today's SB/XFR flow — SwitchboardServer keeps
        // listening, classic clients need it.
        Dialect::Msnp18 => {
            let Some(switchboard) = client.switchboards().get(room.room_id()) else {
                // Failure-path-only race (§1.4-class): the SB connection
                // tore down between the shell's lookup and this one.
                // Pre-D4 the ops rode the just-removed handle and died on
                // its dead sender with a warn; nothing reached the wire
                // then, nothing reaches it now.
                warn!("MSNP|SB: no switchboard for {} at delivery, client gone — {} ops dropped", room.room_id(), out.len());
                return;
            };

            for op in out {
                match op {
                    MessageOp::Sb { label, command } => {
                        // Err = the switchboard's write task is gone (client
                        // disconnected): stop handling the event instead of
                        // panicking the Matrix task (§8 D2, §10).
                        if let Err(error) = switchboard.receive_command(command).await {
                            warn!("MSNP|SB: {} dropped, client gone: {}", label, error);
                            return;
                        }
                    }
                    MessageOp::ReceiveFile { inviter, sender, size, filename, source } => {
                        client.receive_file(room.room_id(), &inviter, &sender, size, filename, source).await;
                    }
                }
            }
        }
    }
}

pub(crate) async fn handle_typing_notice(event: SyncTypingEvent, room: Room, tachyon_client: TachyonClient, client: Client) {
    // Pre-gather gate (D3 shape, preserved): no switchboard for the room =
    // nowhere to deliver typing — skip before gathering members. The
    // handle itself is deliver's classic-arm lookup.
    if !tachyon_client.switchboards().contains(room.room_id()) {
        return;
    }

    let gathered = gather_typing(&event, &room).await;

    deliver(tachyon_client.dialect(), &tachyon_client, &room, decide_typing(gathered)).await;
}

/// Gathered typing facts: the resolved MSN users behind the typing event,
/// own user excluded (the pre-D3 filter), event order preserved.
pub(crate) struct GatheredTyping {
    pub(crate) senders: Vec<MsnUser>,
}

/// Gather: member resolution with the pre-D3 graceful fallback — a member the
/// SDK or the resolver rejects degrades to the bare user-id mapping instead
/// of failing the event (no hazard here pre-D3, none introduced).
async fn gather_typing(event: &SyncTypingEvent, room: &Room) -> GatheredTyping {
    let mut senders = Vec::new();

    for user_id in event.content.user_ids.iter() {
        if user_id != room.own_user_id() {
            // Pre-D3 graceful fallback preserved: a member the SDK cannot
            // look up degrades to the bare user-id mapping instead of
            // failing the event.
            let member = room.get_member_no_sync(&user_id).await;
            let sender = match member {
                Ok(Some(member)) => member_to_msn_user(&member),
                _ => MsnUser::from_user_id(&user_id),
            };
            senders.push(sender);
        }
    }

    GatheredTyping { senders }
}

/// Decide: one typing Control MSG per gathered sender — a 1:1 mapping (§5:
/// passthrough, said honestly), but the family's single decision point.
pub(crate) fn decide_typing(gathered: GatheredTyping) -> Vec<MessageOp> {
    gathered.senders.into_iter().map(|sender| {
        MessageOp::Sb {
            label: "typing notification",
            command: SwitchboardServerCommand::MSG(MsgServer {
                sender: sender.get_email_address().clone(),
                display_name: DisplayName::new_from_ref(sender.compute_display_name()),
                payload: MsgPayload::Control(ControlMessagePayload::new(sender.get_email_address().clone())),
            }),
        }
    }).collect()
}

#[cfg(test)]
mod tests {
    use std::str::FromStr;

    use matrix_sdk::ruma::events::room::MediaSource;
    use matrix_sdk::ruma::{mxc_uri, room_id};
    use matrix_sdk::test_utils::mocks::MatrixMockServer;
    use msnp::msnp::models::contact_list::ContactList;
    use msnp::msnp::notification::models::dialect::Dialect;
    use msnp::msnp::switchboard::command::command::SwitchboardServerCommand;
    use msnp::msnp::switchboard::command::msg::{MsgPayload, MsgServer};
    use msnp::msnp::switchboard::models::session_id::SessionId;
    use msnp::shared::models::display_name::DisplayName;
    use msnp::shared::models::email_address::EmailAddress;
    use msnp::shared::models::endpoint_id::EndpointId;
    use msnp::shared::models::font_name::{DefaultFont, FontName};
    use msnp::shared::models::font_style::FontStyle;
    use msnp::shared::models::msn_object::{FriendlyName, MSNObjectFactory, MsnObject, MsnObjectType};
    use msnp::shared::models::msn_user::MsnUser;
    use msnp::shared::models::ticket_token::TicketToken;
    use msnp::shared::payload::msg::datacast_msg::{Datacast, DatacastType};
    use msnp::shared::payload::msg::text_plain_msg::TextPlainMessagePayload;
    use msnp::shared::traits::IntoBytes;

    use super::{decide_message, decide_typing, deliver, GatheredContent, GatheredMessage, GatheredTyping, MessageOp};
    use crate::switchboard::models::switchboard_handle::SwitchboardHandle;
    use crate::switchboard::switchboard_server::SwitchboardSenderMsg;
    use crate::tachyon::client::tachyon_client::TachyonClient;
    use crate::tachyon::config::tachyon_config::TachyonConfig;

    fn msn_user(email: &str, display_name: &str) -> MsnUser {
        let mut user = MsnUser::new(EndpointId::new(EmailAddress::from_str(email).unwrap(), None));
        user.display_name = Some(display_name.to_string());
        user
    }

    fn sample_object() -> MsnObject {
        MSNObjectFactory::get_display_picture_no_bytes(
            1234,
            &EmailAddress::from_str("peer@test.com").unwrap(),
            "clip.tmp".to_string(),
            FriendlyName::default(),
        )
    }

    fn media_source() -> MediaSource {
        MediaSource::Plain(mxc_uri!("mxc://test.local/abc123").to_owned())
    }

    /// MediaSource carries no PartialEq — assert the gathered mxc survives
    /// into the decided op instead.
    fn assert_media_source_preserved(source: &MediaSource) {
        match source {
            MediaSource::Plain(uri) => assert_eq!(uri, mxc_uri!("mxc://test.local/abc123")),
            _ => panic!("expected the plain gathered media source"),
        }
    }

    fn gathered(content: GatheredContent) -> GatheredMessage {
        GatheredMessage {
            content,
            sender: msn_user("peer@test.com", "Peer"),
            room_user: msn_user("roomhash@test.local", "The Room"),
        }
    }

    /// Unwrap one decided op into its label + MSG parts, failing the test
    /// with a prose message otherwise (MessageOp carries no Debug).
    fn sb_msg(op: &MessageOp) -> (&'static str, &MsgServer) {
        match op {
            MessageOp::Sb { label, command: SwitchboardServerCommand::MSG(msg) } => (label, msg),
            MessageOp::Sb { label, .. } => panic!("op {} decided a non-MSG switchboard command", label),
            MessageOp::ReceiveFile { .. } => panic!("expected an SB MSG op, got a file transfer"),
        }
    }

    fn text_body(msg: &MsgServer) -> &str {
        match &msg.payload {
            MsgPayload::TextPlain(text) => &text.body,
            _ => panic!("expected a TextPlain payload"),
        }
    }

    #[test]
    fn a_text_message_decides_to_a_single_default_style_msg() {
        let ops = decide_message(gathered(GatheredContent::Text("hello there".to_string())));

        assert_eq!(ops.len(), 1);
        let (label, msg) = sb_msg(&ops[0]);
        assert_eq!(label, "message");
        assert_eq!(msg.sender, EmailAddress::from_str("peer@test.com").unwrap());
        assert_eq!(msg.display_name.value(), "Peer");
        match &msg.payload {
            MsgPayload::TextPlain(text) => {
                assert_eq!(text.body, "hello there");
                assert_eq!(text.font_family, FontName::default_font());
            }
            _ => panic!("expected a TextPlain payload"),
        }
    }

    #[test]
    fn a_notice_decides_to_a_notice_style_msg() {
        let ops = decide_message(gathered(GatheredContent::Notice("system notice".to_string())));

        assert_eq!(ops.len(), 1);
        let (label, msg) = sb_msg(&ops[0]);
        assert_eq!(label, "notice");
        match &msg.payload {
            MsgPayload::TextPlain(text) => {
                assert_eq!(text.body, "system notice");
                assert!(text.font_styles.matches(FontStyle::Bold));
                assert!(text.font_styles.matches(FontStyle::Italic));
            }
            _ => panic!("expected a TextPlain payload"),
        }
    }

    #[test]
    fn the_buzz_custom_event_decides_to_a_nudge_datacast() {
        let ops = decide_message(gathered(GatheredContent::Buzz));

        assert_eq!(ops.len(), 1);
        let (label, msg) = sb_msg(&ops[0]);
        assert_eq!(label, "nudge");
        assert_eq!(msg.sender, EmailAddress::from_str("peer@test.com").unwrap());
        match &msg.payload {
            MsgPayload::Datacast(datacast) => assert_eq!(datacast.get_type(), DatacastType::Nudge),
            _ => panic!("expected a Datacast payload"),
        }
    }

    #[test]
    fn audio_with_a_prepared_clip_decides_the_notice_then_the_voice_clip_in_order() {
        let clip = sample_object();
        let expected_sha1d = clip.sha1d.clone();

        let ops = decide_message(gathered(GatheredContent::Audio {
            voice_clip: Some(clip),
            size: 999,
            filename: "unused.wav".to_string(),
            source: media_source(),
        }));

        assert_eq!(ops.len(), 2);

        // The notice first — pre-D3 order; a dropped notice stops the clip.
        let (label, notice) = sb_msg(&ops[0]);
        assert_eq!(label, "voice-clip notice");
        assert_eq!(text_body(notice), "has sent you an audio message");
        assert_eq!(notice.sender, EmailAddress::from_str("peer@test.com").unwrap());

        // Then the clip datacast, sent under the ROOM user's address with the
        // message sender's display name (pre-D3 quirk, preserved).
        let (label, clip_msg) = sb_msg(&ops[1]);
        assert_eq!(label, "voice clip");
        assert_eq!(clip_msg.sender, EmailAddress::from_str("roomhash@test.local").unwrap());
        assert_eq!(clip_msg.display_name.value(), "Peer");
        match &clip_msg.payload {
            MsgPayload::Datacast(datacast) => {
                assert_eq!(datacast.get_type(), DatacastType::MsnObject);
                let Datacast::MsnObject(msn_object) = &datacast.data else {
                    panic!("expected the voice-clip msn object");
                };
                assert_eq!(msn_object.obj_type, MsnObjectType::DisplayPicture);
                assert_eq!(msn_object.sha1d, expected_sha1d);
            }
            _ => panic!("expected a Datacast payload"),
        }
    }

    #[test]
    fn audio_whose_transcode_failed_decides_the_file_notice_then_receive_file() {
        let ops = decide_message(gathered(GatheredContent::Audio {
            voice_clip: None,
            size: 4242,
            filename: "long_audio.ogg".to_string(),
            source: media_source(),
        }));

        assert_eq!(ops.len(), 2);

        let (label, notice) = sb_msg(&ops[0]);
        assert_eq!(label, "file-transfer notice");
        assert_eq!(text_body(notice), "has sent you a file");

        match &ops[1] {
            MessageOp::ReceiveFile { inviter, sender, size, filename, source } => {
                assert_eq!(inviter.get_email_address().as_str(), "roomhash@test.local");
                assert_eq!(sender.get_email_address().as_str(), "peer@test.com");
                assert_eq!(*size, 4242);
                assert_eq!(filename, "long_audio.ogg");
                assert_media_source_preserved(source);
            }
            MessageOp::Sb { label, .. } => panic!("expected the file transfer after the notice, got another {} send", label),
        }
    }

    #[test]
    fn file_image_and_video_decide_to_a_bare_receive_file_without_a_notice() {
        // Pre-D3 (and after): only the AUDIO fallback announces itself with a
        // notice — plain file/image/video events just start the transfer.
        let table: Vec<(&str, GatheredContent, usize, &str)> = vec![
            ("file", GatheredContent::File { size: 1, filename: "doc.pdf".to_string(), source: media_source() }, 1, "doc.pdf"),
            ("image", GatheredContent::Image { size: 2, filename: "pic.png".to_string(), source: media_source() }, 2, "pic.png"),
            ("video", GatheredContent::Video { size: 3, filename: "clip.mp4".to_string(), source: media_source() }, 3, "clip.mp4"),
        ];

        for (name, content, expected_size, expected_filename) in table {
            let ops = decide_message(gathered(content));
            assert_eq!(ops.len(), 1, "{} must decide exactly one op", name);
            match &ops[0] {
                MessageOp::ReceiveFile { inviter, sender, size, filename, source } => {
                    assert_eq!(inviter.get_email_address().as_str(), "roomhash@test.local");
                    assert_eq!(sender.get_email_address().as_str(), "peer@test.com");
                    assert_eq!(*size, expected_size, "{} keeps its gathered size", name);
                    assert_eq!(filename, expected_filename, "{} keeps its gathered filename", name);
                    assert_media_source_preserved(source);
                }
                MessageOp::Sb { label, .. } => panic!("{} must not decide an SB send, got {}", name, label),
            }
        }
    }

    #[test]
    fn ignored_msgtypes_decide_to_no_ops_at_all() {
        // §5 honesty: Emote/Location/ServerNotice/VerificationRequest and
        // unknown custom events were silently ignored pre-D3 — `Ignored`
        // documents that passthrough; the empty vec IS the behavior.
        let ops = decide_message(gathered(GatheredContent::Ignored));
        assert!(ops.is_empty());
    }

    #[test]
    fn every_typing_user_but_us_decides_to_one_control_msg_each() {
        let gathered = GatheredTyping {
            senders: vec![
                msn_user("peer@test.com", "Peer"),
                msn_user("other@test.com", "Other"),
            ],
        };

        let ops = decide_typing(gathered);
        assert_eq!(ops.len(), 2);

        let expected = [("peer@test.com", "Peer"), ("other@test.com", "Other")];
        for (op, (email, display)) in ops.iter().zip(expected) {
            let (label, msg) = sb_msg(op);
            assert_eq!(label, "typing notification");
            assert_eq!(msg.sender, EmailAddress::from_str(email).unwrap());
            assert_eq!(msg.display_name.value(), display);
            match &msg.payload {
                MsgPayload::Control(control) => {
                    assert_eq!(control.typing_user, EmailAddress::from_str(email).unwrap());
                }
                _ => panic!("expected a Control payload"),
            }
        }
    }

    #[test]
    fn typing_with_no_resolved_senders_decides_to_nothing() {
        let ops = decide_typing(GatheredTyping { senders: vec![] });
        assert!(ops.is_empty());
    }

    // ---- the delivery seam (D4 C2) ----

    fn test_client() -> TachyonClient {
        let (snd, _rcv) = tokio::sync::mpsc::channel(10);
        let (kill_snd, kill_recv) = tokio::sync::broadcast::channel::<()>(1);
        let msn_user = MsnUser::new(EndpointId::new(EmailAddress::from_str("aeon@test.com").unwrap(), None));
        TachyonClient::new(TachyonConfig::default(), msn_user, TicketToken::default(), snd, kill_snd, kill_recv, ContactList::default())
    }

    /// §3.3's copy mechanics: the client's dialect defaults to the only
    /// profiled dialect, and set_dialect — the single auth-registration
    /// write — is what the accessor reads back. One owner, one copy, no
    /// parallel flags.
    #[test]
    fn the_dialect_copy_defaults_to_the_profiled_dialect_and_reads_back_what_auth_wrote() {
        let client = test_client();
        assert_eq!(client.dialect(), Dialect::Msnp18);
        client.set_dialect(Dialect::Msnp18);
        assert_eq!(client.dialect(), Dialect::Msnp18);
    }

    /// The classic arm: a room whose switchboard is still Initializing
    /// (RNG invited, the client has not ANSed yet) queues the decided ops
    /// into the handle's pending_events — the pre-D4 receive_command
    /// behavior, now behind the seam. The mock-server room keeps this
    /// socket-free (§9).
    #[tokio::test]
    async fn deliver_queues_decided_ops_into_an_initializing_switchboard() {
        let mock_server = MatrixMockServer::new().await;
        let matrix_client = mock_server.client_builder().build().await;
        let room = mock_server.sync_joined_room(&matrix_client, room_id!("!deliver:localhost")).await;

        let client = test_client();
        client.switchboards().insert(SwitchboardHandle::new(SessionId::new(7), room.room_id().to_owned(), Dialect::Msnp18.profile().max_msg_body_size)).unwrap();

        deliver(client.dialect(), &client, &room, vec![MessageOp::Sb { label: "message", command: SwitchboardServerCommand::OUT }]).await;

        let handle = client.switchboards().get(room.room_id()).expect("the handle survives delivery");
        let pending = handle.pending_events.lock().unwrap();
        assert_eq!(pending.len(), 1);
        assert!(matches!(pending[0], SwitchboardServerCommand::OUT));
    }

    /// The classic arm wire pin: a Ready switchboard gets each decided
    /// command delivered to the write channel in the Single envelope —
    /// the exact bytes the pre-D4 apply put on the wire (the oracle is
    /// the same registered MSG construction, §2's one-object rule).
    #[tokio::test]
    async fn deliver_sends_ready_switchboard_ops_to_the_write_channel_byte_identical() {
        let mock_server = MatrixMockServer::new().await;
        let matrix_client = mock_server.client_builder().build().await;
        let room = mock_server.sync_joined_room(&matrix_client, room_id!("!deliver:localhost")).await;

        let client = test_client();
        let (sb_snd, mut sb_rcv) = tokio::sync::mpsc::channel::<SwitchboardSenderMsg>(10);
        client.switchboards().insert(SwitchboardHandle::new_ready(SessionId::new(7), room.room_id().to_owned(), sb_snd, Dialect::Msnp18.profile().max_msg_body_size)).unwrap();

        fn decided_msg() -> SwitchboardServerCommand {
            SwitchboardServerCommand::MSG(MsgServer {
                sender: EmailAddress::from_str("peer@test.com").unwrap(),
                display_name: DisplayName::new_from_ref("Peer"),
                payload: MsgPayload::TextPlain(TextPlainMessagePayload::new_with_default_style("hello")),
            })
        }
        let expected = decided_msg().into_bytes();

        deliver(client.dialect(), &client, &room, vec![MessageOp::Sb { label: "message", command: decided_msg() }]).await;

        let received = sb_rcv.recv().await.expect("the write channel receives the delivered command");
        let SwitchboardSenderMsg::Single(command) = received else {
            panic!("the classic arm delivers Single envelopes, the pre-D4 shape");
        };
        assert_eq!(command.into_bytes(), expected);
    }

    /// The classic arm's failure path: no switchboard for the room (the
    /// teardown race) warns and drops the ops — no panic in the Matrix
    /// task, nothing on the wire, exactly what the pre-D4 ops-on-a-
    /// removed-handle path amounted to.
    #[tokio::test]
    async fn deliver_without_a_switchboard_drops_the_ops_without_panicking() {
        let mock_server = MatrixMockServer::new().await;
        let matrix_client = mock_server.client_builder().build().await;
        let room = mock_server.sync_joined_room(&matrix_client, room_id!("!deliver:localhost")).await;

        let client = test_client();
        deliver(client.dialect(), &client, &room, vec![MessageOp::Sb { label: "message", command: SwitchboardServerCommand::OUT }]).await;

        assert!(client.switchboards().get(room.room_id()).is_none());
    }
}
