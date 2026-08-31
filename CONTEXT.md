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

- **Login flow** — core's state machine: `Started → AwaitingUser → ProofReceived →
  SessionOpened | Failed`. Backends implement steps, never the choreography.
- **AuthPrompt** — what the backend returns from `begin`: the URL the user's browser
  must visit. For Matrix this is the MAS OAuth authorization URL.
- **CallbackProof** — the raw payload the user's browser brings back to the bridge-web
  callback endpoint. Opaque to core; the backend interprets it (OAuth `code`+`state`
  for Matrix).
- **CredentialBlob** — backend-serialized credentials (`AuthSession` for Matrix) as
  opaque bytes, encrypted at rest by the core-owned store, keyed by `LoginId`.

## Backend seam

- **BackendSession** — the deep port a live backend connection satisfies: messaging,
  typing, presence, media, conversation ops, event stream. Two adapters:
  `tachyon-backend-matrix` (prod) and `FakeBackend` (testkit).
- **BackendEvent** — push events crossing the seam backend → core → bridge (messages,
  membership, `CredentialsRotated`), over a lossless mpsc (one frontend per instance).
- **Dialect** — an MSNP protocol version spoken by a client (18 today; 15 next;
  21/24 = Skype desktop in Messenger mode). A trait in `tachyon-bridge-msn`.

## Deployment vocabulary

- **Loopback bridge** — Tachyon runs beside one patched MSN client; one client per
  instance. TLS, MPOP multiplexing, and multi-user isolation are out of scope.
- **Faked MPOP** — only the endpoint GUID from `USR` is real; others derive from the
  UserId; one endpoint must still be advertised in `JOI`.
