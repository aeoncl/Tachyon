# Tachyon — Architecture: functional core, dialect-aware shell

**Status:** EXECUTED — all stages D0–D6 landed (approval `03d7daf`, 2026-11-05;
execution tip before this update: `fea244e`). This document now describes what
was built: §8 carries the per-stage commit ledger, §12 the executed outcome,
and the evidence tables (§1.1, §1.4) carry built dispositions. It supersedes
the previous top-down proposal (`msn-models` / `tachyon-domain`
re-cratification), deleted unapproved in `f32ab61`: that document imposed
abstractions first; this one extracted machinery only where existing call sites
already demonstrated the need. Before the approval date no code was refactored
toward this document, per AGENTS.md.

**Companion:** `doc/design-decisions-history.md` records how the code got here;
this document recorded where the plan proposed to go and — since D6 — what
actually landed. Every design claim below cites the file:line or commit that
motivates it. File:line references in the evidence sections (§0–§1) are
historical, pinned to `f32ab61` (the pre-D0 tree); built-state references were
spot-verified at the D6 tip. The contributor-facing mirror of the seam
inventory and the honest gates is `doc/adding-a-dialect.md` (shipped by D5,
`6467e32`).

**What this plan bought — demonstrated at HEAD, no more (D0–D5):** a
compiler-enumerated dialect seam — four inventoried sites (§3.3), the
enumeration proven by the executed scratch-variant exercise (`cc5570b`, §9
item 1); socket-free table tests — the `tachyon` lib suite grew from ~27 to
114, the `msnp` baseline from 212/4 to 217/4 (§11); and the retired hazards of
§1.4 (H1–H9, each with its disposition and commit anchor). Passthrough
honesty: most D2b verb migrations are thin `Command(..)` wrappers — the
demonstrated benefit is socket-free tests plus one compiler-enumerated seam,
**not** "every verb has a decision". *Files-touched-per-dialect* remains the
post-release metric, measured when the first contributor dialect lands (§7 M1,
§9 item 3). **Expected runtime change: none** — this is a localhost,
human-scale, I/O-bound bridge (§9 item 5).

**First-release scope:** MSNP18 only (product decision). The dialect *structure*
— negotiated-version storage, per-dialect profile data, seam matches,
conformance harness, contributor contract — must be ready before release so
contributors can add versions; **no second dialect ships in v1** (MSNP21 is
deferred to §7 M1); older (pre-MSNP18) clients are likewise post-release
(§7 M2), with their auth shape already decided (§3.7): the same web password
page, browser launched by Tachyon where the client cannot act on the
in-protocol alert — v1 ships the seam, not the arm. Structure means seams, not
stubs (§6 D).

---

## 0. What the program is

One sentence the architecture must read like:

> Bytes from an unmodified WLM 2011 client arrive on three local sockets
> (Notification Server TCP, Switchboard TCP, HTTP/SOAP) → `msnp` parses them into
> verb enums → handlers matched by connection phase translate them into Matrix
> SDK calls; Matrix sync events flow back through typed handlers into per-client
> command queues → rendered to bytes and written out.

```
WLM 2011 ── NS   :11863 ─► verb enums ─ phase match ─ handlers ─► Matrix SDK
         ── SB   :11864 ─►                                      ▲
         ── HTTP :11866 ─►                                      │
Matrix HS ── sync ─► event handlers ─► per-client queues ───────┘─► bytes
```

Tachyon impersonates the whole MSN server side so the client stays unmodified
(decisions record §1). The spine is already close to what we want: free-function
verb handlers, `match` dispatch over enums, no DI container, no event bus, one
thin composition root (`crates/tachyon/src/lib.rs::run`). `msnp` is a pure sync
codec — no tokio/futures anywhere in its dependencies — with 67 `IntoBytes` and
49 `TryFromRawCommand` impls pinned by byte-exact round-trip fixtures (counts
unchanged through D5). The whole tangle this plan addressed is confined to
`tachyon` (~12k LOC at plan time).

## 1. Why change (evidence)

Two requirements drive everything below:

1. **IO is coupled to business logic on both fronts.** On the MSN side every
   handler signature carries a `Sender<…>` and writes to the socket mid-policy;
   on the Matrix side the event-handler closures call the SDK inline.
2. **More MSNP dialect versions must be supported — structure-first.** The
   first release ships MSNP18 only, but the seams for additional versions must
   exist before release so contributors can add them. Differences are of two
   kinds: sometimes only command shapes change; sometimes behavior does —
   MSNP21 has no switchboard at all.

### 1.1 The same translation exists four times — and has drifted

| Site | Sends text to a room via |
|---|---|
| `switchboard/handlers/ready/msg_handler.rs:79` | `send_with_dedup` (echo-suppressed) |
| `notification/handlers/ready/uum_handler.rs:61` | plain `room.send(content)` — reads as: the UUM path bypasses the dedup extension, so the sender's own echo is not suppressed. Observed gap (no runtime repro taken); retired in D3, not hotfixed |
| `web/soap/ab_service/ab_contact_add.rs:68,77` | a third inline copy |
| `matrix/handlers/*` (reverse direction) | re-implements the mapping and reaches into `switchboards` / `notification_handle` directly |

Every future dialect fork would duplicate all four.

**Built disposition (D3, `643bd73`):** all four sites now route through the
single `RoomOp::SendText` executor — `apply_room_op`
(`matrix/extensions/message_dedup.rs:104`), the only place the event-handler
front touches an SDK text send, so there is exactly one dedup path. Named
behavior delta: the UUM path's plain send gained own-echo suppression and the
per-room send-lock serialization it previously bypassed (`uum_handler.rs:77`);
the SB msg path (`msg_handler.rs:243`) and the two `ab_contact_add` sites
(`ab_contact_add.rs:71,82`) already deduped and now ride the shared executor.
The gap was retired by consolidation, not hotfixed.

### 1.2 IO fused with policy — the specimen

`notification/handlers/ready/chg_handler.rs`, one function: acks over the socket
(:19), spawns a detached task (:27), does three Matrix lookups per contact
(:40–54, with an `.unwrap()` at :54 that silently kills the spawned task on
failure), renders ILN+UBX (:65–86). Four jobs fused, every one of them
dialect-sensitive.

**Built:** `chg` was split in D1 (`f0070e4`) into shell / `decide_chg`
(`chg_handler.rs:123`) / `render` (the seam at `:152`) — the reference pattern
every later migration copied.

### 1.3 Dialect support is structurally absent

`notification/handlers/negotiation.rs:12` accepts only MSNP18 and answers `OUT`
to anything else; the CVR strings (`"14.0.8117.0416"`, `"localhost"`) are
hardcoded at :24. The wire vocabulary is already ahead of the behavior:
`MsnpVersion` (`msnp/notification/models/msnp_version.rs`) declares MSNP17,
MSNP18 **and MSNP21**, and strum parses all three — the enum exists as data,
never as behavior; a single negotiation `if` decides what is served. Meanwhile
the MSNP21 verbs already parse in `msnp` (`sdg.rs`, `ubm.rs`, `uun.rs`,
`uux.rs`),
and `uum_handler.rs` is the NS-IM path half-built — its `todo!()` arms
(TypingUser, Nudge, Raw) are the MSNP21 work list already written. Protocol
semantics must be confirmed against captures / Escargot before post-release
MSNP21 work (§7 M1) builds on them.

