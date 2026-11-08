use crate::matrix::extensions::direct::DirectRoom;
use crate::matrix::extensions::msn_user_resolver::{find_room_from_email, member_to_msn_user, room_to_msn_user_lazy};
use crate::switchboard::handlers::{BootstrapOutcome, ReadyWiring};
use crate::switchboard::models::connection_phase::ConnectionPhase;
use crate::switchboard::models::local_switchboard_data::LocalSwitchboardData;
use crate::switchboard::models::switchboard_token::SwitchboardToken;
use crate::tachyon::client::tachyon_client::TachyonClient;
use crate::tachyon::global_state::GlobalState;
use matrix_sdk::{Client, Room, RoomMemberships};
use msnp::msnp::switchboard::command::cal::{CalServer, CalServerFunction};
use msnp::msnp::switchboard::command::command::{SwitchboardClientCommand, SwitchboardServerCommand};
use msnp::msnp::switchboard::command::iro::IroServer;
use msnp::msnp::switchboard::command::joi::JoiServer;
use msnp::msnp::switchboard::command::usr::UsrClient;
use msnp::msnp::switchboard::models::session_id::SessionId;
use msnp::shared::command::ok::OkCommand;
use msnp::shared::models::endpoint_id::EndpointId;
use msnp::shared::models::msn_user::MsnUser;
use msnp::shared::models::ticket_token::TicketToken;
use std::str::FromStr;

const ROOM_USER_PORTAL_MODE: bool = true;

/// The Authenticating-phase shell (§3.1, pattern A): gathers the
/// token/client/room facts, mutates the connection's LocalSwitchboardData,
/// and returns the decided wire commands plus the post-send ready-wiring —
/// the Sender left this signature in D4. The `?` policies and the TODO
/// error replies are the pre-D4 ones.
pub(crate) async fn handle_auth(command: SwitchboardClientCommand, tachyon_state: &GlobalState, local_switchboard_data: &mut LocalSwitchboardData) -> Result<BootstrapOutcome, anyhow::Error> {

    match command {
        SwitchboardClientCommand::ANS(ans_command) => {
            let token = SwitchboardToken::try_from(ans_command.token.clone())?;

            let Some(tachyon_client) = tachyon_state.tachyon_clients().get(&token.matrix_token) else {
                return Ok(BootstrapOutcome::none());
            };
            let Some(matrix_client) = tachyon_client.matrix_client() else {
                return Ok(BootstrapOutcome::none());
            };
            match matrix_client.get_room(token.room_id.as_ref()) {
                        None => {}
                        Some(room) => {
                            let room_msn_user = room_to_msn_user_lazy(&room, tachyon_client.room_hashes()).await?;
                            local_switchboard_data.token = TicketToken(token.matrix_token);
                            local_switchboard_data.session_id = SessionId::random();
                            local_switchboard_data.email_addr = room_msn_user.get_email_address().clone();
                            local_switchboard_data.endpoint_guid = room_msn_user.endpoint_id.endpoint_guid.clone();
                            local_switchboard_data.tachyon_client = Some(tachyon_client.clone());
                            local_switchboard_data.matrix_client = Some(matrix_client.clone());
                            local_switchboard_data.room_id = Some(room.room_id().to_owned());
                            local_switchboard_data.room = Some(room.clone());
                            local_switchboard_data.phase = ConnectionPhase::Initializing;

                            //Send Initial roster = everyone but me
                            let initial_roster = if ROOM_USER_PORTAL_MODE {
                                vec![room_msn_user.clone()]
                            } else {
                                get_initial_roster_with_room_user(&room, room_msn_user.clone()).await?
                            };

                            let me = tachyon_client.own_user();
                            let ok_response = ans_command.get_ok_response();
                            let tr_id = ans_command.tr_id;

                            local_switchboard_data.phase = ConnectionPhase::Ready;

                            return Ok(BootstrapOutcome {
                                commands: decide_ans_bootstrap(tr_id, ok_response, initial_roster, me),
                                // The handle flip — and its pending-events
                                // flush — must land after these commands; the
                                // dispatcher runs it after the sends.
                                wiring: Some(ReadyWiring::FlipHandleReady { room_id: token.room_id }),
                            });
                        }
                    };


            //TODO send errors
            Ok(BootstrapOutcome::none())

        }
        SwitchboardClientCommand::USR(usr_command) => {

            // §1.4-class, fixed in place (D4): this was `.unwrap()` on
            // client-controlled input — a ticket without the t= prefix
            // panicked the connection task. Now `?` to the dispatcher's
            // error log, the same policy as the ANS arm's token parse
            // above. Failure-path-only delta.
            let token = TicketToken::from_str(&usr_command.token)?;

            let Some(client_data) = tachyon_state.tachyon_clients().get(token.as_str()) else {
                //TODO AUTH error
                return Ok(BootstrapOutcome::none());
            };
            let Some(matrix_client) = client_data.matrix_client() else {
                return Ok(BootstrapOutcome::none());
            };

            local_switchboard_data.email_addr = usr_command.endpoint_id.email_addr.clone();
            local_switchboard_data.endpoint_guid = usr_command.endpoint_id.endpoint_guid.clone();
            local_switchboard_data.token = token;
            local_switchboard_data.tachyon_client = Some(client_data);
            local_switchboard_data.matrix_client = Some(matrix_client);
            local_switchboard_data.session_id = SessionId::random();
            local_switchboard_data.phase = ConnectionPhase::Initializing;
            let email = &local_switchboard_data.email_addr;
            Ok(BootstrapOutcome {
                commands: vec![render_usr_ok(&usr_command, email.to_string())],
                wiring: None,
            })
        }
        // Honest empty arms (§5): CAL/MSG belong to later phases, OUT/RAW
        // during authentication produced nothing pre-D4 — the TODO below is
        // the preserved plan to answer them with an error.
        SwitchboardClientCommand::CAL(_) | SwitchboardClientCommand::MSG(_)
        | SwitchboardClientCommand::OUT | SwitchboardClientCommand::RAW(_) => {
            //TODO: send error when receiving another command
            Ok(BootstrapOutcome::none())
        }
    }
}

