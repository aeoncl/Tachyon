# Plan — Device verification in the new architecture (`Readiness` on the stored session)

Status: **proposed** (2026-09-01), **revised 2026-09-07**. Branch: `refactor/third-times-the-charm`.
Companion to `docs/architecture/tachyon-ports-adapters.md`; subtask 10 folds the outcome back into that doc.


## Revision note (2026-09-07)

The first draft introduced `BackendLogin`, a `LoginRepository`, a `DeviceVerified` witness and a
promotion step that swapped a login object for a session object once the device was verified. A
four-model review of that draft plus the first code slice found that the code had drifted to a flat
`VerificationService` keyed by `LoginId`, that neither shape could be finished without a second
per-login registry, and that the promotion step itself created two races (poll after promotion →
`LoginNotFound`; abandon while a handler still held the login → two `Client`s on one store).

Owner decision: **one `BackendSession` per login for its whole lifetime, stored in
`SessionRepository` together with a core-owned `Readiness { AuthNeeded, VerificationNeeded, Ready }`.**
No promotion, no witness, no second repository, no per-login map in the adapter. The invariant
"bridges only talk to verified sessions" becomes one runtime gate, `SessionRepository::get_ready`,
instead of a type. The object never moves between maps, so the two races above disappear. This also
matches the architecture doc's "Two-frontend call model", which already lists verification and
recovery as direct `BackendSession` calls.

Decisions folded in from the review (each lives in one subtask; revert by editing that subtask):

- **Subtask 0** gets the test gates green before anything else (`domain_is_pure` is red, the
  legacy crate does not compile at HEAD).
- **One current verification flow per session.** No `VerificationFlowId`, no flow map, no
  `?verification=` URL parameter. A second `start_device_verification` cancels and replaces the first.
- **Reset input is `Option<ResetAuth>`**, output gains `IdentityReset::ApprovalPending`, and the
  SDK's `reset_identity()` is called exactly once per user flow (it deletes backups before it
  reports which auth it needs).
- **`VerificationFlowState::Ready` stays**; the driver subscribes to `changes()` *before* reading
  `state()`, calls `start_sas()` on `Ready`, and treats `KeysExchanged { emojis: None }` as a cancel.
- **Legacy bridge separates ticket authorization from the one-shot alert**, so firing the alert
  no longer bounces the browser to `/tachyon/login` while the USR handler finishes.
- Kept from the drifted code: `RecoveryKey` newtype, `recover` returning `DeviceStatus`,
  `IdentityReset` as a value enum, `VerificationError` as the name.


## Context

Device verification (cross-signing the bridge's Matrix device via recovery key, SAS-emoji with
another device, or identity reset) still lives entirely in the legacy `crates/tachyon` crate and
drives `matrix_sdk::Client` directly (`matrix/cross_signing.rs`,
`notification/handlers/auth.rs::sync_with_server_task`, `web/tachyon/matrix_auth.rs`, the
`web/tachyon/{confirm_device,verification}` pages). Core has no notion of verification, and a
session is handed to the bridge *before* the device is trusted — `GlobalState::PreSession` and the
`as_any()` downcasts exist only to paper over that. Known bug: the restore-path `NOT` URL
(`auth.rs:319`) has no `notification_id` while `confirm_device::get_confirm` asserts on it.

Target: verification is a step of core's login flow, owned by a new `DeviceVerificationUseCase`
in `tachyon-core`, Matrix specifics in `tachyon-backend-matrix`. **A bridge can only obtain a
session whose `Readiness` is `Ready`**, through `SessionRepository::get_ready` or
`LoginOutcome::SessionOpened`. The legacy crate keeps its UI/MSNP handlers but calls the use cases.
`tachyon-web` stays a stub.

### Lifecycle of one login

