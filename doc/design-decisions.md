# Tachyon — Design Decisions Record

**Status:** historical record — every entry cites the commit(s) where the decision was
taken. This is the *companion* of `doc/architecture.md`: that document is a *proposal*
for an overhaul not yet started; this one records what the code actually grew and why,
as visible in 372 commits (2022-04 → 2026-11, single author `aeoncl`, tag `0.1.0`).

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
  `matrix/extensions/msn_user_resolver.rs`).
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
- **Credential persistence: sled → bundled SQLite.** Multiple clients sharing one sled
  store was a known bug (`d19f870`); the new auth replaced it with `crates/tachyon-sqlite-store`
  (rusqlite, credentials keyed per user, `506eab3`).
- **Device ID: hostname-derived `TAK-<hostname>` → random, persisted per-user** in the
  data dir's `.device_id` file (`9a45555`; old `TachyonDeviceId::from_hostname` deleted).

## 5. Transport & concurrency choices

- **Command flow uses tokio mpsc, deliberately *unbounded*** — broadcast channels with
  capacity were replaced (`17991e8`) so a slow socket never drops protocol events. The
  flip side (10,000,000-capacity SB channel, `write_all().await.unwrap()`,
  `from_utf8_unchecked`, `rayon` in async) is **known and deliberately unfixed** — logged
  in architecture.md Appendix B and AGENTS.md with the policy "fixing one is a real
  change, not a drive-by cleanup".
- **The parser and the handlers must never spin**: malformed input was made to break out
  of the command-parse loop instead of looping forever (`2beaf6b`), and MSG handling
  stopped awaiting Matrix sends inline on the connection task — payloads are dispatched
  to a spawned task instead (`e5b4897`).
- **Outbound Matrix message dedup is per-room with an async send-lock**, because a global
  `DashMap<(event,room)` key under a `std::sync::Mutex` raced (`6ffaaa2`). DashMap
  re-entrancy got its own rule: remove-by-predicate must use `retain`, never iterate-
  then-remove (deadlock, `550ff20`).
- **Chunked MSG is reassembled at the codec level** (1311-byte chunks, `24d0a1a`),
  failures answer `NAK` (`328aa99`); raw payloads trim stray carriage returns (`13bc537`).

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
  (`b1109ed`, AGENTS.md).
- **A known-red baseline is tolerated, recorded, not chased**: 4 msnobj URL-encoding
  fixture failures in `msnp` are the documented status quo (`e738db8`).
- **Never `cargo fmt --all`** — the tree is deliberately not rustfmt-clean; match the
  surrounding file (`e738db8`).
- **`doc/architecture.md` is a proposal with explicit non-decisions**: Phase 0 not
  approved, other MSN versions out of scope, transport hazards deferred to named phases,
  proxy-group and MPOP deferral documented as intentional (§18). Code refactoring toward
  it is forbidden until approved (AGENTS.md).
