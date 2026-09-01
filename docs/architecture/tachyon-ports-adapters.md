# Tachyon — Ports-and-Adapters Architecture

Status: **accepted** (design accepted 2026-08-23; reconstructed into the repo 2026-08-31 after the original doc was lost uncommitted).
Branch: `refactor/third-times-the-charm`.

Tachyon is an MSNP18 ↔ Matrix bridge. This document records the target architecture, the
vocabulary used across the workspace, and the migration plan from the legacy `crates/tachyon`
binary (~11k LOC) to the new crate layout.

## Deployment model (constraints, not choices)

- Tachyon is a **client-side loopback bridge**: it runs on the same machine as one patched
  MSN client (TLS disabled, loopback-targeted). One client per bridge instance.
- TLS-on-the-wire, multi-user isolation, and per-account session multiplexing are **out of
  scope**. The real cardinality is one core `Session` per account. Each account has two
  frontends: the MSNP bridge (event-stream consumer) and the web bridge (request/response
  consumer of the same `BackendSession`). Backend→bridge events ride a **lossless mpsc
  channel** to the MSNP frontend only — the web bridge does not subscribe to events, so no
  broadcast is needed.
- MPOP is faked: the only real endpoint GUID is the one the client sends in `USR`; others are
  derived from the UserId. An endpoint must still be advertised in `JOI` or WLM falls back to
  P2Pv1.
- All roster contacts are spelled `sha1(room_id)@server` (DMs included); the
  `@user:server ↔ user@server` bijection exists only for inbound resolution
  (ABContactAdd, UUM).

## Crate layout

```
                 ┌────────────────────┐
                 │   tachyon (bin)    │  composition root only
                 └─────┬──────┬───────┘
        ┌──────────────┤      ├────────────────┐
        ▼              ▼      ▼                ▼
┌──────────────┐ ┌───────────────┐ ┌────────────────────┐
│ tachyon-     │ │ tachyon-      │ │ tachyon-backend-   │
│ bridge-msn   │ │ bridge-web    │ │ matrix             │
│ (NS/SB/SOAP/ │ │ (login/verif. │ │ (matrix-sdk        │
│  P2P +       │ │  UI)          │ │  adapter)          │
│  dialects)   │ │               │ │                    │
└──────┬───────┘ └──────┬────────┘ └─────────┬──────────┘
       │                │                    │
       │   ┌────────────▼────────────┐       │
       └──►│       tachyon-core      │◄──────┘
           │ domain + use cases +    │
           │ Session actor + ports   │
           └────────────┬────────────┘
                        │ ports implemented by
           ┌────────────▼────────────┐   ┌─────────────────┐
           │  tachyon-store-sqlite   │   │ tachyon-testkit │
           │ (SQLite: accounts,      │   │ (FakeBackend,   │
           │  credentials, tickets)  │   │  fixtures)      │
           └─────────────────────────┘   └─────────────────┘

           msnp: wire-format library only (no deps on the above)
```

| Crate | Role | Allowed dependencies |
|---|---|---|
| `tachyon-core` | Protocol-neutral domain, use-case services, `Session` actor, all ports | std/tokio only — **no `matrix-sdk`, no `msnp`** |
| `tachyon-backend-matrix` | `ChatBackend`/`BackendSession` adapter over matrix-rust-sdk | `tachyon-core`, `matrix-sdk` |
| `tachyon-bridge-msn` | NS/SB TCP servers, SOAP, P2P, `MsnpSession` actor, one `Dialect` per MSNP version | `tachyon-core`, `msnp` |
| `tachyon-bridge-web` | Login / device-verification / recovery UI (axum); request/response consumer of the account's `BackendSession` | `tachyon-core` |
| `tachyon-store-sqlite` | SQLite (rusqlite) implementation of the store ports | `tachyon-core` |
| `tachyon-testkit` | `FakeBackend`, in-memory repository doubles, fixtures — the second adapter that makes every seam real | `tachyon-core` |
| `tachyon` (bin) | Composition root: config, wiring, startup/shutdown | everything |
| `msnp` | MSNP wire formats, parsing, serialization | none of the above |