```
 start_interactive_login ──► [AuthNeeded] ──finish_interactive_login──► settle ──► [Ready]
                                  │                                        │
                                  │ abandon / cancel                       └─ Unverified ──► [VerificationNeeded]
                                  ▼                                                               │
                               removed                    recover | SAS Done | reset, then restore ─┘──► settle ──► [Ready]
                                                                                                  │ timeout / cancel
 restore(token):  entry Ready ──► SessionOpened                                                   ▼
                  entry VerificationNeeded ──► settle                                           abandon ──► removed
                  no entry ──► auth_service.restore(login_id) ──► insert ──► settle

 settle(login_id, session) = session.device_status()?  Verified   ──► Readiness::Ready              ──► SessionOpened
                                                       Unverified ──► Readiness::VerificationNeeded ──► DeviceVerificationRequired
```

`settle` is the only writer of `Readiness`. It runs under the global lifecycle mutex decided on
2026-09-03 (the one that already guards restore/commit/evict). `restore` for a token whose login is
still `AuthNeeded` cannot happen: the token→login row is only written by `bind_token` after the
OAuth callback.

Build note: subtasks 1–3 change the `AuthService` / `BackendSession` ports, so
`tachyon-backend-matrix` and `tachyon` stop compiling until subtasks 5–9 land. Verify each subtask
with `cargo test -p <crate>`; `cargo test --workspace` is the gate at the end of subtask 9.

---

## Subtask 0 — make the gates green

**Files:** `crates/tachyon-core/src/domain/bridge.rs` (delete), `src/application/ports.rs`,
`crates/tachyon-backend-matrix/src/infrastructure/backend/mod.rs`,
`crates/tachyon/src/{main.rs,notification/handlers/auth.rs,web/tachyon/matrix_auth.rs}`.

- `domain_is_pure` fails: `domain/bridge.rs` holds an `async fn` trait. Move `BridgeHandle` into
  `application/ports.rs` (it is an outbound port, not domain data); delete `domain/bridge.rs`.
- `BridgeRepository` uses bare `async fn` in a trait used as `dyn`: add `#[async_trait]`.
- The legacy crate stays red until subtask 8; subtask 3 changes the very calls a fix here would
  touch, so patching it now is throwaway work.

**Done when:** `cargo test -p tachyon-core` green (both architecture tests).

---

## Subtask 1 — core domain types & errors

**Files:** `crates/tachyon-core/src/domain/{auth.rs,verification.rs,ids.rs}`, `src/application/error.rs`.

- `auth.rs`: `#[derive(Debug, Clone, Copy, PartialEq, Eq)] pub enum Readiness { AuthNeeded, VerificationNeeded, Ready }`
  with `fn can_advance_to(self, next: Readiness) -> bool` (forward only: `AuthNeeded → {VerificationNeeded, Ready}`,
  `VerificationNeeded → Ready`).
- `verification.rs` (already present; adjust):
  - `VerificationFlowState` gains `Ready` between `Requested` and `Started`
    ("the other device accepted; waiting for the emoji exchange to start").
  - `impl VerificationFlowState { pub fn name(&self) -> &'static str }` — stable per variant, the
    poll-dedupe key for subtask 9.
  - `pub struct Password(String)` with the same redacted `Debug` as `RecoveryKey`
    (or rename `RecoveryKey` to a shared `Secret`; pick one).
  - `pub enum ResetAuth { Password(Password), Approved }`.
  - `IdentityReset` gains `ApprovalPending` ("the reset is running against the homeserver; call again").
- `ids.rs`: **remove** `str_id!(VerificationFlowId)`.
- `error.rs`:
  ```rust
  #[derive(Debug, thiserror::Error)]
  pub enum VerificationError {
      #[error("no login with that id")]                 LoginNotFound,
      #[error("login has not finished authenticating")] NotAuthenticated,   // Readiness::AuthNeeded
      #[error("device is already verified")]            AlreadyVerified,     // mutating call on Readiness::Ready
      #[error("no verification in progress")]           NoVerificationInProgress,
      #[error("recovery key rejected")]                 RecoveryKeyRejected,
      #[error("device is still unverified")]            StillUnverified,
      #[error(transparent)] Backend(#[from] BackendError),
      #[error(transparent)] Store(#[from] StoreError),
  }
  ```
  `AuthError` gains `DeviceNotVerified`, `LoginNotFound` (existing names untouched). Add
  `thiserror` to `tachyon-core` if it is not already a dependency; give `BackendError`/`StoreError`
  `Display` while there (the architecture doc asks for `thiserror` at every seam).

