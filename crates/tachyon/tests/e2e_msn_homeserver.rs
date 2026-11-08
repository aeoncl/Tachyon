use std::path::PathBuf;
use std::str::FromStr;
use std::time::Duration;
use std::time::UNIX_EPOCH;

use msnp::msnp::notification::command::command::NotificationClientCommand;
use msnp::msnp::notification::command::command::NotificationServerCommand;
use msnp::msnp::notification::command::command::NotificationServerCommand as Cmd;
use msnp::msnp::notification::command::adl::AdlClient;
use msnp::msnp::notification::command::chg::{ChgClient, ChgServer};
use msnp::msnp::notification::command::cvr::{CvrClient, CvrServer};
use msnp::msnp::notification::command::msg::MsgServer;
use msnp::msnp::notification::command::not::NotServer;
use msnp::msnp::notification::command::usr::{
    AuthOperationTypeClient, AuthPolicy, OperationTypeServer, SsoPhaseClient, SsoPhaseServer,
    UsrClient,
};
use msnp::msnp::notification::command::ver::VerClient;
use msnp::msnp::notification::models::adl_payload::ADLPayload;
use msnp::msnp::notification::models::endpoint_guid::EndpointGuid;
use msnp::msnp::notification::models::dialect::{dialect_for, Dialect};
use msnp::msnp::notification::models::msnp_version::MsnpVersion;
use msnp::msnp::raw_command_parser::ServerCommandParser;
use msnp::shared::models::capabilities::ClientCapabilities;
use msnp::shared::models::email_address::EmailAddress;
use msnp::shared::models::presence_status::PresenceStatus;
use msnp::shared::models::ticket_token::TicketToken;
use msnp::shared::traits::{IntoBytes, TryFromRawCommand};
use tachyon::tachyon::config::tachyon_config::TachyonConfig;
use tachyon::tachyon::identifiers::bridge_link::BridgeLinkToken;
use testcontainers::core::wait::HttpWaitStrategy;
use testcontainers::core::{ContainerPort, WaitFor};
use testcontainers::runners::AsyncRunner;
use testcontainers::{ContainerAsync, GenericImage, ImageExt};
use tokio::io::{AsyncReadExt, AsyncWriteExt, BufReader};
use tokio::net::TcpStream;

const SERVER_NAME: &str = "example.org";
const PASSWORD: &str = "correct-horse-battery-staple";
const REGISTRATION_TOKEN: &str = "tachyon-e2e-registration-token";
const DOMAIN: &str = "example.org";
const CLIENT_USER_AGENT: &str = "MSNMSGR/14.0.8064.0205 msmsgs";
const ENDPOINT_GUID: &str = "{C8D73CAE-A6F5-491B-A74B-A85E657CFC2C}";

// 90s: cold container + matrix-sdk login is the slowest leg
const STAGE_TIMEOUT: Duration = Duration::from_secs(90);

// TACHYON_* env is process-global; hold this across the whole test body
static E2E_LOCK: tokio::sync::Mutex<()> = tokio::sync::Mutex::const_new(());

#[tokio::test(flavor = "multi_thread")]
#[ignore = "requires a reachable Docker daemon (DOCKER_HOST); run with: cargo test -p tachyon --test e2e_msn_homeserver -- --ignored"]
async fn msn_client_logs_in_through_tachyon_against_a_real_homeserver() {
    let _e2e = E2E_LOCK.lock().await;
    init_logging();

    let run = Run::fresh();
    let stack = Stack::start(&run).await;

    let ticket = mint_ticket(stack.http_port, run.email.as_str()).await;

    let mut msn = MsnClient::connect(stack.notification_port).await;
    handshake(&mut msn, &run.email, &ticket).await;
    sign_in_through_web(
        &mut msn,
        stack.http_port,
        &ticket,
        &stack.homeserver_url,
        &run.local_part,
    )
    .await;
    await_ready_sequence(&mut msn, &ticket, &[]).await;
    chg_echo(&mut msn).await;

    stack.shutdown().await;

    assert!(
        any_store_file(&run.home.join("local")),
        "the matrix-sdk store must be materialized under {}",
        run.home.join("local").display()
    );
}

