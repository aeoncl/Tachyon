use std::str::from_utf8_unchecked;
use std::time::Duration;

use anyhow::anyhow;
use log::{debug, error, info, warn};
use msnp::msnp::notification::command::command::NotificationClientCommand;
use msnp::msnp::{notification::command::command::NotificationServerCommand, raw_command_parser::RawCommandParser};
use msnp::shared::traits::{IntoBytes, TryFromRawCommand};
use tokio::{io::{AsyncReadExt, AsyncWriteExt, BufReader}, net::{tcp::{OwnedReadHalf, OwnedWriteHalf}, TcpListener, TcpStream}, sync::{broadcast::{self, Receiver}, mpsc::{self, Sender}}, task::JoinHandle};
use msnp::msnp::raw_command_parser::RawCommand;
use crate::notification::handlers::command_handler::handle_command;
use crate::notification::models::local_client_data::LocalClientData;
use crate::tachyon::client::tachyon_client::TachyonClient;
use crate::tachyon::global_state::GlobalState;
use crate::tachyon::repository::RepositoryStr;

pub struct NotificationServer;


impl NotificationServer {

    pub async fn listen(ip_addr: &str, port: u32, global_shutdown_recv: Receiver<()>, global_state: GlobalState) -> Result<(), anyhow::Error>{
        let listener = Self::bind(ip_addr, port).await?;
        Self::serve(listener, global_shutdown_recv, global_state).await
    }

    async fn bind(ip_addr: &str, port: u32) -> Result<TcpListener, anyhow::Error> {
        TcpListener::bind(format!("{}:{}", ip_addr, port))
            .await.map_err(|e| anyhow!(e))
    }

    async fn serve(listener: TcpListener, global_shutdown_recv: Receiver<()>, global_state: GlobalState) -> Result<(), anyhow::Error> {
        info!("TCP Server started...");

            loop {
                let mut global_shutdown_recv_clone = global_shutdown_recv.resubscribe();
                let global_state_clone = global_state.clone();

                tokio::select! {
                    accepted = listener.accept() => {
                        let (socket, _addr)  = accepted.map_err(|e| anyhow!(e))?;
                        let _handle = tokio::spawn(async move {
                            handle_client(socket, global_shutdown_recv_clone.resubscribe(), global_state_clone).await
                        });
                    }
                    global_shutdown = global_shutdown_recv_clone.recv() => {
                        if let Err(err) = global_shutdown {
                            error!("Unable to listen for global kill: {}", err);
                        }
                        break;
                    }
                }
            }

            info!("TCP Server gracefull shutdown...");
            Ok(())
    }

}



async fn handle_client(socket: TcpStream, mut global_shutdown_recv: broadcast::Receiver<()>, global_state: GlobalState) -> Result<(), anyhow::Error> {
    debug!("Client connected...");

    let (read, write) = socket.into_split();
    let (client_shutdown_snd, client_shutdown_recv) = broadcast::channel::<()>(1);
    let command_sender = start_write_task(write, client_shutdown_recv.resubscribe());

    let mut local_client_data = LocalClientData::new(client_shutdown_snd.clone(), client_shutdown_recv);

    // A handler can park for minutes waiting on the user, so the socket is read by its own
    // task. That is what lets the client leaving reach a sign-in that is still waiting.
    let (commands_snd, mut commands_recv) = mpsc::channel::<Vec<RawCommand>>(32);
    let read_task = start_read_task(read, commands_snd, client_shutdown_snd, global_shutdown_recv.resubscribe());

    loop {
        tokio::select! {
            commands = commands_recv.recv() => {
                match commands {
                    None => break,
                    Some(commands) => handle_commands(commands, &command_sender, &global_state, &mut local_client_data).await,
                }
            },
            global_shutdown = global_shutdown_recv.recv() => {
                if let Err(err) = global_shutdown {
                    error!("Unable to listen for global kill: {}", err);
                }
                break;
            }
        }
    }

    read_task.abort();
    if let Some(guard) = local_client_data.client_drop_guard.take() {
        guard.release().await;
    }

    info!("Client gracefully shutdown...");
    Ok(())

}

fn start_read_task(read: OwnedReadHalf, commands_snd: mpsc::Sender<Vec<RawCommand>>, client_shutdown_snd: broadcast::Sender<()>, mut global_shutdown_recv: Receiver<()>) -> JoinHandle<()> {
    tokio::spawn(async move {
        let mut parser = RawCommandParser::new();
        let mut reader = BufReader::new(read);
        let mut buffer = [0u8; 2048];

        loop {
            tokio::select! {
                bytes_read = reader.read(&mut buffer) => {
                    match bytes_read {
                        Err(e) => {
                            error!("MSNP|NOT: Socket Read Error: {}", e);
                            break;
                        },
                        Ok(0) => break,
                        Ok(bytes_read) => {
                            match parser.parse_message(&buffer[..bytes_read]) {
                                Err(e) => error!("MSNP|NOT: Unable to parse message into commands: {}", e),
                                Ok(commands) => {
                                    if commands_snd.send(commands).await.is_err() {
                                        break;
                                    }
                                }
                            }
                        }
                    }
                },
                global_shutdown = global_shutdown_recv.recv() => {
                    if let Err(err) = global_shutdown {
                        error!("Unable to listen for global kill: {}", err);
                    }
                    break;
                }
            }
        }

        let _result = client_shutdown_snd.send(());
    })
}

