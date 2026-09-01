# Plan — Move device verification into the new architecture (`DeviceVerificationUseCase`)

Status: **proposed** (2026-09-01). Branch: `refactor/third-times-the-charm`.
Companion to `docs/architecture/tachyon-ports-adapters.md`; subtask 10 folds the outcome back into that doc.


## Context

Device verification (cross-signing the bridge's Matrix device via recovery key, SAS-emoji
with another device, or identity reset) still lives entirely in the legacy `crates/tachyon`
crate and drives `matrix_sdk::Client` directly (`matrix/cross_signing.rs`,
`notification/handlers/auth.rs::sync_with_server_task`, `web/tachyon/matrix_auth.rs`, the
`web/tachyon/{confirm_device,verification}` pages). Core has no notion of verification, and
a `BackendSession` is created and stored in `SessionRepository` *before* the device is
trusted — `GlobalState::PreSession` and the `as_any()` downcasts exist only to paper over
that. Known bug: the restore-path `NOT` URL (`auth.rs:319`) has no `notification_id` while
`confirm_device::get_confirm` asserts on it.

Target: verification is a step of core's login flow, owned by a new
`DeviceVerificationUseCase` in `tachyon-core`, Matrix specifics in `tachyon-backend-matrix`.
**No `BackendSession` exists until the login is authenticated *and* verified**, enforced by
the type system. The legacy crate keeps its UI/MSNP handlers but calls the use cases.
`tachyon-bridge-web` stays a stub.

```
Started → AwaitingUser → ProofReceived → Authenticated ──(device verified)──► SessionOpened
                                              │ not verified                       ▲
                                              ▼                                    │
                                   DeviceVerificationRequired ──(recover | SAS | reset)──┘
                                              │ timeout / cancel
                                              ▼
                                            Failed
```
Restoring from stored credentials enters at `Authenticated`.

Build note: subtasks 1–4 change `AuthService`'s return type, so `tachyon-backend-matrix`
and `tachyon` stop compiling until subtasks 5–9 land. Verify each subtask with
`cargo test -p <crate>`; `cargo test --workspace` is the gate at the end of subtask 9.

---

## Subtask 1 — core domain types & errors

**Files:** `crates/tachyon-core/src/domain/{auth.rs,ids.rs}`, `src/application/error.rs`.

- `ids.rs`: `str_id!(VerificationFlowId)`.
- `auth.rs` (sync, plain data, `Debug + Clone`):
  ```rust
  pub enum DeviceStatus { Verified, Unverified }
  pub struct VerificationOptions { pub recovery_available: bool, pub devices: Vec<DeviceSummary> }
  pub struct DeviceSummary { pub id: DeviceId, pub display_name: Option<String> }
  pub enum VerificationFlowState {
      Requested, Ready, Started, CompareEmojis { emojis: Vec<SasEmoji> },
      AwaitingOtherConfirmation, Done, Cancelled { reason: String },
  }
  impl VerificationFlowState { pub fn name(&self) -> &'static str }   // stable per variant (poll dedupe)
  pub struct SasEmoji { pub symbol: String, pub description: String }
  pub enum VerificationAction { Confirm, Mismatch, Cancel }
  pub enum ResetAuth { Password(String), Approved }
  pub struct IdentityReset { pub recovery_key: String }
  ```
  No "accept" anywhere — automatic transitions are the adapter's job.
- `error.rs`: new `#[derive(Debug, thiserror::Error)] DeviceVerificationError { LoginNotFound, FlowNotFound, StillUnverified, RecoveryFailed(String), PasswordRequired, ApprovalRequired { url: String }, ApprovalPending, Backend(#[from] BackendError), Store(#[from] StoreError) }`;
  `AuthError` gains `DeviceNotVerified`, `LoginNotFound` (existing names untouched).

**Done when:** `cargo test -p tachyon-core` passes (`domain_is_pure` still green).

---

## Subtask 2 — core ports & `LoginRepository`

**Files:** `crates/tachyon-core/src/application/ports.rs`, `src/infrastructure/repository.rs`, `src/application/auth_use_case.rs` (witness only).