/// Decide (§3.1): the ANS bootstrap wire sequence — the initial roster's
/// IROs (1-based index, shared count), then the OK ack, then the own-user
/// JOI. Pre-D4 this order was implicit in the interleaved send calls; it
/// is the flow's real decision and is table-tested byte-exact below.
pub(crate) fn decide_ans_bootstrap(tr_id: u128, ok_response: OkCommand, roster: Vec<MsnUser>, me: MsnUser) -> Vec<SwitchboardServerCommand> {
    let count = roster.len() as u32;
    let mut commands = Vec::new();
    // The pre-D4 1-based index counter, as a zip instead of a mutable
    // counter (same wire values, no clippy counter loop).
    for (index, member) in (1u32..).zip(roster.iter()) {
        commands.extend(render_initial_roster_member(tr_id, index, count, member));
    }
    commands.push(SwitchboardServerCommand::OK(ok_response));
    commands.extend(render_initial_joined_member(&me));
    commands
}

/// Render: the USR auth OK response (pre-D4 `get_ok_response_for` inline
/// in the send).
pub(crate) fn render_usr_ok(usr_command: &UsrClient, display_name: String) -> SwitchboardServerCommand {
    SwitchboardServerCommand::USR(usr_command.get_ok_response_for(display_name))
}

/// Render: the CAL RINGING answer (pre-D4 inline CalServer construction).
pub(crate) fn render_cal_ringing(tr_id: u128, session_id: SessionId) -> SwitchboardServerCommand {
    SwitchboardServerCommand::CAL(CalServer {
        tr_id,
        function: CalServerFunction::RINGING,
        session_id,
    })
}

