use crate::matrix::extensions::message_dedup::DedupState;
use crate::matrix::extensions::msn_user_resolver::RoomHashCache;
use crate::matrix::services::login::spawn_session_watcher;
use crate::notification::circle_store::CircleStore;
use crate::notification::handlers::auth::sync_with_server_task;
use crate::notification::models::notification_handle::NotificationHandle;
use crate::notification::models::soap_holder::SoapHolder;
use crate::p2p::client::session::{P2PSession, SessionId};
use crate::p2p::client::transport::Transport;
use crate::switchboard::models::switchboard_handle::SwitchboardHandle;
use crate::tachyon::alert::Alert;
use crate::tachyon::client::voice_clip::VoiceClipStore;
use crate::tachyon::config::tachyon_config::TachyonConfig;
use crate::tachyon::switchboard_service::SwitchboardService;
use dashmap::DashMap;
use matrix_sdk::locks::RwLock;
use matrix_sdk::ruma::OwnedRoomId;
use msnp::msnp::models::contact_list::ContactList;
use msnp::msnp::notification::command::command::NotificationServerCommand;
use msnp::msnp::notification::models::dialect::Dialect;
use msnp::p2p::v2::raw_p2p_payload::RawP2PPayload;
use msnp::shared::models::msn_user::MsnUser;
use msnp::shared::models::ticket_token::TicketToken;
use std::sync::{Arc, Mutex, RwLockWriteGuard};
use tachyon_sqlite_store::SqliteCredentialsStore;
use tokio::sync::{broadcast, mpsc};

/// The matrix backend of the client. It is attached once the matrix login
/// completes and stays attached for the lifetime of the client.
pub enum Backend {
    /// The MSNP session is connected but the matrix login has not completed yet.
    Pending,
    Ready(matrix_sdk::Client),
}

/// An in-flight OAuth login. The SDK client is held here until the homeserver
/// redirects back with the auth code.
pub struct PendingOAuth {
    pub state: String,
    pub client: matrix_sdk::Client,
}

pub struct TachyonClientInner {
    pub backend: RwLock<Backend>,
    pub pending_oauth: Mutex<Option<PendingOAuth>>,
    pub own_user: RwLock<MsnUser>,
    pub ticket_token: TicketToken,
    pub contact_list: Mutex<ContactList>,
    pub soap_holder: SoapHolder,
    pub switchboards: DashMap<OwnedRoomId, SwitchboardHandle>,
    pub alerts: DashMap<i32, Alert>,
    pub circle_store: CircleStore,
    pub notification_handle: NotificationHandle,
    pub config: TachyonConfig,
    pub client_shutdown_snd: broadcast::Sender<()>,
    pub client_shutdown_recv: broadcast::Receiver<()>,
    pub transports: DashMap<OwnedRoomId, Transport>,
    pub sessions: DashMap<SessionId, P2PSession>,
    pub chunked_uploads: DashMap<SessionId, Vec<RawP2PPayload>>,
    pub voice_clips: VoiceClipStore,
    /// Owned dedup state (§3.5, H7): the account's send-lock + sent-event-id
    /// tables. Lifetime = the signed-in account's lifetime — a second account
    /// in the same room can no longer have its sends suppressed or its echoes
    /// misattributed, fixed by ownership rather than a compound key.
    pub dedup: DedupState,
    /// Owned room-id hash cache (§3.5, H7) — the same lifetime story.
    pub room_hashes: RoomHashCache,
    /// §3.3's copy (D4): the dialect the NS connection negotiated at VER.
    /// `SessionState` stays the one owner/negotiator; this is the single
    /// copy into the client, written once at auth registration and read by
    /// the delivery seam — no parallel "is-sb-enabled"-style flags. The
    /// RwLock is the file's existing idiom (backend, own_user); the value
    /// is immutable in practice (one write, at registration). Defaults to
    /// the only profiled dialect, so pre-registration reads behave exactly
    /// as before the copy existed.
    pub dialect: RwLock<Dialect>,
}

#[derive(Clone)]
pub struct TachyonClient {
    pub inner: Arc<TachyonClientInner>
}


impl TachyonClient {
    pub fn new(
        config: TachyonConfig,
        user: MsnUser,
        token: TicketToken,
        notification_sender: mpsc::Sender<NotificationServerCommand>,
        client_shutdown_snd: broadcast::Sender<()>,
        client_shutdown_recv: broadcast::Receiver<()>,
        contact_list: ContactList,
    ) -> TachyonClient {
        TachyonClient {
            inner: Arc::new(TachyonClientInner {
                backend: RwLock::new(Backend::Pending),
                pending_oauth: Mutex::new(None),
                own_user: RwLock::new(user),
                ticket_token: token,
                contact_list: Mutex::new(contact_list),
                soap_holder: Default::default(),
                switchboards: Default::default(),
                alerts: Default::default(),
                circle_store: CircleStore::new(),
                notification_handle: NotificationHandle::new(notification_sender),
                config,
                client_shutdown_snd,
                client_shutdown_recv,
                transports: Default::default(),
                sessions: Default::default(),
                chunked_uploads: Default::default(),
                voice_clips: Default::default(),
                dedup: Default::default(),
                room_hashes: Default::default(),
                dialect: RwLock::new(Dialect::default()),
            })
        }
    }

