# Tachyon Architecture Plan (v2)

**Status:** revised plan — supersedes "Tachyon Architecture Overhaul" (v1). No code changed by this document; implementation not started.
**Revision note:** v1's audit was accurate — it is kept below, re-verified against the tree at commit `8c0f706` (2026-11-04). v1's structural prescription (five crates, one-variant `Dialect` enum, mutex-guarded `Account` aggregate, seven-variant outbound enum) is deferred or dropped: the measured defect surface is hundreds of lines, and the proposed outbound vocabulary was provably incomplete against the tree it would replace (it could not express `RNG`).
**What this document is now:** a defect table with dates and line numbers, an ordered work plan sized to it, and ratchet gates — not a framework proposal.

**Scope decisions carried from v1:** MSNP18 only; one MSN session per Matrix account (policy, with simulated MPOP endpoint ids); group conversations as 1:1 proxy contacts; protocol bytes and timing preserved exactly (moved to data); static dispatch for closed sets. Out of scope: other protocol generations (no captures exist, none scheduled), native group chat, real MPOP, offline-store growth.

## 0. The rule

> Fix demonstrated defects first. Characterize behavior before moving it. Data-ize constants. Introduce structure only where present behavior — or a second real protocol version — forces it. Every acceptance gate is a ratchet (a grep, a count, a LOC delta), not prose.

---

## 1. Measured defect surface

Baseline: **40,394 lines of Rust** across three crates (`find crates -name '*.rs' | xargs wc -l`, 2026-11-04, `8c0f706`). v1 prescribed two new crates, a ~15-method aggregate, and six phases against this:

| # | Defect | Verified location | Measured size |
|---|--------|-------------------|---------------|
| P1 | MSNP18 hardcoded into handshake | `notification/handlers/negotiation.rs` | **30 lines** |
| P2 | Auth fuses MSN wire, credentials, Matrix restore, verification, sync bootstrap | `notification/handlers/auth.rs` | 265 lines / 14,913 B |
| P3 | Matrix/web code constructs MSN wire commands | **11 construction sites outside the protocol trees** — NS: `matrix/oim.rs:141` (`MSG`), `matrix/sync.rs:198` (`NOT`), `web/soap/ab_service/ab_contact_add.rs:142` (`NOT`), `tachyon/switchboard_service.rs:80` (`RNG`); SB: `matrix/handlers/message_handlers.rs:58,64,83,123,138,159,194` (`MSG` ×7). Full census: `NotificationServerCommand::` 52 refs / 21 files; `SwitchboardServerCommand::` 19 refs / 4 files; `use msnp` in the matrix tree: 32 lines / 7 files | 11 leak sites |
| P4 | God objects | `tachyon/global_state.rs` (122 lines) + `tachyon/client/tachyon_client.rs` (191 lines; `TachyonClientInner` = **17 public fields**, lines 40–58 — v1 asserted "~15", now measured) | 313 lines |
| P5 | Implicit session phase | `LocalClientData.phase` + `command_handler.rs` dispatch | field soup + match |
| P6 | Single-session by accident | `get_single_client()` FIXME `global_state.rs:76–77`; sole caller `web/web_endpoints.rs:87` | one registry-key change |
| P7 | Matrix credentials cross the MSN wire | `switchboard_service.rs:80`: `RNG` carries `SwitchboardToken::new(room_id, matrix_ticket)`; parse side `.expect("to be here")` at `switchboard_token.rs:58,65` | one token type |
| P8 | Implicit conversation mapping | `bootstrap_handlers.rs:24` `ROOM_USER_PORTAL_MODE: bool = true` (used :54) | const → per-conversation kind |
| P9 | Web/SOAP is an undocumented third IO boundary | `web/web_server.rs`, `web/soap/*` | documentation only for now |
| P10 | Transport hazards | capacities **10,000,000** (`switchboard_server.rs:164`) vs **300** (`notification_server.rs:135`) vs **10** (`command_handler.rs:124,166`); SB write loop `write_all(...).await.unwrap()` (`switchboard_server.rs:189,194` — panics the task on disconnect); NS write loop *silently discards* write errors (`let _result = ...`, `notification_server.rs:149`); `from_utf8_unchecked` (`notification_server.rs:1,146`); `rayon` in async (`switchboard_server.rs:14–15`); `println!` in tasks (`contact_handlers.rs:23`, `sync.rs:140`, `uum_handler.rs:78`, `request_verification_handlers.rs` ×5) | a few dozen lines |

