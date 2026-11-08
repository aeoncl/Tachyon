# Tachyon — Design Decisions Record

**Status:** historical record — every entry cites the commit(s) where the decision was
taken. This is the *companion* of `doc/architecture.md`: that document was the
bottom-up proposal (IO/logic split + MSNP dialect support; the original top-down
re-cratification proposal was deleted in `f32ab61`) — approved in `03d7daf`, executed to
completion as stages D0–D6, and since D6 it describes the built state. This one records
what the code actually grew and why, as visible in 412 commits (2022-04 → 2026-11,
single author `aeoncl`, tag `0.1.0`). §9 records the decisions the executed plan itself
produced.

## 0. Eras (context for everything below)

| Era | Commits | What happened |
|---|---|---|
| v1 `wlmatrix-rust` (2022-04 → 2023-11) | `a0544bb`…`d6809ca` | Single `WLMatrixRust` crate: NS server, SOAP, P2P v1, voice, e2e on Jest (`43a3c79`). Ends with "TODO: redo the whole P2P Client" (`4a1ce40`). |
| Rewrite (2024-02 → 2024-04) | `e191179`…`58b39c3` | Wire codec extracted into `msnp` lib (`4752e06`); "repo reorganisation" (`58b39c3`, 2024-04-12) creates the `crates/` + `lib/` submodule layout that survives today. |
| Rebuilding features (2024-05 → 2025-06) | `f40d4e5`…`72f916f` | RFC2047, FileTime, ppcrl, RSI, circles, sliding-sync redo (`9ba60a1`), invites. |
| The `tachyon` era (2026-03 → 2026-11) | `d3ebba3`…`e738db8` | Sliding sync promoted (`sync2` → `sync`, `d3ebba3`), real switchboard server (`e470119`), P2PV2/SLP, device verification UI, config file, "new auth" → single-registry auth, architecture + AGENTS docs. |

---

## 1. Product & scope

- **Emulate the whole MSN server side, keep the stock client unmodified.** From the very
  first month the design is: Tachyon *is* the Notification Server, the Switchboard(s) and
  the SOAP/config endpoints a WLM 2011 client calls (ABCH, Sharing, Storage, RSI, RST2
  ticket issuer — `ab49f1c`, `6f2f806`, `ebe95ac`, `2951497`). No client-side plugin.
- **Reproduce the legacy web surface too**, not just messaging: `MsgrConfig.xml`,
  `ppcrlconfig`, `wlidsvcconfig`, `sha1auth` (`84ed96a` "big ppcrl"), and even period-
  accurate **ads and MSN Today** (`175440b`, `394fa4c`) and a Tachyon-branded site
  (`81d8f30`). Faithfulness to the 2011 experience is a feature.
- **MSNP18 is the only negotiated version** (`crates/tachyon/src/notification/handlers/
  negotiation.rs`; confirmed as D1 in the architecture proposal). Other generations must
  be *accommodated*, not implemented.
- **Trust model: localhost, single user.** The RST2 ticket is deliberately *not* a secret:
  "not authenticated: any local process can fabricate one; Tachyon only serves localhost"
  — comment added with the new auth (`506eab3`, `crates/tachyon/src/tachyon/identifiers/bridge_link.rs`).
  Corollary: tickets are passed **in the clear** over MSNP and the local `SecretEncryptor`
  (`local.key`) was removed as dead weight (`6692f58`).

## 2. Technical substrate

- **Rust + tokio, vendored `matrix-rust-sdk` fork as a submodule.** The SDK is tracked as
  a *fork rebased onto upstream* (PR #14 `00c1048` "rebase-matrix-sdk-to-upstream", PR #1
  `960b3a6` "rust-sdk-alignment"), consumed as a path dep, upgraded by moving the
  submodule pointer (`bbd6c61`, `0a51649`, AGENTS.md) — never by version-bumping a crate.
  Ruma, initially a git module, was removed in favour of the crates.io path (`1016a30`).
