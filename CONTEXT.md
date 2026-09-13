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

- **Login flow** is core's state machine over one live login: `Pending` at
  `Step::Authenticate`, then `Pending` at `Step::VerifyDevice`, then `Ready`. Backends
  implement steps, never the choreography. A step that fails drops the login.
- **Login** is the live login `Logins` holds under a `TachyonToken`: `Pending { attempt,
  session, login_id, step, changed }` or `Ready { attempt, session, login_id }`. In memory
  only; the store keeps what rebuilds one after a restart.
- **Step** is what the user still has to do in a browser before a pending login can be
  used: `Authenticate { flow_id, prompt }` or `VerifyDevice`.
- **Attempt** is one occupancy of a token's slot, minted by `Logins` for every login that
  goes in and kept while it advances. `SignIn` hands it to the bridge, and `abandon` and
  the guarded `Logins::replace_if` / `remove_if` act only while the slot still holds it,
  so a caller that gave up on a login cannot take down one that replaced it.
- **SignIn** is what `AuthUseCase::sign_in` hands back: `Ready { session, attempt }` or
  `Pending { step, url, attempt }`, where `url` is the page the browser must visit.
  `AuthUseCase::wait_for_session` resolves once a pending login is usable.
- **settle** is `AuthUseCase::settle_authenticated`, run under the login lifecycle mutex
  once `BackendSession::authenticate` succeeded. It binds the token to the login, reads
  `device_status`, and advances the login. A login still ours that cannot be settled is
  closed and keeps its row for the next sign-in; one that lost its slot meanwhile is
  logged out and discarded, because nothing points at the device it just made.
- **DeviceVerificationUseCase** is core's owner of the verification step, keyed by
  `TachyonToken`: device status, recovery-key import, SAS verification against another
  device, identity reset. It works on logins a bridge cannot reach yet and never advances
  one: the session reports the device as verified and `AuthUseCase::wait_for_session`
  picks that up.
- **VerificationFlowState** is the state of the one live SAS flow on a session
  (`Requested`, `Ready`, `Started`, `CompareEmojis`, `AwaitingOtherConfirmation`, `Done`,
  `Cancelled`). `VerificationFlowState::name()` is the payload-free key a poll endpoint
  compares against the state the page already shows.
- **InteractiveAuthStarted** is what `AuthService::start_interactive_login` returns beside
  the session. For Matrix it is `OAuth { auth_url, csrf_token }`, the MAS authorization
  URL the user's browser must visit.
- **callback query** is the raw query string the redirect endpoint receives from the
  authorization server. Core never parses it. It goes straight to
  `BackendSession::authenticate` as `Credential::OAuthCallback`, and the adapter
  interprets it (OAuth `code` and `state` for Matrix). `Credential::Password` is the
  other credential, for homeservers without OAuth.
- **CredentialBlob** — backend-serialized credentials (`AuthSession` for Matrix) as
  opaque bytes, keyed by `LoginId` in the core-owned store (`tachyon-store-sqlite`).
  Plaintext today; the store schema reserves a format column for encryption at rest.

## Backend seam

- **BackendSession** is the deep port one login owns for its whole lifetime, from the
  first authorization redirect to the last message sent: messaging, typing, presence,
  media, conversation ops, event stream, plus `device_status` and the verification calls.
  `Logins` holds it inside the `Login`, and `AuthUseCase::session` is the only accessor a
  bridge may use: it answers only for a `Ready` login. Two adapters:
  `tachyon-backend-matrix` (prod) and `FakeBackendSession` (testkit).
- **AuthService** is the factory for `BackendSession`. `restore(login_id)` rebuilds one
  from stored credentials. `start_interactive_login(login_id, server_name, user_id,
  redirect_url, bridge_metadata)` builds a fresh one with an `InteractiveAuthStarted`
  prompt. Neither call advances a login.
- **BackendEvent** — push events crossing the seam backend → core → bridge (messages,
  membership, `CredentialsRotated`), over a lossless mpsc (one frontend per instance).
- **Dialect** — an MSNP protocol version spoken by a client (18 today; 15 next;
  21/24 = Skype desktop in Messenger mode). A trait in `tachyon-bridge-msn`.

## Deployment vocabulary

- **Loopback bridge** — Tachyon runs beside one patched MSN client; one client per
  instance. TLS, MPOP multiplexing, and multi-user isolation are out of scope.
- **Faked MPOP** — only the endpoint GUID from `USR` is real; others derive from the
  UserId; one endpoint must still be advertised in `JOI`.