All workspace members belong in `default-members` so architecture tests run on every build.

## Layering inside `tachyon-core`

```
domain          pure types: ids, models, events, errors. Sync. No ports, no async,
                no tokio, no async_trait. Never imports application/.
application     use-case services + ALL ports (traits). Depends on domain only.
infrastructure  production in-memory adapters (sessions), wiring helpers. Depends on both.
```

The dependency arrow points **down only**: `infrastructure → application → domain`.
`tests/architecture.rs` enforces this textually and must stay green:

- `domain_is_pure` — forbids `tokio::`, `async fn`, `async_trait`, `crate::application`
  in `src/domain`.
- ports-location test — every port trait lives in `application/ports.rs` (there is no
  `src/port` directory; the test must scan the real one).

Consequence for the current branch: `domain/backend_ports.rs::AuthService` is a port and
moves to `application/ports.rs`.

## What is backend-agnostic — and therefore lives in core

The centre of gravity of this design is **`tachyon-core`'s use-case layer**, not the backend
adapter. Anything that would read the same if Matrix were swapped for another backend is
core logic; the backend port stays thin and protocol-specific. Concretely:

### Configuration

- `TachyonConfig` is backend-agnostic and owned by core: NS/SB/web ports, web base URL,
  ticket lifetime, secret-key path, log level.
- The backend adapter receives one **opaque config section** (a raw TOML/INI table or
  string) that it interprets itself: homeserver URL override, disable-SSL, store path,
  sync knobs. Core never learns backend vocabulary; adapters never read the config file.

### The interactive login flow

Core owns the **flow shape** as a state machine; a backend implements the steps, not the
choreography:

```
Started ──► AwaitingUser ──► ProofReceived ──► SessionOpened
   │              │                │                 │
   │   core built the callback     │        core persists blob,
   │   URL from ITS config and     │        issues/updates ticket
   │   stored the pending login    │
   └── core issued LoginId         └─ proof is OPAQUE to core
                                      (raw return-URL); the
                                      adapter interprets it
```

- Core issues `LoginId`, builds the redirect/callback URL from its own config (this
  resolves the `//TODO build that with config & url builder service` in
  `auth_use_case.rs`), tracks pending logins, and enforces their expiry.
- **Pending logins are in-memory only** — a crash mid-flow loses them and the user
  simply restarts the login; only completed logins (`CredentialBlob` + ticket) are
  persisted. Cheap to redo on a single-user loopback bridge, so no recovery machinery.
- The adapter fills exactly two slots:
  `begin(login_id, redirect_url, user_hint) -> AuthPrompt { url }` and
  `finish(pending, CallbackProof) -> BackendSession`.
  For Matrix, `begin` is OAuth client registration + authorization URL, and
  `CallbackProof` decodes to `code`+`state`; a future password-style backend fills the
  same slots without core changing.

### Tickets and credentials

- Ticket issuance (`TachyonToken`), expiry, and the ticket → `LoginId` mapping are core
  logic backed by `tachyon-store-sqlite`.
- Credentials persist as an **opaque `CredentialBlob`** keyed by `LoginId` in
  the core-owned store — plaintext today; the schema's `credentials_format` column
  reserves encryption at rest with the local key. The adapter's only job is
  `SessionRestoreData::to_blob()` / `from_blob()` (a versioned JSON envelope).
  Interim until the phase-3 event stream exists: the adapter holds the core
  `CredentialRepository` port and re-persists the whole blob on `TokensRefreshed`; the
  target remains `BackendEvent::CredentialsRotated { new_blob }` with **core**
  re-persisting. (`CredentialsRepository` and its matrix-sdk-typed rows are gone from
  `tachyon-backend-matrix`.)

## Interface style between bounded contexts

**Use-case request/response with exported DTO structs.** Examples:

```rust
AuthUseCase::restore(TachyonToken) -> Result<LoginId, AuthError>
AuthUseCase::start_interactive_login(server, user, BridgeMetadata) -> Result<LoginStart, AuthError>
AuthUseCase::finish_interactive_login(FinishLogin) -> Result<SessionOpened, AuthError>
```