- `DeviceVerified(())` witness in `auth_use_case.rs`: `pub struct` with a `pub(crate)` constructor only.
- `ports.rs`:
  ```rust
  #[async_trait] pub trait BackendLogin: Send + Sync {
      async fn device_status(&self) -> Result<DeviceStatus, BackendError>;
      async fn verification_options(&self) -> Result<VerificationOptions, BackendError>;
      async fn recover(&self, secret: &str) -> Result<(), DeviceVerificationError>;
      async fn request_device_verification(&self, device: &DeviceId) -> Result<Arc<dyn VerificationFlow>, DeviceVerificationError>;
      async fn reset_identity(&self, auth: Option<&ResetAuth>) -> Result<IdentityReset, DeviceVerificationError>;
      async fn open_session(&self, proof: DeviceVerified) -> Result<Arc<dyn BackendSession>, BackendError>; // 2nd call = Technical error
      async fn abandon(&self);
  }
  #[async_trait] pub trait VerificationFlow: Send + Sync {
      fn id(&self) -> &VerificationFlowId;
      fn state(&self) -> VerificationFlowState;   // sync
      async fn confirm(&self) -> Result<(), DeviceVerificationError>;
      async fn mismatch(&self) -> Result<(), DeviceVerificationError>;
      async fn cancel(&self) -> Result<(), DeviceVerificationError>;
  }
  pub trait LoginRepository: Send + Sync {   // authenticated, not yet promoted
      fn insert(&self, LoginId, Arc<dyn BackendLogin>); fn get(&self, &LoginId) -> Option<Arc<dyn BackendLogin>>;
      fn remove(&self, &LoginId) -> Option<Arc<dyn BackendLogin>>;   // removes its flows too
      fn insert_flow(&self, &LoginId, Arc<dyn VerificationFlow>);
      fn get_flow(&self, &LoginId, &VerificationFlowId) -> Option<Arc<dyn VerificationFlow>>;
  }
  ```
- `AuthService::restore_login` / `finish_interactive_login` now return `Arc<dyn BackendLogin>`.
  `BackendSession` untouched.
- `LoginRepositoryInMem` in `repository.rs` (DashMaps, next to `SessionRepositoryInMem`).

**Done when:** `cargo build -p tachyon-core` (AuthUseCase will be adjusted in subtask 3; stub it to compile).

---

## Subtask 3 — `AuthUseCase`: `LoginOutcome` + promotion invariant

**Files:** `crates/tachyon-core/src/application/auth_use_case.rs`, `src/infrastructure/app_state.rs`.

- API:
  ```rust
  pub enum LoginOutcome { SessionOpened { login_id, session: Arc<dyn BackendSession> }, DeviceVerificationRequired { login_id } }
  restore(&TachyonToken) -> Result<LoginOutcome, AuthError>
  finish_interactive_login(&LoginId, &str) -> Result<LoginOutcome, AuthError>
  abandon_login(&LoginId) -> Result<(), AuthError>     // Ok(()) when nothing pending
  // start_interactive_login, link_token unchanged; RestoredLogin removed
  ```
- `restore` = single promotion point, whole body under a `tokio::sync::Mutex<()>` field
  (also held by the tail of `finish_interactive_login`) so concurrent `USR S` can't restore
  twice onto one sqlite crypto store or promote twice:
  1. in `SessionRepository` → `SessionOpened`;
  2. in `LoginRepository` → `promote_if_verified`;
  3. no `LoginId` for the token → `BackendCredentialsNotInStore` (unchanged);
  4. `auth_service.restore_login` → `promote_if_verified`.
- `promote_if_verified(login_id, login)`: `device_status()`; `Verified` →
  `open_session(DeviceVerified(()))`, `session_repository.insert`, then
  `login_repository.remove` → `SessionOpened`; `Unverified` → `login_repository.insert`
  (idempotent) → `DeviceVerificationRequired`.
- `abandon_login`: remove entry, `login.abandon().await`, drop.
- `AppState` builds `LoginRepositoryInMem`, passes it to `AuthUseCase::new`; signature of
  `AppState::new` unchanged.
- Tests (`#[cfg(test)]`, hand-written `FakeAuthService` / `FakeBackendLogin` with scriptable
  `device_status` and an `open_session` counter): verified restore opens a session; unverified →
  `DeviceVerificationRequired`, `SessionRepository` empty; restore after flip promotes exactly once
  and empties `LoginRepository`; two concurrent restores → one `open_session`; `abandon_login` on
  nothing → `Ok`; `finish_interactive_login` unverified inserts into `LoginRepository`.