**Verdict.** Every defect above is locally fixable in days, with the existing e2e as the safety net, and none of them requires a new crate. The ratio — ~59 KB of v1 design against a defect surface measured in hundreds of lines — is why this document exists.

**The finding that killed v1's core enum.** v1's `MsnOutbound` had seven variants (`RosterAck`, `PresenceNotice`, `EndpointData`, `Notification`, `ProfileMessage`, `OfflineMailData`, `KeepAlive`) and could not express **`RNG`** — the only command that pages an idle client into a switchboard, emitted today from `switchboard_service.rs:80`. The NS server vocabulary also includes `UBM`, `UUX`, `BLP`, `NFY`, `CHG`, `FQY`, `ILN`, `NLN`, `XFR`, and 13 ready-phase verb handlers exist (`adl chg fqy png prp put rml url usr uum uun uux xfr`). A vocabulary designed from imagined flows missed the operation the Matrix→MSN delivery path depends on; v1's own acceptance criterion ("a new version requires zero domain changes") was therefore falsified *for the version in scope*, before any code moved. Consequence: **no outbound vocabulary may exist until the §3 emission inventory exists, and it must be derived from it.**

---

## 2. Immediate fixes — independent PRs, no architecture approval needed

Each PR ships alone; gate = `cargo test` (baseline 212 pass / 4 known-fail per AGENTS.md) + e2e + the stated ratchet. No sequencing between them except A6 before A7's key change if they touch the same registry code.

| PR | Change | Ratchet (→ target) |
|----|--------|--------------------|
| A1 | One bounded-capacity policy for all NS/SB channels (start 256; backpressure for message-class, drop/coalesce for keepalive-class). Kills the 10,000,000 / 300 / 10 split. | `rg 'mpsc::channel\(10000000\)' crates/tachyon/src` → 0 |
| A2 | Write failures → typed session teardown. SB: delete the two `unwrap()`s; NS: stop discarding `_result`. Both loops currently fail — one loudly, one silently. | `rg 'write_all.*unwrap'` → 0; `rg 'let _result = write'` → 0 |
| A3 | `from_utf8_unchecked` → `from_utf8_lossy` in debug logging | `rg from_utf8_unchecked crates/tachyon/src` → 0 |
| A4 | `rayon` out of async paths; `println!` → `log` in spawned tasks | `rg rayon crates/tachyon/src` → 0 |
| A5 | `.expect("...be here")` → typed errors (SB bootstrap, token parse) | `rg 'expect\(".*be here'` → 0 |
| A6 | **P7:** opaque random switchboard ticket: `ticket → (email, conversation_id)`, TTL, single-use, revoked on teardown; no Matrix token or room id on the wire. Requires a minimal `ConversationId(Uuid)` + persisted `conversation_directory` table — dozens of lines, not a crate. | `rg SwitchboardToken crates/tachyon/src/matrix` → 0; token payload carries no Matrix fields |
| A7 | **P6:** registry keyed by email; single-session policy enforced at insert; delete `get_single_client()` + FIXME | `rg get_single_client` → 0 |

---

## 3. Phase 0 — characterize before any move (kept from v1, hardened)

1. **Emission inventory** — deliverable `doc/protocol-emission-inventory.md`: every NS/SB server command the system can emit, with call sites, triggers, timing constraints, and protocol-local vs externally-triggered classification. The §1 census (52/19 refs) is the floor, not the inventory; counts include match arms, so the inventory refines them by hand. **No outbound enum merges until this artifact exists.**
2. **Transcript harness** — recordings from the running system against a fake Matrix backend: handshake + RST2 interlude, post-login sequence, ADL→NLN/UBX, message both directions, **RNG invite → SB bootstrap → pending-message flush**, OIM NS `MSG`, roster `NOT`, presence, disconnect-mid-write.
3. **Determinism harness, named:** injected `SessionId`/UUID sources (the invite path calls `SessionId::random()` twice, `switchboard_service.rs:68,82`) and a pausable clock. Without injection, "byte-identical" is a property nobody can check.
4. **Two-tier fixtures:** *shape* (command sequence, tr_id/session-id/timestamps normalized) and *constants* (exact bytes for fixed payloads: CVR `14.0.8117.0416`/`localhost`, SHIELDS, 1311-byte chunking). **Timing is asserted separately** — ordering plus windows (e.g. ADL ack → NLN within 100 ms ± tolerance), never bytes across a delay that crosses task boundaries; v1's byte-identical gate would flake under any new task topology.