**Done when:** `cargo test -p tachyon-core` green.

---

## Subtask 2 — ports & `SessionRepository`

**Files:** `crates/tachyon-core/src/application/ports.rs`, `src/infrastructure/repository.rs`;
**delete** `crates/tachyon-backend-matrix/src/infrastructure/backend/verification_service.rs` and its `pub mod` line.

- `BackendSession` becomes the per-login object (auth → verification → messaging later):
  ```rust
  #[async_trait] pub trait BackendSession: Send + Sync {
      // authentication (valid while Readiness::AuthNeeded)
      async fn finish_interactive_login(&self, callback_query: &str) -> Result<(), BackendError>;
      // device trust
      async fn device_status(&self) -> Result<DeviceStatus, BackendError>;
      async fn verification_options(&self) -> Result<VerificationOptions, VerificationError>;
      async fn recover(&self, key: &RecoveryKey) -> Result<DeviceStatus, VerificationError>;
      async fn start_device_verification(&self, device: &DeviceId) -> Result<(), VerificationError>;  // replaces any current flow
      fn verification_state(&self) -> Option<VerificationFlowState>;                                  // None = no flow started
      async fn verification_action(&self, action: VerificationAction) -> Result<(), VerificationError>;
      async fn reset_identity(&self, auth: Option<ResetAuth>) -> Result<IdentityReset, VerificationError>;
      // lifecycle
      async fn close(&self);            // idempotent; every later call returns BackendError::Technical("session closed")
      fn as_any(&self) -> &dyn Any;     // FIXME: until the messaging port exists
  }
  ```
- `AuthService` shrinks to the two factory methods:
  ```rust
  async fn restore(&self, login_id: &LoginId) -> Result<Arc<dyn BackendSession>, BackendError>;
  async fn start_interactive_login(&self, login_id: &LoginId, server_name: &str, user_id: Option<UserId>,
                                   redirect_url: &str, bridge_metadata: &BridgeMetadata)
      -> Result<(Arc<dyn BackendSession>, InteractiveAuthStarted), BackendError>;
  ```
  `finish_interactive_login` moves onto the session. Delete `VerificationService`.
- `SessionRepository` stores readiness next to the session:
  ```rust
  pub struct SessionEntry { pub session: Arc<dyn BackendSession>, pub readiness: Readiness }
  pub trait SessionRepository: Send + Sync {
      fn insert(&self, login_id: LoginId, session: Arc<dyn BackendSession>, readiness: Readiness) -> Option<SessionEntry>;
      fn get(&self, login_id: &LoginId) -> Option<SessionEntry>;
      /// The only accessor bridges may use.
      fn get_ready(&self, login_id: &LoginId) -> Option<Arc<dyn BackendSession>>;
      /// Forward transitions only (`Readiness::can_advance_to`); returns the previous readiness.
      fn set_readiness(&self, login_id: &LoginId, readiness: Readiness) -> Result<Readiness, ReadinessError>;
      fn remove(&self, login_id: &LoginId) -> Option<SessionEntry>;
  }
  ```
  `SessionRepositoryInMem`: one `DashMap<LoginId, SessionEntry>`. `ReadinessError { NotFound, Backwards { from, to } }` in `error.rs`.
- Existing callers of `SessionRepository::get` in the legacy crate become `get_ready`.

**Done when:** `cargo build -p tachyon-core` (AuthUseCase stubbed to compile; fixed in subtask 3).

---

## Subtask 3 — `AuthUseCase`: `LoginOutcome` + `settle`

**Files:** `crates/tachyon-core/src/application/auth_use_case.rs`, `src/infrastructure/app_state.rs`.

- API:
  ```rust
  pub enum LoginOutcome { SessionOpened { login_id, session: Arc<dyn BackendSession> }, DeviceVerificationRequired { login_id } }
  start_interactive_login(server_name, user_id, &BridgeMetadata) -> Result<LoginStart, AuthError>   // inserts AuthNeeded
  finish_interactive_login(&LoginId, &str) -> Result<LoginOutcome, AuthError>
  restore(&TachyonToken) -> Result<LoginOutcome, AuthError>                                          // was restore_session
  abandon_login(&LoginId) -> Result<(), AuthError>                                                   // Ok(()) when nothing pending
  bind_token(TachyonToken, LoginId) unchanged; RestoredLogin removed
  ```