**Done when:** `cargo test -p tachyon-core` green.

---

## Subtask 4 — `DeviceVerificationUseCase`

**Files:** new `crates/tachyon-core/src/application/device_verification_use_case.rs`, `application/mod.rs`, `infrastructure/app_state.rs`.

- Keyed by `TachyonToken` (→ `LoginId` via `AccountRepository` → `LoginRepository`):
  ```rust
  status(token) -> DeviceStatus            // Verified if the login is already in SessionRepository (reload after Done)
  options(token) -> VerificationOptions
  recover(token, secret) -> ()             // re-check device_status → StillUnverified
  start_device_verification(token, &DeviceId) -> VerificationFlowId   // stores the flow
  verification_state(token, &VerificationFlowId) -> VerificationFlowState
  verification_action(token, &VerificationFlowId, VerificationAction) -> ()
  reset_identity(token, Option<ResetAuth>) -> IdentityReset   // re-check device_status after
  ```
  all `Result<_, DeviceVerificationError>`; anything but `status` → `LoginNotFound` when not pending.
- `AppState::device_verification_use_case()`.
- Tests with the subtask-3 fakes + `FakeVerificationFlow`: `recover` leaving unverified →
  `StillUnverified`; flow stored and reachable; unknown flow → `FlowNotFound`; `status` after
  promotion → `Verified`.

**Done when:** `cargo test -p tachyon-core` green; `application_does_not_know_infrastructure` green.

---

## Subtask 5 — matrix adapter: `BackendLoginMatrix` & token handover

**Files:** `crates/tachyon-backend-matrix/src/infrastructure/{backend.rs,mappers.rs,mod.rs}`, new `infrastructure/login.rs`, `Cargo.toml` (+ `futures-util`).

- `BackendLoginMatrix { client, tasks_token: std::sync::Mutex<Option<CancellationToken>>, verification_sync: Mutex<Option<CancellationToken>>, reset: Mutex<Option<PendingReset>> }`
  — **no `matrix_client()` accessor**. `std::sync::Mutex` (never held across await).
- Move the `restore_session` + `whoami` block from `BackendSessionMatrix::restore`
  (`backend.rs:49-86`) to `BackendLoginMatrix::restore`; `restore_login` /
  `finish_interactive_login` return `Arc<BackendLoginMatrix>`; the token-refresh watcher
  (`backend.rs:384-425`) is still spawned per login, its token stored in `tasks_token`.
- `device_status`: `wait_for_e2ee_initialization_tasks()`, `request_user_identity(own)`,
  `get_own_device()?.is_cross_signed_by_owner()` (port of `cross_signing::check_device_is_crossed_signed`).
- `open_session`: cancel `verification_sync`, `take()` `tasks_token` (`None` →
  `BackendError::Technical("already promoted")`), `BackendSessionMatrix::new(client.clone(), token)`
  (add `pub(crate) fn new`). `Drop`: cancel `verification_sync`; cancel `tasks_token` if never taken.
- Other `BackendLogin` methods `todo!()`-free stubs returning `Technical` until subtasks 6–7.
- `mappers.rs`: `DeviceId` ↔ `OwnedDeviceId`.
- Tests (wiremock client builder from `backend.rs` tests): promoted → login `Drop` leaves the token
  uncancelled; never promoted → cancelled; second `open_session` errors.

**Done when:** `cargo test -p tachyon-backend-matrix` green.

---

## Subtask 6 — matrix adapter: SAS flow & to-device sync

**Files:** new `crates/tachyon-backend-matrix/src/infrastructure/verification.rs`, `login.rs`.

- Port `build_to_device_only_sliding_sync` / `cross_sign_sync_task` from
  `crates/tachyon/src/matrix/cross_signing.rs:157-233` (drop the duplicate `add_list` at `:199`),
  driven by a child `CancellationToken` stored in `verification_sync`. Started lazily by
  `request_device_verification` **before** `device.request_verification_with_methods([SasV1])`
  (`.ready/.start/.key/.mac` are to-device events). Not needed for recover/reset.
- `VerificationFlowMatrix { request: VerificationRequest, id, driver: JoinHandle<()> }`, `Drop`
  aborts the driver. No stored SAS: `state()`/actions derive from `request.state()`
  (`Transitioned { verification }` → `verification.sas()` → `sas.state()`).