async fn handle_commands(commands: Vec<RawCommand>, command_sender: &Sender<NotificationServerCommand>, global_state: &GlobalState, local_client_data: &mut LocalClientData) {
    for command in commands {
        debug!("NS << | {}", command.get_command());

        let notification_command = NotificationClientCommand::try_from_raw(command);
        match notification_command {
            Err(e) => {
                error!("MSNP|NOT: Unable to parse command: {}", e);
                debug!("{:?}", e);
            },
            Ok(notification_command) => {
                let command_result = handle_command(notification_command, command_sender.clone(), &global_state, local_client_data, &global_state.get_config()).await;

                if let Err(error) = command_result {
                    error!("MSNP|NS: An error has occured handling a notification command: {}", &error);
                    debug!("MSNP|NS: {:?}", &error);
                    //TODO SEND ERROR BACK TO Client
                }
            }
        }
    }
}

fn start_write_task(mut write: OwnedWriteHalf, mut kill_recv: Receiver<()>) -> Sender<NotificationServerCommand> {
    println!("Socket write task started...");
    let (sender, mut receiver) = mpsc::channel::<NotificationServerCommand>(300);

    let _result = tokio::spawn(async move {
        loop {
            tokio::select! {
                command = receiver.recv() => {
                    if let Some(command) = command {

                        let bytes = command.into_bytes();

                        unsafe {
                            debug!("NS >> | {}", from_utf8_unchecked(&bytes));
                        }

                        let _result = write.write_all(&bytes).await;
                    }
                },
                _kill_signal = kill_recv.recv() => {
                    break;
                }
            }
        }
        println!("Socket write task gracefully shutdown...");
    } );
    sender
}

#[cfg(test)]
mod tests {
    use super::*;
    use tachyon_core::domain::ids::BridgeId;
    use msnp::shared::models::client_version::ClientVersion;
    use msnp::shared::models::email_address::EmailAddress;
    use std::net::SocketAddr;
    use std::str::FromStr;
    use std::sync::Arc;
    use std::time::Duration;
    use tachyon_core::application::ports::AccountRepository;
    use tachyon_core::domain::ids::LoginId;
    use tachyon_core::domain::verification::DeviceStatus;
    use tachyon_core::infrastructure::app_state::AppState;
    use tachyon_testkit::fakes::FakeAuthService;
    use tachyon_testkit::repositories::AccountRepositoryInMem;
    use tokio::task::JoinHandle;
    use tokio::time::timeout;

    const TEST_EMAIL: &str = "aeon@shlasouf.local";
    const TEST_ENDPOINT_GUID: &str = "{55192CF5-588E-4ABE-9CDF-395B616ED85B}";
    const REPLY_WINDOW: Duration = Duration::from_secs(2);

    fn v14() -> ClientVersion {
        "14.0".parse().expect("a valid client version")
    }

    fn email() -> EmailAddress {
        EmailAddress::from_str(TEST_EMAIL).expect("a valid test address")
    }

    async fn test_state(auth_service: Arc<FakeAuthService>) -> GlobalState {
        let (global_state, account_repository) = test_state_without_login(auth_service);

        account_repository
            .save_login_for_token(global_state.token_for(&email(), &v14()), LoginId::new("login-1"))
            .await
            .expect("the in-memory repository should accept the login");

        global_state
    }

    /// An instance that has never authenticated the test account, so a sign-in has to go
    /// through the interactive login.
    fn test_state_without_login(
        auth_service: Arc<FakeAuthService>,
    ) -> (GlobalState, Arc<AccountRepositoryInMem>) {
        let account_repository = Arc::new(AccountRepositoryInMem::default());
        let app_state = Arc::new(AppState::new(
            auth_service,
            account_repository.clone(),
            "http://127.0.0.1:11866/tachyon".to_string(),
        ));
        let global_state = GlobalState::new(Default::default(), BridgeId::new("msn"), app_state);
        (global_state, account_repository)
    }

    struct TestServer {
        addr: SocketAddr,
        shutdown: broadcast::Sender<()>,
        serve: JoinHandle<Result<(), anyhow::Error>>,
    }