    /// The matrix backend, if the login has completed.
    pub fn matrix_client(&self) -> Option<matrix_sdk::Client> {
        match &*self.inner.backend.read() {
            Backend::Ready(client) => Some(client.clone()),
            Backend::Pending => None,
        }
    }

    pub fn is_backend_ready(&self) -> bool {
        self.matrix_client().is_some()
    }

    pub fn matrix_client_ready(&self) -> matrix_sdk::Client {
        self.matrix_client().expect("backend is attached in every flow that needs it")
    }

    pub fn set_pending_oauth(&self, oauth: PendingOAuth) {
        if let Ok(mut slot) = self.inner.pending_oauth.lock() {
            *slot = Some(oauth);
        }
    }

    pub fn take_pending_oauth(&self) -> Option<PendingOAuth> {
        self.inner.pending_oauth.lock().ok()?.take()
    }

    pub fn pending_oauth_state(&self) -> Option<String> {
        self.inner.pending_oauth.lock().ok()?.as_ref().map(|oauth| oauth.state.clone())
    }

    /// Attach the matrix backend. The first caller wins: the concurrent login
    /// paths (session restore, login page, OAuth callback) observe the winner
    /// instead of double-attaching.
    pub fn attach_backend(&self, matrix_client: matrix_sdk::Client, credentials: Arc<SqliteCredentialsStore>) {
        {
            let mut backend = self.inner.backend.write();
            if matches!(*backend, Backend::Ready(_)) {
                return;
            }
            *backend = Backend::Ready(matrix_client.clone());
        }
        log::info!("matrix backend attached for {}", self.inner.ticket_token.as_str());
        spawn_session_watcher(matrix_client.clone(), self.inner.ticket_token.as_str().to_owned(), credentials, self.shutdown_receiver());
        sync_with_server_task(self.clone(), &matrix_client);
    }

    pub fn config(&self) -> TachyonConfig {
        self.inner.config.clone()
    }

    pub fn switchboards(&self) -> SwitchboardService {
        SwitchboardService::new(self.clone(), self.inner.config.switchboard_port)
    }

    pub fn alerts(&self) -> &DashMap<i32, Alert> {
        &self.inner.alerts
    }

    pub fn own_user(&self) -> MsnUser {
        self.inner.own_user.read().clone()
    }

    pub fn own_user_mut(&self) -> RwLockWriteGuard<'_, MsnUser> {
        self.inner.own_user.write()
    }

    pub fn soap_holder(&self) -> &SoapHolder {
        &self.inner.soap_holder
    }

    pub fn get_contact_list(&self) -> &Mutex<ContactList> {
        &self.inner.contact_list
    }

    pub fn ticket_token(&self) -> TicketToken {
        self.inner.ticket_token.clone()
    }

    pub fn shutdown_sender(&self) -> broadcast::Sender<()> {
        self.inner.client_shutdown_snd.clone()
    }

    pub fn shutdown(&self) {
        let _ = self.inner.client_shutdown_snd.send(());
    }

    pub fn shutdown_receiver(&self) -> broadcast::Receiver<()> {
        self.inner.client_shutdown_recv.resubscribe()
    }

    pub fn notification_handle(&self) -> NotificationHandle {
        self.inner.notification_handle.clone()
    }

    /// The account's owned dedup state — the single dedup path (§3.5).
    pub fn dedup(&self) -> &DedupState {
        &self.inner.dedup
    }

    /// The account's owned room-id hash cache (§3.5).
    pub fn room_hashes(&self) -> &RoomHashCache {
        &self.inner.room_hashes
    }

    /// The §3.3 dialect copy — negotiated once on the NS connection,
    /// written once at auth registration (`set_dialect`), read by the
    /// delivery seam (§3.4).
    pub fn dialect(&self) -> Dialect {
        *self.inner.dialect.read()
    }

    /// The auth-time copy of the negotiated dialect (§3.3). Called exactly
    /// once, at the SSO registration point, before the client becomes
    /// discoverable to any reader.
    pub fn set_dialect(&self, dialect: Dialect) {
        *self.inner.dialect.write() = dialect;
    }
}