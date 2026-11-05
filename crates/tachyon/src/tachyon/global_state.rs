use crate::matrix::services::homeserver::HomeserverResolver;
use crate::matrix::services::login::MatrixLoginService;
use crate::matrix::verification_request_repository::VerificationRequestRepository;
use crate::tachyon::client::tachyon_client::TachyonClient;
use crate::tachyon::client::tachyon_client_repository::TachyonClientRepository;
use crate::tachyon::config::tachyon_config::TachyonConfig;
use crate::tachyon::mappers::user_id::MatrixIdCompatible;
use std::sync::Arc;
use tachyon_sqlite_store::SqliteCredentialsStore;

pub struct GlobalStateInner {
    config: TachyonConfig,
    tachyon_clients: TachyonClientRepository,
    pending_verification_requests: VerificationRequestRepository,
    matrix_login_service: Box<dyn MatrixLoginService>,
    credentials: Arc<SqliteCredentialsStore>,
    homeserver_resolver: HomeserverResolver,
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

        log::debug!("Client Drop Guard dropped");
    }
}

impl GlobalState {

    pub fn new(config: TachyonConfig, matrix_login_service: Box<dyn MatrixLoginService>, credentials: Arc<SqliteCredentialsStore>) -> Self {
        let homeserver_resolver = HomeserverResolver::new(!config.strict_ssl);
        Self {
            inner: Arc::new(GlobalStateInner {
                config,
                tachyon_clients: Default::default(),
                pending_verification_requests: Default::default(),
                matrix_login_service,
                credentials,
                homeserver_resolver,
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

}