    async fn start_server(global_state: GlobalState) -> TestServer {
        let listener = NotificationServer::bind("127.0.0.1", 0)
            .await
            .expect("the test server should bind");
        let addr = listener.local_addr().expect("a bound listener has an address");
        let (shutdown, shutdown_recv) = broadcast::channel::<()>(1);
        let serve = tokio::spawn(NotificationServer::serve(listener, shutdown_recv, global_state));

        TestServer {
            addr,
            shutdown,
            serve,
        }
    }

    async fn wait_for(mut condition: impl FnMut() -> bool) -> bool {
        timeout(REPLY_WINDOW, async {
            while !condition() {
                tokio::time::sleep(Duration::from_millis(10)).await;
            }
        })
        .await
        .is_ok()
    }

    struct TestClient {
        stream: TcpStream,
        received: String,
    }

    impl TestClient {
        async fn connect(addr: SocketAddr) -> Self {
            let stream = TcpStream::connect(addr)
                .await
                .expect("the test client should reach the server");
            Self {
                stream,
                received: String::new(),
            }
        }

        async fn send(&mut self, command: &str) {
            self.stream
                .write_all(command.as_bytes())
                .await
                .expect("the test client should be able to write");
        }

        async fn read_until(
            &mut self,
            predicate: impl Fn(&str) -> bool,
        ) -> Result<(), anyhow::Error> {
            let deadline = tokio::time::Instant::now() + REPLY_WINDOW;
            let mut chunk = [0u8; 4096];

            while !predicate(&self.received) {
                let read = tokio::time::timeout_at(deadline, self.stream.read(&mut chunk)).await;
                let read = match read {
                    Err(_) => return Err(anyhow!("timed out, got so far: {}", self.received)),
                    Ok(Err(e)) => return Err(anyhow!("socket read failed: {}", e)),
                    Ok(Ok(read)) => read,
                };

                if read == 0 {
                    return Err(anyhow!("the server hung up, got so far: {}", self.received));
                }

                self.received
                    .push_str(&String::from_utf8_lossy(&chunk[..read]));
            }

            Ok(())
        }

        async fn sign_in(&mut self, global_state: &GlobalState) -> Result<(), anyhow::Error> {
            self.sign_in_as(global_state, "14.0.8117.0416").await
        }

        async fn sign_in_as(
            &mut self,
            global_state: &GlobalState,
            client_build: &str,
        ) -> Result<(), anyhow::Error> {
            let client: ClientVersion = client_build.parse().expect("a valid client build");
            self.send("VER 1 MSNP18 MSNP17 CVR0\r\n").await;
            self.send(&format!(
                "CVR 2 0x0409 winnt 6.2.0 i386 MSNMSGR {} msmsgs {}\r\n",
                client_build, TEST_EMAIL
            ))
            .await;
            self.send(&format!("USR 3 SSO I {}\r\n", TEST_EMAIL)).await;
            self.read_until(|received| received.contains("USR 3 SSO S")).await?;

            self.send(&format!(
                "USR 4 SSO S t={} challenge {}\r\n",
                global_state.ticket_for(&email(), &client).as_str(),
                TEST_ENDPOINT_GUID
            ))
            .await;

            Ok(())
        }
    }

    #[tokio::test]
    async fn a_second_client_is_served_while_the_first_sign_in_is_parked() {
        let auth_service = FakeAuthService::minting([DeviceStatus::Unverified]);
        let global_state = test_state(auth_service).await;
        let server = start_server(global_state.clone()).await;

        let mut first = TestClient::connect(server.addr).await;
        first.sign_in(&global_state).await.unwrap();
        first.read_until(|received| received.contains("USR 4 OK")).await.unwrap();
        first.read_until(|received| received.contains("confirm_device")).await.unwrap();

        let mut second = TestClient::connect(server.addr).await;
        second.sign_in(&global_state).await.unwrap();
        second
            .read_until(|received| received.contains("USR 4 OK"))
            .await
            .expect("the reconnecting client should be signed in while the first sign-in waits");
    }

    #[tokio::test]
    async fn a_second_connection_of_the_same_version_is_sent_out() {
        let auth_service = FakeAuthService::minting([DeviceStatus::Unverified]);
        let global_state = test_state(auth_service.clone()).await;
        let server = start_server(global_state.clone()).await;

        let mut first = TestClient::connect(server.addr).await;
        first.sign_in(&global_state).await.unwrap();
        first.read_until(|received| received.contains("confirm_device")).await.unwrap();

        let mut second = TestClient::connect(server.addr).await;
        second.sign_in(&global_state).await.unwrap();
        second
            .read_until(|received| received.contains("OUT\r\n"))
            .await
            .expect("the second client of the same version should be sent OUT");

        assert_eq!(auth_service.session(0).close_calls(), 0, "the first sign-in is untouched");
        assert_eq!(auth_service.restore_calls(), 1);
        let token = global_state.token_for(&email(), &v14());
        assert!(global_state.is_token_linked(token.as_str()));
    }

