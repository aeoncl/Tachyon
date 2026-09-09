use std::str::from_utf8_unchecked;

use anyhow::anyhow;
use log::{debug, error, info, warn};
use msnp::msnp::notification::command::command::NotificationClientCommand;
use msnp::msnp::{notification::command::command::NotificationServerCommand, raw_command_parser::RawCommandParser};
use msnp::shared::traits::{IntoBytes, TryFromRawCommand};
use tokio::{io::{AsyncReadExt, AsyncWriteExt, BufReader}, net::{tcp::OwnedWriteHalf, TcpListener, TcpStream}, sync::{broadcast::{self, Receiver}, mpsc::{self, Sender}}};
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

    let mut parser = RawCommandParser::new();
    let mut reader = BufReader::new(read);
    let mut buffer= [0u8; 2048];

    loop {
        tokio::select! {
            bytes_read = reader.read(&mut buffer) => {
                match bytes_read {
                    Err(e) => {
                        error!("MSNP|NOT: Socket Read Error: {}", e);
                        break;
                    },
                    Ok(bytes_read) => {

                        if bytes_read == 0 {
                            break;
                        }

                        let data = &buffer[..bytes_read];

                        let commands = parser.parse_message(data);

                        match commands {
                            Err(e) => error!("MSNP|NOT: Unable to parse message into commands: {}", e),
                            Ok(commands) => {
                                handle_commands(commands, &command_sender, &global_state, &mut local_client_data).await;
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

    info!("Client gracefully shutdown...");
    Ok(())

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
    use async_trait::async_trait;
    use msnp::shared::models::email_address::EmailAddress;
    use std::any::Any;
    use std::net::SocketAddr;
    use std::str::FromStr;
    use std::sync::atomic::{AtomicUsize, Ordering};
    use std::sync::{Arc, Mutex};
    use std::time::Duration;
    use tachyon_core::application::error::{BackendError, VerificationError};
    use tachyon_core::application::ports::{AccountRepository, AuthService, BackendSession};
    use tachyon_core::domain::auth::{BridgeMetadata, InteractiveAuthStarted};
    use tachyon_core::domain::ids::{DeviceId, LoginId, UserId};
    use tachyon_core::domain::verification::{
        DeviceStatus, IdentityReset, RecoveryKey, ResetAuth, VerificationAction,
        VerificationFlowState, VerificationOptions,
    };
    use tachyon_core::infrastructure::app_state::AppState;
    use tachyon_testkit::repositories::AccountRepositoryInMem;
    use tokio::task::JoinHandle;
    use tokio::time::timeout;

    const TEST_SECRET: [u8; 4] = [1, 2, 3, 4];
    const TEST_EMAIL: &str = "aeon@shlasouf.local";
    const TEST_ENDPOINT_GUID: &str = "{55192CF5-588E-4ABE-9CDF-395B616ED85B}";
    const REPLY_WINDOW: Duration = Duration::from_secs(2);

    fn email() -> EmailAddress {
        EmailAddress::from_str(TEST_EMAIL).expect("a valid test address")
    }

    fn unsupported<T>() -> Result<T, VerificationError> {
        Err(VerificationError::Backend(BackendError::Technical(anyhow!(
            "the fake backend session does not support this call"
        ))))
    }

    struct FakeBackendSession {
        close_calls: AtomicUsize,
    }

    impl FakeBackendSession {
        fn new() -> Arc<Self> {
            Arc::new(Self {
                close_calls: AtomicUsize::new(0),
            })
        }

        fn close_calls(&self) -> usize {
            self.close_calls.load(Ordering::SeqCst)
        }
    }

    #[async_trait]
    impl BackendSession for FakeBackendSession {
        async fn finish_interactive_login(&self, _callback_query: &str) -> Result<(), BackendError> {
            Err(BackendError::Technical(anyhow!(
                "the fake backend session does not support this call"
            )))
        }

        async fn device_status(&self) -> Result<DeviceStatus, BackendError> {
            Ok(DeviceStatus::Unverified)
        }

        async fn verification_options(&self) -> Result<VerificationOptions, VerificationError> {
            unsupported()
        }

        async fn recover(&self, _key: &RecoveryKey) -> Result<DeviceStatus, VerificationError> {
            unsupported()
        }

        async fn start_device_verification(
            &self,
            _device: &DeviceId,
        ) -> Result<(), VerificationError> {
            unsupported()
        }

        fn verification_state(&self) -> Option<VerificationFlowState> {
            None
        }

        async fn verification_action(
            &self,
            _action: VerificationAction,
        ) -> Result<(), VerificationError> {
            unsupported()
        }

        async fn reset_identity(
            &self,
            _auth: Option<ResetAuth>,
        ) -> Result<IdentityReset, VerificationError> {
            unsupported()
        }

        async fn close(&self) {
            self.close_calls.fetch_add(1, Ordering::SeqCst);
        }

        fn as_any(&self) -> &dyn Any {
            self
        }
    }

    #[derive(Default)]
    struct FakeAuthService {
        sessions: Mutex<Vec<Arc<FakeBackendSession>>>,
    }

    impl FakeAuthService {
        fn new() -> Arc<Self> {
            Arc::new(Self::default())
        }

        fn restore_calls(&self) -> usize {
            self.sessions.lock().unwrap().len()
        }

        fn session(&self, index: usize) -> Arc<FakeBackendSession> {
            self.sessions.lock().unwrap()[index].clone()
        }
    }

    #[async_trait]
    impl AuthService for FakeAuthService {
        async fn restore(
            &self,
            _login_id: &LoginId,
        ) -> Result<Arc<dyn BackendSession>, BackendError> {
            let session = FakeBackendSession::new();
            self.sessions.lock().unwrap().push(session.clone());
            Ok(session)
        }

        async fn start_interactive_login(
            &self,
            _login_id: &LoginId,
            _server_name: &str,
            _user_id: Option<UserId>,
            _redirect_url: &str,
            _bridge_metadata: &BridgeMetadata,
        ) -> Result<(Arc<dyn BackendSession>, InteractiveAuthStarted), BackendError> {
            Err(BackendError::Technical(anyhow!(
                "the fake auth service has no interactive login"
            )))
        }
    }

    async fn test_state(auth_service: Arc<FakeAuthService>) -> GlobalState {
        let account_repository = Arc::new(AccountRepositoryInMem::default());
        let app_state = Arc::new(AppState::new(
            auth_service,
            account_repository.clone(),
            "http://127.0.0.1:11866/tachyon/login/callback".to_string(),
        ));
        let global_state = GlobalState::new(Default::default(), TEST_SECRET.to_vec(), app_state);

        account_repository
            .save_login_for_token(global_state.token_for(&email()), LoginId::new("login-1"))
            .await
            .expect("the in-memory repository should accept the login");

        global_state
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
            self.send("VER 1 MSNP18 MSNP17 CVR0\r\n").await;
            self.send(&format!(
                "CVR 2 0x0409 winnt 6.2.0 i386 MSNMSGR 14.0.8117.0416 msmsgs {}\r\n",
                TEST_EMAIL
            ))
            .await;
            self.send(&format!("USR 3 SSO I {}\r\n", TEST_EMAIL)).await;
            self.read_until(|received| received.contains("USR 3 SSO S")).await?;

            self.send(&format!(
                "USR 4 SSO S t={} challenge {}\r\n",
                global_state.ticket_for(&email()).as_str(),
                TEST_ENDPOINT_GUID
            ))
            .await;

            Ok(())
        }
    }

    #[tokio::test]
    async fn a_second_client_is_served_while_the_first_sign_in_is_parked() {
        let auth_service = FakeAuthService::new();
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
    async fn a_client_that_disconnects_during_verification_is_abandoned_promptly() {
        let auth_service = FakeAuthService::new();
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
    async fn global_shutdown_ends_a_parked_sign_in() {
        let auth_service = FakeAuthService::new();
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
