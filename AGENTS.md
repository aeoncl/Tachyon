# Tachyon

MSNP18 ↔ Matrix translation layer in Rust. A Windows Live Messenger 2011/WSM 8.5
client talks to three servers that this repo hosts: a Notification Server (TCP), a
Switchboard/Mixer server (TCP, one connection per room), and an axum HTTP server for
the SOAP services the client expects (ABCH address book, Sharing, Storage, RSI, RST2
ticket issuer) plus the login/device-verification web UI. Outbound side is the
vendored matrix-rust-sdk.

## Layout

- `crates/msnp` — wire codec only: framing (`raw_command_parser`), one file per
  verb (`adl.rs`, `usr.rs`, `msg.rs`…), `<Verb>Client`/`<Verb>Server` structs
  implementing `TryFromRawCommand`/`IntoBytes`, plus SOAP envelopes (yaserde).
- `crates/tachyon` — the app: `notification/`, `switchboard/`, `web/` (SOAP + UI),
  `matrix/` (handlers, sync, services), `p2p/`, `audio/`, `tachyon/` (domain-ish
  objects, config, `global_state`). `main.rs` is a thin composition root; `lib.rs`
  exposes `run(config, shutdown_signal)`.
- `crates/tachyon-sqlite-store` — rusqlite credentials store (bundled SQLite).
- `lib/libsiren`, `lib/matrix-rust-sdk` — git submodules (the SDK is a fork on
  branch `upstream-main`, excluded from the workspace, consumed as a path dep).
- `doc/architecture.md` — **proposal, not the current code** (see Pitfalls).

## Setup and commands

```sh
git submodule update --init --recursive    # build fails without libsiren + the SDK
cargo build                                # build.rs needs a C compiler (cc) for libsiren
cargo test                                 # workspace unit tests
cargo test -p tachyon --lib               # app unit tests only (~27 tests, fast)
cargo test -p msnp <test_name>             # single wire-codec test
cargo test -p tachyon --test e2e_msn_homeserver -- --ignored   # needs Docker
cargo clippy --workspace
```

Rust 2021, edition-pinned per crate; `cargo test` on a warm target is ~45 s.
Runtime requirement (not compile-time): `ffmpeg` with `msnsiren` **and** `libopus`
on `PATH`, used for voice clips (`src/audio/ffmpeg.rs`).

## Conventions actually followed here

- Tests are inline `#[cfg(test)] mod tests` next to the code, with prose names
  (`a_dead_http_candidate_stays_unreachable_not_policy`). One integration test only.
- NS/SB handlers are free functions `handle_<verb>(command_sender:
  Sender<...ServerCommand>) -> Result<(), anyhow::Error>` in
  `*/handlers/ready/<verb>_handler.rs`; `command_handler.rs` dispatches them by
  connection `phase`.
- Wire round-trips are asserted as exact byte strings in `msnp` tests — change a
  renderer and you must update its fixture deliberately.
- Errors: `thiserror` for typed library errors, `anyhow` at handler/server edges.
- `mod.rs` for module declarations; CRLF line endings; 4-space indent.
- Commits: Conventional Commits, lowercase — `fix: …`, `feat(slop): …`,
  `refactor(auth): …`, `chore: bump matrix-rust-sdk`.
- MSNP18 is the only negotiated version (`notification/handlers/negotiation.rs`).

## Pitfalls

- **`cargo test` is red on a clean `main`.** 4 known failures in `msnp`
  (`nln`/`iln` msnobj URL-encoding fixtures). Don't chase them; treat 212 passed /
  4 failed as the baseline, and make sure your change doesn't add a 5th.
- **Never run `cargo fmt --all`** — the tree is not rustfmt-clean (a ~40k-line
  diff). Match the surrounding file instead.
- The root workspace manifest is committed as **`cargo.toml` (lowercase)**. It only
  resolves because this filesystem and `core.ignorecase=true` are case-insensitive;
  don't rename it or add a second `Cargo.toml` without checking on Linux.
- `doc/architecture.md` describes crates that **don't exist yet** (`msn-models`,
  `tachyon-domain`, `msn/dialect/`). Phase 0 is explicitly unapproved — do not
  start refactoring toward it unless the user asks.
- Ports: code defaults are `11863/11864/11866` bound to `127.0.0.1`, not the
  `1863/1864` in the README. A stock `msnmsgr.exe` needs the `zathras` patch +
  config.ini to talk to it.
- Config and state live outside the repo: `TACHYON_HOME` env var if set, else
  platform `ProjectDirs("Tachyon")` (`config.ini` there is auto-written from
  defaults on first run). `logs_enabled` defaults to **false** — turn it on before
  debugging, and note `logs.txt` is truncated with `File::create` each start.
- The logger sets `matrix-sdk` to `LevelFilter::Off`, so Matrix-side problems are
  invisible in logs by default.
- The e2e test mutates process-global `TACHYON_*` env and serializes itself with a
  static mutex; run it with `--ignored` alone, not mixed into a normal test run.
- `lib/matrix-rust-sdk` is a fork: bumping it means updating the submodule pointer
  (see `chore: bump matrix-rust-sdk`), not editing a version in `Cargo.toml`.
- Known transport hazards (logged in architecture.md Appendix B, deliberately not
  fixed): 10,000,000-capacity switchboard mpsc, `write_all(...).await.unwrap()` in
  both write loops, `from_utf8_unchecked` in debug logs, `rayon` inside async code.
  Fixing one is a real change, not a drive-by cleanup.