#[tokio::test(flavor = "multi_thread")]
#[ignore = "requires a reachable Docker daemon (DOCKER_HOST); run with: cargo test -p tachyon --test e2e_msn_homeserver -- --ignored"]
async fn msn_client_reconnects_to_a_persisted_session() {
    let _e2e = E2E_LOCK.lock().await;
    init_logging();

    let run = Run::fresh();
    let mut stack = Stack::start(&run).await;

    let ticket = mint_ticket(stack.http_port, run.email.as_str()).await;
    let mut msn = MsnClient::connect(stack.notification_port).await;
    handshake(&mut msn, &run.email, &ticket).await;
    sign_in_through_web(
        &mut msn,
        stack.http_port,
        &ticket,
        &stack.homeserver_url,
        &run.local_part,
    )
    .await;
    await_ready_sequence(&mut msn, &ticket, &[]).await;
    chg_echo(&mut msn).await;

    drop(msn);
    tokio::time::sleep(Duration::from_secs(1)).await;
    stack.kill_tachyon().await;

    let replayed = mint_ticket(stack.http_port, run.email.as_str()).await;
    assert_eq!(replayed, ticket, "the ticket is stable per client identity");

    let mut msn2 = MsnClient::connect(stack.notification_port).await;
    handshake(&mut msn2, &run.email, &ticket).await;
    await_ready_sequence(
        &mut msn2,
        &ticket,
        &["Sign in required", "Device verification required"],
    )
    .await;

    stack.shutdown().await;
}

#[tokio::test(flavor = "multi_thread")]
#[ignore = "requires a reachable Docker daemon (DOCKER_HOST); run with: cargo test -p tachyon --test e2e_msn_homeserver -- --ignored"]
async fn abch_pulls_while_login_is_in_flight_answer_empty() {
    let _e2e = E2E_LOCK.lock().await;
    init_logging();

    let run = Run::fresh();
    let stack = Stack::start(&run).await;

    let ticket = mint_ticket(stack.http_port, run.email.as_str()).await;

    let mut msn = MsnClient::connect(stack.notification_port).await;
    handshake(&mut msn, &run.email, &ticket).await;

    // USR OK has just been sent: the matrix login is still in flight, and the Messenger client
    // fires these two ABCH pulls right after it, so they must not fault.
    let contacts = soap_pull(
        &stack,
        "/abservice/abservice.asmx",
        "http://www.msn.com/webservices/AddressBook/ABFindContactsPaged",
        &find_contacts_paged_request(&ticket),
    )
    .await;
    assert_eq!(contacts.status, 200, "the contact pull must not error before login completes: {}", contacts.body);
    assert!(!contacts.body.contains("<soap:Fault"), "no SOAP fault before login completes: {}", contacts.body);
    assert!(contacts.body.contains("ABFindContactsPagedResult"), "the pull must carry the AB result: {}", contacts.body);
    assert!(contacts.body.contains("12r1:e2e-cache-key"), "the cache key must be echoed back: {}", contacts.body);
    assert_eq!(contacts.body.matches("<passportName>").count(), 1, "only the own contact is expected before login completes: {}", contacts.body);

    let memberships = soap_pull(
        &stack,
        "/abservice/SharingService.asmx",
        "http://www.msn.com/webservices/AddressBook/FindMembership",
        &find_membership_request(&ticket),
    )
    .await;
    assert_eq!(memberships.status, 200, "the membership pull must not error before login completes: {}", memberships.body);
    assert!(!memberships.body.contains("<soap:Fault"), "no SOAP fault before login completes: {}", memberships.body);
    assert!(memberships.body.contains("FindMembershipResult"), "the pull must carry the membership result: {}", memberships.body);
    assert!(memberships.body.contains("12r1:e2e-cache-key"), "the cache key must be echoed back: {}", memberships.body);
    assert_eq!(memberships.body.matches("PassportMember").count(), 0, "no members are expected before login completes: {}", memberships.body);

    stack.shutdown().await;
}

