# Tachyon Architecture Overhaul

**Status:** proposal — no code changed by this document; **implementation not started** (Phase 0 awaits approval)
**Basis:** full audit of `crates/msnp`, `crates/tachyon`, `crates/tachyon-sqlite-store` (see appendix A for file mapping)

**Scope (confirmed):** MSNP18 is the **only** protocol version implemented by this overhaul. Supporting other MSN client generations is **out of scope** — the architecture must simply *accommodate* them, so each future version lands as a profile + one dialect module + one `Dialect` variant + fixtures, with **zero changes to the domain** (§6.4, appendix C). Legacy SOAP/web endpoints carry no version-sensitive state (thin pass-throughs, §8.4). No protocol captures exist for other generations, and none are needed for Phases 0–6.

---

## 0. Decisions & assumptions (correct me here)

These were confirmed in the design discussion; the scope questions (§18) were answered and are folded in below. They drive everything.

| # | Decision | Consequence |
|---|----------|-------------|
| D1 | **MSNP18 is the only protocol version in this overhaul.** Other generations are out of scope now; the architecture must *accommodate* them — a future version = profile + dialect module + `Dialect` variant, zero domain changes (§6.4, appendix C). | Protocol support is modeled as a **dialect seam** (§6): a closed `Dialect` enum, MSNP18 as the sole variant today. |
| D2 | **One MSN session per Matrix account** for now. MPOP is *simulated*: endpoint ids are derived (from the account's email); the only *real* endpoint id is the one the client provides. | Single-session is a **policy flag on the account**, not a structural constraint. The session registry keys by email and holds a `Vec` of sessions (len enforced to 1). `get_single_client()` FIXME disappears. |
| D3 | **Group conversations are simulated as 1:1 proxy contacts** (proxy room email ⇒ simulated contact). Clients restrict group messages to text; the 1:1 simulation unlocks full message types. No canonical rooms exist in the Matrix spec for this, and tracking contact relations on the client side was painful — this is the chosen approach and may be revisited. | `Conversation` is an explicit domain aggregate with `kind: Direct \| ProxyGroup` (§7.4). Future native group support is a confined upgrade, not a rewrite. |
| D4 | **Protocol timing is intentional and must be preserved exactly** (USR S + GCF together, SBS → profile MSG → UBX before sync, ADL ack → 100 ms → NLN/UBX, 1311-byte message chunking, 120 s QNG keepalive, SHIELDS payload, CVR build strings `14.0.8117.0416` / `localhost`). | All of this moves from *code* to **profile data** (§6.2). Behavior is byte-identical; nothing is randomized. |
| D5 | Keep it simple. **Static dispatch** (enums + `match`) wherever the set is closed; dynamic dispatch only at two ports (§7.7). | No `dyn` trait labyrinths, no generics over version, one actor at most (§7.6 explains why there is none). |

---

## 1. Current state — the problems, with evidence

The workspace is three crates:

- `crates/msnp` — MSNP wire codec: framing (`RawCommandParser`), notification/switchboard/P2Pv2/SOAP command structs, version-agnostic shared models (`MsnUser`, `ContactList`, `PresenceStatus`, `TicketToken`, `EndpointId`, …). `MsnpVersion` enum today has only `MSNP17 | MSNP18 | MSNP21` variants.
- `crates/tachyon` — the bridge application: MSN notification server (TCP 1863), switchboard server (TCP 1864), axum HTTP server (SOAP + web UI), Matrix sync/handlers, P2P transports, audio (siren/ffmpeg), domain objects, composition root.
- `crates/tachyon-sqlite-store` — SQLite persistence (credentials keyed by ticket token).
- Vendored `matrix-rust-sdk` submodule.

The code works, but every seam the new requirements need to cut through is load-bearing:

| # | Problem | Evidence | What it blocks |
|---|---------|----------|----------------|
| P1 | **MSNP18 is hardcoded into the handshake.** `negotiation.rs` accepts only candidates containing `MSNP18`, replies VER, and builds the CVR response with hardcoded version strings. | `crates/tachyon/src/notification/handlers/negotiation.rs` | Adding any other generation requires rewriting the handshake path. |
| P2 | **IO and business logic are fused on the MSN side.** `auth.rs` (~16 KB) performs MSN USR SSO I/S **and** Matrix session restore, cross-signing checks, verification alerting, SBS, profile MSG, UBX, and sync bootstrap inside the NS handler. `adl_handler.rs` does roster mutation + Matrix presence lookups. | `notification/handlers/auth.rs`, `notification/handlers/ready/adl_handler.rs` | The NS layer cannot support versions with different auth (MD5/TWN pre-13) or roster flows (SYN/LST) without forking the whole handler tree. |
| P3 | **The mirror-image leak on the Matrix side.** Matrix event handlers and the sync loop construct MSN *wire* commands directly (`SwitchboardServerCommand::MSG`, `NotificationServerCommand` for "abch updated"). `bootstrap_handlers.rs` imports `matrix_sdk::{Client, Room, RoomMemberships}` and does room resolution inside SB auth. | `matrix/handlers/message_handlers.rs`, `matrix/sync.rs`, `switchboard/handlers/bootstrap_handlers.rs` | Neither side can be tested or replaced independently; version changes ripple through both directions at once. |
| P4 | **God objects.** `GlobalState` (Arc of everything, passed to every server) and `TachyonClient` (~15 public fields: backend, contact list, `DashMap<OwnedRoomId, SwitchboardHandle>` keyed by a *Matrix* type, SOAP holder, alerts, P2P sessions, …). `get_single_client()` exists as a FIXME because sessions are keyed by ticket token. | `tachyon/global_state.rs`, `tachyon/client/tachyon_client.rs` | No encapsulation boundary is enforceable; every handler can touch everything. |
| P5 | **Implicit state machine.** `LocalClientData.phase` (`Negotiating | Authenticating | Ready`) plus a field soup drives dispatch in `command_handler.rs`; same pattern in `LocalSwitchboardData`. | `notification/models/local_client_data.rs`, `notification/handlers/command_handler.rs` | Phase-legal command sequences are not expressed in the types; version-specific phases cannot be added. |
| P6 | **Single session by accident, not by design.** A second login of the same email (different ticket token) inserts a second `TachyonClient`; nothing enforces or models the limit. | `global_state.rs` (`tachyon_clients: DashMap<TicketToken, …>`), `auth.rs` | MPOP simulation and multi-endpoint presence cannot be reasoned about. |
| P7 | **Matrix credentials cross the MSN wire.** The switchboard token encodes the room id **and** the Matrix ticket token; the SB bootstrap looks the account up by `token.matrix_token`. | `switchboard/models/switchboard_token.rs`, `bootstrap_handlers.rs` | Wire tokens leak the internal credential scheme; no expiry, no revocation. |
| P8 | **Conversation mapping is implicit.** "Rooms" are keyed by `OwnedRoomId` everywhere on the MSN side; `ROOM_USER_PORTAL_MODE: bool = true` bakes the 1:1 room-user assumption into SB bootstrap. Groups-as-proxies exists in practice but not as a domain concept. | `switchboard/models/switchboard_handle.rs`, `bootstrap_handlers.rs` | Group proxy model (D3) cannot be extended; room/1:1/group semantics are scattered. |
| P9 | **The web/SOAP surface is an undocumented third IO boundary.** `web_server.rs` hosts SOAP services (ABService = address book, SharingService, StorageService, RSI, **RST2 = ticket issuer**), legacy Passport endpoints (`ppcrlcheck`, `sha1auth`, `wlidsvcconfig`, `MsgrConfig`), ads/MSN-Today, profile pictures, and the login/device-verification UI. These are IO adapters over the *same* domains as NS/SB (auth, roster, presence) — RST2 is the token source for the SSO handshake itself. | `web/web_server.rs`, `web/soap/*` | Multi-version support also needs version-aware SOAP/config responses; today there is no boundary to hang that on. |
| P10 | **Transport robustness smells.** SB write channel has capacity **10,000,000** (NS: 300 — inconsistent); `write_all(...).await.unwrap()` panics the write task on client disconnect; `from_utf8_unchecked` in debug logging; `rayon` parallel iterators imported into async server code; `println!` in a spawned writer task. | `switchboard/switchboard_server.rs` L164/L189, `notification/notification_server.rs` | Memory blowup risk, panics in transport, no backpressure policy. |

---

## 2. Design principles

1. **Boundaries hide knowledge that varies independently.** Each boundary below is justified by the one thing that varies across it. If a module can't state what it protects, it doesn't exist.
2. **Crate edges are the compiler-enforced boundaries.** Module boundaries rot; `Cargo.toml` dependencies don't. The one rule that does most of the work: **`tachyon-domain` may not depend on `msnp` (wire), `matrix-sdk`, `axum`, or `sqlx/rusqlite`.** If the domain compiles without them, the leaks P2/P3 cannot reappear.
3. **Static dispatch for closed sets.** Protocol dialects, command kinds, outbound operations, and domain events are *closed enums dispatched by `match`*. Runtime selection happens exactly once — during `VER` negotiation — and then every command dispatches statically through the chosen dialect variant. One `match` arm per command is free; this satisfies "static dispatch as much as possible" without 21 monomorphized connection types.
4. **Dynamic dispatch only at two ports** — the Matrix port and the persistence port (§7.7). One trait, one implementation each, constructed once at startup. Everything else is `enum` + `match`.
5. **One owner per piece of state.** Protocol session state is owned by its connection task (no sharing). Account domain state is owned by the `Account` aggregate and touched only through its API. There is no global mutable object.
6. **Preserve wire behavior; move it to data.** D4. Transcript tests (§16) are the safety net for every phase.
7. **No golden interfaces.** A trait/interface whose method list mirrors the protocol's command list (`handle_ver`, `handle_cvr`, …, `handle_xfr`) is an application-layer boundary wearing a domain costume — it carries no invariant knowledge and exists only to be implemented once per version. The dialect is therefore a *module + enum variant + concrete state struct*, not a trait. The `VER` negotiation match (§6) is the only version dispatch; the two `dyn` ports (§7.7) are the only dynamic dispatch in the system.

---

## 3. Target layout

### 3.1 Crate graph

```
                 ┌────────────────────────────────────────────┐
                 │              tachyon  (bin + app)          │
                 │  msn-io (NS/SB sessions, dialects)         │
                 │  matrix-adapter   web/SOAP adapter         │
                 │  composition root (main, config)          │
                 └──────┬──────────────┬──────────────┬──────┘
                        │              │              │
        ┌───────────────┘              │              └───────────────┐
        ▼                              ▼                              ▼
┌──────────────┐   depends    ┌────────────────┐          ┌────────────────────────┐
│     msnp     │◄─────────────│  msn-models    │◄─────────│   tachyon-domain       │
│ (wire codec) │              │ (MSN data types│          │ (bridge domain core)   │
└──────────────┘              │  only, no wire)│          │ no msnp / no matrix-sdk│
        ▲                     └────────────────┘          └───────────┬────────────┘
        │                                                             │ implements
        │ (dialects in app render to these)                           ▼
        │                                          ┌────────────────────────────────┐
        └──────────────────────────────────────────│  tachyon-sqlite-store          │
                                                   │ (repository impl, port traits) │
                                                   └────────────────────────────────┘
```

Five crates (three exist today; two are created by moving files):

| Crate | Role | May depend on |
|-------|------|---------------|
| `msn-models` (new) | Version-agnostic MSN *data* vocabulary: `MsnUser`, `EmailAddress`, `EndpointId`, `PresenceStatus`, `Contact`, `ContactList`, `TicketToken`, `Capabilities`, `MsnpVersion`, payload content types. **No wire framing, no policy.** | std, uuid, serde (only where needed) |
| `msnp` (existing, trimmed) | Pure wire: framing (`RawCommandParser`), NS/SB/P2Pv2/SOAP command structs, `IntoBytes`, `TryFromRawCommand`. | `msn-models` |
| `tachyon-domain` (new) | The bridge domain: account, identity, roster, presence, conversations, messaging, delivery, alerts, policy, ports. | `msn-models`, std, uuid, async trait plumbing for ports |
| `tachyon-sqlite-store` (existing) | Persistence implementing domain port traits. | `tachyon-domain` (for port traits), `msn-models`, rusqlite |
| `tachyon` (existing, bin) | Adapters + composition root: MSN IO (NS/SB + **dialect modules**), Matrix adapter, web/SOAP adapter, P2P transports, media codecs, config, `main`. | all of the above |

### 3.2 Module tree of `crates/tachyon/src`

```
src/
  main.rs                  # composition root: config, logging, services, spawn, shutdown
  config.rs                # (existing, extended: enabled_versions)
  msn/
    connection_core.rs     # shared TCP framing/read/write/kill loop (§8.1)
    ns_server.rs           # accept loop → spawn NS session task
    sb_server.rs           # accept loop → spawn SB session task
    ns_session.rs          # NS session task: owns MsnpSession + outbound select (§8.2)
    sb_session.rs          # SB session task: owns SbSession (§8.3)
        dialect/
      mod.rs               # Dialect enum (one variant today), negotiate(), PROFILES registry
      profile.rs           # MsnpProfile + Capabilities (data)
      msnp18.rs            # MSNP18 dialect: dedicated VER/CVR/USR handshake + ready-phase handlers + render_*
      # future versions land here as sibling modules (appendix C); no empty modules today
    p2p/                   # (existing transports/sessions — IO level, driven by domain ops)
    media/                 # siren/ffmpeg codec utilities (moved here)
  matrix_adapter/
    client.rs              # matrix_sdk Client wrapper: login/restore, sliding sync loop
    events.rs              # SDK events → domain events (thin normalizers, no MSNP types)
    send.rs                # domain MatrixOutbound ops → SDK calls
    directory.rs           # ConversationId ↔ RoomId map, email → user resolver
    verification.rs        # cross-signing, SAS, device verification mechanics
  web/
    web_server.rs          # axum router (routes unchanged)
    soap/                  # each service: decode XML → domain call → render (thin)
    ui/                    # login page, device verification, oauth callback
    legacy.rs              # MsgrConfig, ppcrl*, sha1auth, wlidsvcconfig, ads, today
  services.rs              # Services struct (replaces GlobalState)
```

### 3.3 Dependency rules (enforced by Cargo + one CI grep)

1. `tachyon-domain` imports nothing from `msnp`, `matrix_sdk`, `axum`, `rusqlite`. *(Cargo-enforced.)*
2. `matrix_adapter/*` imports nothing from `msnp` (no wire command construction). *(CI grep: `grep -r "use msnp" crates/tachyon/src/matrix_adapter` must be empty.)*
3. `msn/dialect/*` and `msn/*session*.rs` import nothing from `matrix_adapter` or `matrix_sdk`. *(CI grep.)*
4. `msn-models` and `msnp` contain no `tokio` sockets, no policy, no `log` of business decisions. *(Code review; they are small.)*
5. The dialect is the only place that turns *domain events* into *wire bytes* and the only place that turns *wire commands* into *domain events* (+ local protocol replies).

---

## 4. `msn-models` — the MSN data vocabulary

A pure file move from `msnp::shared::models` and `msnp::shared::traits` (data parts). Types the domain and the wire share:

```rust
pub enum MsnpVersion { Msnp17, Msnp18, Msnp21 }   // closed wire vocabulary: versions that can appear in VER; new variants added only as versions come into scope
pub struct EmailAddress(pub String);
pub struct EndpointId { pub email_addr: EmailAddress, pub endpoint_guid: Option<Uuid> }
pub struct MsnUser { pub endpoint_id: EndpointId, pub display_name: String, pub status: PresenceStatus, pub avatar: Option<AvatarRef>, /* … */ }
pub struct Contact { pub user: MsnUser, pub notes: String, pub capabilities: Capabilities, /* … */ }
pub struct ContactList { /* membership, block list, … */ }
pub struct TicketToken(pub String);          // opaque auth token (RST2-issued)
pub struct PresenceStatus { /* away/busy/brb/offline/online + substates */ }
```

Plus *content* types (text, url, image, file, voice-clip, system, group) used by both sides' `MessageContent`. Nothing here knows about sockets, XML, or Matrix.

---

## 5. `msnp` — wire only (what stays)

Unchanged in spirit: `RawCommandParser`, per-protocol command enums and structs, `IntoBytes`/`TryFromRawCommand`. Two refinements:

- `MsnpVersion` moves to `msn-models` as the closed wire vocabulary (negotiation policy lives in the dialect; variants are added only as versions come into scope).
- SOAP request/response structs stay here (they are wire); **SOAP semantics** (what ABCH *means*, which operations are legal) move to the domain.

---

## 6. The dialect — protocol semantics per generation

This is the boundary that kills P1 and makes D1 tractable. It lives in `msn/dialect/` (app crate) and is the *only* code that knows both MSNP wire shapes and the domain vocabulary.

### 6.1 Structure: families, versions, profiles

Design note for future onboarding (appendix C): behavior does **not** change per version — it changes per *generation family* (roughly: MSNP10–12 `SYN`/`LST` + MD5/TWN; MSNP13–16 `ADL`/`RML` + SSO; MSNP17–18 SSO I/S + GCF/SHIELDS; MSNP21+ a further rework), and within a family individual versions differ by small capability deltas. **Today only MSNP18 is implemented**; the note is kept so future profiles inherit the domain knowledge, but no other dialect modules are built now.

So the unit of *code* is one dialect module per implemented version; the unit of *data* is the version profile. If a second version of a family later lands, code-sharing *within* the family is an implementation detail of that moment — not a structure to build ahead of time.

```rust
// dialect/profile.rs — DATA only
pub struct MsnpProfile {
    pub version: MsnpVersion,
    pub family: Family,
    pub server_versions: &'static [MsnpVersion],   // advertised in VER, best-first
    pub cvr: Option<CvrTemplate>,                   // client id, version/build/product strings, "localhost" entries
    pub auth: AuthScheme,                           // Md5 | Twn | SsoI_S { shields_payload }
    pub roster: RosterStyle,                        // SynLst | AdlRml
    pub caps: Capabilities,                         // has_gcf, has_uux, has_oim, has_msgr, has_ubx, has_wimp, has_p2pv2, max_msg_size, …
    pub post_login: PostLoginSync,                  // ordered sequence: [Sbs, ProfileMsg, Ubx, InitialMail, …]
}
pub const PROFILES: &[MsnpProfile] = &[ /* MSNP18 (today's hardcoded values live here), MSNP17, … */ ];

pub fn negotiate(candidates: &[MsnpVersion]) -> Option<&'static MsnpProfile> {
    // highest mutual version by server preference; None → reject
}
```

**Today's hardcoded values become data:** the `14.0.8117.0416`/`localhost` CVR strings, the GCF SHIELDS payload, the post-login sequence — all of it moves into `PROFILES` entries (D4: byte-identical, now inspectable).

### 6.2 The `Dialect` enum — static dispatch, closed set

```rust
// dialect/mod.rs
pub enum Dialect {
    Msnp18(Msnp18),    // sole variant today — current behavior lives here
    // future variants (Msnp17, Msnp21, …) are sibling modules — appendix C
}

impl Dialect {
    pub fn handle(&mut self, cmd: NotificationClientCommand) -> Result<Step, DialectError>;
    pub fn render_ns(&self, op: MsnOutbound) -> Vec<NotificationServerCommand>;
    pub fn render_sb(&self, op: SbOutbound) -> Vec<SwitchboardServerCommand>;
    pub fn caps(&self) -> &Capabilities;
}

pub struct Step {
    pub local_replies: Vec<NotificationServerCommand>,  // protocol-local: OK/NAK with tr_id, QNG, handshake steps
    pub domain_events: Vec<MsnEvent>,                   // semantic, tr_id-free (§7.5)
}
```

`handle` is an exhaustive `match` on the dialect variant (one arm today) → `Msnp18::handle` is an exhaustive `match` on the command. No `dyn`, no generics over version — this is the **single static dispatch point** for the negotiated version, chosen once during `VER`.

The MSNP18 dialect module has **explicit dedicated handshake handlers** (`on_ver`, `on_cvr`, `on_usr_i`, `on_usr_s`), as requested:

```rust
// dialect/msnp18.rs (MSNP18 = today's behavior, lifted verbatim)
pub struct Msnp18 { profile: &'static MsnpProfile, state: Msnp18State, tr: TrTable }

enum Msnp18State {
    ExpectCvr,
    ExpectUsrI,
    ExpectUsrS,
    Ready,
}

impl Msnp18 {
    fn on_ver(&mut self, ver: VerClient) -> Step { /* advertise profile.server_versions */ }
    fn on_cvr(&mut self, cvr: CvrClient) -> Step { /* local: CVR from profile.cvr */ }
    fn on_usr_i(&mut self, email: EmailAddress) -> Step { /* local: USR S + GCF(SHIELDS from profile); domain: none yet */ }
    fn on_usr_s(&mut self, s: UsrSClient) -> Step {
        /* local: USR OK;
           domain: MsnEvent::ProtocolReady { email, ticket_token, client_version, endpoint_guid: s.guid } */
    }
    // ready-phase: on_png → local QNG; on_adl → local OK + domain RosterSynced; on_chg → domain PresenceChanged; …
}
```

**Handshake state machine** (pre-`VER` is shared and trivial: expect `VER`, else disconnect):

```
MSNP18:   VER → CVR → USR I → [client fetches RST2 token over HTTPS] → USR S → Ready
```

Future generations may have *drastically* different handshakes (challenge-response auth without CVR, different post-auth sequences, …). That is exactly why the handshake lives *inside* the dialect module, with its own state enum, rather than in a shared version-neutral state machine: each future dialect brings its own (appendix C).

Note the cross-surface interlude: for 13+ the client *leaves the NS mid-handshake* to get a ticket via HTTPS **RST2** and returns on the same NS connection. The NS state machine therefore has an "awaiting token" state, and RST2 (web adapter) must call the **domain** ticket service — this is why auth is a domain service and not an NS-handler detail (§9.3).

**Outbound rendering** is where version behavior diverges (D4 timing lives in the rendering + domain policy, keyed by `caps`):

```rust
// MSNP18: MsnOutbound::Presence { … } → [NLN, UBX]  (the ADL 100 ms delay is a domain delivery delay, not a render delay)
// MSNP18: MsnOutbound::Message { … } → chunked MSG at profile.caps.max_msg_size (1311)
// future versions render the same domain ops to different wire shapes from their own render_*
```

### 6.3 The MSNP18 profile data (today)

MSNP18's behavior is *data*, not code — it lives in the `PROFILES` entry (D4) and is pinned against today's behavior by the Phase-0 transcript fixtures (§16):

| Aspect | MSNP18 value (current behavior, preserved byte-identically) |
|---|---|
| Auth | SSO I/S + GCF/SHIELDS — SHIELDS payload in `profile.auth` |
| Roster | ADL/RML |
| CVR strings | `14.0.8117.0416` / `localhost` — in `profile.cvr` |
| Post-login sequence | SBS → profile MSG → UBX → offline mail; ADL ack → 100 ms → NLN/UBX — in `profile.post_login` |
| Message chunking | 1311 bytes — `profile.caps.max_msg_size` |
| Keepalive | 120 s QNG |

The multi-generation capability matrix (families, per-version cells, capture-verify markers) is demoted to **appendix C** — it is onboarding material for future versions, not design work for this overhaul.

### 6.4 Procedure: how to add a version later (the D1 payoff — and the acceptance criterion)

1. Add the `MsnpVersion` variant (msn-models) if the version can appear in `VER`.
2. Add an `MsnpProfile` const (data: advertised versions, CVR, auth, caps, post-login sequence).
3. Add a sibling dialect module (`msn/dialect/msnpNN.rs`): dedicated handshake state machine + command legality + `render_*`.
4. Add one `Dialect` variant + one negotiation arm.
5. Record transcript fixtures (§16): handshake, roster sync, one message each direction, presence change, file/voice clip if applicable; flip the profile on in config; e2e with a generation-appropriate client (e.g. zathras).

**Acceptance criterion:** steps 1–5 are the *entire* change — no edits to `tachyon-domain`, the matrix adapter, the web/SOAP adapter, or the store (they are version-blind by construction), and all MSNP18 fixtures remain green. (MSNP10, if ever wanted, is exactly this procedure and is out of scope today.)

---

## 7. The domain — `crates/tachyon-domain`

No sockets, no XML, no Matrix SDK, no SQL. Depends on `msn-models` only.

### 7.1 The `Account` aggregate (replaces `TachyonClient` + half of `GlobalState`)

```rust
pub struct Account {
    // identity (immutable after creation)
    email: EmailAddress,
    own_user: MsnUser,                      // derived endpoint id (D2)
    // state — single owner = the account task-context; touched only via API
    sessions: Mutex<SessionSlots>,          // Vec<SessionSlot>; policy enforces len ≤ 1 (D2)
    matrix: MatrixState,                    // Pending | Ready { backend: Arc<dyn MatrixPort> }
    roster: Mutex<Roster>,                  // contacts, membership, block list, group proxy contacts
    presence: Mutex<PresenceTable>,         // target → status/extended data
    conversations: DashMap<ConversationId, Conversation>,
    outbound_ns: Sender<MsnOutbound>,      // per active NS session (1:1 with the NS session task)
    delivery: Delivery,                    // pending events, dedupe log, offline buffer
    alerts: Alerts,                         // login toast, verification, info (web URLs)
    tickets: TicketRegistry,               // RST2 + opaque SB tickets, TTL, revocation (§12)
    config: Arc<DomainConfig>,
}
```

### 7.2 The account API — the only entry points for any IO layer

All methods are **fast**: they mutate state, enqueue outbound ops, and `spawn` anything slow (Matrix round-trips) internally. This is the contract that lets the NS/SB session tasks call them inline without blocking the socket.

```rust
impl Account {
    // — MSN protocol inbound (from dialect, via NS session task) —
    pub async fn protocol_ready(&self, info: ProtocolReadyInfo) -> Vec<MsnOutbound>;
    pub async fn roster_synced(&self, contacts: Roster, initial: bool) -> Vec<MsnOutbound>;
    pub async fn roster_action(&self, action: RosterAction) -> Vec<MsnOutbound>;   // ADD/REM/ADL delta, SOAP ABCH
    pub async fn presence_changed(&self, target: EmailAddress, status: PresenceStatus) -> Vec<MsnOutbound>;
    pub async fn endpoint_data_changed(&self, target: EmailAddress, data: EndpointData) -> Vec<MsnOutbound>;
    pub async fn alert_action(&self, alert_id: i32) -> Vec<MsnOutbound>;

    // — switchboard inbound (from SB session task) —
    pub async fn conversation_connected(&self, conversation: ConversationId, sb: SbHandle) -> Vec<SbOutbound>;
    pub async fn conversation_command(&self, conversation: ConversationId, cmd: SbInbound) -> Vec<SbOutbound>;
    pub async fn conversation_disconnected(&self, conversation: ConversationId) -> Vec<MsnOutbound>;

    // — matrix inbound (from matrix adapter handlers) —
    pub fn matrix_event(&self, ev: MatrixEvent);   // sync, fast, spawns slow work

    // — web/SOAP inbound —
    pub async fn issue_ticket(&self, email: &EmailAddress) -> TicketToken;          // RST2
    pub async fn soap_query(&self, q: SoapQuery) -> SoapResult;                       // AB, sharing, storage, rsi

    // — lifecycle —
    pub async fn restore_matrix_session(&self) -> Result<(), BridgeError>;   // from auth or web login
    pub fn kick_session(&self, session: &SessionSlot, reason: KickReason);
}
```

### 7.3 Domain submodules and the invariants they protect

| Module | Owns | Invariant it protects (Muratori justification) |
|---|---|---|
| `identity` | email ↔ account, `own_user` derivation, client version, `BridgeLinkToken`, endpoint ids (derived vs client-provided, D2) | One canonical identity per email; endpoint guid fan-out rules; opaque wire tokens (§12) |
| `roster` | contacts (incl. **group proxy contacts**, D3), membership, block list | Roster is a total, consistent view for *all* sessions; proxy contacts never leak a room id |
| `presence` | per-target status + extended data, mapping rules (matrix ↔ msn), fan-out to sessions | Presence is consistent across NS sessions, SOAP, and Matrix; the 100 ms post-ADL ordering (D4) |
| `conversation` | `Conversation { id, peer: ContactId, kind: Direct \| ProxyGroup, room: Option<RoomRef>, members }` | 1:1 conversation ↔ contact ↔ (room) is a bijection managed in one place; `ROOM_USER_PORTAL_MODE` disappears as a global — it's a `kind` |
| `messaging` | inbound/outbound message flow, content normalization, dedupe log, offline buffer, file/voice-clip policy (uses dialect `caps`) | No message duplicated across the bridge; chunking limits respected per dialect |
| `delivery` | outbound op queues (NS ops, per-conversation SB ops), pending events until a conversation is connected | Ops are not lost between "domain decided" and "wire sent"; ordering preserved |
| `alert` | client alerts with web URLs (login, verification, info), alert resolution | Exactly one pending login alert per account; alert ids stable across sessions |
| `policy` | single-session enforcement (D2), unknown-contact degradation, offline behavior | The MPOP limit is a flag, not a structure (D2) |
| `auth_service` | RST2 ticket issuance/lookup, credential lookup, matrix session restore orchestration, verification gating | Credentials never appear on the MSNP wire; auth is one service, not one NS handler detail |

### 7.4 Conversations and the group-proxy model (D3)

```rust
pub struct ConversationId(pub Uuid);
pub enum ConversationKind { Direct, ProxyGroup }
pub struct Conversation {
    pub id: ConversationId,
    pub kind: ConversationKind,
    pub peer: ContactId,              // for ProxyGroup: the proxy contact (simulated contact)
    pub room: Option<RoomRef>,        // RoomRef is an opaque domain token; the matrix adapter owns the real RoomId
    pub members: Mutex<Vec<ContactId>>,
}
```

- A Matrix room maps to exactly one `Conversation` (the directory in the matrix adapter owns `ConversationId ↔ RoomId`, §9.4).
- Group rooms are surfaced to MSN clients as a `Contact` of kind *proxy*: the client sees a 1:1 contact whose messages land in the group room (text-only restriction is enforced in `messaging` from the proxy kind, matching today's behavior).
- Future native group support = change `kind` semantics + per-dialect SB rendering. Nothing outside `conversation`/`messaging`/`dialect` is touched.

### 7.5 Inbound/outbound vocabulary (closed enums — the leak-killers)

```rust
// domain → MSN NS (dialect renders these)
pub enum MsnOutbound {
    RosterAck,
    PresenceNotice { target: EmailAddress, status: PresenceStatus, display_name: String, avatar: Option<AvatarRef> },
    EndpointData { target: EmailAddress, data: EndpointData },
    Notification { alert: Alert, kind: NotifyKind },   // login toast, verification, info
    ProfileMessage,
    OfflineMailData { target: EmailAddress, data: MailData },
    KeepAlive,
}

// domain → MSN SB (dialect renders these)
pub enum SbOutbound {
    MemberJoined { contact: MsnUser },
    MemberLeft { contact: MsnUser },
    RosterDelta { added: Vec<MsnUser>, removed: Vec<MsnUser> },
    Message { from: MsnUser, content: MessageContent },
    TypingNotice { from: MsnUser },
    MemberAction { contact: MsnUser, action: MemberAction },
}

// matrix adapter → domain
pub enum MatrixEvent {
    IdentityReady { user: MsnUser },
    SessionExpired,
    VerificationRequired,
    ContactStatusChanged { contact: ContactId, status: PresenceStatus },
    ConversationMemberJoined { conversation: ConversationId, contact: ContactId },
    ConversationMemberLeft   { conversation: ConversationId, contact: ContactId },
    ConversationMessageReceived { conversation: ConversationId, sender: ContactId, content: MessageContent },
    TypingStarted { conversation: ConversationId, user: ContactId },
    MediaDownloaded { conversation: ConversationId, source: MediaSourceRef, bytes: Vec<u8> },
}

// domain → matrix adapter (adapter executes these)
pub enum MatrixOutbound {
    SendRoomMessage { conversation: ConversationId, content: MessageContent },
    SendTyping { conversation: ConversationId, duration: Duration },
    Invite { conversation: ConversationId, contact: ContactId },
    Join { conversation: ConversationId },
    Leave { conversation: ConversationId },
    DownloadMedia { source: MediaSourceRef },
    SetTypingListener { conversation: ConversationId },
}
```

Because these enums are the *only* types that cross the MSN boundary and the Matrix boundary, `SwitchboardServerCommand::MSG` in `message_handlers.rs` (P3) and Matrix login inside `auth.rs` (P2) become unexpressible — the types simply don't exist on the other side of the boundary.

### 7.6 Why there is no central actor

Considered: a single `BridgeCore` task with `BridgeCommand`/`BridgeEvent` message enums (every IO layer sends to it, oneshot replies for queries). Rejected because it would add reply plumbing for every query (SOAP, SB bootstrap, presence lookups), serialize the whole bridge through one task, and be a large behavioral change to an already-working concurrency shape. The existing shape — per-connection tasks + shared account aggregate + outbound queues — is sound once the aggregate has a real API. **Rule: the account is touched only through its API; protocol state is never shared.** That gives the same invariants without the maze.

### 7.7 Ports (the only `dyn` in the design)

```rust
// tachyon-domain/ports.rs — one implementation each, constructed once in main()
pub trait MatrixPort: Send + Sync {
    async fn restore_session(&self, account: &AccountHandle) -> Result<(), BridgeError>;
    async fn send_message(&self, conversation: ConversationId, content: MessageContent) -> Result<(), BridgeError>;
    async fn send_typing(&self, conversation: ConversationId, duration: Duration) -> Result<(), BridgeError>;
    async fn invite(&self, conversation: ConversationId, contact: ContactId) -> Result<(), BridgeError>;
    async fn join(&self, conversation: ConversationId) -> Result<(), BridgeError>;
    async fn leave(&self, conversation: ConversationId) -> Result<(), BridgeError>;
    async fn resolve_contact(&self, email: &EmailAddress) -> Result<Option<MsnUser>, BridgeError>;
    async fn download_media(&self, source: MediaSourceRef) -> Result<MediaDownload, BridgeError>;
    async fn cross_signing_ok(&self) -> bool;
    async fn start_verification(&self) -> Result<(), BridgeError>;
}

pub trait CredentialStore: Send + Sync {
    fn find_by_token(&self, token: &TicketToken) -> Option<CredentialRecord>;
    fn save(&self, token: &TicketToken, record: CredentialRecord) -> StoreResult;
    fn find_account(&self, email: &EmailAddress) -> Option<CredentialRecord>;
}
```

`MatrixPort` is implemented by `matrix_adapter`; `CredentialStore` by `tachyon-sqlite-store` (SQLite behind `spawn_blocking`). A fake `MatrixPort` gives the domain its whole test suite (§16).

---

## 8. The MSN IO adapters (app crate)

### 8.1 `connection_core` — one framing loop for NS and SB

Today `notification_server.rs` and `switchboard_server.rs` duplicate the accept/read/parse/write/kill loop, the parser, and the write task, with different (wrong) channel sizes. One generic core:

```rust
pub struct ConnectionCore<Cmd, Resp> {
    parser: RawCommandParser,
    reader: BufReader<OwnedReadHalf>,
    writer: OwnedWriteHalf,
    out: mpsc::Sender<Resp>,          // bounded: 256
    kill: broadcast::Receiver<()>,
}
// loop: select! { socket read → parse → yield Vec<Cmd>; out.recv() → write_all (error → teardown, NO unwrap); kill }
```

IO contract (fixes P10):
- **Bounded channels** (256) with explicit policy: full ⇒ drop-oldest for keepalive-class ops, backpressure (await) for message-class ops.
- **Write error ⇒ session teardown**, never `unwrap()` in a transport task.
- Debug logging uses `from_utf8_lossy`; no `from_utf8_unchecked`, no `println!`, no rayon inside async tasks.

### 8.2 NS session task (one per client connection)

```
owns: MsnpSession { pre_handshake: PreVer | negotiated: Dialect, tr_table, dialect state }
loop select! {
  bytes → cmds = parse
       for cmd: step = dialect.handle(cmd)
         write(step.local_replies)                 // OK/NAK/QNG/handshake — protocol-local
         for ev in step.domain_events: write(account.<method>(ev).await)   // fast by contract (§7.2)
  op = outbound_ns.recv()
       write(dialect.render_ns(op))
  kill → teardown (account.kick_session, remove SB handles)
}
```

The phase field is gone: the dialect's state machine *is* the phase, and each dialect owns its legal command set (P5, P1). `LocalClientData` and `command_handler.rs`'s phase match disappear.

### 8.3 SB session task (one per conversation connection)

Owns an `SbSession` (dialect-rendered, state: `Auth | Initializing | Ready` — the ANS+USR bootstrap, CAL→IRO/JOI, then `Ready`):

```
Auth:        ANS { opaque ticket } → domain validates ticket → (email, conversation) → USR OK / error
             USR { endpoint } → record client endpoint guid (D2) → OK
Initializing: CAL { email } → RING; domain supplies member roster → IRO(s) + JOI(me) → Ready
Ready:       MSG → SbInbound::Message → domain → SbOutbound rendered by dialect
             USR/JAN/ACK keepalives handled locally
disconnect:  domain.conversation_disconnected (delivery re-queues pending events)
```

Bootstrap no longer imports `matrix_sdk`, does no room scanning, and never sees a Matrix token (P3, P7): the opaque ticket resolves to `(email, ConversationId)` in the domain (§12).

### 8.4 The web/SOAP adapter (third surface, P9)

Routes stay as they are (`web_server.rs`); each handler becomes a three-line translation:

| Route | Decode | Domain call | Render |
|---|---|---|---|
| `/RST2.srf` | POST ticket request | `account.issue_ticket(email)` | SOAP token envelope |
| `/abservice/abservice.asmx` | ABCH XML | `account.soap_query(GetContacts/Add/Remove/…) → RosterAction` | ABCH response (dialect profile decides schema nuances) |
| `/abservice/SharingService.asmx` | sharing XML | `account.soap_query(FileTransfer/Invite/…) → SbOutbound + MsnOutbound` | XML |
| `/storageservice/…` | storage XML | `account.soap_query(OIM/…) → MsnOutbound::OfflineMailData` | XML |
| `/rsi/rsi.asmx` | rsi XML | `account.soap_query` | XML |
| `MsgrConfig`, `ppcrl*`, `sha1auth`, `wlidsvcconfig` | — | thin stateless pass-throughs (confirmed: no version-sensitive state) | XML/JSON |
| `/tachyon/login`, device-verification, `/oauth/callback` | form/URL | `account.restore_matrix_session()` / `MatrixPort::start_verification` | pages (unchanged UI) |
| ads / matrix_today / profile pictures | — | read-only | unchanged |

Version-awareness (D1) lands here naturally: SOAP/config **rendering** takes the active profile; *semantics* are already domain calls.

---

## 9. The Matrix adapter

### 9.1 What moves where

- `matrix/client.rs` (from `sync.rs` + `auth.rs` restore bits): login, session restore, sliding-sync loop. The sync loop's current inline `NOT (abch updated)` becomes `MatrixEvent` → domain → `MsnOutbound` (P3).
- `matrix/events.rs` (from `matrix/handlers/*`): **thin normalizers only.** Each SDK event handler: extract fields → normalize to `MessageContent` → `account.matrix_event(ev)`. No MSNP imports (rule 2), no room logic, no MSN command construction.
- `matrix/send.rs`: executes `MatrixOutbound` ops via `MatrixPort` (send message, typing, invite, join, leave, media download with the existing retry/timeout behavior).
- `matrix/directory.rs`: `ConversationId ↔ RoomId` (bidirectional, persisted), email → matrix user resolver (existing `FindRoomFromEmail`/`ToMsnUser` moved here), room display-name mapping. `ROOM_USER_PORTAL_MODE` becomes a per-conversation fact in the domain.
- `matrix/verification.rs`: cross-signing, SAS, device verification mechanics (from `auth.rs` + `verification_request_repository`). The domain decides *policy* ("verification required" → alert); the adapter does the *mechanics*.

### 9.2 The Matrix side of the login flow (today's `auth.rs`, 16 KB, split across three owners)

| Today (in `auth.rs`) | Target owner |
|---|---|
| USR SSO I/S state machine, GCF/SHIELDS | `dialect/msnp18.rs` (data: `MsnpProfile`) |
| Credential lookup by ticket token | domain `auth_service` (+ `CredentialStore` port) |
| Matrix session restore / login toast alert | domain `auth_service` + `MatrixPort::restore_session` + `Alerts` |
| Cross-signing check + verification alert | domain policy + `matrix/verification.rs` |
| SBS, profile MSG, UBX, initial mail, 100 ms NLN/UBX ordering | domain `post_login` sequence (data: `MsnpProfile.post_login`), rendered by dialect |
| Sliding-sync bootstrap | `matrix/client.rs` |

### 9.3 Auth as a domain service (the before/after that proves the design)

**Before (MSN side, one 16 KB file):**
```
USR S → look up credentials (sqlite) → restore matrix session inline
      → cross-signing inline → spawn sync_with_server_task → SBS/profile/UBX/mail inline
      → insert TachyonClient into GlobalState → phase = Ready
```

**After (four small pieces, each on its side of a boundary):**
```
NS session:  USR S → dialect local: USR OK
             → domain event: ProtocolReady { email, ticket, version, guid }
domain:      protocol_ready():
             account = registry.get_or_create(email)          // single-session policy: kick old session
             account.sessions.insert(slot, outbound_ns)
             if !credentials_found(ticket) → Alert(login toast)      // D2/D4: exactly one pending
             else spawn restore_matrix_session()
             → MsnOutbound sequence from profile.post_login
matrix:      restore_session() → on success: MatrixEvent::IdentityReady
             → domain: MatrixEvent → MsnOutbound::ProfileMessage/UBX/OfflineMail (rendered by dialect)
web:         RST2 → account.issue_ticket(email)               // the cross-surface token step
```

No MSN wire type appears below the dialect; no Matrix type appears above the adapter.

---

## 10. Composition root (replaces `GlobalState`)

```rust
pub struct Services {
    pub accounts: DashMap<EmailAddress, Arc<Account>>,   // replaces tachyon_clients (keyed by email, P4/P6)
    pub matrix: Arc<MatrixAdapter>,                      // owns the SDK client(s), directory, verification
    pub store: Arc<dyn CredentialStore>,                 // the one dyn (with sqlite impl)
    pub profiles: &'static [MsnpProfile],                // filtered by config.enabled_versions
    pub config: Arc<TachyonConfig>,
}
```

- Every server/handler receives **only** the fields it needs (NS session gets `&Services.accounts` + its own outbound channel; SB bootstrap gets the ticket registry + conversations; SOAP gets the account lookup). No "Arc of everything".
- `TachyonClient` is deleted: session slot = `{ session_id, email, dialect-caps snapshot, outbound_ns, sb_handles }`, owned by the NS session task; account-level state lives in `Account`.
- Shutdown: existing broadcast-kill pattern kept, plus per-account teardown (kick sessions, close SB handles, persist state).

---

## 11. Key flows (sequences)

**A. Full login (MSNP18, byte-identical to today):**
```
client: VER [MSNP18, …] → server: VER [MSNP18]                     (dialect, local)
client: CVR  → server: CVR (profile.cvr)                           (dialect, local)
client: USR SSO I <email> → server: USR S + GCF(SHIELDS)           (dialect, local)
client: (HTTPS) RST2 → ticket token                                (web → domain auth_service)
client: USR SSO S { token, guid } → server: USR OK                (dialect, local)
       → domain: protocol_ready → account created, session slot, post_login sequence
       → domain: credentials? restore matrix session | login toast NOT
matrix:  IdentityReady → domain → ProfileMessage, UBX, OfflineMail  (dialect renders)
client: ADL → server: OK (local) → 100 ms → NLN+UBX per contact     (domain, D4)
         roster synced, phase Ready
```

**B. Message MSN → Matrix:**
```
SB MSG → SbSession (dialect) → SbInbound::Message → account.conversation_command
       → domain messaging: dedupe, normalize, content policy (voice clip vs file, per caps)
       → MatrixOutbound::SendRoomMessage → matrix_adapter send.rs → SDK send
```

**C. Message Matrix → MSN:**
```
SDK room message event → matrix/events.rs normalizer → MatrixEvent::ConversationMessageReceived
→ account.matrix_event → domain messaging → SbOutbound::Message
→ conversation's SB handle (if connected) → dialect.render_sb → chunked MSG write
  (else delivery re-queues; on SB connect, flush — existing pending-event behavior)
```

**D. Group message via proxy (D3):** group room event → normalizer attributes sender via directory → domain maps proxy contact → 1:1 SB message to the proxy contact; outbound text-only restriction enforced in `messaging` (kind = ProxyGroup).

**E. File transfer:** SB MSG(P2P) → domain messaging (caps allow P2Pv2) → p2p/ transport (moved, unchanged machinery) → media/ codec for voice clips (siren/ffmpeg).

**F. Device verification:** `MatrixEvent::VerificationRequired` → domain alert (web URL) → client opens `/tachyon/verify` → web → `MatrixPort::start_verification` → SAS in adapter → domain alert resolved → client proceeds.

---

## 12. Identity, credentials, tickets (P7 fix)

- **Account key = email** (stable, domain-owned). Ticket tokens are *credentials*, not identity.
- **RST2 ticket**: opaque, stored `token ↔ email` with TTL; issued by `auth_service`.
- **Switchboard ticket**: new, opaque, short-lived, issued at conversation-announcement time, mapping `ticket ↔ (email, ConversationId)`; the wire `ANS` carries only this ticket. No Matrix identifier ever crosses the MSNP wire; tickets are revocable on account teardown.
- **MPOP simulation (D2)**: `Account.own_user` endpoint guid is derived (from email); the client's real guid (from CVR/USR) is stored per session and used where the client asserts identity. `sessions: Vec` with policy `max_sessions = 1`; raising the limit later fans presence/roster out over the slots — no structural change.

---

## 13. Persistence (`tachyon-sqlite-store`)

- Implements `CredentialStore` (existing credentials table, refactored from the 20 KB monolith into per-aggregate methods).
- New small tables: `ticket_tokens (token, email, created, expires)`, `switchboard_tickets (ticket, email, conversation_id, expires)`, `conversation_directory (conversation_id, room_id, kind)` (the directory in §9.4 is persisted, not rebuilt by scanning).
- SQLite access behind `spawn_blocking`; migration on startup (existing pattern).
- The domain never sees SQL; the app never sees table layouts.

---

## 14. Error handling

- Wire parse error ⇒ log + session teardown (no recovery, as today).
- Dialect error (illegal command for state) ⇒ protocol-local error reply where the protocol defines one (currently missing `ERR` codes are a known gap — the dialect's `DialectError` gives every handler one place to add them) else teardown.
- Domain error ⇒ log + targeted NAK/teardown; **no panics below the composition root**; transport write errors ⇒ session teardown (§8.1).
- Matrix SDK errors ⇒ domain policy decides (retry / alert / session teardown), never a transport panic.

---

## 15. Configuration

`config.ini` gains:
```ini
[tachyon]
enabled_versions = msnp18            # profile registry filter (future rollout valve)
[ports] notification = 1863, switchboard = 1864, http = 80
[account] max_sessions = 1               # D2 policy flag
```
Everything else (homeserver, strict_ssl, logs) unchanged.

---

## 16. Testing strategy (the safety net for every phase)

1. **Golden transcript tests per dialect** (highest leverage; built in Phase 0 before any refactor): recorded raw byte streams (client side) fed to dialect + domain (with fake `MatrixPort`); assert exact server bytes out + domain events. Fixtures start with the MSNP18 flows of §11 (handshake, ADL, MSG both ways, presence, file/voice), pinning the `PROFILES` data (§6.3) against today's behavior. No captures exist, so fixtures are derived from *observed current behavior* — they pin the status quo before refactoring, not wire truth.
2. **Dialect contract test**: for every enabled profile, handshake → ready, and every `MsnOutbound`/`SbOutbound` renders non-empty legal commands.
3. **Domain unit tests** with the fake `MatrixPort`: single-session policy, dedupe, offline re-queue, proxy group text-only, alert exactly-once.
4. **Existing e2e** (testcontainers Matrix + zathras) stays green after every phase — it is the acceptance gate.

---

## 17. Migration plan (each phase ships green: `cargo test` + e2e)

| Phase | What | Acceptance |
|---|---|---|
| **0. Characterize** | Golden transcript fixtures for MSNP18 (§16.1) against the *current* code; capture behavior byte-exactly. | Fixtures pass pre-refactor. |
| **1. Split crates** | Create `msn-models` (move `msnp::shared`); `msnp` depends on it. No behavior change. | Builds, fixtures pass. |
| **2. Domain skeleton** | Create `tachyon-domain`; move identity/roster/presence/conversation/messaging/delivery/alerts/policy + ports; `Account` API; legacy `TachyonClient`/`GlobalState` become thin shims over it. | Builds; e2e green; no logic changed yet. |
| **3. Dialect + NS session** | `msn/dialect` family C lifted verbatim from `negotiation.rs`/`auth.rs`/`ready/*`; `MsnpSession` replaces `LocalClientData`; NS session task per §8.2. | Phase-0 fixtures pass byte-identically; `negotiation.rs`/`auth.rs` deleted. |
| **4. Matrix adapter** | Extract §9; handlers → normalizers; **delete every `use msnp::` from matrix code**; sync loop emits `MatrixEvent`. | CI grep rule 2 passes; e2e green. |
| **5. SB + web rewire** | `SbSession` + opaque SB tickets (§8.3, §12); SOAP handlers → thin translations (§8.4); `connection_core` shared loop; bounded channels; no-unwrap writes. | e2e green; P7/P10 closed. |
| **6. Demolish god objects** | Delete `GlobalState`, `TachyonClient`; `Services` composition root; narrow dependencies everywhere; `tachyon-sqlite-store` implements ports + new tables. | e2e green; grep: no handler takes `GlobalState`. |


Ordering rationale: behavior is pinned before it moves (0); the leak-killers come before the god-object demolition (4 before 6) so each deletion is mechanically checkable.

**Out of this plan (by scope decision):** implementing other MSN generations. Their onboarding — family design notes, capability matrix, per-generation phases — is demoted to **appendix C**, to be picked up as separate efforts when a version is actually wanted.

**Status:** Phases 0–6 are a plan; **Phase 0 has not started** (approval pending).

---

## 18. Scope decisions (confirmed) & remaining risks

**Confirmed:**

1. **Other MSN versions are out of scope** for this overhaul — the architecture only needs to *accommodate* them (§6.4, appendix C). No protocol captures exist for other generations and none are needed for Phases 0–6; future onboarding records its own fixtures (appendix C, step C1).
2. **MSNP10 is out of scope** (as are 11–16 and 21); trivial to add later via the §6.4 procedure.
3. **Legacy SOAP/web endpoints** (`ppcrl*`, `sha1auth`, ads, MSN-Today) carry **no hidden version-sensitive state** — they stay thin pass-throughs (§8.4) with no version machinery.
4. **Phase 0 does not start now** — the §17 plan awaits approval.

**Remaining risks:**

1. **Fixture brittleness**: byte-exact golden tests on an evolving codebase can become maintenance noise. Mitigation: fixtures are small, per-flow, and only *handshake + one round-trip per domain op*; e2e covers integration.
2. **Phase 3 is the critical one** (lifting 18's behavior verbatim into the dialect). Mitigation: Phase 0 fixtures make "byte-identical" a checkable property, and `auth.rs`'s current inline flow maps 1:1 to the §9.3 table (no behavior invention).
3. **Not addressed here (deferred, by design):** native MSN group chat modeling (D3 keeps the proxy model), MPOP beyond the policy flag (D2), offline-message store growth, profile/avatar sync refinements, per-conversation read receipts. Each has a clean home in the domain and a defined extension point.

### Appendix A — current file → target mapping (excerpt)

| Current | Target |
|---|---|
| `notification/handlers/negotiation.rs` | `msn/dialect/profile.rs` + `msnp18::on_ver/on_cvr` |
| `notification/handlers/auth.rs` | split: dialect `msnp18::on_usr_*`; domain `auth_service`, `alert`; `matrix/client.rs`; domain post-login sequence (data) |
| `notification/handlers/command_handler.rs` + `models/local_client_data.rs` | `msn/ns_session.rs` + `MsnpSession` (dialect state machines) |
| `notification/handlers/ready/*` (adl, presence, …) | dialect ready-phase handlers + domain `roster`/`presence`/`messaging` |
| `notification/notification_server.rs` | `msn/connection_core.rs` + `ns_server.rs` (accept only) |
| `switchboard/switchboard_server.rs` | `msn/connection_core.rs` + `sb_server.rs` |
| `switchboard/handlers/bootstrap_handlers.rs` | `msn/sb_session.rs` + domain tickets/conversations |
| `switchboard/models/switchboard_token.rs` | domain `tickets` (opaque) + wire encoding in `msnp` |
| `tachyon/global_state.rs` | `services.rs` (`Services`) |
| `tachyon/client/tachyon_client.rs` | domain `Account` + NS `SessionSlot` |
| `tachyon/client/{messaging, repository}.rs` | domain `messaging`, `auth_service` |
| `matrix/sync.rs` | `matrix_adapter/client.rs` |
| `matrix/handlers/*` | `matrix_adapter/events.rs` (normalizers) |
| `web/soap/*` | thin SOAP handlers over domain `soap_query` (§8.4) |
| `msnp/src/msnp/shared/*` | `crates/msn-models` (models) + `msnp` (wire traits) |
| `tachyon-sqlite-store/src/lib.rs` | per-aggregate repositories + `CredentialStore` impl + new ticket tables |

### Appendix B — adjacent code health issues (fixed as part of Phases 3/5, listed so they aren't lost)

- 10,000,000-capacity SB mpsc (vs 300 for NS) → bounded 256 + policy (§8.1).
- `write_all(...).await.unwrap()` in both write loops → teardown on error.
- `from_utf8_unchecked` in debug logs → `from_utf8_lossy`.
- `rayon` parallel iterators in async server code → remove (blocking in async).
- `println!` in spawned writer task → `log`.
- `ROOM_USER_PORTAL_MODE` global const → `ConversationKind` (§7.4).
- `.expect("To be here")` in SB bootstrap → domain ticket validation returns a typed error.
- TODO "send error back to client" in several SB/NS handlers → `DialectError` mapping (§14).

### Appendix C — future protocol version onboarding (out of scope for this overhaul)

Material for when another MSN generation is actually wanted. Procedure: §6.4. Acceptance criterion: the last paragraph of §6.4 (steps 1–5 are the entire change; MSNP18 fixtures stay green).

**Family design notes** (from the audit — verify against captures at onboarding time):

- **MSNP10–12** (Messenger 6.x): `SYN`/`LST` roster, `ADD`/`REM`, MD5/TWN auth, no `ADL`, no SOAP ABCH, no GCF. A drastic break from everything later. (MSNP10 itself is out of scope even for future effort — trivial to add as a profile.)
- **MSNP13–16** (WLM 2005 → 8.5 SP2): `ADL`/`RML` + BLP, ticket/SSO auth, SOAP ABCH, GCF, UUX/OIM/MSGR/UBX appearing across 14–16.
- **MSNP21** (WLM 2012): SOAP-heavy rework, presence-model changes.

**Capability matrix** (fill from captures at onboarding time; "verified" = MSNP18 current behavior):

| Feature | 10–12 | 13–16 | 17–18 | 21 |
|---|---|---|---|---|
| CVR | ? | ✓ | ✓ verified | ? |
| Auth | MD5 / TWN | TWN → SSO | SSO I/S + GCF/SHIELDS — verified | SSO I/S (+ extensions?) |
| Roster sync | SYN/LST | ADL/RML | ADL/RML — verified | ADL/RML (SOAP shift?) |
| Contact add/remove | ADD/REM | ADL/RML + SOAP ABCH | ADL/RML + SOAP ABCH — verified | ? |
| GCF | — | 13+ | ✓ | ✓ |
| UUX / OIM | — | 15+ | ✓ | ✓ |
| MSGR (ext. presence) | — | 16+? | ✓ | ✓ |
| UUM / UBX | — | ? | ✓ | ✓ |
| WIMP | — | — | ✓ | ? |
| P2P | v1? | v2 | v2 — verified | v2 |
| SOAP ABCH / RST2 | — | ✓ | ✓ — verified | ✓ |
| Group chat | legacy SB groups | ? | ✓ (proxy model, D3) | ? |

**Onboarding phases** (per generation, separate effort):

| Step | What | Acceptance |
|---|---|---|
| C1 | Record protocol captures (Wireshark/zathras); fill the capability-matrix column | No unverified cells for the target version |
| C2 | `MsnpVersion` variant + `MsnpProfile` const | Builds; profile data matches captures |
| C3 | Dialect module: dedicated handshake state machine + ready-phase handlers + `render_*` | Dialect contract test: handshake → ready |
| C4 | Transcript fixtures: handshake, roster sync, message both ways, presence, file/voice clip | All fixtures green |
| C5 | Enable in config; e2e with a generation-appropriate client | e2e green; MSNP18 fixtures still green |
