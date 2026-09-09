# Tachyon — Domain Glossary

Terms used across code, docs, and reviews. If a name here and a name in code disagree,
one of them is wrong — fix it or fix this file.

## Identity & naming

- **LoginId** — Tachyon's own stable identifier for one authenticated account. Opaque
  UUID; survives token rotation. The key for sessions and credentials.
- **TachyonToken** (a.k.a. **ticket**) — the opaque, random, *expiring* value the MSN
  client holds and echoes back (RST2, cookies, `USR`). Maps to a `LoginId` server-side.
  It is **not** a backend credential and is never derived from one.
- **ConversationId** — core's opaque identifier for a conversation. Core never sees
  Matrix room ids or MSN addresses.
- **ContactNameTable** — module in `tachyon-bridge-msn`, one instance per `MsnpSession`
  (per-session lifetime, never process-wide), holding the `sha1(room_id)@server` ↔
  conversation bijection and MSN address rules (64-char fallback, inbound
  `@user:server ↔ user@server` resolution).

## Auth flow (shape owned by core)

- **Login flow** is core's state machine over one login's `Readiness`:
  `AuthNeeded → VerificationNeeded → Ready`, plus `Failed` when a step errors and the
  login is dropped. Backends implement steps, never the choreography.
- **Readiness** is how far one login has come: `AuthNeeded`, `VerificationNeeded`, or
  `Ready`. A `Copy` enum in `domain::auth`, held in `SessionRepository` next to the
  session, and forward-only through `Readiness::can_advance_to`.
- **LoginOutcome** is what `AuthUseCase::restore` and `AuthUseCase::finish_interactive_login`
  hand back: `SessionOpened { login_id, session }` or
  `DeviceVerificationRequired { login_id }`.
- **settle** is `AuthUseCase::settle`, the only writer of `Readiness`. It reads
  `BackendSession::device_status()` under the login lifecycle mutex, sets the readiness,
  and returns the matching `LoginOutcome`.
- **DeviceVerificationUseCase** is core's owner of the verification step, keyed by
  `TachyonToken`: device status, recovery-key import, SAS verification against another
  device, identity reset. It works on logins a bridge cannot reach yet and never writes
  `Readiness`.
- **VerificationFlowState** is the state of the one live SAS flow on a session
  (`Requested`, `Ready`, `Started`, `CompareEmojis`, `AwaitingOtherConfirmation`, `Done`,
  `Cancelled`). `VerificationFlowState::name()` is the payload-free key a poll endpoint
  compares against the state the page already shows.
- **InteractiveAuthStarted** is what `AuthService::start_interactive_login` returns beside
  the session. For Matrix it is `OAuth { auth_url, csrf_token }`, the MAS authorization
  URL the user's browser must visit.
- **callback query** is the raw query string the redirect endpoint receives from the
  authorization server. Core never parses it. It goes straight to
  `BackendSession::finish_interactive_login`, and the adapter interprets it (OAuth `code`
  and `state` for Matrix).
- **CredentialBlob** — backend-serialized credentials (`AuthSession` for Matrix) as
  opaque bytes, keyed by `LoginId` in the core-owned store (`tachyon-store-sqlite`).
  Plaintext today; the store schema reserves a format column for encryption at rest.

## Backend seam

- **BackendSession** is the deep port one login owns for its whole lifetime, from the
  first authorization redirect to the last message sent: messaging, typing, presence,
  media, conversation ops, event stream, plus `device_status` and the verification calls.
  `SessionRepository` tracks its `Readiness`, and `SessionRepository::get_ready` is the
  only accessor a bridge may use. Two adapters: `tachyon-backend-matrix` (prod) and
  `FakeBackend` (testkit).
- **AuthService** is the factory for `BackendSession`. `restore(login_id)` rebuilds one
  from stored credentials. `start_interactive_login(login_id, server_name, user_id,
  redirect_url, bridge_metadata)` builds a fresh one with an `InteractiveAuthStarted`
  prompt. Neither call sets `Readiness`.
- **BackendEvent** — push events crossing the seam backend → core → bridge (messages,
  membership, `CredentialsRotated`), over a lossless mpsc (one frontend per instance).
- **Dialect** — an MSNP protocol version spoken by a client (18 today; 15 next;
  21/24 = Skype desktop in Messenger mode). A trait in `tachyon-bridge-msn`.

## Deployment vocabulary

- **Loopback bridge** — Tachyon runs beside one patched MSN client; one client per
  instance. TLS, MPOP multiplexing, and multi-user isolation are out of scope.
- **Faked MPOP** — only the endpoint GUID from `USR` is real; others derive from the
  UserId; one endpoint must still be advertised in `JOI`.