// ==================== PER-DIALECT CONFORMANCE SCRIPT (D5) ====================
//
// The MSNP18 script below is the shape a contributor copies for a new dialect
// (doc/adding-a-dialect.md). It is a wire-contract conformance run: a fake MSN
// client (MsnClient) drives a real Tachyon stack (Stack) over real sockets and
// asserts, step by step, what the dialect promised — with every expectation
// that IS profile data read from `Dialect::profile()` itself, so the script
// pins the record-to-wire mapping instead of a second copy of the record.
//
// To write the script for a new dialect, COPY `msnp18_dialect_conformance_script`
// and change exactly three things:
//   1. the dialect: `Dialect::Msnp18` -> the new variant. The compiler prints
//      the work list: every seam in doc/architecture.md §3.3's inventory must
//      have grown an arm before this line compiles at all;
//   2. the negotiation offer: `handshake` offers MSNP18/MSNP17 and pins the
//      MSNP18 SSO shape — a new script offers the versions ITS dialect
//      negotiates (a v21 script offers MSNP21) and inlines its own exchange
//      where the handshake shape itself differs;
//   3. the steps whose contract differs per the inventory's NAMED differences
//      (e.g. §7 M1: no switchboard — presence and delivery ride SDG/UUM over
//      the NS; §7 M2: the auth trigger is a host-launched browser, not the
//      in-protocol NOT alert this script asserts through sign_in_through_web).
// What does NOT change: the harness (Run/Stack/MsnClient/helpers — reused, never
// re-invented), the step structure (negotiation -> auth trigger -> ready
// sequence -> presence echo), and the profile-read discipline.
//
// The chunked-MSG contract is deliberately NOT exercised here: this harness
// has no switchboard client, and inventing one would violate the reuse rule.
// The chunker is anchored where the machinery already exists — the msg_handler
// chunk table tests pin the split behavior against the literal cap, and the
// dialect-profile test in msnp pins the cap value itself (1311).
// ==============================================================================

#[tokio::test(flavor = "multi_thread")]
#[ignore = "requires a reachable Docker daemon (DOCKER_HOST); run with: cargo test -p tachyon --test e2e_msn_homeserver -- --ignored msnp18_dialect_conformance_script"]
async fn msnp18_dialect_conformance_script() {
    let _e2e = E2E_LOCK.lock().await;
    init_logging();

    // The dialect under conformance and its profile record: the single source
    // of every profile-data expectation below.
    let dialect = Dialect::Msnp18;
    let profile = dialect.profile();

    let run = Run::fresh();
    let stack = Stack::start(&run).await;
    let ticket = mint_ticket(stack.http_port, run.email.as_str()).await;

    let mut msn = MsnClient::connect(stack.notification_port).await;

    // Step 1 — negotiation (seam: dialect.rs profile()/dialect_for). The
    // offered wire vocabulary resolves to THIS dialect, the agreed version
    // maps back to it through the one exhaustive mapping, and the CVR reply
    // carries exactly the profile's strings — record and wire agree.
    let negotiated = handshake(&mut msn, &run.email, &ticket).await;
    assert_eq!(
        dialect_for(negotiated.agreed_version),
        Some(dialect),
        "the agreed wire version must map to the dialect under conformance"
    );
    assert_eq!(negotiated.cvr.rec_client_ver, profile.cvr_version);
    assert_eq!(negotiated.cvr.rec_client_ver2, profile.cvr_version);
    assert_eq!(negotiated.cvr.min_client_ver, profile.cvr_version);
    assert_eq!(negotiated.cvr.client_dl_url, profile.cvr_host);
    assert_eq!(negotiated.cvr.client_info_url, profile.cvr_host);

    // Step 2 — auth trigger (seam: auth.rs render_auth_trigger; §3.7). The
    // ticket is the boundary: the in-protocol NOT alert this dialect renders
    // must carry the web-login URL keyed by the ticket (asserted inside), and
    // the password crosses only the web page — never the MSN wire. USR OK +
    // the SSO policy shape are asserted inside handshake.
    sign_in_through_web(
        &mut msn,
        stack.http_port,
        &ticket,
        &stack.homeserver_url,
        &run.local_part,
    )
    .await;

    // Step 3 — ready sequence: the backend attaches with no repeated auth
    // trigger (none of the unexpected toasts fired) and the profile MSG
    // payload — this harness's NS message flow — reaches the client.
    await_ready_sequence(&mut msn, &ticket, &[]).await;

    // Step 4 — presence render (seam: chg_handler.rs render). CHG NLN echoes
    // back NLN in this dialect's presence wire shape.
    chg_echo(&mut msn).await;

    stack.shutdown().await;
}

struct SoapResponse {
    status: u16,
    body: String,
}

async fn soap_pull(stack: &Stack, path: &str, soap_action: &str, body: &str) -> SoapResponse {
    let client = reqwest::Client::new();
    let response = client
        .post(format!("http://127.0.0.1:{}{}", stack.http_port, path))
        .header("SOAPAction", format!("\"{soap_action}\""))
        .header("User-Agent", CLIENT_USER_AGENT)
        .header("content-type", "application/soap+xml")
        .body(body.to_string())
        .send()
        .await
        .expect("soap request");
    let status = response.status().as_u16();
    let text = response.text().await.unwrap_or_default();
    SoapResponse { status, body: text }
}