    #[tokio::test]
    async fn a_client_of_another_version_signs_in_beside_the_first() {
        let auth_service = FakeAuthService::minting([DeviceStatus::Unverified]);
        let global_state = test_state(auth_service.clone()).await;
        let server = start_server(global_state.clone()).await;

        let mut first = TestClient::connect(server.addr).await;
        first.sign_in(&global_state).await.unwrap();
        first.read_until(|received| received.contains("confirm_device")).await.unwrap();

        let mut second = TestClient::connect(server.addr).await;
        second.sign_in_as(&global_state, "8.5.1302.1018").await.unwrap();
        second
            .read_until(|received| received.contains("login/start"))
            .await
            .expect("a client of another version gets its own login");

        assert_eq!(auth_service.restore_calls(), 1, "the stored login belongs to 14.0");
        assert_eq!(auth_service.start_calls(), 1, "8.5 starts a login of its own");
        assert_eq!(auth_service.session(0).close_calls(), 0);
    }

    #[tokio::test]
    async fn a_client_that_reconnects_right_after_leaving_signs_in_again() {
        let auth_service = FakeAuthService::minting([DeviceStatus::Unverified]);
        let global_state = test_state(auth_service.clone()).await;
        let server = start_server(global_state.clone()).await;

        let mut first = TestClient::connect(server.addr).await;
        first.sign_in(&global_state).await.unwrap();
        first.read_until(|received| received.contains("confirm_device")).await.unwrap();
        drop(first);

        let mut second = TestClient::connect(server.addr).await;
        second.sign_in(&global_state).await.unwrap();
        second
            .read_until(|received| received.contains("confirm_device"))
            .await
            .expect("the reconnecting client should sign in, not be sent OUT");

        assert_eq!(auth_service.restore_calls(), 2);
    }

    #[tokio::test]
    async fn a_client_that_disconnects_during_verification_is_abandoned_promptly() {
        let auth_service = FakeAuthService::minting([DeviceStatus::Unverified]);
        let global_state = test_state(auth_service.clone()).await;
        let server = start_server(global_state.clone()).await;

        let mut first = TestClient::connect(server.addr).await;
        first.sign_in(&global_state).await.unwrap();
        first.read_until(|received| received.contains("USR 4 OK")).await.unwrap();
        first.read_until(|received| received.contains("confirm_device")).await.unwrap();

        let abandoned = auth_service.session(0);
        drop(first);

        assert!(
            wait_for(|| abandoned.close_calls() == 1).await,
            "the parked sign-in should close its backend session when the client leaves, close_calls = {}",
            abandoned.close_calls()
        );

        let mut second = TestClient::connect(server.addr).await;
        second.sign_in(&global_state).await.unwrap();
        second.read_until(|received| received.contains("USR 4 OK")).await.unwrap();
        second.read_until(|received| received.contains("confirm_device")).await.unwrap();

        assert_eq!(
            auth_service.restore_calls(),
            2,
            "the abandoned session should have been rebuilt for the reconnecting client"
        );
    }

    #[tokio::test]
    async fn a_client_that_disconnects_during_interactive_login_is_abandoned_promptly() {
        let auth_service = FakeAuthService::minting([DeviceStatus::Unverified]);
        let (global_state, _) = test_state_without_login(auth_service.clone());
        let server = start_server(global_state.clone()).await;

        let mut first = TestClient::connect(server.addr).await;
        first.sign_in(&global_state).await.unwrap();
        first.read_until(|received| received.contains("USR 4 OK")).await.unwrap();
        first.read_until(|received| received.contains("login/start")).await.unwrap();

        let abandoned = auth_service.session(0);
        drop(first);

        assert!(
            wait_for(|| abandoned.discard_calls() == 1).await,
            "a sign-in that never authenticated should be discarded when the client leaves, discard_calls = {}",
            abandoned.discard_calls()
        );
    }

    #[tokio::test]
    async fn global_shutdown_ends_a_parked_sign_in() {
        let auth_service = FakeAuthService::minting([DeviceStatus::Unverified]);
        let global_state = test_state(auth_service).await;
        let server = start_server(global_state.clone()).await;

        let mut first = TestClient::connect(server.addr).await;
        first.sign_in(&global_state).await.unwrap();
        first.read_until(|received| received.contains("USR 4 OK")).await.unwrap();
        first.read_until(|received| received.contains("confirm_device")).await.unwrap();

        server.shutdown.send(()).unwrap();

        let stopped = timeout(REPLY_WINDOW, server.serve).await;
        assert!(stopped.is_ok(), "serve should return once the global shutdown fires");
        stopped.unwrap().unwrap().unwrap();
    }
}