/// The Initializing-phase shell (§3.1, pattern A): same contract as
/// handle_auth — facts in, decided commands + wiring out, no Sender.
pub(crate) async fn handle_init(command: SwitchboardClientCommand, tachyon_client: TachyonClient, matrix_client: Client, local_switchboard_data: &mut LocalSwitchboardData) -> Result<BootstrapOutcome, anyhow::Error> {
    match command {
        // Honest empty arms (§5): ANS/USR were consumed by the
        // Authenticating phase; strays produced nothing pre-D4 and produce
        // nothing here.
        SwitchboardClientCommand::ANS(_) => {}
        SwitchboardClientCommand::USR(_) => {}
        SwitchboardClientCommand::CAL(cal_client) => {
            let email = cal_client.email_addr;

            let mut commands = vec![render_cal_ringing(cal_client.tr_id, local_switchboard_data.session_id.clone())];

            if email == local_switchboard_data.email_addr {
                // It's me !
                let me = tachyon_client.own_user();
                commands.extend(render_initial_joined_member(&me));
                return Ok(BootstrapOutcome { commands, wiring: None });
            }

            // §1.4 (D4): pre-D4 this was `.unwrap()` — a resolver failure
            // panicked the connection task and tore the whole SB connection
            // down. Now `?` to the dispatcher's error log, the policy of
            // the room_to_msn_user_lazy `?` right below it. Failure-path-
            // only delta; the success path is unchanged.
            let maybe_found = find_room_from_email(&matrix_client, tachyon_client.room_hashes(), &email)?;
            if let Some(room) = maybe_found {
                let target_room_user = room_to_msn_user_lazy(&room, tachyon_client.room_hashes()).await?;

                local_switchboard_data.room_id = Some(room.room_id().to_owned());
                local_switchboard_data.room = Some(room.clone());
                local_switchboard_data.phase = ConnectionPhase::Ready;

                commands.extend(render_initial_joined_member(&target_room_user));

                return Ok(BootstrapOutcome {
                    commands,
                    wiring: Some(ReadyWiring::InsertReadyHandle {
                        session_id: local_switchboard_data.session_id.clone(),
                        room_id: room.room_id().to_owned(),
                    }),
                });
            }
            //TODO: handle room not found
            return Ok(BootstrapOutcome { commands, wiring: None });
        }
        SwitchboardClientCommand::MSG(_) => {}
        SwitchboardClientCommand::OUT => {}
        SwitchboardClientCommand::RAW(_) => {}
    };
    Ok(BootstrapOutcome::none())
}

async fn get_initial_roster(room: &Room) -> Result<Vec<MsnUser>, anyhow::Error> {
    let mut out = Vec::new();

    let direct_target = room.get_single_direct_target();
    let members = room.members(RoomMemberships::JOIN).await?;

    for member in members {
        if let Some(direct_target_user_id) = direct_target.as_ref() {
            if member.user_id() == direct_target_user_id || member.user_id() == room.own_user_id() {
                continue;
            }
        }

        out.push(member_to_msn_user(&member));
    }

    Ok(out)
}

async fn get_initial_roster_with_room_user(room: &Room, room_msn_user: MsnUser) -> Result<Vec<MsnUser>, anyhow::Error> {
    let mut out = get_initial_roster(room).await?;
    out.push(room_msn_user);
    Ok(out)
}

/// Render: one initial-roster member's IRO pair — the bare-email IRO
/// always, the endpoint-guid IRO only when the member has a guid (the
/// pre-D4 send_initial_roster_member, byte-identical).
pub(crate) fn render_initial_roster_member(tr_id: u128, index: u32, count: u32, member: &MsnUser) -> Vec<SwitchboardServerCommand> {
    let mut commands = vec![SwitchboardServerCommand::IRO(IroServer::new(
        tr_id,
        index,
        count,
        member.compute_display_name().to_string(),
        member.endpoint_id.strip_endpoint_guid(),
        member.capabilities.clone()
    ))];

    if member.endpoint_id.endpoint_guid.is_some() {
        commands.push(SwitchboardServerCommand::IRO(IroServer::new(
            tr_id,
            index,
            count,
            member.compute_display_name().to_string(),
            member.endpoint_id.clone(),
            member.capabilities.clone()
        )));
    }
    commands
}