fn find_contacts_paged_request(ticket: &BridgeLinkToken) -> String {
    format!(
        r#"<?xml version="1.0" encoding="utf-8"?>
        <soap:Envelope xmlns:soap="http://schemas.xmlsoap.org/soap/envelope/" xmlns:xsi="http://www.w3.org/2001/XMLSchema-instance" xmlns:xsd="http://www.w3.org/2001/XMLSchema" xmlns:soapenc="http://schemas.xmlsoap.org/soap/encoding/">
            <soap:Header>
                <ABApplicationHeader xmlns="http://www.msn.com/webservices/AddressBook">
                    <ApplicationId>AAD9B99B-58E6-4F23-B975-D9EC1F9EC24A</ApplicationId>
                    <IsMigration>false</IsMigration>
                    <PartnerScenario>Initial</PartnerScenario>
                    <CacheKey>12r1:e2e-cache-key</CacheKey>
                </ABApplicationHeader>
                <ABAuthHeader xmlns="http://www.msn.com/webservices/AddressBook">
                    <ManagedGroupRequest>false</ManagedGroupRequest>
                    <TicketToken>t={ticket}</TicketToken>
                </ABAuthHeader>
            </soap:Header>
            <soap:Body>
                <ABFindContactsPaged xmlns="http://www.msn.com/webservices/AddressBook">
                    <filterOptions>
                        <DeltasOnly>true</DeltasOnly>
                        <LastChanged>2022-04-21T19:49:28Z</LastChanged>
                        <ContactFilter>
                            <IncludeHiddenContacts>true</IncludeHiddenContacts>
                        </ContactFilter>
                    </filterOptions>
                    <abView>MessengerClient8</abView>
                    <extendedContent>AB AllGroups CircleResult</extendedContent>
                </ABFindContactsPaged>
            </soap:Body>
        </soap:Envelope>"#
    )
}

fn find_membership_request(ticket: &BridgeLinkToken) -> String {
    format!(
        r#"<?xml version="1.0" encoding="utf-8"?>
        <soap:Envelope xmlns:soap="http://schemas.xmlsoap.org/soap/envelope/" xmlns:xsi="http://www.w3.org/2001/XMLSchema-instance" xmlns:xsd="http://www.w3.org/2001/XMLSchema" xmlns:soapenc="http://schemas.xmlsoap.org/soap/encoding/">
            <soap:Header>
                <ABApplicationHeader xmlns="http://www.msn.com/webservices/AddressBook">
                    <ApplicationId>AAD9B99B-58E6-4F23-B975-D9EC1F9EC24A</ApplicationId>
                    <IsMigration>false</IsMigration>
                    <PartnerScenario>Initial</PartnerScenario>
                    <CacheKey>12r1:e2e-cache-key</CacheKey>
                </ABApplicationHeader>
                <ABAuthHeader xmlns="http://www.msn.com/webservices/AddressBook">
                    <ManagedGroupRequest>false</ManagedGroupRequest>
                    <TicketToken>t={ticket}</TicketToken>
                </ABAuthHeader>
            </soap:Header>
            <soap:Body>
                <FindMembership xmlns="http://www.msn.com/webservices/AddressBook">
                    <serviceFilter>
                        <Types>
                            <ServiceType>Messenger</ServiceType>
                            <ServiceType>SocialNetwork</ServiceType>
                            <ServiceType>Space</ServiceType>
                            <ServiceType>Profile</ServiceType>
                        </Types>
                    </serviceFilter>
                    <View>Full</View>
                    <deltasOnly>true</deltasOnly>
                    <lastChange>2022-04-20T13:03:28Z</lastChange>
                </FindMembership>
            </soap:Body>
        </soap:Envelope>"#
    )
}

fn any_store_file(dir: &std::path::Path) -> bool {
    fn walk(dir: &std::path::Path) -> bool {
        let Ok(entries) = std::fs::read_dir(dir) else {
            return false;
        };
        entries.filter_map(|entry| entry.ok()).any(|entry| {
            entry.file_name().to_string_lossy().contains(".sqlite3")
                || (entry.path().is_dir() && walk(&entry.path()))
        })
    }
    walk(dir)
}

/// The negotiation evidence a conformance script asserts on: the agreed
/// wire version and the full CVR reply. The flow tests above discard the
/// return value; the D5 conformance script reads it and checks it against
/// the dialect's own profile record. Behavior is unchanged for every
/// existing caller.
struct Negotiated {
    agreed_version: MsnpVersion,
    cvr: CvrServer,
}

