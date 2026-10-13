use std::path::PathBuf;
use std::str::FromStr;
use std::time::Duration;
use std::time::UNIX_EPOCH;

use msnp::msnp::notification::command::command::NotificationClientCommand;
use msnp::msnp::notification::command::command::NotificationServerCommand;
use msnp::msnp::notification::command::command::NotificationServerCommand as Cmd;
use msnp::msnp::notification::command::adl::AdlClient;
use msnp::msnp::notification::command::chg::{ChgClient, ChgServer};
use msnp::msnp::notification::command::cvr::CvrClient;
use msnp::msnp::notification::command::msg::MsgServer;
use msnp::msnp::notification::command::not::NotServer;
use msnp::msnp::notification::command::usr::{
    AuthOperationTypeClient, AuthPolicy, OperationTypeServer, SsoPhaseClient, SsoPhaseServer,
    UsrClient,
};
use msnp::msnp::notification::command::ver::VerClient;
use msnp::msnp::notification::models::adl_payload::ADLPayload;
use msnp::msnp::notification::models::endpoint_guid::EndpointGuid;
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

async fn handshake(msn: &mut MsnClient, email: &str, ticket: &BridgeLinkToken) {
    msn.send(NotificationClientCommand::VER(VerClient::new(1, MsnpVersion::MSNP18, MsnpVersion::MSNP17)))
        .await;
    let Cmd::VER(ver) = msn.next().await else {
        panic!("expected the agreed version");
    };
    assert_eq!(ver.agreed_version, MsnpVersion::MSNP18);

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