- One `tokio::sync::Mutex<()>` field — the global lifecycle mutex from the 2026-09-03 decisions.
  `finish_interactive_login`'s tail, all of `restore`, and `abandon_login` run under it. Keep the
  critical sections in private `async fn`s that never call a public `AuthUseCase` method (the mutex
  is not reentrant).
- `finish_interactive_login`: entry must exist with `AuthNeeded` else `LoginNotFound`;
  `session.finish_interactive_login(cb)`; then `settle`.
- `restore`:
  1. no `LoginId` for the token → `BackendCredentialsNotInStore` (unchanged);
  2. entry `Ready` → `SessionOpened`;
  3. entry `VerificationNeeded` → `settle`;
  4. entry `AuthNeeded` → `LoginNotFound` (cannot happen, see lifecycle; keep the arm explicit);
  5. no entry → `auth_service.restore(login_id)` → `insert(.., AuthNeeded)` → `settle`.
     A `LoggedOut` / `SoftLoggedOut` here is the `LoginDead` eviction path of the 2026-09-03
     decisions; do not add a second one.
- `settle(login_id, session)`: `device_status()`; `Verified` → `set_readiness(Ready)` →
  `SessionOpened`; `Unverified` → `set_readiness(VerificationNeeded)` (a no-op when already there) →
  `DeviceVerificationRequired`.
- `abandon_login`: `remove` → `session.close().await`. The `Arc` may still be held by a web handler;
  `close()` makes every later call on it fail (subtask 5).
- `AppState` unchanged in signature; `SessionRepositoryInMem` gains nothing new to wire.
- Tests (`FakeAuthService` / `FakeBackendSession` — scriptable `device_status` sequence, `finish`
  and `close` counters — in a `#[cfg(test)] pub(crate) mod test_support` inside `tachyon-core`;
  a `tachyon-testkit` dev-dependency would be a cycle, and in-crate unit tests would then see a
  different `tachyon_core` than the fakes):
  verified restore → `SessionOpened`, entry `Ready`; unverified → `DeviceVerificationRequired`,
  entry `VerificationNeeded`, `get_ready` is `None`; restore after the fake flips → `Ready`,
  `device_status` called once per restore, `auth_service.restore` called once overall; two
  concurrent restores on an empty repository → one `auth_service.restore`; `abandon_login` on
  nothing → `Ok`; `abandon_login` → `close` called once and `get` is `None`;
  `set_readiness(Ready → VerificationNeeded)` → `Backwards`.

**Done when:** `cargo test -p tachyon-core` green.

---

## Subtask 4 — `DeviceVerificationUseCase`

**Files:** new `crates/tachyon-core/src/application/device_verification_use_case.rs` (replaces the empty
`verification_use_case.rs`), `application/mod.rs`, `infrastructure/app_state.rs`.