async fn handshake(msn: &mut MsnClient, email: &str, ticket: &BridgeLinkToken) -> Negotiated {
    msn.send(NotificationClientCommand::VER(VerClient::new(1, MsnpVersion::MSNP18, MsnpVersion::MSNP17)))
        .await;
    let Cmd::VER(ver) = msn.next().await else {
        panic!("expected the agreed version");
    };
    assert_eq!(ver.agreed_version, MsnpVersion::MSNP18);
    let agreed_version = ver.agreed_version;

    msn.send(NotificationClientCommand::CVR(CvrClient::new(
        2,
        0x0409,
        "winnt".to_string(),
        "6.2.0".to_string(),
        "i386".to_string(),
        "MSNMSGR".to_string(),
        "14.0.8064.0205".to_string(),
        "msmsgs".to_string(),
        email.to_string(),
    )))
    .await;
    let Cmd::CVR(cvr) = msn.next().await else {
        panic!("expected the CVR reply");
    };
    assert_eq!(cvr.tr_id, 2);

    msn.send(NotificationClientCommand::USR(UsrClient {
        tr_id: 3,
        auth_type: AuthOperationTypeClient::Sso(SsoPhaseClient::I {
            email_addr: EmailAddress::from_str(email).expect("email"),
        }),
    }))
    .await;
    let nonce = match msn.next().await {
        Cmd::USR(usr) => match usr.auth_type {
            OperationTypeServer::Sso(SsoPhaseServer::S { policy, nonce }) => {
                assert!(matches!(policy, AuthPolicy::MbiKeyOld));
                nonce
            }
            _ => panic!("expected the SSO challenge, got {usr}"),
        },
        command => panic!("expected the SSO challenge, got {command}"),
    };
    assert!(!nonce.is_empty());

    let gcf = msn.next().await;
    assert!(
        matches!(gcf, Cmd::GCF(_)),
        "expected the shields payload after the challenge"
    );

    msn.send(NotificationClientCommand::USR(UsrClient {
        tr_id: 4,
        auth_type: AuthOperationTypeClient::Sso(SsoPhaseClient::S {
            ticket_token: TicketToken::from_str(&format!("t={ticket}")).expect("ticket"),
            challenge: nonce,
            endpoint_guid: EndpointGuid::from_str(ENDPOINT_GUID).expect("endpoint guid"),
        }),
    }))
    .await;
    let usr = msn.next().await;
    let Cmd::USR(usr) = usr else {
        panic!("expected USR OK after the handshake");
    };
    let OperationTypeServer::Ok { email_addr, .. } = &usr.auth_type else {
        panic!("expected USR OK, got {usr}");
    };
    assert_eq!(email_addr.as_str(), email);

    Negotiated { agreed_version, cvr }
}

async fn sign_in_through_web(
    msn: &mut MsnClient,
    http_port: u16,
    ticket: &BridgeLinkToken,
    homeserver_url: &str,
    local_part: &str,
) {
    msn.send(NotificationClientCommand::PNG).await;
    let (mut toast, mut qng) = (None, false);
    while toast.is_none() || !qng {
        match msn.next().await {
            Cmd::NOT(not) => toast = Some(not),
            Cmd::QNG(_) => qng = true,
            _ => {}
        }
    }
    let toast_text = toast_payload(toast.expect("sign-in toast"));
    assert!(
        toast_text.contains("/tachyon/login?") && toast_text.contains(ticket.as_str()),
        "the sign-in toast must carry the login URL with the ticket, got: {toast_text}"
    );

    let adl_payload = r#"<ml l="1"></ml>"#;
    msn.send(NotificationClientCommand::ADL(AdlClient {
        tr_id: 7,
        payload: ADLPayload::from_str(adl_payload).expect("adl payload"),
    }))
    .await;
    let Cmd::OK(ok) = msn.next().await else {
        panic!("expected the ADL acknowledgement");
    };
    assert_eq!((ok.operand.as_str(), ok.tr_id), ("ADL", 7));

    let web = reqwest::Client::builder()
        .redirect(reqwest::redirect::Policy::none())
        .timeout(Duration::from_secs(120))
        .build()
        .expect("web client");
    let response = web
        .post(format!(
            "http://127.0.0.1:{http_port}/tachyon/login?t={ticket}"
        ))
        .header("content-type", "application/x-www-form-urlencoded")
        .body(format!(
            "username={local_part}&homeserver_url={homeserver_url}&password={PASSWORD}"
        ))
        .send()
        .await
        .expect("login POST");
    let status = response.status();
    assert_eq!(
        status,
        reqwest::StatusCode::SEE_OTHER,
        "password login failed: {}",
        response.text().await.unwrap_or_default()
    );

    let response = web
        .post(format!(
            "http://127.0.0.1:{http_port}/tachyon/confirm_device/enable?t={ticket}"
        ))
        .send()
        .await
        .expect("enable POST");
    let status = response.status();
    let page = response.text().await.unwrap_or_default();
    assert_eq!(
        status,
        reqwest::StatusCode::OK,
        "secret storage setup failed: {page}"
    );
    assert!(
        page.contains("recovery_key/saved"),
        "the recovery key page must be offered, got: {page}"
    );

    let response = web
        .post(format!(
            "http://127.0.0.1:{http_port}/tachyon/confirm_device/recovery_key/saved?t={ticket}"
        ))
        .send()
        .await
        .expect("recovery key saved POST");
    assert_eq!(
        response.status(),
        reqwest::StatusCode::SEE_OTHER,
        "the acknowledgement must redirect back to the hub"
    );
}