- **Wire codec is its own crate, `msnp` — "wire codec only."** Deliberate split
  (`4752e06`, `58b39c3`): framing + one file per verb + `<Verb>Client/Server` structs
  implementing `TryFromRawCommand`/`IntoBytes`; the app crate owns all behaviour. Codec
  correctness is pinned by **byte-exact round-trip fixtures** (AGENTS.md convention).
- **`libsiren` is compiled from source by `build.rs`** (C compiler required), and voice
  clips need **`ffmpeg` on PATH** at runtime — with an explicit degradation policy:
  without ffmpeg, audio messages fall back to plain file transfers (`5e3cbc1`,
  `src/audio/ffmpeg.rs` "TODO replace with compiled FFMPEG lib to remove dependency from the PATH").
- **Dead code left by a refactor is deleted in a dedicated follow-up chore** — not left
  rotting (`6692f58` after `68f3a89`; `bb7ceaa` "removed tachyon legacy").

## 3. MSN ↔ Matrix identity & conversation model

- **A Matrix room must present as MSN identities.** Room IDs are too long and malformed
  for the email shape MSN expects, so the bridge synthesises room emails from a **SHA-1
  hash of the room ID**, kept in a bidirectional DashMap cache (`04d3112`,
  `matrix/extensions/msn_user_resolver.rs`). **Extended, not reversed (D3, `643bd73`):**
  the same SHA-1 synthesis and the same bidirectional shape, but the cache is now the
  *owned* `RoomHashCache { by_hash, by_room }` field on `TachyonClientInner`
  (`tachyon/client/tachyon_client.rs:67`) — the process-global `lazy_static` map is gone,
  and cache lifetime equals the signed-in account's lifetime (architecture.md §1.4 H7).
- **Group chats are simulated as one "room user" contact** — `ROOM_USER_PORTAL_MODE =
  true` (`cfde039`): the SB roster at ANS time contains only the room-user, and active
  members arrive as notices. The stated rationale (architecture doc D3): Matrix has no
  canonical room shape for MSN groups, and tracking contact relations client-side "was
  painful". This is a *chosen* approach, explicitly "may be revisited".
- **Datacast media is sent from the fake room user; file/voice arrivals get plain-text
  "…has sent you a file" notices** — a workaround for WLM's handling of media from
  unexpected senders (`8fe8abb`, `3509fa6`, `1ac966f`). Reverted once for voice clips:
  audio/datacast must come from the *real* sender (`3509fa6`).
- **Presence is derived, not forwarded.** Matrix presence support was judged unusable
  ("terrible presence support in the spec") — presence is instead driven by membership/
  endpoint data with PSM payloads (`06ff5df`, `942136c` part 2).
- **A disconnecting contact goes HDN (hidden), never FLN.** The roster entry survives with
  `PresenceStatus::HDN` instead of being removed (`a009252`; still mapped in
  `tachyon/mappers/presence_state.rs`). Profile churn propagates as NLN+UBX, replacing an
  earlier FLN flash (`a009252`).
- **One Switchboard connection per conversation**, reached via `XFR` (`1b9e75d`, `e45eb06`,
  SB server `e470119` with `LocalSwitchboardData` session state).
- **Contact/membership sync is push-based through SOAP** (AB pull + `PRP` + mail-data/OIM
  notifications — `a13e8f6`, `89b42b5`, `29faa66`), with **lazy resolution** of MSN user
  details so roster pulls don't block on Matrix lookups (`5b24999`).

## 4. Auth & identity lifecycle (the longest arc)

The login story went through five designs; the last two define today's behaviour:

1. password → "encrypted password" (`b2de452`)
2. **Matrix auth through the browser** instead of the MSN UI (`7df652a`, `36e78aa`/`81358c0`),
   token decoupling (`27ce905`), dedicated login service (`99b2717`)
3. device verification pages: SAS emoji compare, confirm-with-other-device, **identity
   reset as the escape hatch** (`aac3804`, `45b50d3`, `ae2b119`); the web RSI endpoint
   must return `X-IC-CancelPolling` 404 when no client is live (`a33af87`)
