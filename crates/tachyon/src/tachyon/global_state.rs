use std::mem;
use crate::matrix::services::homeserver::HomeserverResolver;
use crate::matrix::services::login::MatrixLoginService;
use crate::matrix::verification_request_repository::VerificationRequestRepository;
use crate::notification::handlers::auth::sync_with_server_task;
use crate::tachyon::alert::AlertReceiver;
use crate::tachyon::client::tachyon_client::TachyonClient;
use crate::tachyon::client::tachyon_client_repository::TachyonClientRepository;
use crate::tachyon::config::secret_encryptor::SecretEncryptor;
use crate::tachyon::config::tachyon_config::TachyonConfig;
use crate::tachyon::mappers::user_id::MatrixIdCompatible;
use crate::tachyon::repository::RepositoryStr;
use dashmap::DashMap;
use matrix_sdk::Client;
use msnp::msnp::notification::command::command::NotificationServerCommand;
use msnp::msnp::notification::models::endpoint_guid::EndpointGuid;
use msnp::shared::models::email_address::EmailAddress;
use msnp::shared::models::endpoint_id::EndpointId;
use msnp::shared::models::msn_user::MsnUser;
use msnp::shared::models::ticket_token::TicketToken;
use std::sync::{Arc, Mutex};
use tachyon_sqlite_store::SqliteCredentialsStore;
use tokio::sync::{broadcast, mpsc};
use msnp::msnp::models::contact_list::ContactList;

pub struct PendingOAuth {
    pub state: String,
    pub client: Client,
}

pub struct PendingLogin {
    pub email: EmailAddress,
    pub endpoint_guid: EndpointGuid,
    pub notification_sender: mpsc::Sender<NotificationServerCommand>,
    pub client_shutdown_snd: broadcast::Sender<()>,
    pub client_shutdown_recv: broadcast::Receiver<()>,
    pub pending_oauth: Mutex<Option<PendingOAuth>>,
    pub guard: Mutex<Option<ClientDropGuard>>,
    pub contact_list: Mutex<ContactList>,
}

pub struct GlobalStateInner {
    config: TachyonConfig,
    tachyon_clients: TachyonClientRepository,
    token_validator: SecretEncryptor,
    pending_alerts: DashMap<i32, AlertReceiver>,
    pending_verification_requests: VerificationRequestRepository,
    matrix_login_service: Box<dyn MatrixLoginService>,
    credentials: Arc<SqliteCredentialsStore>,
    homeserver_resolver: HomeserverResolver,
    pending_logins: DashMap<String, Arc<PendingLogin>>,
}

#[derive(Clone)]
pub struct GlobalState {
    inner: Arc<GlobalStateInner>,
}

pub struct ClientDropGuard {
    global_state: GlobalState,
    key: String
}

impl ClientDropGuard {
    pub fn new(global_state: GlobalState, key: String) -> Self {
        Self { global_state, key }
    }
}

impl Drop for ClientDropGuard {
    fn drop(&mut self) {
        let tachyon_client = self.global_state.tachyon_clients().remove(&self.key);
        if let Some(client) = tachyon_client {
            client.shutdown();

            //Todo change this so we use a neutral key
            let own_user_id = client.own_user().get_email_address().to_owned_user_id();
            self.global_state.pending_verification_requests().remove_for(&own_user_id);
        }

        println!("Client Drop Guard dropped");
    }
}

impl GlobalState {

    pub fn new(config: TachyonConfig, token_validator: SecretEncryptor, matrix_login_service: Box<dyn MatrixLoginService>, credentials: Arc<SqliteCredentialsStore>) -> Self {
        let homeserver_resolver = HomeserverResolver::new(!config.strict_ssl);
        Self {
            inner: Arc::new(GlobalStateInner {
                config,
                tachyon_clients: Default::default(),
                token_validator,
                pending_alerts: Default::default(),
                pending_verification_requests: Default::default(),
                matrix_login_service,
                credentials,
                homeserver_resolver,
                pending_logins: Default::default(),
            })
        }
    }

    pub fn get_config(&self) -> &TachyonConfig {
        &self.inner.config
    }

    //FIXME: remove this and fix everywhere it's called to get the client using the key.
    pub fn get_single_client(&self) -> Option<TachyonClient> {
        self.tachyon_clients().single()
    }

    pub fn tachyon_clients(&self) -> &TachyonClientRepository {
        &self.inner.tachyon_clients
    }