async fn await_ready_sequence(msn: &mut MsnClient, ticket: &BridgeLinkToken, unexpected: &[&str]) {
    let mut profile_seen = false;
    let mut msg_count = 0;
    loop {
        match msn.next().await {
            Cmd::SBS(_) => {}
            Cmd::UBX(_) => {}
            Cmd::NOT(not) => {
                let text = toast_payload(not);
                for fragment in unexpected {
                    assert!(
                        !text.contains(fragment),
                        "unexpected toast mentioning {fragment}: {text}"
                    );
                }
            }
            Cmd::MSG(msg) => {
                msg_count += 1;
                if msg_payload_text(msg).contains(ticket.as_str()) {
                    profile_seen = true;
                }
            }
            _ => {}
        }
        if profile_seen && msg_count >= 2 {
            break;
        }
    }
}

async fn chg_echo(msn: &mut MsnClient) {
    msn.send(NotificationClientCommand::CHG(ChgClient {
        tr_id: 5,
        presence_status: PresenceStatus::NLN,
        client_capabilities: ClientCapabilities::default(),
        avatar: None,
    }))
    .await;
    loop {
        match msn.next().await {
            Cmd::CHG(chg) => {
                let ChgServer { presence_status, .. } = chg;
                assert!(matches!(presence_status, PresenceStatus::NLN));
                return;
            }
            _ => panic!("expected the CHG echo"),
        }
    }
}

fn init_logging() {
    let _ = env_logger::Builder::new()
        .filter_module("tachyon", log::LevelFilter::Debug)
        .filter_module("msnp", log::LevelFilter::Debug)
        .filter_module("matrix_sdk", log::LevelFilter::Warn)
        .filter_module("matrix_sdk::http_client", log::LevelFilter::Debug)
        .filter_level(log::LevelFilter::Warn)
        .try_init();
}

struct Run {
    local_part: String,
    email: String,
    home: PathBuf,
}

impl Run {
    fn fresh() -> Self {
        let nanos = std::time::SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .expect("clock")
            .as_nanos();
        let local_part = format!("e2e{nanos}");
        let home = std::env::temp_dir().join(format!("tachyon-e2e-{nanos}"));
        let _ = std::fs::remove_dir_all(&home);
        std::fs::create_dir_all(&home).expect("create TACHYON_HOME");
        Self {
            email: format!("{local_part}@{DOMAIN}"),
            local_part,
            home,
        }
    }
}

struct Stack {
    homeserver: ContainerAsync<GenericImage>,
    homeserver_url: String,
    notification_port: u16,
    http_port: u16,
    shutdown_snd: tokio::sync::broadcast::Sender<()>,
    tachyon_server: tokio::task::JoinHandle<()>,
    outcome_rcv: tokio::sync::oneshot::Receiver<String>,
}

impl Stack {
    async fn start(run: &Run) -> Self {
        let homeserver = start_homeserver().await;
        let homeserver_url = homeserver_url(&homeserver).await;
        register_homeserver_user(&homeserver_url, &run.local_part).await;

        std::env::set_var("TACHYON_HOME", &run.home);

        let mut stack = Self {
            homeserver,
            homeserver_url,
            notification_port: 0,
            http_port: 0,
            shutdown_snd: tokio::sync::broadcast::channel::<()>(1).0,
            tachyon_server: tokio::spawn(async {}),
            outcome_rcv: tokio::sync::oneshot::channel().1,
        };
        stack.boot_tachyon().await;
        stack
    }