/// Render: a joining member's JOI pair — the same bare-then-guid rule as
/// the roster render (the pre-D4 send_initial_joined_member, byte-
/// identical).
pub(crate) fn render_initial_joined_member(member: &MsnUser) -> Vec<SwitchboardServerCommand> {
    let mut commands = vec![SwitchboardServerCommand::JOI(JoiServer {
        display_name: member.compute_display_name().to_string(),
        endpoint_id: EndpointId {
            email_addr: member.get_email_address().clone(),
            endpoint_guid: None,
        },
        capabilities: member.capabilities.clone(),
    })];

    if  member.endpoint_id.endpoint_guid.is_some() {
        commands.push(SwitchboardServerCommand::JOI(JoiServer {
            display_name: member.compute_display_name().to_string(),
            endpoint_id: member.endpoint_id.clone(),
            capabilities: member.capabilities.clone(),
        }));
    }

    commands
}

#[cfg(test)]
mod tests {
    use std::str::FromStr;
    use std::sync::Arc;

    use matrix_sdk::ruma::room_id;
    use msnp::msnp::switchboard::command::ans::AnsClient;
    use msnp::msnp::switchboard::command::command::{SwitchboardClientCommand, SwitchboardServerCommand};
    use msnp::msnp::switchboard::command::usr::UsrClient;
    use msnp::msnp::switchboard::models::session_id::SessionId;
    use msnp::shared::command::ok::OkCommand;
    use msnp::shared::models::email_address::EmailAddress;
    use msnp::shared::models::endpoint_id::EndpointId;
    use msnp::shared::models::msn_user::MsnUser;
    use msnp::shared::models::ticket_token::TicketToken;
    use msnp::shared::traits::IntoBytes;
    use tachyon_sqlite_store::SqliteCredentialsStore;

    use super::{decide_ans_bootstrap, handle_auth, render_cal_ringing, render_initial_joined_member, render_initial_roster_member, render_usr_ok};
    use crate::matrix::services::login::MatrixLoginServiceImpl;
    use crate::switchboard::models::connection_phase::ConnectionPhase;
    use crate::switchboard::models::local_switchboard_data::LocalSwitchboardData;
    use crate::switchboard::models::switchboard_token::SwitchboardToken;
    use crate::tachyon::global_state::GlobalState;

    /// The derived endpoint guid is deterministic (a seed over the email):
    /// aeon@test.com always renders this guid — the same literal the chg
    /// presence pin captured.
    const AEON_GUID: &str = "{1F8BFCBF-4F72-587D-8A18-489C017B448B}";
    /// ClientCapabilities::default() renders as this pair (the chg ILN pin
    /// captured it).
    const DEFAULT_CAPS: &str = "2754973748:3734755248";

    /// The guid derivation is part of the codec (a seeded Uuid), so a
    /// second email's literal is computed the same deterministic way the
    /// chg pin test computes its expected guid.
    fn derived_guid(email: &str) -> String {
        EndpointId::from_email_addr(EmailAddress::from_str(email).unwrap())
            .endpoint_guid.unwrap().to_string()
    }

    fn user_with_guid(email: &str, display: &str) -> MsnUser {
        let mut user = MsnUser::new(EndpointId::from_email_addr(EmailAddress::from_str(email).unwrap()));
        user.display_name = Some(display.to_string());
        user
    }

    fn user_bare(email: &str, display: &str) -> MsnUser {
        let mut user = MsnUser::without_endpoint_guid(EmailAddress::from_str(email).unwrap());
        user.display_name = Some(display.to_string());
        user
    }

    fn rendered(command: SwitchboardServerCommand) -> String {
        String::from_utf8(command.into_bytes()).unwrap()
    }

    fn test_credentials_store(name: &str) -> Arc<SqliteCredentialsStore> {
        let dir = std::env::temp_dir().join(format!("tachyon-{}-test-{}", name, std::process::id()));
        let _ = std::fs::remove_dir_all(&dir);
        Arc::new(SqliteCredentialsStore::open(&dir.join("credentials.db")).expect("store opens"))
    }

    fn test_state() -> GlobalState {
        GlobalState::new(Default::default(), Box::new(MatrixLoginServiceImpl::new()), test_credentials_store("sb-bootstrap"))
    }

    fn test_lsd() -> LocalSwitchboardData {
        let (_, kill_recv) = tokio::sync::broadcast::channel::<()>(1);
        LocalSwitchboardData::new(kill_recv)
    }