**Built (D1 `f0070e4`, D5 `515edc3`):** VER picks the highest mutual version
that has a profile (`dialect_for` lookup, `negotiation.rs:17`); the CVR
strings are profile data in `msnp/src/msnp/notification/models/dialect.rs`
(`profile()` at `:37`), pinned by its test mod. `uum_handler.rs`'s `todo!()`
arms (TypingUser, Nudge, Raw) are still `todo!()` at HEAD (`:85–94`,
documented as such at `:25`) — the UUM migration (`5def2d5`) moved the verb to
the decide seam without inventing v21 behavior; they remain M1's work list.

Per product decision, MSNP18 is the only dialect in the first release: the seams
must exist and be proven on MSNP18 itself, so a contributor's dialect work lands
as additions at compiler-enumerated points (§3.3), not as archaeology through
handler bodies.

"Additional versions" spans both directions — newer (MSNP21, §7 M1) and older
(pre-MSNP18 clients via the browser-auth decision, §7 M2) — and both land as
additions at the same compiler-enumerated seams.

### 1.4 Carry-over hazards (fixed once, in place, during the migration)

| # | Hazard (as read in code) | Site |
|---|---|---|
| H1 | `from_utf8_unchecked` on wire bytes that legitimately carry binary (P2P MSG blobs) — UB inside `debug!` | `notification_server.rs:145`, `switchboard_server.rs:104`, `msnp/p2p/v2/mod.rs:140,177,221,405` |
| H2 | SB write task panics on a dead socket: `write_all(&bytes).await.unwrap()` | `switchboard_server.rs:189,194` |
| H3 | NS write task silently ignores write errors — opposite policy, same failure | `notification_server.rs:149` |
| H4 | SB channel bound 10,000,000 vs NS 300 — a made-up bound that reads as bounded | `switchboard_server.rs:164` vs `notification_server.rs:135` |
| H5 | Dead parallel state reading as live: `AlertRepository` (0 callers), `Repository<K,V>` (0 impls after it), `GlobalState.pending_alerts` + accessors (0 callers); the live alert path is `tachyon_client.alerts()` | `tachyon/alert/alert_repository.rs`, `tachyon/repository.rs:7`, `global_state.rs:17,94–100` |
| H6 | Ritual deps: `rayon` imported, zero `par_iter` uses repo-wide; `console-subscriber` used only in a comment; `msnp18` cargo feature with zero `cfg` references | `switchboard_server.rs:14–15`, `main.rs:18`, `msnp/Cargo.toml` |
| H7 | Process-global `lazy_static` caches — unowned, never reset, shared across accounts; `DEDUP_STATE` is keyed by room only, so a second account in the same room has its sends suppressed and its echoes misattributed | `msn_user_resolver.rs:19–20`, `message_dedup.rs:12`, `message_handlers.rs:27` |
| H8 | `get_single_client()` (with its own FIXME) hides a single-account agreement; `single()` returns `None` at ≥2 clients → profile pics break the moment a second account signs in | `global_state.rs:76–79`, `web_endpoints.rs:87`, `tachyon_client_repository.rs:18` |
| H9 | SB teardown reads as a chain: write-task panic (H2) → `client_kill_snd.send(())?` fails → `?` skips `switchboards().remove(&room_id)` → stale `SwitchboardHandle` leaks | `switchboard_server.rs:141–148` |

**Built dispositions (all nine retired; anchors verified at the D6 tip):**

- **H1 — retired for its two *active* sites (D2a, `03383fc`):** checked
  `from_utf8` with a `<binary N bytes>` fallback at both live debug-log sites
  (`notification_server.rs:165–170`, `switchboard_server.rs:103–108`). The
  four `msnp/p2p/v2/mod.rs` sites were never active UB: all nine
  `from_utf8_unchecked` hits in that file are dead text inside the
  block-commented test region (`:127–424`) — a known-deferred
  comment-accuracy item (`msnp` is frozen by policy), recorded, not fixed.
- **H2 + H3 — retired (D2a, `2040958`):** both write loops warn and exit via
  a labeled break (`'write_loop`, `switchboard_server.rs:189–225`; same policy
  on the NS side, `notification_server.rs:158–178`) — one failure policy on
  both servers, no `unwrap()`, no silent ignore.
- **H4 — retired (D2a, `3a62018`):** one `COMMAND_CHANNEL_BOUND: usize =
  1024` (`lib.rs:23`) + `send().await` on both servers
  (`notification_server.rs:155`, `switchboard_server.rs:186`); §8's
  sub-decision **(b) chosen** — bounded, never drops.
- **H5 — deleted (D0, `44e722f`):** `alert_repository.rs`,
  `Repository<K,V>` and `GlobalState.pending_alerts` + accessors are gone;
  `RepositoryStr` was inlined onto its one impl.
- **H6 — dropped (D0, `44e722f`):** `rayon`, `console-subscriber` and the
  dead `msnp18` cargo feature are gone from the manifests.
- **H7 — retired by ownership (D3, `643bd73`):** `DedupState` and
  `RoomHashCache` became `TachyonClientInner` fields
  (`tachyon/client/tachyon_client.rs:65,67`) — the 3 process-global mutable
  maps went to 0, and cache lifetime is now exactly the signed-in account's
  lifetime. This *extends* `68f3a89`'s one-registered-object pattern; no
  parallel registries were added.
- **H8 — retired (D3, `26a6604`):** the usertile URL is keyed by client —
  the sha256-hex `BridgeLinkToken` (`tachyon/identifiers/bridge_link.rs:65`),
  the pre-existing SOAP repository key, extending `506eab3`'s
  tickets-are-not-secret localhost trust model. Renderer:
  `msnp/src/soap/storage_service/get_profile.rs:263–277`; route
  `/storage/usertile/{client_key}/{image_mxid}/{image_type}`
  (`web_server.rs:52`); caller `storage_service.rs:46–50`;
  `get_single_client()` deleted. Deliberate fixture update
  (`get_profile.rs:314`). **Honesty note: no runtime e2e coverage — the msnp
  fixture pin carries this fix.**
- **H9 — retired (D4, `9dd3da7`), with the honest reachability finding:**
  post-D2a the leak chain was *structurally latent*, not live —
  `LocalSwitchboardData` parks its own never-read broadcast receiver, and
  tokio's broadcast `send` errors only at zero receivers, so the kill-send
  could not actually fail while the connection tail ran. The fix makes the
  cleanup unconditional **by construction** regardless:
  `teardown_switchboard()` was hoisted *before* the kill-send and extracted
  into its own function (`switchboard_server.rs:167`), and the kill-send now
  warns instead of `?`-skipping cleanup. The hoist also narrows the window in
  which a concurrent `switchboards().insert()` (with its pending-events
  merge) could be evicted by a later-running remove. Socket-free tests pin
  the invariant; the full tail stays e2e-anchored.