    async fn boot_tachyon(&mut self) {
        let [notification_port, switchboard_port, http_port] = free_ports();
        let config = TachyonConfig {
            notification_port: notification_port as u32,
            switchboard_port: switchboard_port as u32,
            http_port: http_port as u32,
            allow_plain_http: true,
            ..TachyonConfig::default()
        };

        let (shutdown_snd, shutdown_rcv) = tokio::sync::broadcast::channel::<()>(1);
        let (outcome_snd, mut outcome_rcv) = tokio::sync::oneshot::channel::<String>();
        let tachyon_server = tokio::spawn(async move {
            let result = tachyon::run(config, shutdown_rcv).await;
            let _ = outcome_snd.send(format!("{result:?}"));
        });
        wait_for_web(http_port, &mut outcome_rcv).await;

        self.notification_port = notification_port;
        self.http_port = http_port;
        self.shutdown_snd = shutdown_snd;
        self.tachyon_server = tachyon_server;
        self.outcome_rcv = outcome_rcv;
    }

    async fn kill_tachyon(&mut self) {
        self.shutdown_snd.send(()).ok();
        let task = std::mem::replace(&mut self.tachyon_server, tokio::spawn(async {}));
        task.abort();
        let _ = task.await;
        tokio::time::sleep(Duration::from_millis(500)).await;
        self.boot_tachyon().await;
    }

    async fn shutdown(self) {
        self.shutdown_snd.send(()).expect("shutdown signal");
        tokio::time::timeout(Duration::from_secs(60), self.tachyon_server)
            .await
            .expect("run() to finish after the shutdown signal")
            .expect("run() task must not panic");
        let outcome = self.outcome_rcv.await.expect("run() outcome");
        assert!(outcome.starts_with("Ok"), "servers stopped cleanly: {outcome}");
        drop(self.homeserver);
    }
}

async fn start_homeserver() -> ContainerAsync<GenericImage> {
    let image = GenericImage::new("ghcr.io/continuwuity/continuwuity", "latest")
        .with_exposed_port(ContainerPort::Tcp(8008))
        .with_wait_for(WaitFor::Http(Box::new(
            HttpWaitStrategy::new("/_matrix/client/versions")
                .with_port(ContainerPort::Tcp(8008))
                .with_expected_status_code(200u16),
        )))
        .with_env_var("CONTINUWUITY_server_name", SERVER_NAME)
        .with_env_var("CONTINUWUITY_database_path", "/var/lib/conduwuit")
        // 0.0.0.0: a loopback bind is unreachable through the Docker NAT; first-run mode hijacks registration until the first account exists
        .with_env_var("CONTINUWUITY_address", "0.0.0.0")
        .with_env_var("CONTINUWUITY_allow_registration", "true")
        .with_env_var("CONTINUWUITY_registration_token", REGISTRATION_TOKEN)
        .with_env_var("CONTINUWUITY_force_disable_first_run_mode", "true");
    image
        .start()
        .await
        .expect("continuwuity container to start and serve /_matrix/client/versions")
}

async fn homeserver_url(homeserver: &ContainerAsync<GenericImage>) -> String {
    let port = homeserver
        .get_host_port_ipv4(ContainerPort::Tcp(8008))
        .await
        .expect("mapped homeserver port");
    format!("http://127.0.0.1:{port}")
}

async fn register_homeserver_user(base_url: &str, local_part: &str) -> (String, String) {
    let body = format!(
        r#"{{"username":"{local_part}","password":"{PASSWORD}","auth":{{"type":"m.login.registration_token","token":"{REGISTRATION_TOKEN}"}}}}"#
    );
    let response = reqwest::Client::new()
        .post(format!("{base_url}/_matrix/client/v3/register"))
        .header("content-type", "application/json")
        .body(body)
        .send()
        .await
        .expect("registration request");
    let status = response.status();
    let text = response.text().await.unwrap_or_default();
    assert!(status.is_success(), "registration failed ({status}): {text}");
    let parsed: serde_json::Value = serde_json::from_str(&text).expect("registration json");
    (
        parsed["user_id"].as_str().expect("user_id").to_string(),
        parsed["access_token"].as_str().expect("access_token").to_string(),
    )
}