- Driver: act on `request.state()` first, then consume `request.changes()` (streams start after
  subscription). `Ready { their_methods, .. }` → cancel only if SAS-v1 missing; **no
  `accept_with_methods`** (no-op for the initiator, `matrix-sdk-crypto/src/verification/requests.rs:927`;
  legacy `verification/mod.rs:97` was dead). `Transitioned` with SAS → `sas.accept().await`
  immediately (idempotent, `sas/mod.rs:436`; a SAS born from the other side's `.start` is already
  `Started`, so waiting on the stream would hang). `Done`/`Cancelled` → stop.
- Pure mappers `map_request_state` / `map_sas_state`: `Created|Requested → Requested`,
  `Ready → Ready`, SAS `Created|Started|Accepted → Started`, `KeysExchanged → CompareEmojis`,
  `Confirmed → AwaitingOtherConfirmation`, `Done`, `Cancelled { reason: cancel_info.reason() }`,
  `Transitioned` without SAS → `Cancelled`.
- `confirm/mismatch` → SAS; `cancel` → request.
- Tests: mapper fns on SDK-constructible variants (`Created`, `Done`, `Cancelled`, SAS `Confirmed`).

**Done when:** `cargo test -p tachyon-backend-matrix` green.

---

## Subtask 7 — matrix adapter: options, recovery, identity reset

**Files:** `crates/tachyon-backend-matrix/src/infrastructure/login.rs`.

- `verification_options`: `secret_storage().is_enabled()` + `get_user_devices(own)` filtered as
  `other_device.rs:29-33` (cross-signed, curve25519 key, not dehydrated) → `DeviceSummary`.
- `recover(secret)`: `recovery().recover(secret)` (HTTP only; `import_secrets` self-signs,
  `secret_store.rs:426-441`).
- `reset_identity(auth)`: `recovery().reset_identity()`; handle `Uiaa` → needs
  `ResetAuth::Password` else `PasswordRequired`; `OAuth` → first call returns
  `ApprovalRequired { url }` and **spawns** `handle.reset(None)` (tight HTTP retry loop,
  `encryption/mod.rs:297-319` — never inside a request handler), keeping handle + `JoinHandle` in
  `reset`; `ResetAuth::Approved` → `ApprovalPending` while the task runs. After reset:
  `recovery().enable()` → `IdentityReset { recovery_key }`.
- `abandon`: `handle.cancel()` on a pending reset.

**Done when:** `cargo test -p tachyon-backend-matrix` green; manual check of recover against the MAS homeserver.

---

## Subtask 8 — legacy: `GlobalState`, USR handler, OAuth callback

**Files:** `crates/tachyon/src/tachyon/global_state.rs`, `notification/handlers/auth.rs`, `web/tachyon/matrix_auth.rs`, `web/tachyon/middleware.rs`.

- `global_state.rs`: delete `PreSession`, `pre_sessions`, `pending_verification_requests`,
  `insert_pre_session/remove_pre_session/confirmation_client/has_confirmation_alert/take_confirmation_alert`,
  the `remove_for` call in `ClientDropGuard`; add `pending_verifications: DashMap<String /*ticket*/, Alert>`
  with `store_/has_/take_pending_verification`. `is_session_token` = `TachyonClient` **or** pending verification.
- `auth.rs`: `authenticate` matches `LoginOutcome`; on `DeviceVerificationRequired { login_id }`
  a new `device_verification(..)` step: **first** `store_pending_verification(ticket, alert)`,
  **then** send the `NOT` → `/tachyon/confirm_device?t=<ticket>` (no `notification_id`), then
  `timeout_at(deadline, receiver.recv())`; on timeout/failure `take_pending_verification` +
  `abandon_login(&login_id)`; on success `restore(&token)` again, require `SessionOpened`.
  `interactive_login` returns its trailing `restore`'s `LoginOutcome`; its `abandon` closure =
  `take_pending_login` + `take_pending_verification` + `abandon_login`. `sync_with_server_task`
  loses lines 294-363 and the `deadline` param. `USR OK` still first; `CLIENT_SIGN_IN_WINDOW` unchanged.
- `matrix_auth.rs::get_login_callback`: `LoginOutcome`; `link_token` failure →
  `abandon_login(&pending.login_id)`; `SessionOpened` → fire alert + success page;
  `DeviceVerificationRequired` → `store_pending_verification(ticket, pending.alert)` + redirect to
  `/tachyon/confirm_device?t=<ticket>`. Delete downcast + cross-signing import.