- Async only where there is I/O.
- **No command-enum-with-reply-channels at context seams.** The
  `TachyonEvent::Bridge*` request variants + `EventSender` + `plumbing_event_listener`
  shape is rejected: bridges hold `Arc<AuthUseCase>` (and future use cases) and call
  methods. Deleting the bus must not remove any capability — that is the test.
- The **actor pattern is reserved for the live-session module**: a per-account `Session`
  actor behind a cheap-`Clone` `SessionHandle` with methods; its `SessionCommand` enum is
  private. Stateless reads call the backend directly.
- Events flow **one direction only**: backend → core → bridge as `BackendEvent` /
  domain events over lossless mpsc.

### Type mapping at the seams

Each adapter owns the mappers for its own vocabulary, symmetrically:
`tachyon-backend-matrix` maps ruma/matrix-sdk types ↔ core ids/DTOs (its
`mappers.rs`), and **`tachyon-bridge-msn` maps core DTOs/events ↔ `msnp` wire types**.
Core types never appear in the `msnp` crate, wire types never cross into core, and no
mapper lives outside the adapter whose vocabulary it translates.

### Error strategy

Typed errors at every seam, composed with `thiserror`:

- `domain` has pure error enums (`TachyonError`), no I/O variants.
- Every port defines its own error enum (`BackendError`, `StoreError`); use cases
  compose them into use-case errors (`AuthError`) via `#[from]`.
- `anyhow` is confined to **adapter internals** and surfaces at the seam only as an
  opaque technical variant (`BackendError::Technical`). No `anyhow::Result` in any
  port or use-case signature.

## Ports

All ports live in `tachyon-core/src/application/ports.rs`.

- **`BackendSession`** — the deep port at the centre of the design. Its interface is derived
  from what the MSNP handlers actually consume, roughly:
  send message / typing / presence, media fetch, conversation ops (create DM, join,
  invite, members), profile/avatar, finish-login, and an event stream. Two adapters make
  the seam real: `tachyon-backend-matrix` in production, `FakeBackend` in tests.
  It must not stay a marker trait — an empty trait forces downcasting later.
- **`AuthService`** — the thin, protocol-specific slice of the login flow (core owns the
  flow shape — see "What is backend-agnostic"): `begin_interactive_login(login_id,
  redirect_url, user_hint) -> AuthPrompt`, `finish_interactive_login(pending,
  CallbackProof) -> BackendSession`, `restore(CredentialBlob) -> BackendSession`.
- **`AccountRepository` / `CredentialRepository`** — ticket → `LoginId` mapping and
  `LoginId` → `CredentialBlob` persistence; implemented by `tachyon-store-sqlite`
  (in-memory doubles in `tachyon-testkit`).
- **`SessionRepository`** — live sessions by `LoginId`.

## Auth and credentials

The invariant: **the Matrix token never reaches the client or IDCRL.**

- Matrix is OAuth-era (MAS): access + refresh tokens persist as the opaque
  `CredentialBlob` (serialized matrix-sdk `AuthSession`) in the **core-owned** store —
  the adapter serializes (see "Tickets and credentials" above).
- MSN-side tickets are **opaque, random, expiring** (`TachyonToken`), mapped
  ticket → `LoginId` → credentials. The ticket is *not* the encrypted access token and is
  *not* the repository primary key — the legacy `TicketToken` conflated all three roles and
  leaked onto seven wires (RST2 body, MSG profile, NOT URLs, XFR, RNG blob, cookie, query
  strings).
- Token rotation: the Matrix adapter subscribes to session changes and re-persists on
  `TokensRefreshed` (surfaced as `BackendEvent::CredentialsRotated`); tickets are unaffected
  by rotation. This permanently fixes the legacy bug where restore hard-coded
  `refresh_token: None` and rotated tokens were lost.
- Optional (open): a Tachyon-local "Messenger password" (argon2) to authenticate RST2.