## 2. Non-negotiable constraints (decisions record + AGENTS.md)

- `msnp` fixture baseline: the four known-reds (msnobj URL-encoding,
  `nln`/`iln` — recorded in AGENTS.md, `e738db8`; names verbatim in
  `doc/adding-a-dialect.md` §2) are **invariant and never fixed** as a side
  effect of this plan; no fifth red. The pass count grew 212 → **217** by
  deliberate, attributed fixture additions (+2 D0 `44e722f`, +1 D3
  `26a6604`, +2 D5 `515edc3`) — §9 item 4.
- Byte-exact round-trip fixtures: changing a renderer means a deliberate fixture
  update in the same commit.
- Never `cargo fmt --all`; match surrounding style; inline prose-named tests;
  Conventional Commits, lowercase.
- Channel-policy rationale (`17991e8`): a slow socket never drops protocol
  events.
- One registered object with an internal state field (`68f3a89`:
  `Backend::Pending | Ready` inside `TachyonClient`) — no parallel registries.
  This plan extends that pattern; it never regresses it.
- E2e pin: `cargo test -p tachyon --test e2e_msn_homeserver -- --ignored`
  (Docker, env-driven), including the SOAP-degrades-while-pending contract
  (`433cb67`) — now **four** `--ignored` tests: the three classic flows plus
  `msnp18_dialect_conformance_script` (`cc5570b`).
- CI stays deleted (`45467fa`). The only new harness was the D5 dialect
  script, reusing the e2e fake-client machinery — shipped same-file in
  `e2e_msn_homeserver.rs` (`cc5570b`).

## 3. Target shape: gather → decide → render → execute, per flow

No trait tower, no ports-and-adapters, no new crate. Two small plain-data
vocabularies, one capability enum and one profile table — that is *all* the new
machinery.

```
socket bytes ─► msnp parse ─► decide() ─Vec<ClientOut>─► render(dialect) ─► existing write task
                                     │
Matrix event ─► shell/gather ─► fact ┴─Vec<RoomOp>─► apply() ─► Matrix SDK
```

### 3.1 Handlers stop taking `Sender<…>`

That parameter is why they can't be pure — today they write to the socket
mid-policy. Decision functions become:

```rust
fn decide_chg(cmd: ChgClient, state: &mut SessionState, dialect: Dialect)
    -> Vec<ClientOut>;
```

The shell gathers inputs, calls decide, renders (`match dialect`), and sends
through the existing `NotificationHandle` / `SwitchboardHandle` sinks — which
already serve both producers (read loop and Matrix side). The write-task
machinery stays; only the producers change.

**Built shape:** signatures varied per flow — `chg`'s ended as
`decide_chg(gathered: Vec<GatheredContact>) -> Vec<ClientOut>`
(`chg_handler.rs:123`); the plan snippet's `cmd`/`state`/`dialect` params
proved unused and were dropped (`2c0b81b`). The dialect is consumed at the
render seam, not inside decide. Two Sender disciplines coexist, named in the
D2b commits: **pattern A** — `Sender` leaves the handler signature entirely
(decide/render are socket-free); **pattern B** — a sanctioned detached spawn
keeps holding its `Sender` (`adl`'s avatar fan-out, `5193cfc`; SB `ready`/`msg`
in D4, `89c9494`).

### 3.2 The vocabularies (plain data)

```rust
enum ClientOut {           // toward MSN; produced by pure decide fns
    DeliverText { to: EmailAddress, text: String, format: FontFormat }, // semantic
    ContactPresence { user: NetworkIdEmail, status: PresenceStatus, /* .. */ },
    Ping,
    Command(NotificationServerCommand), // escape hatch
}

enum RoomOp {              // toward Matrix; executed by apply()
    SendText { room: OwnedRoomId, content: RoomMessageEventContent },
    Invite { room: OwnedRoomId, mxid: OwnedUserId },
    Typing { room: OwnedRoomId, on: bool },
}

// Two version types, one job each (see §3.3): MsnpVersion (EXISTING, msnp) is
// what the wire can say; Dialect (NEW, msnp) is what we can serve
enum MsnpVersion { MSNP17, MSNP18, MSNP21 }   // strum-parsed from VER candidates
enum Dialect { Msnp18 /* Msnp21 in M1, .. */ } // negotiated once, stored in SessionState
```

**The escape-hatch rule is load-bearing:** a `ClientOut` variant is promoted to
semantic *only on demonstrated difference* — for v1 that means MSNP18's own
drift (§1.1: four divergent send paths; §1.2: fused presence rendering) or a
recorded product decision with a named, deferred second renderer (§3.7's
web-login trigger); never speculation. Everything else rides `Command(..)`.
Promoting a variant is a decision to remove the escape hatch for that behavior
and needs the same evidence bar as any abstraction. This is the valve that keeps the vocabulary
from ballooning into a universal model nobody has evidence for.