- Keyed by `TachyonToken` (→ `LoginId` via `AccountRepository` → `SessionRepository::get`):
  ```rust
  status(token) -> DeviceStatus                        // Ready → Verified without asking the backend
  options(token) -> VerificationOptions
  recover(token, &RecoveryKey) -> ()                   // Unverified after → StillUnverified
  start_device_verification(token, &DeviceId) -> ()
  verification_state(token) -> VerificationFlowState   // None from the session → NoVerificationInProgress
  verification_action(token, VerificationAction) -> ()
  reset_identity(token, Option<ResetAuth>) -> IdentityReset
  ```
  all `Result<_, VerificationError>`. Resolution: no entry → `LoginNotFound`; `AuthNeeded` →
  `NotAuthenticated`; `Ready` → fine for `status`, `options`, `verification_state` (the finished
  flow is still on the session, so the poll page's last tick after `Done` just works), `AlreadyVerified`
  for the four mutating calls. The use case never writes `Readiness`; `restore` does, via `settle`.
- `AppState::device_verification_use_case()`.
- Tests with the subtask-3 fakes: `recover` leaving unverified → `StillUnverified`; `status` on a
  `Ready` entry → `Verified` with zero `device_status` calls; `verification_state` on a `Ready`
  entry returns the fake's `Done`; `recover` on `Ready` → `AlreadyVerified`; unknown token → `LoginNotFound`.

**Done when:** `cargo test -p tachyon-core` green; `application_does_not_know_infrastructure` green.

---

## Subtask 5 — matrix adapter: `BackendSessionMatrix` owns the login end to end

**Files:** `crates/tachyon-backend-matrix/src/infrastructure/backend/{auth_service.rs,session.rs}`,
`infrastructure/mappers.rs`, `Cargo.toml` (+ `futures-util`).

- `BackendSessionMatrix { client, login_id, credential_repository, tasks_token: CancellationToken,
  verification: std::sync::Mutex<Option<VerificationFlowMatrix>>, reset: std::sync::Mutex<Option<PendingReset>>,
  closed: AtomicBool }`. `std::sync::Mutex` only, never held across an await. **No `matrix_client()`
  accessor** once subtask 9 lands.
- `AuthServiceMatrixSdk::start_interactive_login`: build the `Client`, register the OAuth client,
  build the authorization URL, return `(Arc<BackendSessionMatrix>, InteractiveAuthStarted::OAuth {..})`.
  **Delete** `pending_clients`, `PendingClient`, `PENDING_CLIENT_TTL` and the two purge tests — the
  session in `SessionRepository` is the pending client now, and `abandon_login` is its expiry
  (owner: no TTL).
- `BackendSessionMatrix::finish_interactive_login`: `client.oauth().finish_login(..)`, store the
  credential blob (moved from `AuthServiceMatrixSdk::store_credentials`), then spawn the
  token-refresh watcher (`subscribe_to_session_tokens`, moved here) under `tasks_token`.
- `AuthServiceMatrixSdk::restore`: build the client, `restore_session` + `whoami` (the body of the
  current `BackendSessionMatrix::restore`, error mapping unchanged), spawn the watcher, return the session.
- `device_status`: `wait_for_e2ee_initialization_tasks()`, `request_user_identity(own)`,
  `get_own_device()?.is_cross_signed_by_owner()` (port of `cross_signing::check_device_is_crossed_signed`).
- `close()`: set `closed`; cancel `tasks_token`; take and drop the current `VerificationFlowMatrix`
  (its `Drop` aborts the driver and cancels the to-device sync); cancel a pending reset. `Drop` calls
  the same routine. Every trait method starts with `self.ensure_open()?` → `Technical("session closed")`.
- Store dir: per the 2026-09-03 decision the sqlite store is `store_root/logins/<login_id>`;
  `build_client` takes the `LoginId` instead of the user id. If that change has not landed yet,
  keep the current per-user path and leave a `TODO(login-store-dir)`.
- `mappers.rs`: `DeviceId` ↔ `OwnedDeviceId`.
- Tests (wiremock client builder from the existing tests): `close()` cancels `tasks_token`;
  `Drop` without `close()` cancels it too; a method after `close()` → `Technical`; `start_interactive_login`
  returns a session and stores nothing in the service.

**Done when:** `cargo test -p tachyon-backend-matrix` green.

---

## Subtask 6 — matrix adapter: SAS flow & to-device sync

**Files:** new `crates/tachyon-backend-matrix/src/infrastructure/backend/verification.rs`, `session.rs`.

- Port `build_to_device_only_sliding_sync` / `cross_sign_sync_task` from
  `crates/tachyon/src/matrix/cross_signing.rs:157-233` (drop the duplicate `add_list` at `:199`).
  The sync is owned by the flow: `VerificationFlowMatrix { request: VerificationRequest,
  driver: JoinHandle<()>, to_device_sync: CancellationToken }`; `Drop` aborts the driver and cancels
  the sync. Not needed for recover/reset.
- `start_device_verification(device)`: take and drop any current flow (cancelling its request), start
  the to-device sync **before** `device.request_verification_with_methods([SasV1])`
  (`.ready/.start/.key/.mac` are to-device events), spawn the driver, store the flow.
- Driver, in this order: `let mut changes = request.changes();` **first** — eyeball's `subscribe()`
  only yields values set after the call, so reading `state()` first loses a `Transitioned` that
  lands in between — then act on `request.state()`, then act on each item of `changes`. Write every
  step as "act on the state I observe now", never "wait for state X" (subscribers are latest-value).
  - `Ready { their_methods, .. }`: SAS-v1 missing → `request.cancel()`; present → `request.start_sas()`
    once (the initiator's request never advances on its own; `start_sas` returns `Ok(None)` if the peer
    started first). **No `accept_with_methods`** — a no-op for the initiator
    (`matrix-sdk-crypto/src/verification/requests.rs:927`); legacy `verification/mod.rs:97` was dead.
  - `Transitioned` with SAS → `sas.accept().await` immediately (idempotent, `sas/mod.rs:436`).
  - SAS `KeysExchanged { emojis: None, .. }` → `sas.cancel()`; the peer negotiated decimal-only and
    the page has nothing to show.
  - `Done` / `Cancelled` → cancel `to_device_sync`, exit.
- `verification_state()`: derive from `request.state()` (`Transitioned { verification }` →
  `verification.sas()` → `sas.state()`); `None` when no flow. Pure mappers `map_request_state` /
  `map_sas_state`: `Created|Requested → Requested`, `Ready → Ready`, SAS `Created|Started|Accepted → Started`,
  `KeysExchanged { emojis: Some } → CompareEmojis`, `Confirmed → AwaitingOtherConfirmation`, `Done`,
  `Cancelled { reason: cancel_info.reason() }`, `Transitioned` without SAS → `Cancelled`.
- `confirm/mismatch` → SAS; `cancel` → request. Any action with no flow → `NoVerificationInProgress`.
- Tests: mapper fns on SDK-constructible variants (`Created`, `Ready`, `Done`, `Cancelled`, SAS
  `Confirmed`, `KeysExchanged` with and without emojis).

**Done when:** `cargo test -p tachyon-backend-matrix` green.

---

## Subtask 7 — matrix adapter: options, recovery, identity reset

**Files:** `crates/tachyon-backend-matrix/src/infrastructure/backend/session.rs`.

- `verification_options`: run the initial key query first — `request_user_identity(own)` or
  `has_devices_to_verify_against()` as the legacy `other_device.rs:25` did — then
  `secret_storage().is_enabled()` + `get_user_devices(own)` filtered as `other_device.rs:29-33`
  (cross-signed, curve25519 key, not dehydrated) → `DeviceSummary`. `get_user_devices` is local-only;
  without the query the device card is empty on a wiped store.
- `recover(key)`: `recovery().recover(key)` (HTTP only; `import_secrets` self-signs,
  `secret_store.rs:426-441`); then `device_status()` and return it. Error mapping:
  `RecoveryError::SecretStorage(_)` → `RecoveryKeyRejected`; `Sdk(_)` / `BackupExistsOnServer` → `Backend(Technical)`.
- `reset_identity(auth)`, one SDK `recovery().reset_identity()` per user flow — it deletes backups and
  disables secret storage *before* reporting the auth type (`recovery/mod.rs:432-437`), so it must
  never run twice:
  - `None` with no pending reset → call it, keep the `IdentityResetHandle` in `reset` for **both**
    auth types; `Uiaa` → `PasswordRequired`; `OAuth` → `ApprovalRequired { url }`.
  - `None` with a pending reset → report its current state (`PasswordRequired` / `ApprovalRequired` /
    `ApprovalPending`), never restart.
  - `Some(Password(p))` → `handle.reset(Some(password auth))` inline (one round trip).
  - `Some(Approved)` → if not already running, spawn `handle.reset(None)` — the SDK loop
    (`encryption/mod.rs:297-319`) retries HTTP with no sleep, so wrap it in `tokio::time::timeout`
    bounded by the remaining sign-in window and a child of the session's cancellation — and record
    the task outcome as `Done | Failed(e) | Cancelled`; while running → `ApprovalPending`.
  - After a successful reset: `recovery().enable()` → `Done { recovery_key }`; then `device_status()`
    is expected `Verified`.
  - `close()` cancels the handle and aborts the task. A cancelled reset leaves the account with backups
    deleted and no new identity; the page copy must present reset as last resort (subtask 9).

**Done when:** `cargo test -p tachyon-backend-matrix` green; manual check of recover against the MAS homeserver.

---

## Subtask 8 — legacy: `GlobalState`, USR handler, OAuth callback

**Files:** `crates/tachyon/src/tachyon/global_state.rs`, `notification/handlers/auth.rs`,
`web/tachyon/matrix_auth.rs`, `web/tachyon/middleware.rs`.

- `global_state.rs`: delete `PreSession`, `pre_sessions`, `pending_verification_requests`,
  `insert_pre_session/remove_pre_session/confirmation_client/has_confirmation_alert/take_confirmation_alert`,
  the `remove_for` call in `ClientDropGuard`. Add two separate things:
  `authorized_tickets: DashSet<String>` (a ticket may use the confirm/verification pages) and
  `pending_verifications: DashMap<String /*ticket*/, Alert>` (the one-shot completion channel).
  `is_session_token` = `TachyonClient` **or** authorized ticket. The ticket is de-authorized only
  after `insert_clients`, or on abandon — never when the alert fires.
- `auth.rs`: `authenticate` matches `LoginOutcome`; on `DeviceVerificationRequired { login_id }` a
  new `device_verification(..)` step: authorize the ticket, `store_pending_verification(ticket, alert)`,
  send the `NOT` → `/tachyon/confirm_device?t=<ticket>` (no `notification_id`), then
  `select!` on `timeout_at(deadline, receiver.recv())` and the connection's shutdown signal
  (`client_shutdown_recv`, as the deleted `sync_with_server_task` did). On timeout / shutdown /
  failure: `take_pending_verification`, de-authorize, `abandon_login(&login_id)`. On success:
  `restore(&token)`; a `DeviceVerificationRequired` here is usually the peer's signature upload
  racing our `/keys/query` (the SDK sends `.mac`/`.done` before the signature), so retry `restore`
  with a 500 ms backoff until the deadline before failing. `interactive_login` returns its trailing
  `restore`'s `LoginOutcome`; its `abandon` closure = `take_pending_login` + `take_pending_verification`
  + de-authorize + `abandon_login`. `sync_with_server_task` loses lines 294-363 and the `deadline`
  param. `USR OK` still first; `CLIENT_SIGN_IN_WINDOW` unchanged.
- `matrix_auth.rs::get_login_callback`: `LoginOutcome`; `bind_token` failure → `abandon_login`;
  `SessionOpened` → fire alert + success page; `DeviceVerificationRequired` → authorize ticket +
  `store_pending_verification(ticket, pending.alert)` + redirect to `/tachyon/confirm_device?t=<ticket>`.
  Delete the downcast and the cross-signing import.
- Web pages may not compile yet — stub them if needed; they are subtask 9.

**Done when:** `cargo build -p tachyon` (pages stubbed) and the existing `command_handler.rs` tests pass.

---

## Subtask 9 — legacy: web pages on the use case + deletions

**Files:** `crates/tachyon/src/web/tachyon/{mod.rs,confirm_device/*.rs,verification/*.rs}`,
`crates/tachyon/assets/web/tachyon/verify.js`,
`crates/tachyon/src/matrix/{mod.rs,cross_signing.rs,verification_request_repository.rs}`.

- All pages call `state.app_state().device_verification_use_case()` with `TachyonToken::new(&token)`
  from the request extension; replace `unwrap/assert!/panic!` with the `error_page` style of
  `matrix_auth.rs:196`, rendering `VerificationError`'s `Display`.
  - `get_confirm` → `status` then `options`; `Verified` → fire the alert + success page; recovery
    card only if `recovery_available`, device card only if `devices` non-empty; reset card last,
    labelled as destructive.
  - `post_recover` → `recover`; success fires the alert; `RecoveryKeyRejected` re-renders the form,
    `Backend` renders the error page (do not tell the user to retype a key over a network error).
  - `other_device` → `options` / `start_device_verification` → `X-IC-Redirect` to `/tachyon/verification`.
  - `get_verification_poll` → `verification_state`; 204 when `state.name()` == `state=` param;
    fold `sas_v1.rs` rendering in (`CompareEmojis` → `emoji_table`, description-based images);
    `Done` fires the alert, `Cancelled` fails it; drop `user_id`; delete `sas_v1.rs`.
  - `/verification/sas_v1/{action}` (`confirm|mismatch|cancel`) → `verification_action`; drop `accept`.
  - `reset_identity` → `reset_identity(token, ..)`: first `GET` calls with `None`;
    `PasswordRequired` → password form posting `Some(Password)`; `ApprovalRequired { url }` → link +
    "I've approved, continue" form posting `Some(Approved)`; `ApprovalPending` re-renders with a poll;
    `Done { recovery_key }` shows the key once and fires the alert.
  - `verify.js:5,93`: `restore-method` → `restore_method` (matches `recover.rs:43`).
- Delete `cross_signing.rs`, `verification_request_repository.rs` and their `pub mod` lines.
  `matrix/handlers/request_verification_handlers.rs` (in-session inbound, auto-confirm stub) is out of scope.

**Done when:** `cargo test --workspace` green;
`grep -rn matrix_sdk crates/tachyon/src/web/tachyon` → no hits in `confirm_device/`, `verification/`, `matrix_auth.rs`;
`grep -rn "as_any\|BackendSessionMatrix" crates/tachyon/src` → only the `TachyonClient::new` scaffold in `auth.rs`.

---

## Subtask 10 — docs

**Files:** `docs/architecture/tachyon-ports-adapters.md`, `CONTEXT.md`.

- Replace the §"The interactive login flow" diagram with the lifecycle above (keep `Failed`); in
  §"Ports" describe `BackendSession` as the per-login object with `Readiness` tracked by
  `SessionRepository`, `AuthService` as its factory, `get_ready` as the only bridge accessor;
  `restore(login_id)` / `start_interactive_login(..)` return `BackendSession` there and in
  correction 8; keep "verification, recovery" in the `BackendSession` direct-call list
  (§"Two-frontend call model" — it was right); add a "Device verification" paragraph under
  §"Auth and credentials" (use case, `settle` as the only `Readiness` writer, to-device sync only
  while a SAS flow is live); startup step 5 → "restores a `BackendSession`, settles its readiness".
- `CONTEXT.md`: `Login flow` states → `AuthNeeded → VerificationNeeded → Ready`; add `Readiness`,
  `LoginOutcome`, `DeviceVerificationUseCase`, `settle`; no `BackendLogin` / `LoginRepository` anywhere.

**Done when:** doc and glossary names match the code (`CONTEXT.md` rule).

---

## End-to-end verification (after subtask 9)

Patched WLM client against the MAS homeserver (`/run` skill), watching the log:
1. Fresh instance: sign in → login `NOT` → OAuth → callback lands on confirm-device → recover
   with the recovery key → confirmation page → client leaves the sign-in screen (`SBS`). Log:
   `DeviceVerificationRequired` … `SessionOpened`; `get_ready` returns `None` until the second.
   The browser is never redirected to `/tachyon/login` during the flow.
2. Restart with `store_root` and the Tachyon DB intact (credentials are in sqlite, so there is no
   OAuth re-run) → `restore` → `device_status` reads `Verified` from the crypto store → `SessionOpened`,
   no verification alert, no callback.
3. Wipe the crypto store, keep the DB → verify with another device: the device card is populated;
   emojis match Element; "They match" → `Done` → client signs in. Cancel from Element →
   "cancelled" page, client refused. Repeat with Element X (it does not auto-start; `start_sas` must).
4. Let the verification alert time out (~5 min) → client refused, login abandoned, no further
   to-device `/sync` lines, a fresh sign-in starts cleanly.
5. Ctrl-C while the alert is pending → the process exits promptly (connection shutdown arm).