    /// Wire pin (§2): the JOI pair for a member WITH an endpoint guid — the
    /// bare-email JOI first, then the guid JOI — byte-identical to the
    /// pre-D4 send_initial_joined_member construction the render copied.
    #[test]
    fn render_initial_joined_member_pins_the_joi_pair_bytes() {
        let rendered: Vec<String> = render_initial_joined_member(&user_with_guid("aeon@test.com", "Aeon Test"))
            .into_iter().map(rendered).collect();

        assert_eq!(rendered, vec![
            format!("JOI aeon@test.com Aeon Test {}\r\n", DEFAULT_CAPS),
            format!("JOI aeon@test.com;{} Aeon Test {}\r\n", AEON_GUID, DEFAULT_CAPS),
        ]);
    }

    /// Wire pin (§2): a member WITHOUT a guid renders exactly one JOI —
    /// the bare-address form (the pre-D4 conditional preserved).
    #[test]
    fn render_initial_joined_member_renders_a_single_joi_without_a_guid() {
        let rendered: Vec<String> = render_initial_joined_member(&user_bare("peer@test.com", "Peer"))
            .into_iter().map(rendered).collect();

        assert_eq!(rendered, vec![
            format!("JOI peer@test.com Peer {}\r\n", DEFAULT_CAPS),
        ]);
    }

    /// Wire pin (§2): the IRO pair — same tr_id/index/count on both lines,
    /// bare address first, guid second (pre-D4 send_initial_roster_member).
    #[test]
    fn render_initial_roster_member_pins_the_iro_pair_bytes() {
        let rendered: Vec<String> = render_initial_roster_member(4, 1, 2, &user_with_guid("aeon@test.com", "Aeon Test"))
            .into_iter().map(rendered).collect();

        assert_eq!(rendered, vec![
            format!("IRO 4 1 2 aeon@test.com Aeon Test {}\r\n", DEFAULT_CAPS),
            format!("IRO 4 1 2 aeon@test.com;{} Aeon Test {}\r\n", AEON_GUID, DEFAULT_CAPS),
        ]);
    }

    /// The ANS bootstrap sequence (§3.1 decide): roster IROs in order with
    /// the shared count, then the OK, then the own-user JOI — the pre-D4
    /// send order, now a pure decision. Byte pins on every line.
    #[test]
    fn decide_ans_bootstrap_orders_roster_then_ok_then_own_joi() {
        let roster = vec![
            user_with_guid("room@test.local", "The Room"),
            user_bare("peer@test.com", "Peer"),
        ];
        let me = user_with_guid("aeon@test.com", "Aeon Test");
        let ok = OkCommand { operand: "ANS".to_string(), tr_id: 4 };

        let commands = decide_ans_bootstrap(4, ok, roster, me);

        let room_guid = derived_guid("room@test.local");
        let rendered: Vec<String> = commands.into_iter().map(rendered).collect();
        assert_eq!(rendered, vec![
            format!("IRO 4 1 2 room@test.local The Room {}\r\n", DEFAULT_CAPS),
            format!("IRO 4 1 2 room@test.local;{} The Room {}\r\n", room_guid, DEFAULT_CAPS),
            format!("IRO 4 2 2 peer@test.com Peer {}\r\n", DEFAULT_CAPS),
            "ANS 4 OK\r\n".to_string(),
            format!("JOI aeon@test.com Aeon Test {}\r\n", DEFAULT_CAPS),
            format!("JOI aeon@test.com;{} Aeon Test {}\r\n", AEON_GUID, DEFAULT_CAPS),
        ]);
    }

    /// Wire pin: the RINGING answer carries the connection's session id
    /// (pre-D4 inline CalServer).
    #[test]
    fn render_cal_ringing_pins_the_ringing_bytes() {
        let command = render_cal_ringing(9, SessionId::new(4242));
        assert_eq!(rendered(command), "CAL 9 RINGING 4242\r\n");
    }

    /// Wire pin: the USR OK response echoes tr_id + email and carries the
    /// display name the shell resolved (pre-D4 get_ok_response_for).
    #[test]
    fn render_usr_ok_pins_the_usr_response_bytes() {
        let usr = UsrClient {
            tr_id: 3,
            endpoint_id: EndpointId::from_email_addr(EmailAddress::from_str("aeon@test.com").unwrap()),
            token: "t=TICKET".to_string(),
        };
        assert_eq!(rendered(render_usr_ok(&usr, "aeon@test.com".to_string())), "USR 3 aeon@test.com aeon@test.com OK\r\n");
    }