Gate: fixtures pass against *current* code before refactoring starts. Honestly labeled: pinned status quo, not wire truth.

---

## 4. Data-ization — the piece of v1 that survives on merit

Move into a `const`/`static` MSNP18 profile living where the code already lives (`negotiation.rs`/`auth.rs` neighborhood). No framework, no new crate:

| Constant | Today | Target |
|----------|-------|--------|
| CVR strings `14.0.8117.0416` / `localhost` | hardcoded in negotiation | `MSNP18_PROFILE.cvr` |
| GCF/SHIELDS payload | hardcoded in auth | `MSNP18_PROFILE.auth` |
| 1311-byte MSG chunk limit | message-send path | `MSNP18_PROFILE.caps.max_msg_size` |
| 120 s QNG keepalive | hardcoded | `MSNP18_PROFILE.keepalive` |
| Post-login sequence (SBS → profile MSG → UBX → mail; ADL ack → 100 ms → NLN/UBX) | imperative order inside `auth.rs` | `MSNP18_PROFILE.post_login` (ordered data) |

Gate: `rg '14\.0\.8117' crates/tachyon/src` hits only the profile module; Phase-0 fixtures green.

---

## 5. Local structural fixes — subtractive, no new crates

**5.1 Explicit session state (P5).** `LocalClientData.phase` → typed enum owned by the NS session task (`ExpectCvr | ExpectUsrI | AwaitingSsoToken | ExpectUsrS | Ready` — final states derived from the inventory); SB likewise (`Auth | Initializing | Ready`). Illegal command for state → typed `SessionError` → protocol error reply or teardown. Handler return keeps v1's best-taste idea: `Step { local_replies, events }` — protocol-local replies (OK/NAK/QNG/handshake) never leave the session; semantic events go to services. That is the shape a future dialect would take; no dialect is built now.