## Sessions and lifecycle

- Per-account `Session` actor + cheap `Clone` handles replace `GlobalState`,
  `TachyonClient`, `LocalClientData`, `LocalSwitchboardData`, and the `lazy_static`
  globals.
- Lifecycle uses `CancellationToken` (tokio-util) everywhere — no broadcast kill channels
  (`resubscribe()` races miss early signals; `Drop`-guard teardown is fragile).
- State partitions of the legacy `TachyonClient` god object:
  - backend handle → `BackendSession` (behind the port),
  - MSNP session state (contact list, own_user, switchboards, SOAP projection) →
    `tachyon-bridge-msn`,
  - P2P transports/sessions/voice clips → `tachyon-bridge-msn`,
  - lifecycle → core `Session`.

### Two-frontend call model

Each account's `BackendSession` is shared by both frontends via `Arc<dyn BackendSession>`.
The split is:

- **Direct calls** (no actor hop): stateless operations — send message, typing, presence,
  media fetch, conversation reads, profile/avatar, verification, recovery. Both the MSNP
  bridge and the web bridge call `BackendSession` methods directly.
- **Actor-mediated calls** (via `SessionHandle`): stateful coordination — lifecycle
  (start/stop/close), file-transfer exclusivity, credential rotation re-persist. Only the
  `Session` actor owns the `CancellationToken` tree and serializes these operations.
- **Event stream** (one-directional): `BackendSession` → mpsc → `Session` actor → mpsc →
  MSNP bridge. The web bridge does not consume the event stream.

### Startup & shutdown