    /// The token-validation facts handle_auth gathers on: the switchboard
    /// ticket is a base64 "room_id;matrix_token" csv — valid tickets parse,
    /// everything else is a CommandError the shell `?`s away (the pre-D4
    /// `?`, unchanged).
    #[test]
    fn a_switchboard_ticket_round_trips_and_bad_tickets_error() {
        let room_id = room_id!("!sbroom:localhost").to_owned();
        let token: TicketToken = SwitchboardToken::new(room_id.clone(), "matrix-token".to_string()).into();

        let parsed = SwitchboardToken::try_from(token).expect("the encoded ticket parses");
        assert_eq!(parsed.room_id, room_id);
        assert_eq!(parsed.matrix_token, "matrix-token");

        // Not base64 at all.
        assert!(SwitchboardToken::try_from(TicketToken("!!!not-base64!!!".to_string())).is_err());
        // Base64, but not a two-part csv.
        let one_part = TicketToken(encode_csv("only-a-room"));
        assert!(SwitchboardToken::try_from(one_part).is_err());
        // Two parts, but the room id is not a room id.
        let bad_room = TicketToken(encode_csv("not-a-room-id;token"));
        assert!(SwitchboardToken::try_from(bad_room).is_err());
    }

    fn encode_csv(csv: &str) -> String {
        use base64::engine::general_purpose;
        use base64::Engine;
        general_purpose::STANDARD.encode(csv)
    }

    /// Phase/token table: an ANS whose ticket names an unregistered matrix
    /// token produces NO wire commands and NO wiring, and the connection
    /// stays Authenticating — the pre-D4 silent return, now returned as
    /// data. Socket-free.
    #[tokio::test]
    async fn ans_with_an_unknown_token_produces_nothing_and_stays_authenticating() {
        let state = test_state();
        let mut lsd = test_lsd();

        let token: TicketToken = SwitchboardToken::new(room_id!("!sbroom:localhost").to_owned(), "unknown-matrix-token".to_string()).into();
        let ans = SwitchboardClientCommand::ANS(AnsClient {
            tr_id: 4,
            endpoint_id: EndpointId::from_email_addr(EmailAddress::from_str("aeon@test.com").unwrap()),
            token,
            session_id: 1234,
        });

        let outcome = handle_auth(ans, &state, &mut lsd).await.expect("an unknown token is not an error, it is silence");

        assert!(outcome.commands.is_empty());
        assert!(outcome.wiring.is_none());
        assert!(matches!(lsd.phase, ConnectionPhase::Authenticating));
    }

    /// Phase/token table: the USR arm behaves the same for an unregistered
    /// ticket — and a ticket without the t= prefix now `?`s to an error
    /// instead of panicking (§1.4-class, fixed in place in D4).
    #[tokio::test]
    async fn usr_with_an_unknown_ticket_produces_nothing_and_a_malformed_ticket_errors() {
        let state = test_state();
        let mut lsd = test_lsd();

        let usr = SwitchboardClientCommand::USR(UsrClient {
            tr_id: 3,
            endpoint_id: EndpointId::from_email_addr(EmailAddress::from_str("aeon@test.com").unwrap()),
            token: "t=unknown-ticket".to_string(),
        });
        let outcome = handle_auth(usr, &state, &mut lsd).await.expect("an unknown ticket is not an error, it is silence");
        assert!(outcome.commands.is_empty());
        assert!(outcome.wiring.is_none());
        assert!(matches!(lsd.phase, ConnectionPhase::Authenticating));

        let malformed = SwitchboardClientCommand::USR(UsrClient {
            tr_id: 3,
            endpoint_id: EndpointId::from_email_addr(EmailAddress::from_str("aeon@test.com").unwrap()),
            token: "missing-prefix".to_string(),
        });
        assert!(handle_auth(malformed, &state, &mut lsd).await.is_err());
        assert!(matches!(lsd.phase, ConnectionPhase::Authenticating));
    }
}