**5.2 Split `auth.rs` by moving code (P2).** Each concern moves into the file that owns it: Matrix restore/verification → `matrix/`; credential lookup → auth-service module; post-login → driven by `MSNP18_PROFILE.post_login`; handshake stays in the NS session. Move-and-delete as you go — no "thin shim" phase running two state models in parallel (v1's Phase 2 shim is rejected).

**5.3 Kill the god objects by subtraction (P4).** No `Account` aggregate is built. Handlers receive only the services they use (ticket registry, conversation directory, their own connection's outbound handle); protocol state is never shared; `GlobalState` shrinks to composition-root wiring, then disappears. Any shared service that survives exposes a **synchronous** API — mutate, enqueue, return `Vec<Op>` — so v1's "all methods are fast" contract is type-checked (no `async fn` on shared mutable state): nothing inside can await Matrix or SQLite. Slow work crosses as data on bounded queues; completions come back as events. *Judgment call, not measurement:* if narrowed handlers prove insufficient, revisit the aggregate; the sync-API rule governs whatever emerges.

**5.4 Matrix-boundary ratchet (P3).** Handlers become normalizers: extract → normalized event → enqueue; wire rendering moves app-side, beside the protocol code that owns it. If an outbound vocabulary is introduced, it is derived from the inventory: every variant maps to ≥1 real emit site, every inventoried command is reachable from some variant, `RNG` is first-class (it is what wakes an idle client), and `KeepAlive` is *not* an op — QNG is protocol-local (correcting v1 §7.5, which also contradicted its own §6.2 by putting the ADL ack in both `local_replies` and a `RosterAck` variant). Matrix-adapter outbound shape: one bounded per-account channel to the adapter; completions return as events. Ratchets, monotonic to zero:

- `rg -c 'use msnp' crates/tachyon/src/matrix` — today 32 lines / 7 files → 0
- `rg 'NotificationServerCommand::|SwitchboardServerCommand::' crates/tachyon/src/matrix crates/tachyon/src/web` → 0

Enforcement is these greps in CI (plus `clippy::disallowed_types` where handy) — **not crate walls**. The lint was always what did the work; crate edges pour concrete before the seams are validated.

**5.5 Conversation directory (P8).** `ConversationId(Uuid)` + `kind: Direct | ProxyGroup` (introduced with A6), persisted; `ROOM_USER_PORTAL_MODE` becomes per-conversation data; the proxy-group text-only policy stays in messaging. The group-proxy model itself (v1 D3) is unchanged and unchallenged.

**5.6 Net-LOC goal.** §2–§5 combined should be **net-negative** against the 40,394-line baseline. A refactor of a few-hundred-line defect surface that grows the codebase has failed, whatever the tests say.

---

## 6. Deferred — with an explicit trigger

Deferred: the `Dialect` enum + profile registry + negotiation framework; the five-crate split (`msn-models`, `tachyon-domain`); the `Account` aggregate; v1's Appendix C (family notes, capability matrix, onboarding phases) — **dropped**; it was onboarding material for unscheduled work whose "?" cells were guesses.

**Trigger:** a second MSN protocol version is actually scheduled *and* captures exist. Then: implement it the dumb way, allow duplication where versions genuinely differ, extract the seam from two real implementations, and promote only boundaries that have stayed stable for a release to crate edges. Semantic compression requires ≥2 instances; there is one today, and v1's own text says future handshakes "may be drastically different" — shared by name only, not by meaning.

What survives into that future: Phase-0 fixtures, the emission inventory, the profile data (§4), the `Step` shape, the session-state enums.

---

## 7. Verification against Muratori's principles

Framed as the questions each principle forces, with the concrete test. Where this plan is silent because work is deferred, it says so rather than claiming a pass.

| Question the principle forces | Test | Answer |
|---|---|---|
| Does each abstraction delete more code than it adds? (semantic compression) | count the instances before building | `Dialect`: 1 instance → deferred (§6). Outbound vocabulary: 11 real emit sites → allowed, but only derived from the inventory (§5.4). Profile consts: delete scattered literals → kept (§4). |
| Is the rendering next to the data it renders? (merger of function and data) | locate renderer, enum, and user | v1 split them across three crates; v2 keeps profile consts beside the protocol code and session state inside the session task (§4, §5.1). |
| Name the present consumer of every new type. (no speculative generality) | grep for consumers | `ConversationId`: A6 tickets + directory + SB bootstrap. `Msnp18Profile`: negotiation/auth/render today. `Dialect`: none → not built. |
| Are the panics, the 10-million-slot channel, and the credential leak ahead of all structural work? (bystander effect / reliability first) | document order | yes — §2 is first in the document and in the work order. |
| Is every claim checkable with `wc`/`grep`? (the tree is ground truth) | re-verify citations | every number in §1–§5 was re-measured against `8c0f706`, dated. Deferred sections make no claims about the tree. |
| Does every acceptance gate count something monotonic? (ratchet analysis) | greps vs prose | §2/§4/§5.4 gates are grep ratchets; §5.6 is net LOC; §3 is fixture count. "e2e green" survives only as an *additional* gate, never the only one. |
| Can someone change one handler without understanding the whole bridge? (local reasoning) | after §5.1–5.3 | yes: narrow inputs, session-owned state, sync service APIs. v1's three-mutex aggregate would have failed this test; this plan doesn't build it (judgment call — revisit per §5.3). |
| Do we watch it run before moving it? | §3 harness | recordings of the live system with injected determinism, honestly labeled pinned status quo, not wire truth. |
| Does any surviving enum restate the protocol's command list? (no golden interfaces) | compare against inventory | any that do are protocol-side render inputs, not a "domain" costume (§5.4); the inventory proves coverage in both directions. |
| Is every step independently green and revertible? (always shippable) | per PR/step | §2's PRs are independent; §5's steps each end green with e2e; no step needs the next one to make sense — v1's Phase-2 shim state was the counterexample and is gone. |

**Honest silences:** the concurrency question v1 refused (Matrix event + ADL + second login + socket death, simultaneously) is answered only *provisionally*: session tasks own protocol state; the sync-API rule (§5.3) bounds what shared state can cost; a single-owner actor becomes a triggered decision only if contention is ever measured — a trigger, not a hope. MPOP, native groups, offline-store growth: out of scope, unchanged from v1's scope decisions.