    pub fn insert_clients(&self, key: String, tachyon_client: TachyonClient) -> ClientDropGuard {
        self.inner.tachyon_clients.insert(key.clone(), tachyon_client);
        ClientDropGuard::new(self.clone(), key)
    }

    pub fn get_clients(&self, key: &str) -> Option<TachyonClient> {
        self.inner.tachyon_clients.get(key)
    }

    pub fn store_pending_alert(&self, key: i32, receiver: AlertReceiver) {
        self.inner.pending_alerts.insert(key, receiver);
    }

    pub fn take_pending_alert(&self, key: &i32) -> Option<AlertReceiver> {
        self.inner.pending_alerts.remove(key).map(|(_, recv)| recv)
    }

    pub fn pending_verification_requests(&self) -> &VerificationRequestRepository {
        &self.inner.pending_verification_requests
    }

    pub fn matrix_login_service(&self) -> &Box<dyn MatrixLoginService> {
        &self.inner.matrix_login_service
    }

    pub fn homeserver_resolver(&self) -> &HomeserverResolver {
        &self.inner.homeserver_resolver
    }

    pub fn credentials(&self) -> &SqliteCredentialsStore {
        &self.inner.credentials
    }

    pub fn credentials_handle(&self) -> Arc<SqliteCredentialsStore> {
        self.inner.credentials.clone()
    }

    pub fn secret_encryptor(&self) -> &SecretEncryptor {
        &self.inner.token_validator
    }

    pub fn register_pending_login(&self, token: String, pending: PendingLogin) {
        self.inner.pending_logins.insert(token, Arc::new(pending));
    }

    pub fn pending_login(&self, token: &str) -> Option<Arc<PendingLogin>> {
        self.inner.pending_logins.get(token).map(|entry| entry.value().clone())
    }

    pub fn remove_pending_login(&self, token: &str) -> Option<Arc<PendingLogin>> {
        self.inner.pending_logins.remove(token).map(|(_, recv)| recv)
    }

    pub fn find_pending_login_by_oauth_state(&self, state: &str) -> Option<(String, Arc<PendingLogin>)> {
        self.inner.pending_logins.iter()
            .find(|entry| {
                entry.value().pending_oauth.lock().map(|oauth| oauth.as_ref().map(|o| o.state.as_str()) == Some(state)).unwrap_or(false)
            })
            .map(|entry| (entry.key().clone(), entry.value().clone()))
    }

    pub fn take_pending_oauth(&self, token: &str) -> Option<PendingOAuth> {
        let entry = self.inner.pending_logins.get(token)?;
        let mut oauth = entry.value().pending_oauth.lock().ok()?;
        oauth.take()
    }

    pub fn hold_pending_oauth(&self, token: &str, oauth: PendingOAuth) {
        if let Some(entry) = self.inner.pending_logins.get(token) {
            if let Ok(mut slot) = entry.value().pending_oauth.lock() {
                *slot = Some(oauth);
            }
        }
    }

    pub async fn finish_login(&self, token: &str, matrix_client: Client) {
        // the restore path and the login page can race; whichever lands first wins
        if self.get_clients(token).is_some() {
            return;
        }

        let Some(pending) = self.pending_login(token) else {
            return;
        };

        let endpoint_id = EndpointId::new(pending.email.clone(), Some(pending.endpoint_guid.clone()));
        let msn_user = MsnUser::new(endpoint_id);
        let ticket_token = TicketToken(token.to_string());

        let contact_list = pending.contact_list.lock().expect("to be here").clone();

        let tachyon_client = TachyonClient::new(
            matrix_client.clone(),
            self.get_config().clone(),
            msn_user.clone(),
            ticket_token.clone(),
            pending.notification_sender.clone(),
            pending.client_shutdown_snd.clone(),
            pending.client_shutdown_recv.resubscribe(),
            contact_list
        );
        let drop_guard = self.insert_clients(token.to_owned(), tachyon_client.clone());

        sync_with_server_task(
            &pending.notification_sender,
            &pending.client_shutdown_snd,
            &pending.client_shutdown_recv,
            &ticket_token,
            &matrix_client,
            &msn_user,
            tachyon_client,
            self.get_config(),
        ).await.ok();

        if let Ok(mut slot) = pending.guard.lock() {
            *slot = Some(drop_guard);
        }

        log::info!("login completed for {}", token);
    }

}