4. **login actually gates on cross-signing** — restore "succeeds" only if
   `check_device_is_crossed_signed` passes; false positives push the user to identity
   reset (`3143be6`)
5. **"new auth" + single registry** (`506eab3` → `2f21be2` → `433cb67` → `68f3a89`).
   The settled design (from the `68f3a89` message):
   - `TachyonClient` is created **the moment the USR S S ticket validates**, *before* the
     Matrix login completes. No separate `pending_logins` registry.
   - `Backend: Pending | Ready` lives inside the client; `attach_backend` is
     **first-caller-wins**. `PendingOAuth` moved from global state into the client.
   - `ConnectionPhase` loses `AwaitingLogin`; the `Ready` phase keeps the QNG/ADL
     keep-alive path for the window where the MSN client is live but the Matrix backend
     is still pending.
   - **SOAP consumers must degrade gracefully while pending**: ABCH/Sharing answer
     contact pulls with an *empty book*, not an error (`433cb67`) — behaviour pinned by
     the Docker e2e test (`b1109ed`, `tests/e2e_msn_homeserver.rs`).
   - The **client drop guard moved onto `LocalClientData`** (`49d4341` introduced it) so
     dropping the NS socket releases store handles deterministically.
   - **The one-registered-object pattern was extended, never reversed (D3/D4):** the
     per-account `DedupState` and `RoomHashCache` (`643bd73`) and the negotiated dialect
     copy (`dialect: RwLock<Dialect>`, `0015cbe`) all live as `TachyonClientInner` fields
     — still one registry, still no parallel registries, still `Backend::Pending | Ready`
     inside the client (architecture.md §3.3/§3.5).
- **Credential persistence: sled → bundled SQLite.** Multiple clients sharing one sled
  store was a known bug (`d19f870`); the new auth replaced it with `crates/tachyon-sqlite-store`
  (rusqlite, credentials keyed per user, `506eab3`).
- **Device ID: hostname-derived `TAK-<hostname>` → random, persisted per-user** in the
  data dir's `.device_id` file (`9a45555`; old `TachyonDeviceId::from_hostname` deleted).

## 5. Transport & concurrency choices

- **Command flow uses tokio mpsc, deliberately *unbounded*** — broadcast channels with
  capacity were replaced (`17991e8`) so a slow socket never drops protocol events. The
  flip side (10,000,000-capacity SB channel, `write_all().await.unwrap()`,
  `from_utf8_unchecked`, `rayon` in async) was **known and deliberately unfixed** at the
  time — logged in the then-current hazards appendix of architecture.md and in AGENTS.md
  with the policy "fixing one is a real change, not a drive-by cleanup".
  **Extended, not reversed (D2a):** the *rationale* — a slow socket never drops protocol
  events — is preserved exactly, and the plan's §8 sub-decision (b) satisfied it with
  **one bounded policy instead**: `COMMAND_CHANNEL_BOUND: usize = 1024` + `send().await`
  on both servers (`3a62018`) — bounded memory, never drops (await ≠ drop), backpressure
  propagates to the Matrix-side senders. It was safe only after the write loops stopped
  panicking/silently ignoring (`2040958`: warn + labeled break on both servers) and the
  debug logs stopped using `from_utf8_unchecked` (`03383fc`: checked UTF-8 with a
  `<binary N bytes>` fallback); `rayon` left with the D0 ritual deps (`44e722f`). The
  "deliberately unfixed" text above and the old Appendix B pointer are stale — the
  dispositions now live in architecture.md §1.4.
- **The parser and the handlers must never spin**: malformed input was made to break out
  of the command-parse loop instead of looping forever (`2beaf6b`), and MSG handling
  stopped awaiting Matrix sends inline on the connection task — payloads are dispatched
  to a spawned task instead (`e5b4897`).