**Built state — the valve held:** `ClientOut`
(`notification/models/client_out.rs:12`) shipped exactly three variants:
`ContactPresence` (the §1.2 fused-presence difference), `Command(..)` (what
every D2b verb migration rides — the passthrough honesty of the header), and
`DirectToWebLogin { url }` (§3.7's recorded product decision; D2c `8871758`).
The snippet's `DeliverText` and `Ping` never earned promotion. On the Matrix
front, `RoomOp` shipped `SendText`-only (`message_dedup.rs:97`), and the
event-handler front got `MessageOp { Sb, ReceiveFile }`
(`message_handlers.rs:37`) — one or two ops per event, dying in `apply()`;
boxing the transfer variant was rejected as indirection with no measured
benefit (§9 item 5).

### 3.3 Dialect is runtime data, negotiated once

Built (D1 `f0070e4`, D4 `0015cbe`): `negotiation.rs:12`'s hardcoded `==
MSNP18` check became "pick the highest mutual version that has a
`DialectProfile`" — a `dialect_for` lookup (`negotiation.rs:17`). The result
is stored **once** in `SessionState`
(`notification/models/session_state.rs:11`) and copied into
`TachyonClientInner` at auth — exactly one write, `set_dialect` at the NS
SSO-S registration **before** `insert_clients` makes the client discoverable
(`auth.rs:73`), held as `dialect: RwLock<Dialect>`
(`tachyon/client/tachyon_client.rs:76`) — one owner, no parallel
"is-sb-enabled"-style flags anywhere. The CVR strings (pre-D1
`negotiation.rs:24`) are profile data in
`msnp/src/msnp/notification/models/dialect.rs`, landed in the same commit as
negotiation because negotiation itself consumes them. No global dialect flag;
no `if dialect >= 21` anywhere — one `match dialect` at the top of the
affected flow's dispatch/render.

```rust
struct DialectProfile {        // plain data in msnp, one record per Dialect variant
    dialect: Dialect,
    cvr_version: &'static str, // pre-D1 hardcoded at negotiation.rs:24
    cvr_host: &'static str,    // ditto
    max_msg_body_size: usize,  // joined at D5 (515edc3, = 1311, dialect.rs:43);
    // the file-local MAX_MSG_BODY_SIZE const at switchboard_handle.rs was
    // PHYSICALLY RETIRED in the same commit — single source of truth, the
    // chunker byte-identical. Stored at construction
    // (SwitchboardHandle::new(.., max_msg_body_size), switchboard_handle.rs:47;
    // consumer reads dialect().profile().max_msg_body_size,
    // switchboard/handlers/mod.rs:98) because the P2P transport holds only
    // the handle (TransportSender::SBBridge, p2p/client/transport.rs:41).
    // Auth scheme / verb set / config blobs join only with a real second
    // dialect — fields move with their consumers, never ahead of them
}

fn dialect_for(v: MsnpVersion) -> Option<Dialect> // one exhaustive match:
                                                  // the wire→capability seam
```

**Parse vocabulary ≠ serve capability — and that is not a parallel registry.**
`MsnpVersion` (existing: MSNP17/18/21) is what clients may *say* — MSNP17 must
stay parseable because WLM 2011 offers it as second candidate (pinned by the
`command_handler` handshake fixture), and MSNP21 stays parseable so a v21 client
earns a clean `OUT` instead of a parse error. `Dialect` is what Tachyon can
*serve* — exactly one variant at release. A single mapping function connects
them; `SessionState` stores only `Dialect`; no call site ever consults both.
`msnp` stays a pure sync codec with zero new deps: the profile is data beside
the enum, which is what keeps the codec crate contributor-safe.

**"Structure ready" has a mechanical definition at release:** `Dialect` has
exactly one variant; every seam is an exhaustive `match dialect` over it, with
no `_` wildcard and no `#[non_exhaustive]` — either one defeats the compiler
enumeration the whole structure exists for. The day a contributor adds
`Dialect::Msnp21`, the compiler prints their work list: every site that needs a
decision — no registry, no runtime dispatch, no hidden agreements.
Single-variant matches are deliberate: the seam exists, speculative arms do not.

**Seam discipline (the anti-Option-B tripwire):** every `match dialect` site
names, in a comment, the concrete difference it anticipates ("MSNP21 has no
switchboard"). A site that cannot name one is not a seam — inline v18 and delete
it; re-add it with the contributor who needs it. The inventory of sites is
maintained in this document (each §8 stage records the sites it adds); budget:
**≤6 sites in `tachyon`** — negotiation/profile, auth (CVR strings + web-login
trigger, §3.7), deliver, presence render, message path, session-lifecycle/XFR.
A seventh requires deleting one or stating why the budget moved.

**Seam site inventory — final built state (4 of ≤6; mirrors
`doc/adding-a-dialect.md` §1, anchors verified at the D6 tip):**

- **Site 1 (D1)** `crates/msnp/src/msnp/notification/models/dialect.rs` —
  `Dialect::profile()` (`:37`), the negotiation/profile-lookup site, with the
  `dialect_for` mapping beside it (`:52`); reached from the VER and CVR arms
  in `negotiation.rs` (`:17`). The CVR strings and `max_msg_body_size: 1311`
  are profile data here. Named difference: MSNP21 serves SDG/UUM over the NS
  and has no switchboard (§7 M1); pre-MSNP18 clients need the browser-auth
  trigger (§7 M2).
- **Site 2 (D1)** `crates/tachyon/src/notification/handlers/ready/chg_handler.rs:152`
  — `render()`, the chg presence-render site. Named difference: the MSNP21
  presence payload / UBX endpoint-data shape differs, and MSNP21 has no
  switchboard (§7 M1).
- **Site 3 (D2c, `8871758`)** `crates/tachyon/src/notification/handlers/auth.rs:191`
  — `render_auth_trigger()`, the auth web-login trigger site (§3.7). Named
  difference: v1 renders the in-protocol NOT alert; M2's arm launches the
  host browser for pre-MSNP18 clients that cannot act on the alert (§7 M2) —
  the fork is the trigger only, never the login.
- **Site 4 (D4, `0015cbe`)** `crates/tachyon/src/matrix/handlers/message_handlers.rs:351`
  — `deliver()` (fn at `:350`), the SB-vs-NS delivery site (§3.4). Named
  difference: the classic arm dispatches the decided ops through the room's
  switchboard handle (today's SB/XFR flow); MSNP21 has no switchboard — the
  M1 arm delivers the same ops as SDG/UUM over the NS writer and refuses XFR
  instead (§7 M1) — the fork is the transport only, never the
  translation/dedup/room resolution shared above the arms.
- **Two budget slots stay reserved, deliberately unbuilt:** the *message
  path* and *session-lifecycle/XFR* slots of the ≤6 budget have no sites.
  The UUM/NS-IM path (`5def2d5`) and the `xfr` migration (`18ee489`) landed
  dialect-free on purpose: their only anticipated fork is transport, which
  site 4 already owns (the M1 arm refuses XFR there). A new seam at either
  slot requires naming a difference site 4 cannot carry.

Sites 2–4 are the exactly-three `match dialect` matches in
`crates/tachyon/src` (grep-verified at the D6 tip); site 1 lives in `msnp`.
The scratch-variant exercise (`cc5570b`, §9 item 1) demonstrated the
inventory is compiler-enforced: all four sites light up, zero non-inventoried
sites do.

### 3.4 The switchboard question dissolves instead of being abstracted

`deliver(dialect, client, room, out)` is a function, not a trait — built
exactly as specified (D4, `0015cbe`; `message_handlers.rs:350`, seam match
`:351`):

- at release there is exactly **one arm**: today's SB/XFR flow —
  `SwitchboardServer` keeps listening, classic clients need it;
- constructed **and** called from day one: two production call sites pass
  `client.dialect()` (`message_handlers.rs:119` — the room-message flow;
  `:395` — the typing flow);
- `deliver()` exists as the seam anyway, because SB-vs-NS dispatch is where a
  second dialect provably forks (MSNP21 has no switchboard). The second arm —
  SDG/UUM over the NS writer, XFR refusal for v21 — lands in §7 M1; the fork
  point is named, owned by one function, and compiler-checked the moment
  `Dialect::Msnp21` exists (demonstrated by the scratch exercise, `cc5570b`).

Nothing is deleted when v21 lands; it is purely additive. The shared body
(translation, dedup, room resolution) sits above the arms.

### 3.5 State split

`LocalClientData` mixed decision state (phase, endpoint data, email, token)
with shell state (shutdown channels, drop guard). Split into `SessionState`
(decision inputs, including `dialect`) + shell-owned fields — landed as its
own mechanical commit (D1, `c4b0b97`).

The process globals became owned fields of `TachyonClientInner`
(`room_hashes: RoomHashCache`, `dedup: DedupState` —
`tachyon/client/tachyon_client.rs:65,67`; D3, `643bd73`): cache lifetime is
now exactly the signed-in account's lifetime, and H7 was fixed by ownership,
not by a compound key. The one `RoomOp::SendText` executor
(`message_dedup.rs:104`) means one dedup path, which retired the §1.1 echo
gap instead of patching it — with the named delta recorded there (the UUM
path gained suppression + serialization it previously lacked).

### 3.6 Spawns are shell policy, not effects

`msg_handler.rs:62–68`'s ordering rule (P2P packets handled inline because chunk
reassembly breaks if reordered; everything else spawned) stays exactly that: a
rule written once in the shell. `sync_with_server_task` /
`spawn_session_watcher` likewise — the auth shell knows "on Ready, start sync +
watcher". No `Out::StartTask` ceremony; a spawn "requested" by a decide fn is a
smell that the flow was split wrong. Don't spawn a task per effect: sequential
read→decide→apply in the connection task is the least-concept version; the write
task exists only because Matrix events arrive from another task and must
interleave.

**Built:** the rule held through both front migrations — the P2P-inline
ordering rule and the chunk-reassembly state stayed in the SB shell when
`ready`/`msg` migrated (D4, `89c9494`), and the bootstrap's "on Ready, start
sync + watcher" wiring stayed shell-side as `ReadyWiring` executed by the
dispatcher after the sends, preserving the pre-D4 wire order including
`set_state(Ready)`'s pending flush.

### 3.7 Auth: the ticket is the boundary; only the trigger forks

Today's MSNP18 flow, read-verified: `USR S S I` → SSO policy + hardcoded nonce
(`auth.rs:38`); the client returns with a ticket at `USR S S S` → `TachyonClient`
registered **before** the OK (`auth.rs:51-63`, `68f3a89`); then
`spawn_restore_or_toast` either restores a saved session or **fires a NOT
alert** — "Sign in required - click here to continue", URL = the web login page
keyed by the ticket token (`auth.rs:132-155`). The page collects Matrix
credentials (password form or OAuth SSO, `login.rs:64-115`) and calls
`attach_backend` (first-caller-wins); the toast is fire-and-forget — nothing
blocks on it (`login.rs:310`).

**Product decision (recorded):** pre-MSNP18 clients that cannot act on the
in-protocol alert get the same password auth page with **Tachyon opening the
browser itself**; the flow after the page is identical to MSNP18's. The server
never receives, stores, or verifies an MSN password in-protocol —
challenge-hash *verification* is off the table, consistent with the
tickets-are-not-secret localhost trust model (decisions record §1).

- `TicketToken` is the boundary: everything downstream is dialect-independent
  shared machinery (`MatrixLoginService`, `Backend::Pending→Ready`, watcher,
  sync, presence). Auth is the one place a dialect may fork, and what forks is
  **the trigger**: in-protocol NOT alert vs host-launched browser.
- The trigger is behavior, not command shape → one `match dialect` arm at the
  already-budgeted auth site (§3.3), **not** a `DialectProfile` field: a data
  field would let a new dialect compile with the wrong trigger silently
  constructed; a match arm is a compiler error = a contributor work-list item.
- Opening a browser is host IO → shell only (§3.6 discipline): decide fns emit a
  semantic `ClientOut::DirectToWebLogin { url }` (the URL carries the ticket
  token so the page finds its session); the v1 renderer arm produces today's NOT
  alert, M2's arm launches the host browser. Acceptable only because listeners
  bind 127.0.0.1 — auto-launching a browser in response to a remote network
  event would be phishing-shaped.
- The same NOT-alert trigger pattern also drives device verification
  (`auth.rs:176-183`); M2's capture work decides whether the legacy arm
  generalizes to every "open this page" alert or only to login.
- v1 ships the seam single-arm; the legacy arm lands with M2 (§6 D — no stubs).
  The dormant primitives (`Alert::new_weblogin` / `WebLoginAlertContent` — zero
  call sites, no expiry fields unlike ConfirmDevice's) stay available but are
  not load-bearing: the live flow awaits no alert, and whether M2 needs one is
  a capture decision.

**Built (D2c, `8871758` + `b5e8c3a`):** `decide_web_login_trigger`
(`auth.rs:170`) emits `ClientOut::DirectToWebLogin { url }` — the URL carries
the ticket token; `render_auth_trigger` (`auth.rs:188`, seam match `:191`) is
site 3. The Msnp18 arm reproduces today's NOT alert byte-identically
(oracle-first pin: captured against the pre-rewiring path, passed twice for
determinism, reproduced unchanged after — the seam moved no wire byte). The
new variant collided with three exhaustive 2-arm `ClientOut` matches;
wildcards being barred, the collision was resolved with **three
defect-canary arms** — chg render (warn + `vec![]`), the adl send loop (warn
+ continue), and `send_client_outs` (warn + skip,
`notification/handlers/ready/mod.rs:75`) — loud, unreachable by construction,
and documented as such at each site. The ruling: compiler enumeration
preserved, no wildcards, no panic arms (`8871758`). The `usr` migration
(`b5e8c3a`) completed the front.

## 4. What stays, and why (demonstrated benefit, not taste)

- **The 3-crate seam** (`msnp` / `tachyon` / `tachyon-sqlite-store`): real and
  pinned. Merging `<Verb>Client`/`<Verb>Server` pairs is churn across ~28k
  fixture-pinned LOC with no measured gain.
- **`Box<dyn MatrixLoginService>`** — the only dyn in the repo, two real impls
  (prod + `TestMatrixLoginService`); it is what makes
  `handshake_happy_flow_test` / `auth_i_test` run against `MatrixMockServer`
  with no homeserver. A mockability interface that has demonstrated its benefit
  stays.
- **`TachyonClient` with `Backend::Pending | Ready`** — the single-registry
  pattern (`68f3a89`); §3.3 extends it.
- **`SwitchboardHandle`** (`Initializing | Ready{sender}` + buffer-then-flush).
- **Extension traits** (`ToRoomId`, `ToEmailAddress`, `FindRoomFromEmail`, …) —
  idiomatic, zero-cost static dispatch; deleting them is churn without a metric.
  Exception: `PresenceStateMapper` (single impl, pure `match`) may become a free
  function in D0 if it falls out for free. (It didn't — D0 `44e722f` left it
  alone, so it stays, by its own demonstrated-benefit bar.)

## 5. Two kinds of dialect difference, two mechanisms

| Kind of difference | Mechanism | Example |
|---|---|---|
| **Command shape** — renamed verb, param order, payload encoding, new/removed verbs | Data in `msnp`: per-dialect verb files/structs (the per-verb layout is made for this; pre-ABCH dialects would need `syn`/`lst`/`lsg`), plus a `DialectProfile` **struct of data** (verb set, body-size limits, auth scheme, CVR/config blobs). Never generics over a version parameter — that infects every call site | MSNP21 `USR` payload |
| **Behavior/topology** — no switchboard, different auth flow, different conversation lifecycle | One `match dialect` at the top of the affected flow's dispatch/render. If the paper exercise (§5.1) shows the phase machines genuinely fork, duplicate into `dialect::classic::run` vs `dialect::v21::run`, sharing codec, vocabularies and apply | MSNP21 conversation flow; legacy web-login trigger (§3.7 — decided, deferred to M2) |

**5.1 The paper exercise is mandatory before M1 (MSNP21) work:** count shared
vs forked lines between the two phase machines on paper. ~90% shared → one
driver with a field wins; ~50% → two drivers win. Decide with that number, not
with taste. The structural half — one `match dialect` at the top of a flow,
never `if dialect >= 21` inside handlers — is built in D1–D4 regardless, because
that is what a contributor inherits and cannot retrofit cheaply. For v1 the
exercise itself is deferred with the dialect (§7).

## 6. Rejected alternatives

| Option | Content | Verdict |
|---|---|---|
| **A** | Functional core / imperative shell per flow (§3); runtime `Dialect`; match renderers; strangler migration flow-by-flow; conversation abstraction deferred (§7) | **This document.** ~+400–800 LOC of machinery; unmigrated handlers keep the `Sender` signature while migrated ones return effects — coexistence, never big bang |
| **B** | Full layered bridge core now (`account`/`contacts`/`conversations`/`presence`), `ConversationEndpoint` enum, `MatrixGateway` wrapper | Rejected: top-down normalization before a second dialect exists; risks a universal model MSNP21 then hacks around. Its one load-bearing insight is honored via §7's extraction trigger |
| **C** | Inline `match dialect` in existing handlers, no IO split | Rejected: every dialect fork duplicates the IO+logic tangle per verb; conformance tests still need sockets and a homeserver; the coupling that motivated this plan survives |
| **D** | Ship MSNP21 verb stubs, a second `deliver()` arm, v21 profile data, or a placeholder test-only dialect pre-release to "prove" the structure | Rejected: stubs are top-down guesses about a dialect nobody has verified, and wrong stubs become the de-facto spec a contributor must work around; a fake second dialect invites test-only paths into the release. Structure = seams (§3.3), not stubs. A timeboxed post-1.0 spike may exist — discarded, never merged. The recorded legacy-auth decision (§3.7) shows why: nothing challenge-hash-shaped is ever implemented — legacy clients route to a page whose behavior already ships, so the arm lands with the dialect that needs it |

Also rejected: `trait MsnDialect` / dyn dispatch and handler registries
(`HashMap<String, Box<dyn Handler>>`) — indirection that hides control flow for
zero measured benefit over a `match` on an enum; cargo-feature dialect gating —
the dead `msnp18` feature is the evidence, and features cannot do
per-connection coexistence (an 18 client and a 21 client connect to the same NS
port); a global dialect flag.

## 7. Explicitly deferred, with triggers

- **MSNP21 — post-release phase M1, contributor-facing.** v1 ships MSNP18 only
  (product decision); what ships pre-release is the structure (§3.3, D5). What
  does **not** ship in v1: v21 verb files, the second `deliver()` arm, a
  `Dialect::Msnp21` variant + profile record, the §5.1 paper exercise, and the
  files-touched measurement (§9 item 3). **Trigger:** a contributor picks up M1.
  **Entry requirements:** verb-semantics verification against captures/Escargot
  (§1.3) + the §5.1 paper exercise. M1-internal deferrals: MPOP endpoint
  multiplexing; multi-recipient/group SDG.
- **`ConversationRegistry` / `ConversationEndpoint`.** The instinct that
  "switchboard must not be a universal concept in the core" is right — but the
  correct shape can't be known until SDG-over-NS actually works. Trigger: when
  deliver-to-room logic exists at 3+ call sites *rendering differently per
  dialect* (candidates visible today: `msg_handler`, `uum_handler`,
  `ab_contact_add`), pull the transport enum out of the working `deliver()`
  arms. Extract, don't impose.
- **Pre-MSNP18 dialects — post-release phase M2, contributor-facing.** The
  product decision is recorded (§3.7): auth runs through the same web password
  page — Tachyon opens the browser for clients that cannot act on the
  in-protocol alert — and the server never verifies an MSN password
  in-protocol. What does **not** ship in v1: pre-MSNP18 `MsnpVersion`/`Dialect`
  variants + profile records, the browser-launch trigger arm, and `syn`/`lst`/
  `lsg` verb files. **Trigger:** a contributor picks up M2. **Entry
  requirements:** capture verification of the target client's actual handshake —
  does it reach a ticket-presentation step Tachyon can key the session on, and
  does it *require* a completed challenge-hash exchange regardless (if so,
  decide complete-without-verification — consistent with the
  tickets-are-not-secret trust model — vs refusal, and record it; write no
  `syn.rs` until captures say it is mandatory and unavoidable). Name the target
  client version in M2's profile record, not here. Don't let it accrete in
  `usr.rs` (already 485 lines).
- **No new crates** (no `bridge-core`, no `msn-models`).
- **No performance work** beyond the hazard fixes: there is no hot loop; SoA /
  arena treatment here would be theater.

## 8. Migration (one commit per stage; each compiles standalone)

The earlier untangle-in-place proposal (hazard fixes, owned caches,
`get_single_client` removal) is absorbed into D0/D2/D3/D4 — one stage scheme,
not two. **Release gate: D0–D6, all pre-release.** MSNP21 (M1) is post-release
and deliberately outside this table (§7).

| Step | Content | Risk | Anchors |
|---|---|---|---|
| **D0** `chore: remove dead registries and ritual deps` | Delete H5 (`alert_repository.rs`, `Repository<K,V>`, `GlobalState.pending_alerts` + accessors; inline `RepositoryStr` onto its one impl); drop H6 deps/feature. Then pin current VER/CVR behavior with fixtures *before* anything touches negotiation (pre-flight read of `ver.rs`/`usr.rs`) | ~zero (grep-verified; compiler is the backstop) | `cargo check --workspace`; `cargo test -p tachyon --lib`; msnp 212/4 |
| **D1** `refactor:` proof seam | Split **one** handler — `chg` — into shell/decide/render; introduce `Dialect` + `DialectProfile` (Msnp18 record; CVR strings from `negotiation.rs:24` move in — negotiation consumes them) + the `dialect_for` mapping; VER negotiation picks the highest profiled mutual version (`MsnpVersion` already parses 17/18/21 — no codec change); `SessionState` split. **Stop condition: if the `chg` split takes more than a screen, or the whole stage more than ~a day, the shape is wrong — stop and redesign** | low–med | e2e `--ignored`; new chg table test; handshake test's VER fixture (`MSNP18 MSNP17`) still green |
| **D2** `refactor:` NS front | Migrate `notification/handlers/**` verb-by-verb (own commit each); `Sender` leaves handler signatures. Absorb H1–H4 **here**: checked UTF-8 with a `<binary>` fallback (grep `from_utf8_unchecked` for the authoritative list); SB `unwrap()` → log+break; NS ignored error → same policy; one channel bound (1024) + `send().await` on both servers. Ordering is load-bearing: a silently-dead write task must be fixed *before* senders start awaiting, or bounded channels deadlock. The auth seam is named here, single-arm: `DirectToWebLogin` renders as today's NOT alert (§3.7); the browser-launch arm is M2, not shipped | med | e2e; per-verb table tests; handshake/auth tests get simpler — cite the LOC delta |
| **D3** `refactor:` Matrix front | Event-handler closures → shells emitting facts; `apply()` owns all SDK calls; `DEDUP_STATE`/`ROOM_HASH_*` → owned `TachyonClientInner` fields (H7); the single dedup path retires the §1.1 echo gap; fold in H8 (key the usertile URL by client — renderer and route are both ours: `msnp/src/soap/storage_service/get_profile.rs:263–277` — the plan-era `:263–269` anchor drifted with the H8 edit itself; `web_server.rs:52`; deliberate fixture update) | med, wide but mechanical | e2e message flow; table tests per handler family |
| **D4** `refactor:` SB front + deliver() | SB handlers to the same shape; `deliver(dialect, …)` with the classic arm only; H9 fix (unconditional teardown; remove-before-early-return) | med | e2e SB flows byte-identical |
| **D5** `feat:` dialect structure ready (contributor contract) | Ship the structure, **not** a second dialect: (1) `MAX_MSG_BODY_SIZE` (`switchboard_handle.rs:27`) completes the profile record if D4 hasn't already — the chunker must stay byte-identical (e2e-anchored, not a drive-by constant move); (2) formalize the e2e fake client into a per-dialect conformance script — the MSNP18 script is the shape contributors copy; (3) write `doc/adding-a-dialect.md`: the seam inventory with named differences, the fixture rule, the scratch-variant exercise (§9 item 1), the 212/4 regression gate, and the sanctioned auth approach (§3.7: browser page; no password handling in `msnp`). MSNP21 itself is §7 M1 | low–med | MSNP18 conformance script green; every `match dialect` exhaustive over the single variant, zero wildcards; chunked-message e2e byte-identical |
| **D6** `docs:` | This document updated to describe what was built; decisions-record entry; extend `68f3a89` / `17991e8` / `04d3112`, don't reverse them | zero | — |

**Executed — the commit ledger (D6 record; one row per §8 stage, oldest
first):**

| Stage | Commits |
|---|---|
| approval | `03d7daf` |
| D0 | `44e722f` |
| D1 | `c4b0b97` (SessionState split), `f0070e4` (dialect seam + chg) |
| D2a (H1–H4) | `2c0b81b`, `03383fc` (H1), `2040958` (H2/H3), `3a62018` (H4) |
| D2b-i (png/put/url/fqy/prp/rml) | `4e1767b`, `a51c6b9`, `2b3ef01`, `191e65b`, `cd88ab1`, `d9277f8` |
| D2b-ii (uun/uux/uum/xfr/adl) | `c70d9e1`, `8009a00`, `5def2d5`, `18ee489`, `5193cfc`, `eccaf83` |
| D2c (adl-avatar fix, auth seam, decide+usr) | `17e1fa5`, `8871758`, `b5e8c3a` |
| D3 (matrix front, H7, H8) | `561f486`, `8922c77`, `643bd73`, `26a6604` |
| D4 (SB front, deliver(), H9) | `89c9494`, `0015cbe`, `9dd3da7` |
| D4 addendum (fixture isolation) | `b951cc7` |
| D5 (profile cap, conformance script, contributor contract) | `515edc3`, `cc5570b`, `6467e32` |
| D5 addenda | `a048b5c` (doc anchor), `fea244e` (`never_loop` template hygiene) |
| D6 | this update |

**Sub-decision in D2 — RESOLVED: (b) chosen** (landed `2040958` + `3a62018`,
in that order — the write-loop fix first, because the bound is only safe once
a dead write task warns and exits instead of panicking or silently ignoring):
one `COMMAND_CHANNEL_BOUND: usize = 1024` (`lib.rs:23`) + `send().await` on
both servers — bounded memory, never drops (await ≠ drop), satisfies
`17991e8`'s actual never-drop rationale, and backpressure propagates to the
Matrix-side senders. Option (a) (`unbounded_channel`) was not taken.

## 9. Acceptance criteria

1. **(1.0) The contributor path exists and is enforced, not asserted:**
   `doc/adding-a-dialect.md` is complete; every `match dialect` site is in the
   §3.3 inventory with a named anticipated difference; `Dialect` has exactly one
   variant and every seam match is exhaustive — no `_` arms, no
   `#[non_exhaustive]`. The seam test: add a scratch `Dialect::Msnp21` variant on
   a branch and the compiler must error at every inventoried site — then delete
   it.
   **Executed at D5 (`cc5570b`) — a real run, recorded verbatim in
   `doc/adding-a-dialect.md` §3:** two passes (cargo stops at `msnp`). Pass 1,
   variant only: `error[E0004]` at `dialect.rs:39` — `profile()`'s match.
   Pass 2, with a scratch profile arm so `msnp` compiles: `error[E0004]` at
   exactly the three `tachyon` seams — `message_handlers.rs:351` (deliver),
   `chg_handler.rs:152` (render), `auth.rs:191` (render_auth_trigger). All
   four inventoried sites lit; **zero** non-inventoried sites did; the file
   was restored, the tree verified clean, and the scratch variant was never
   committed (`git log -p` on `dialect.rs` never shows `Msnp21`).
   "Enforced, not asserted" is now demonstrated.
2. **Table tests per flow:** `(command bytes, state, dialect) → expected
   Vec<ClientOut>` — no sockets, no homeserver, no mock login service.
   **Landed** with each migration stage (D1–D5); the lib suite at HEAD is
   114 tests (§11), up from ~27 at plan time.
3. **(post-1.0, M1) files touched to add a dialect** = new `msnp` verb files +
   one arm per inventoried seam + one profile record. If the first contributor
   dialect touches twelve files, the inventory was wrong — falsifying the ≤6-site
   budget is M1's stated purpose (§7), not a surprise.
4. msnp baseline at HEAD: **217 passed / 4 known-red** (plan-era text said
   212/4; the growth is deliberate and attributed: +2 D0 `44e722f` ver/cvr
   negotiation pins, +1 D3 `26a6604` the H8 usertile fixture, +2 D5
   `515edc3` the `dialect.rs` pins). The four known-reds (`nln`/`iln` msnobj
   URL-encoding; names verbatim in `doc/adding-a-dialect.md` §2) are
   invariant at every step and were never fixed; fixtures untouched except
   the deliberate updates named in their commits.
5. **Runtime: no measurable change expected.** The metrics are (1)–(3). Anyone
   promising throughput from this plan is selling something.

End-state deltas (measured, not estimated): direct deps −2 (`rayon`,
`console-subscriber`, D0 `44e722f` — plus the dead `msnp18` cargo feature);
**active** UB sites 2→0 (H1's two live debug-log sites, D2a `03383fc`; the
remaining 9 `from_utf8_unchecked` hits in `msnp/p2p/v2/mod.rs` are dead text
inside the block-commented test region `:127–424` — a known-deferred
comment-accuracy item, `msnp` frozen by policy); write-panic sites 2→0
(H2/H3, D2a `2040958`); channel bounds 10,000,000/300 → one 1024 bound (H4,
D2a `3a62018`); process-global mutable maps 3→0 (H7, D3 `643bd73`); alert
registries 3→1 (H5, D0 `44e722f`); teardown-skip bug 1→0 (H9, D4 `9dd3da7`);
§1.1 divergent send paths 4→1 executor (D3 `643bd73`). LOC: the plan's
"≈ −150 before D1's machinery" estimate measured **+30 net in D0** — the
negotiation fixture pins outweighed the deleted registries (`44e722f`: 30
files, +115/−85); D0–D5 total **+4,765/−1,090 across 76 files** in `crates/`,
dominated by new socket-free tests, not machinery.

## 10. Pitfalls

- **Semantic-vocabulary explosion** is Option B sneaking back in — enforce the
  escape-hatch rule (§3.2) ruthlessly.
- **Backpressure deadlock chain:** bounded channel + `send().await` + dead write
  task = hung Matrix tasks. D2's ordering is load-bearing, not tidy.
- **`ConnectionPhase` is dialect-sensitive** (SSO/MBI vs MD5/TWN handshake
  shapes) and nobody has touched it — the trigger fork is decided (§3.7) and
  built (D2c), but the legacy handshake *shape* is not; it stays budgeted for
  M2, don't discover it there. Resolve it as arms in the auth decider, never
  `if dialect == …` inside `auth.rs`.
- **The auth seam renders the trigger, it doesn't own the login** — if the site
  starts holding session state, the fork point has drifted (§3.7). And never add
  password/hash handling to `usr.rs` or a future `syn.rs`: credentials are
  collected only by the web page — that keeps the ticket model honest for every
  dialect.
- **Don't delete `SwitchboardServer` when MSNP21 lands** — classic stays the
  default; v21 is purely additive.
- **Renames in separate mechanical commits** (`LocalClientData` →
  `SessionState`), or every diff is polluted; never `cargo fmt --all`.
- **Gather must not become a mega-context:** per-flow gathers over only the
  fields the flow reads; `&Client` never enters a decide fn — if a flow needs IO
  mid-decision, it is two decide calls, not one with IO inside.
- **Don't spawn a task per effect** — see §3.6.
- **Single-variant erosion is the most likely 1.0 failure:** a reviewer
  "simplifies" a seam away, or a new handler inlines `MSNP18`. Anchors: no `_`
  wildcards in `match dialect` (a wildcard silently re-hides the seam the
  structure exists to expose); the literal `MSNP18` in `crates/tachyon/src`
  appears only in negotiation's profile lookup and tests — any appearance inside
  a handler body is a defect, not a style nit.
- **Seams are informed, not yet proven.** The single-variant structure is
  grounded in MSNP18's own fork points (negotiation, render, SB-vs-NS
  dispatch), and the scratch-variant exercise (`cc5570b`) demonstrated the
  compiler enumeration — but that is a weaker claim than a real second
  dialect exercising the seams; expect the first contributor dialect to force
  seam changes. That is budgeted (M1, §9 item 3), not a failure of this plan —
  and it is why no v21 stubs ship (§6 D). When a seam changes, update this
  document, `doc/adding-a-dialect.md` §1 and the decisions record in the same
  commit.

## 11. Verification anchors (at HEAD, D6)

- Per stage as executed: `cargo check --workspace` green; `cargo test -p
  tachyon --lib` — **114 passed / 0 failed / 1 ignored** (was ~27 at plan
  time); `cargo test -p msnp` — **217 passed / 4 known-red** (§9 item 4
  attribution); `cargo clippy --workspace` — 1816 warnings, all in generated
  code (msnp-gen 1503 / tachyon-gen 311), set-diffed per commit; `cargo
  clippy -p tachyon --tests` clean, and `--all-targets` clean since `fea244e`.
- `cargo test -p tachyon --test e2e_msn_homeserver -- --ignored` (Docker, run
  alone) — now **four** tests, 4/4: the three classic flows
  (`msn_client_logs_in_through_tachyon_against_a_real_homeserver`,
  `msn_client_reconnects_to_a_persisted_session`,
  `abch_pulls_while_login_is_in_flight_answer_empty` — pinning
  SOAP-degrades-while-pending, `433cb67`) plus
  `msnp18_dialect_conformance_script` (`cc5570b`): wire-level against a real
  `Stack` + `MsnClient` sockets, every profile-data expectation read from
  `Dialect::profile()`, so it pins the record↔wire mapping rather than a copy
  of the record. Same-file shape — a `tests/common` extraction is deferred
  until M1's second script needs it (fields move in with their consumers).
- `handshake_happy_flow_test` / `auth_i_test` against `MatrixMockServer`
  survived the signature changes and stay socket-free at HEAD.
- Table tests per flow (§9 item 2) landed with each migration stage; the
  conformance script is the contributor template
  (`doc/adding-a-dialect.md` §6).

## 12. Phase approval (required before any code)

| Phase | Approved? | Notes |
|---|---|---|
| D0 | ☑ | pure deletion + negotiation fixtures; near-zero risk |
| D1 | ☑ | proof seam; stop condition applies |
| D2 | ☑ | includes H1–H4 and the channel sub-decision |
| D3 | ☑ | includes H7, H8, echo-gap retirement |
| D4 | ☑ | includes H9 |
| D5 | ☑ | dialect structure ready + contributor contract; MSNP18 only — M1 stays deferred (§7) |
| D6 | ☑ | docs only |

All rows approved by the user in chat on 2026-11-05 (`03d7daf`): execute
D0–D6 to completion, in order, one commit per stage (D2: one commit per verb
migration, as the stage row specifies).

**Executed to completion.** The per-stage commit ledger is §8; the built
state is §3.3's final inventory (4 of ≤6 sites), §1.4's dispositions (all
nine hazards retired), §9's acceptance record (scratch exercise executed,
217/4 baseline) and §11's verification anchors (114 lib / 217 msnp / four
e2e). Release gate = D0–D6, met; MSNP18 only. M1 (MSNP21) and M2
(pre-MSNP18 dialects + browser auth) remain post-release (§7) — deliberately
deferred, not forgotten; the first contributor dialect picks up from
`doc/adding-a-dialect.md`.