- Web pages may not compile yet — stub them if needed; they are subtask 9.

**Done when:** `cargo build -p tachyon` (pages stubbed) and the existing `command_handler.rs` tests pass.

---

## Subtask 9 — legacy: web pages on the use case + deletions

**Files:** `crates/tachyon/src/web/tachyon/{mod.rs,confirm_device/*.rs,verification/*.rs}`, `assets/web/tachyon/verify.js`, `crates/tachyon/src/matrix/{mod.rs,cross_signing.rs,verification_request_repository.rs}`.

- All pages call `state.app_state().device_verification_use_case()` with
  `TachyonToken::new(&token)` from the request extension; replace `unwrap/assert!/panic!` with the
  `error_page` style of `matrix_auth.rs:196`.
  - `get_confirm` → `options`; recovery card only if `recovery_available`, device card only if
    `devices` non-empty; if `status` is `Verified` fire the alert + success page.
  - `post_recover` → `recover`; success fires the alert.
  - `other_device` → `options` / `start_device_verification` → `X-IC-Redirect` to
    `/tachyon/verification?verification=<id>`.
  - `get_verification_poll` → `verification_state`; 204 when `state.name()` == `state=` param;
    fold `sas_v1.rs` rendering in (`CompareEmojis` → `emoji_table`, description-based images);
    `Done` fires the alert, `Cancelled` fails it; drop `user_id`; delete `sas_v1.rs`.
  - `/verification/{action}` (`confirm|mismatch|cancel`) → `verification_action`; drop `accept`.
  - `reset_identity` → `reset_identity(token, Some(Password))`; `ApprovalRequired { url }` →
    link + "I've approved, continue" form posting `auth=approved`; `ApprovalPending` re-renders it.
  - `verify.js:5,93`: `restore-method` → `restore_method` (matches `recover.rs:43`).
- Delete `cross_signing.rs`, `verification_request_repository.rs` and their `pub mod` lines.
  `matrix/handlers/request_verification_handlers.rs` (in-session inbound, auto-confirm stub) is out of scope.

**Done when:** `cargo test --workspace` green;
`grep -rn matrix_sdk crates/tachyon/src/web/tachyon` → no hits in `confirm_device/`, `verification/`, `matrix_auth.rs`;
`grep -rn "as_any\|BackendSessionMatrix" crates/tachyon/src` → only the `TachyonClient::new` scaffold in `auth.rs`.

---

## Subtask 10 — docs

**Files:** `docs/architecture/tachyon-ports-adapters.md`, `CONTEXT.md`.

- Replace the §"The interactive login flow" diagram with the one above (keep `Failed`); add
  `BackendLogin`, `VerificationFlow`, `LoginRepository` to §"Ports"; `finish(...)` /
  `restore(CredentialBlob)` return `BackendLogin` there and in correction 8; remove "verification,
  recovery" from the `BackendSession` direct-call list (§"Two-frontend call model"); add a
  "Device verification" paragraph under §"Auth and credentials" (use case, promotion invariant,
  `DeviceVerified` witness, to-device sync only during SAS); startup step 5 → "gets a
  `BackendLogin`, promotes it once verified".
- `CONTEXT.md`: update `Login flow` states; add `BackendLogin`, `LoginRepository`,
  `DeviceVerificationUseCase`, `VerificationFlow`.

**Done when:** doc and glossary names match the code (`CONTEXT.md` rule).

---

## End-to-end verification (after subtask 9)

Patched WLM client against the MAS homeserver (`/run` skill), watching the log:
1. Fresh instance: sign in → login `NOT` → OAuth → callback lands on confirm-device → recover
   with the recovery key → confirmation page → client leaves the sign-in screen (`SBS`). Log:
   `DeviceVerificationRequired` … `SessionOpened`, nothing session-related before.
2. Restart with `store_root` intact (credentials in-memory ⇒ OAuth re-runs, crypto store already
   verified) → callback goes straight to the success page, no verification alert.
3. Wipe the crypto store → verify with another device: emojis match Element; "They match" →
   `Done` → client signs in. Cancel from Element → "cancelled" page, client refused.
4. Let the verification alert time out (~5 min) → client refused, login abandoned, no further
   to-device `/sync` lines, a fresh sign-in starts cleanly.