- **Outbound Matrix message dedup is per-room with an async send-lock**, because a global
  `DashMap<(event,room)` key under a `std::sync::Mutex` raced (`6ffaaa2`). DashMap
  re-entrancy got its own rule: remove-by-predicate must use `retain`, never iterate-
  then-remove (deadlock, `550ff20`). **Extended (D3, `643bd73`):** the per-room
  send-lock + dedup now live in the *owned* `DedupState` on `TachyonClientInner`, and
  one executor — `apply_room_op`'s `RoomOp::SendText`
  (`matrix/extensions/message_dedup.rs:104`) — covers every §1.1 send path
  **including the UUM NS-IM path** that previously bypassed dedup entirely; named
  behavior delta: UUM's plain send gained own-echo suppression and per-room
  serialization.
- **Chunked MSG is reassembled at the codec level** (1311-byte chunks, `24d0a1a`),
  failures answer `NAK` (`328aa99`); raw payloads trim stray carriage returns (`13bc537`).
  **Extended (D4/D5):** the NAK policy is now the pure ack table in the SB msg decide
  path (`89c9494`), and the 1311 cap is `DialectProfile::max_msg_body_size` (`515edc3`)
  — the file-local const retired, the chunker byte-identical, one source of truth per
  dialect.

## 6. P2P

- **P2P v1 inside the app was abandoned** ("TODO: redo the whole P2P Client", `4a1ce40`).
  Early on, the echo/direct P2P servers were disabled (`dc422a6`, 2022-06) and sessions
  were driven for real from the Matrix side through switchboard-invited SLP sessions
  (`fe190d1`, `6580e43`).
- **P2P v2 lives in `msnp` as SLP with concrete typed payloads** (`f72dfb4`…`7b3b7c3`,
  `0f50f5c`, `e3bdb8d`, `885a83b`): TLV lists, strict length computation, RAK/ACK session
  semantics (`e79ca9c`, `13d8995`), session BYE (`1de52d5`), SLP packet de-chunking and a
  duplicated `Content-Length` fix (`5cc03df`, `35acc60`), `uux` payload-size fix (`c8dfbdf`).
- **MSNObject serialization** (`c74f479`, `10fa8af`, `63ff4e0` friendly form) + SLP
  EUF-GUID (`d3a2b7f`) carry file metadata; display pictures ride the same MSNObject
  machinery (`25566b8`, re-added on the new stack as `a9f9647`).
- **Matrix → MSN file transfer landed first** (`df148a7`, `1a7074f` era); incoming files
  remained wip (`45447ee`, `aca1b6d`, `1ac966f` notice workaround).

## 7. Configuration & runtime

- **Config format: JSON → INI.** A JSON tachyon config existed briefly (`c4c7718`,
  2024-06); the durable decision is an **auto-written `config.ini` from defaults on first
  run**, under platform `ProjectDirs("Tachyon")` unless `TACHYON_HOME` overrides
  (`d523c58`, `tachyon/config/tachyon_config.rs`, paths `paths.rs` — the Option-returning
  path API was later made infallible with the dir guaranteed present, `9a45555`).
- **Dev-port defaults `11863/11864/11866` on 127.0.0.1**, not the README's 1863/1864;
  a stock client needs the `zathras` patch + config.ini (AGENTS.md). The switchboard port
  stopped being hardcoded — `TachyonConfig` is threaded down into handlers (`f6ea56e`).
- **`strict_ssl` is a config toggle** because local self-signed homeservers are the
  normal deployment for this toy-era stack (`f53a034`, before it every login passed
  `disable_ssl: true`).
- **Logging is off by default** (`logs_enabled: false`), one `logs.txt` truncated per
  start, and **`matrix-sdk` is silenced to `LevelFilter::Off`** so bridge logs stay
  readable — with the accepted downside that Matrix-side problems are invisible
  (main.rs logger setup; AGENTS.md pitfall).

## 8. Engineering practice (how the repo runs)