Startup, in order (all in the `tachyon` bin's composition root):

1. Parse `TachyonConfig`; hand the opaque backend section to the adapter's constructor.
2. Open `tachyon-store-sqlite` (accounts, tickets, blobs); build core use cases with the store
   and backend ports.
3. Create the **root `CancellationToken`**; wire Ctrl-C to `root.cancel()`.
4. Spawn the frontends with `root.child_token()` each: NS + SB listeners
   (`tachyon-bridge-msn`) and the web server (`tachyon-bridge-web`).
5. No sessions exist yet. A `Session` actor is spawned lazily by
   `AuthUseCase::restore` (ticket redeemed over `USR`) or by a completed interactive
   login: core calls `AuthService::restore(blob)` / `finish(...)`, gets a
   `BackendSession`, then spawns the actor with `session_token = root.child_token()`;
   backend tasks (sync loop, token-refresh watcher) run under
   `session_token.child_token()`.
   The web bridge receives its `Arc<dyn BackendSession>` from the spawned actor (or from
   `AuthUseCase::finish_interactive_login` during the initial login flow) and uses it for
   request/response operations only.

Shutdown is cancellation cascading down that tree: `root.cancel()` → listeners stop
accepting → each `Session` actor observes its token, tells its `BackendSession` to stop
(sync loop and watchers exit via their child tokens), flushes any pending
`CredentialBlob` write, drops its MSNP connections, and completes. The bin `join!`s the
frontends and the store closes last. Killing one session (`Session::close()`, logout,
`UnknownToken`) cancels only that session's token — nothing above it.

### Concurrency

One `Session` actor per account; its private `SessionCommand` mpsc is **small and
bounded** (≈64) — callers `await` the send, so a stuck actor backpressures instead of
ballooning. Backend → bridge events ride one bounded-but-lossless mpsc per session
(≈256; the sender awaits, never drops — only the MSNP frontend subscribes, so no
broadcast anywhere). Socket writer tasks get small bounded channels too (the legacy 10,000,000-cap
switchboard channel is exactly what this forbids). Shared maps (`DashMap`) are allowed
only *inside* repositories and stores — never as a channel substitute, and no
`lazy_static` mutable state anywhere.

## Identity mapping

- Core speaks opaque `ConversationId`; it never sees MSN addresses or Matrix room ids.
- The **`ContactNameTable`** is a module in **`tachyon-bridge-msn`**, one instance per
  `MsnpSession` (per-session lifetime, not process-wide, not in core). It holds the
  `sha1(room_id)@server` bijection, the 64-char MSN address fallback rules, and inbound
  resolution. The legacy `lazy_static` DashMaps (`ROOM_HASH_TABLE`, `ROOM_HASH_CACHE`)
  are deleted.

## Dialects

- "Skype" means Skype desktop clients in Messenger mode speaking MSNP21/24 → handled as
  **dialects** of the MSN bridge (a `Dialect` trait per MSNP version); it is not the P2P
  Skype protocol. MSNP15 is the next dialect after 18; nothing else is planned.

## Migration plan

Six phases, strangler-style; the legacy binary keeps working throughout.

| Phase | Content | Status (2026-08-31) |
|---|---|---|
| 0 | Scaffold workspace crates | done (crates exist; `default-members` still to fix) |
| 1 | Core types, use cases, ports, store | store done 2026-09-01 (`tachyon-store-sqlite`); use-case corrections below |
| 2 | Matrix adapter (`tachyon-backend-matrix`) | started (auth/restore/token-refresh done; credentials persist as blobs through the core port) |
| 3 | Core `Session` actor | not started |
| 4 | MSNP bridge extraction (`tachyon-bridge-msn`) | not started |
| 5 | Dialects | not started |
| 6 | Web UI (`tachyon-bridge-web`) | not started |

### Immediate corrections on the current branch (from the 2026-08-30 review)

Ordered; 1–3 are prerequisites for shaping the port in 4.

- [ ] 1. **Add all workspace crates to `default-members`** — prerequisite for the
  architecture tests (and everything else in the new crates) to compile and run on a
  plain `cargo build` / `cargo test`.
- [ ] 2. **Un-invert the layering** — move `AuthService` from `domain/backend_ports.rs`
  to `application/ports.rs`; fix `tests/architecture.rs` paths so both tests scan real
  directories and pass.
- [ ] 3. **Delete the plumbing event bus** — remove `TachyonEvent` request variants,
  `EventSender`, `plumbing_event_listener`; bridges call use cases directly.
- [ ] 4. **Deepen `BackendSession`** — derive the interface from the legacy handlers'
  real consumption; implement in the Matrix adapter; ship `FakeBackend` in the testkit.
- [ ] 5. **`ContactNameTable`** — replace the identity-mapping globals with the
  per-`MsnpSession` module in `tachyon-bridge-msn`.
- [ ] 6. **Split `TachyonClient`** — first replace the 15+ external `.inner.*` field
  pokes with methods, then partition per "Sessions and lifecycle" above.
- [x] 7. **Break the ticket = token identity** — done 2026-09-01: the `TachyonToken →
   LoginId → CredentialBlob` flow persists end-to-end in `tachyon-store-sqlite`. Tickets
   are still derived rather than issued; expiry is pending the ticket-issuance use case.
- [ ] 8. **Pull the flow back into core** — partially done 2026-09-01: credentials are
  opaque blobs behind core's `CredentialRepository` and the matrix-typed rows are gone.
  Remaining: the `begin`/`finish`/`restore(CredentialBlob)` reshape — core builds URLs
  from config, tracks pending logins, persists blobs; the adapter only
  serializes/interprets (today it still holds the store port and persists directly).

## Legacy defects the target design retires

For the record (found in the 2026-08-30 review of `crates/tachyon`):

- NS accept loop `await`s each spawned handler — serves one client at a time
  (`notification_server.rs`); the SB server spawns correctly.
- Two divergent `ConnectionPhase` enums; duplicate `msn_user_resolver` modules; dead
  `EventDeduplicator`, dead `AlertRepository`; `mockall`/`wiremock` declared but unused.
- Switchboard writer channel of capacity 10,000,000; `unwrap()` on socket writes.
- Ticket-token cookie without `HttpOnly`/`Secure`; token in query strings; full
  request/response bodies (including tokens) logged at debug level.
- 13 test functions for 10,876 LOC; everything touching `matrix_sdk::Client` requires a
  mock homeserver because only `MatrixLoginService` was a trait.