async fn mint_ticket(http_port: u16, email: &str) -> BridgeLinkToken {
    let expected = tachyon::tachyon::identifiers::bridge_link::BridgeLinkToken::from_parts(
        tachyon::tachyon::identifiers::bridge_link::BridgeType::Msn,
        email,
        tachyon::tachyon::identifiers::bridge_link::ClientVersion(14),
    );
    let rst2_body = format!(
        r#"<?xml version="1.0" encoding="UTF-8"?><s:Envelope xmlns:s="http://www.w3.org/2003/05/soap-envelope" xmlns:wsse="http://docs.oasis-open.org/wss/2004/01/oasis-200401-wss-wssecurity-secext-1.0.xsd" xmlns:wsu="http://docs.oasis-open.org/wss/2004/01/oasis-200401-wss-wssecurity-utility-1.0.xsd"><s:Header><wsse:Security><wsse:UsernameToken wsu:Id="user"><wsse:Username>{email}</wsse:Username><wsse:Password>{PASSWORD}</wsse:Password></wsse:UsernameToken></wsse:Security></s:Header><s:Body></s:Body></s:Envelope>"#
    );
    let response = reqwest::Client::new()
        .post(format!("http://127.0.0.1:{http_port}/RST2.srf"))
        .header("User-Agent", CLIENT_USER_AGENT)
        .body(rst2_body)
        .send()
        .await
        .expect("RST2 request");
    let status = response.status();
    let body = response.text().await.unwrap_or_default();
    assert!(status.is_success(), "RST2 failed ({status}): {body}");
    assert!(
        body.contains(expected.as_str()),
        "the RST2 response must carry the minted bridge link ticket"
    );
    expected
}

fn free_ports<const N: usize>() -> [u16; N] {
    let listeners: Vec<std::net::TcpListener> = (0..N)
        .map(|_| {
            std::net::TcpListener::bind(("127.0.0.1", 0)).expect("probe listener to bind")
        })
        .collect();
    let ports: [u16; N] = listeners
        .iter()
        .map(|listener| listener.local_addr().expect("probe local addr").port())
        .collect::<Vec<_>>()
        .try_into()
        .expect("exactly N ports");
    drop(listeners);
    ports
}

async fn wait_for_web(
    http_port: u16,
    outcome: &mut tokio::sync::oneshot::Receiver<String>,
) {
    let client = reqwest::Client::new();
    for _ in 0..120 {
        if let Ok(message) = outcome.try_recv() {
            panic!("tachyon::run finished early: {message}");
        }
        if let Ok(response) = client
            .get(format!("http://127.0.0.1:{http_port}/tachyon"))
            .send()
            .await
        {
            if response.status().is_success() {
                return;
            }
        }
        tokio::time::sleep(Duration::from_millis(250)).await;
    }
    panic!("the tachyon web server did not come up on port {http_port}");
}

fn toast_payload(not: NotServer) -> String {
    match not.payload {
        msnp::msnp::notification::command::not::NotificationPayloadType::Raw(text) => text,
        msnp::msnp::notification::command::not::NotificationPayloadType::Normal(_) => String::new(),
    }
}

fn msg_payload_text(msg: MsgServer) -> String {
    let bytes = msg.payload.into_bytes();
    String::from_utf8_lossy(&bytes).into_owned()
}

struct MsnClient {
    reader: BufReader<tokio::net::tcp::OwnedReadHalf>,
    writer: tokio::net::tcp::OwnedWriteHalf,
    parser: ServerCommandParser,
    pending: std::collections::VecDeque<NotificationServerCommand>,
}

impl MsnClient {
    async fn connect(port: u16) -> Self {
        let mut stream = None;
        for _ in 0..40 {
            match TcpStream::connect(("127.0.0.1", port)).await {
                Ok(connected) => {
                    stream = Some(connected);
                    break;
                }
                Err(_) => tokio::time::sleep(Duration::from_millis(250)).await,
            }
        }
        let (read, write) = stream.expect("notification server to accept").into_split();
        Self {
            reader: BufReader::new(read),
            writer: write,
            parser: ServerCommandParser::new(),
            pending: std::collections::VecDeque::new(),
        }
    }

    async fn send(&mut self, command: NotificationClientCommand) {
        self.writer
            .write_all(&command.into_bytes())
            .await
            .expect("write command");
    }

    async fn next(&mut self) -> NotificationServerCommand {
        tokio::time::timeout(STAGE_TIMEOUT, async {
            loop {
                if let Some(command) = self.pending.pop_front() {
                    return command;
                }
                let mut chunk = [0u8; 8192];
                let read = self.reader.read(&mut chunk).await.expect("read socket");
                if read == 0 {
                    panic!("the notification server closed the connection");
                }
                let raws = self
                    .parser
                    .parse_message(&chunk[..read])
                    .expect("frame server commands");
                for raw in raws {
                    let command = NotificationServerCommand::try_from_raw(raw)
                        .expect("deserialize server command");
                    self.pending.push_back(command);
                }
            }
        })
        .await
        .expect("a server command within the stage timeout")
    }
}