- **Conventional Commits, lowercase** (`feat:`/`fix:`/`refactor(auth):`/`chore:`) —
  uniform from 2026 onward; `wip:`/`feat(slop):` labels are used honestly for
  experiments (`a0a0fe6`, `506eab3`).
- **Tests are inline and prose-named; wire round-trips are asserted as exact bytes** —
  changing a renderer means deliberately updating its fixture (AGENTS.md;
  `crates/msnp` tests from the 2024 extraction).
- **CI was deliberately deleted** (`.github/workflows/rust-clippy.yml`, created `ce9fda8`
  in 2022, removed `45467fa`). The Jest e2e harness (`43a3c79`) died with v1; the only
  integration test today is the Rust Docker e2e, `--ignored` and env-driven
  (`b1109ed`, AGENTS.md). **Extended (D5):** CI stays deleted; the e2e family grew to
  **four** `--ignored` tests — the three classic flows plus
  `msnp18_dialect_conformance_script` (`cc5570b`), the per-dialect wire-level template
  that reads every profile-data expectation from `Dialect::profile()`.
- **A known-red baseline is tolerated, recorded, not chased**: 4 msnobj URL-encoding
  fixture failures in `msnp` are the documented status quo (`e738db8`). **Extended:**
  the count is now **217 passed / 4 failed** — the growth is deliberate, attributed
  fixture additions (+2 `44e722f`, +1 `26a6604`, +2 `515edc3`); the four names live
  verbatim in `doc/adding-a-dialect.md` §2, are invariant at every step of D0–D6, and
  were never fixed.
- **Never `cargo fmt --all`** — the tree is deliberately not rustfmt-clean; match the
  surrounding file (`e738db8`).
- **`doc/architecture.md` is a proposal with explicit non-decisions**: Phase 0 not
  approved, other MSN versions out of scope, transport hazards deferred to named phases,
  proxy-group and MPOP deferral documented as intentional (§18). Code refactoring toward
  it is forbidden until approved (AGENTS.md). **Superseded (D6):** the plan was approved
  by the user (`03d7daf`, 2026-11-05) and executed to completion — stages D0–D6, ledger
  in architecture.md §8. The refactoring ban was lifted by that approval, not around it:
  no code moved toward the plan before `03d7daf`. `doc/architecture.md` now describes the
  built state, and `doc/adding-a-dialect.md` (`6467e32`) is the contributor contract.
  MSNP21 (M1) and pre-MSNP18 dialects + browser auth (M2) remain deferred exactly as the
  proposal said (§7) — the seams shipped, the dialects did not.

## 9. The dialect-structure era (D0–D6, 2026-11)

Decisions the executed plan itself produced — each extends an earlier entry above;
none reverses one. Full per-stage ledger in architecture.md §8.

- **The negotiated dialect is copied once, owned by the client, never flagged in
  parallel.** `SessionState` negotiates (VER, `dialect_for`); the client carries —
  `dialect: RwLock<Dialect>` on `TachyonClientInner` with exactly **one** write,
  `set_dialect` at the NS auth SSO-S registration *before* `insert_clients` makes the
  client discoverable (`auth.rs:73`), so every reader (the delivery seam) sees it
  (`0015cbe`; architecture.md §3.3's "one owner, no parallel flags").
- **`deliver()` is a function, not a trait — classic arm only, constructed *and*
  called.** `deliver(dialect, client, room, out)` at `message_handlers.rs:350`, one arm
  at release, two production call sites passing `client.dialect()` (`:119`, `:395`); the
  v21 arm (SDG/UUM over the NS writer, XFR refusal) is M1's addition, compiler-checked
  the moment `Dialect::Msnp21` exists (`0015cbe`; architecture.md §3.4).
- **H9: unconditional teardown by construction — with the reachability honesty kept.**
  `teardown_switchboard()` hoisted *before* the kill-send and extracted into its own
  function; the kill-send warns, never `?`-skips cleanup. The recorded finding: post-D2a
  the leak chain was *structurally latent* (a parked never-read broadcast receiver means
  tokio's `send` cannot fail at zero-receivers), so the fix is not "bug reproduced" — it
  makes the invariant structural regardless, and narrows the concurrent-insert eviction
  window (`9dd3da7`).
- **H7 retired by ownership, not by compound keys** — extending `68f3a89`'s
  one-registered-object pattern (§4 above): `DedupState` + `RoomHashCache` as client
  fields; the 3 process-global mutable maps went to 0 (`643bd73`).
- **H8: the usertile URL is keyed by client** — the sha256-hex `BridgeLinkToken`
  (`bridge_link.rs:65`), the pre-existing SOAP repository key, extending `506eab3`'s
  tickets-are-not-secret localhost trust model rather than inventing an auth mechanism.
  Coverage honesty: no runtime e2e test exercises the route; the msnp fixture pin
  (`get_profile.rs:314`) carries the fix (`26a6604`).
- **The §1.1 echo gap was retired by consolidation, with the delta named** — one
  `RoomOp::SendText` executor; the UUM path gained own-echo suppression + per-room
  send-lock serialization it previously bypassed (`643bd73`; §5 above).
- **Defect canaries are what compiler enumeration looks like when it bites.** Adding
  `ClientOut::DirectToWebLogin` broke three exhaustive 2-arm matches; wildcards being
  barred, the ruling was three loud warn+drop canary arms (chg render, adl send loop,
  `send_client_outs`) — unreachable by construction, documented at each site, no panic
  arms. The compiler enumerated the affected sites; the canaries make any future
  misrouting loud instead of silent (`8871758`).
- **Sender discipline: pattern A default, pattern B sanctioned and named.** `Sender`
  left handler signatures so decide/render are socket-free (pattern A); where a detached
  spawn legitimately keeps its `Sender` (`adl`'s avatar fan-out, SB `ready`/`msg`), that
  is pattern B — recorded in the commit bodies, not hidden in a signature
  (`4e1767b`…`eccaf83`, `5193cfc`, `89c9494`). `xfr` stayed deliberately dialect-free:
  the SB-vs-NS fork belongs to `deliver()` (`18ee489`).
- **The conformance script stays same-file.** `msnp18_dialect_conformance_script` lives
  in `e2e_msn_homeserver.rs`, reusing `Run`/`Stack`/`MsnClient`; a `tests/common`
  extraction is deferred until M1's second script needs it — the "fields move in with
  their consumers" doctrine applied to test harnesses (`cc5570b`).
- **Fixture isolation is a rule, not a preference.** Two bootstrap tests raced one shared
  temp-dir credentials store (SQLite error 14, ~40% failure under parallel scheduling);
  the fix is per-test unique names, codified in `doc/adding-a-dialect.md` §4 (`b951cc7`).
- **The contributor template gets template hygiene.** A pre-existing `clippy::never_loop`
  error in the e2e `chg_echo` helper — invisible to the workspace gate, which never
  compiles test targets, and predating the plan — was fixed because `cc5570b` canonized
  the harness as the template contributors copy; `--all-targets` clippy is clean for the
  first time since before D0 (`fea244e`).
- **Quirks preserved by design, recorded not fixed:** the double `SessionId::random()`
  in `switchboard_service` initialize (handle vs RNG, `switchboard_service.rs:69,85`),
  `insert()`'s pending-events merge, the SB-local `ConnectionPhase` enum,
  `expect("To be here")` + its TODO (`switchboard/handlers/mod.rs:90`),
  `ROOM_USER_PORTAL_MODE` (`bootstrap_handlers.rs:22`), and the
  `//TODO SEND ERROR BACK TO Client` pair (`notification_server.rs:146`,
  `switchboard_server.rs:123`) — plus the four known-red fixtures (§8) and the
  comment-interior `from_utf8_unchecked` dead text in `msnp/p2p/v2/mod.rs` (a
  comment-accuracy item; `msnp` frozen by policy). M1 (MSNP21) and M2 (pre-MSNP18 +
  browser auth) stay deferred with their entry requirements (architecture.md §7).
