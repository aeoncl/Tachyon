# Plan — `BridgeLinkToken`: core mints the token a bridge names an account by

Status: **proposed** (2026-09-13), **implemented 2026-09-14** (subtasks 0 to 5 on
`refactor/third-times-the-charm`; the end-to-end run with the client is still to do). Branch:
`refactor/third-times-the-charm`.
Follows `docs/plans/2026-09-13-auth-use-case-login-identity.md`. Companion to
`docs/architecture/tachyon-ports-adapters.md`; subtask 3 folds the outcome back into the docs.

## Context

Today the MSN bridge derives the ticket it hands the client from `local.key` and the address,
`sha1(secret:email)`, in `crates/tachyon/src/tachyon/identifiers/ticket.rs`, and wraps the same
string as core's `TachyonToken`. Two things are wrong with that. The value that decides whether
two bridges for the same user get two backend logins is minted outside core, in one bridge, so a
second bridge would have to copy the recipe. And the glossary calls the token "random, expiring"
when it is neither, and never needed to be.

The decision: a **`BridgeLinkToken`** is a bridge's name for one client of an account,
deterministic from `(BridgeId, UserId, ClientVersion)`, minted by core, opaque to everyone else.
`TachyonToken` already plays that role minus the version, so this is a rename plus a
constructor. The MSN bridge wraps the minted value as the wire `TicketToken`; RST2 returns it;
USR compares what the client echoes to what core mints. The secret file goes.

The client version is in the token so that two MSN versions can be signed in to one account at
once, each with its own backend login and device. `ClientVersion` is major and minor only,
"14.0" from `14.0.8117.0416`: a build update must not orphan a login. A machine id is not an
input. The store is per machine and not synced, so a token never meets one from another
machine, and computing a machine id is infrastructure core cannot do; it can be added as a
value the bin passes in if a shared store ever exists.

One login per token becomes a refusal (subtask 4), which with the version in the token means
one client of a given version per user per bridge: a second connection for the same token,
while a login for it is live, is denied instead of replacing a pending login or joining a ready
one. The cost is that a dead connection must release its login
quickly, or the same client reconnecting after a network blip is locked out until the old socket
is noticed. Subtask 4 carries that prerequisite. Once it holds, the `Attempt` from the previous
plan is retired (subtask 5): the slot can only change hands after the old connection's own
teardown, so callers without a session can abandon by token again, and callers with a session
identify their login by that session.

## Scope

In: `BridgeId`, `BridgeLinkToken::mint`, the rename, the MSN bridge minting through core, the
secret's removal, the docs.

Out: checking the password at RST2 (the argon2 item in the architecture doc), token expiry or
rotation, two clients for one user on one bridge, anything in `AuthUseCase`.

## Constraints

- Minting is a pure function of two values. It lives in `domain`, not behind a use case, so a
  test fixture can name a token without building the use case, and `domain_is_pure` stays green.
- No new crate in core. `uuid` is already a dependency; v5 over `bridge:user` is deterministic,
  fixed-length and URL-safe, which the ticket needs because it travels in query strings and
  cookies. Enable the `v5` feature.
- Address case must not matter. Today the derivation lowercases the address, and the MSN client
  is not consistent about case between RST2 and USR. `EmailAddress::to_owned_user_id` does not
  normalise, so `mint` lowercases its input. The bridge-level test for this moves to core.
- `BridgeLinkToken::new(raw)` stays. It is the boundary parser for a value read back from the
  wire, a cookie or the store. Only `mint` creates a new one.
- The version reaches RST2 in the request's `User-Agent` header, whose tail reads
  `App msnmsgr.exe, 14.0.8117.416, {7108E71A-9926-4FCB-BCC9-9A9D3F32E423}` (captured
  2026-09-14 from WLM 2009). `CVR` spells the same build `14.0.8117.0416`, so the two sources
  agree only on major and minor, which is one more reason the token stops there. The `msnp`
  crate gets the header parser; the SOAP envelope's `ps:AuthInfo` is not needed.
- The `Debug` impl stays redacted.

## Existing data

Every stored login is bound to a token from the old derivation, and nothing in the store can
rebuild the new token from the old row. Pre-alpha with one user, so no migration: wipe the data
directory before the first start after subtask 2. That removes the stale token rows, the store
directories they kept alive, and `local.key` itself, which no code reads any more. The old
device stays on the homeserver until removed from another client.

## Alternatives

**A. Keep the secret, add the bridge id to the hash.** Rejected. The secret adds nothing on a
loopback bridge: the ticket is not a credential (RST2 checks no password) and the only thing a
secret bought, "not derivable without `local.key`", protects nothing the network cannot already
reach. Removing it removes a file, a startup step and a test.

**B. Mint behind `AuthUseCase::issue_token`.** Rejected. A use-case call to compute a hash, and
every fixture would need the use case to name a token.

**C. Rename `TachyonToken` in place plus a `mint` constructor in the domain.** Chosen.

## Applicable skills

`tdd` for subtask 1 (the constructor's tests come first). `/deslop` on each diff. `unslop` on
the doc comments and this file. `babysit` after the PR opens. `interrogate` is not needed; the
design was settled in conversation and has one moving part.

## Subtask 0 — rename `TachyonToken` to `BridgeLinkToken`

**Files:** every `.rs` that names it (22 files across `tachyon-core`, `tachyon-store-sqlite`,
`tachyon-testkit`, `tachyon`), `CONTEXT.md`, `docs/architecture/tachyon-ports-adapters.md`.

- A mechanical rename, one commit, nothing else in it. Build the lever: a `sed` over the file
  list, then `cargo build --workspace` to prove it. The old plan documents keep the old name;
  they are history.
- The doc comment on the type says what it is: a bridge's name for an account, minted by
  `mint`, parsed by `new` at boundaries.

**Done when:** `cargo test --workspace --exclude msnp` green and `grep -rn TachyonToken crates`
is empty.

---

## Subtask 1 — `BridgeId` and `BridgeLinkToken::mint`

**Files:** `crates/tachyon-core/src/domain/ids.rs`, `src/domain/auth.rs`,
`crates/tachyon-core/Cargo.toml`.

- `str_id!(BridgeId)` and `str_id!(ClientVersion)` next to the other ids. Core does not parse
  version strings; the bridge hands it "14.0".
- `impl BridgeLinkToken { pub fn mint(bridge: &BridgeId, user: &UserId, client: &ClientVersion)
  -> Self }`, a v5 UUID in simple form over `"{bridge}:{user}:{client}"` with `user` lowercased,
  under one fixed namespace UUID declared beside it.
- Tests in `domain/auth.rs`, written first: the same inputs mint the same token; two bridges
  mint different tokens for one user; two users mint different tokens on one bridge; two client
  versions mint different tokens for one user on one bridge; the user's case does not change
  the token. These replace the four tests in `ticket.rs`, with "different secrets" dropped
  because there is no secret.

**Done when:** the five tests green, `domain_is_pure` green.

---

## Subtask 2 — the MSN bridge mints through core, the secret goes

**Files:** `crates/msnp/src/shared/models/client_version.rs` (new),
`crates/tachyon/src/tachyon/global_state.rs`, `src/main.rs`,
`src/notification/handlers/negotiation.rs`, `src/notification/handlers/auth.rs`,
`src/notification/models/local_client_data.rs`, `src/web/soap/rst2.rs`,
`src/tachyon/identifiers/ticket.rs` (delete), `src/tachyon/identifiers/mod.rs`, `Cargo.toml` of
`tachyon` if `rand` and `sha1` lose their last use.

- `msnp` gets a `ClientVersion` parsed from two places: the `CVR` `client_ver` field, and the
  `App <exe>, <version>, {<guid>}` tail of the RST2 `User-Agent`. Both reduce to major and
  minor. Tests: `14.0.8117.0416` and `14.0.8117.416` both parse to `14.0`; a `User-Agent`
  without the `App` tail does not parse. (User-Agent example String: "Mozilla/4.0 (compatible; MSIE 6.0; Windows NT 6.2; WOW64; .NET4.0C; .NET4.0E; .NET CLR 2.0.50727; .NET CLR 3.0.30729; .NET CLR 3.5.30729; IDCRL 5.000.819.1; IDCRL-cfg 16.0.27832.0; App msnmsgr.exe, 14.0.8117.416, {7108E71A-9926-4FCB-BCC9-9A9D3F32E423})")
- The `CVR` handler keeps the version on the connection's local data. Today it answers and
  drops it.
- `GlobalState` holds `bridge: BridgeId` set to `"msn"` by the bin, in place of `token_secret`.
  `token_for(email, client)` mints from the address's user id and the version;
  `ticket_for(email, client)` wraps the same value in `TicketToken`. RST2 mints with the version
  from its `User-Agent` and refuses a request without one; `USR S` mints with the connection's
  `CVR` version and compares, as it does today, so a client that says one thing to RST2 and
  another to the notification server fails the ticket check. `derive_ticket`, `derive_token`
  and their tests go.
- `main.rs` loses `setup_key` and the `local.key` read, so the bridge never creates or opens
  that file again. `GlobalState::new` loses the secret parameter. If `rand` has no other use in
  the `tachyon` crate, it leaves `Cargo.toml` too.
- The notification server tests already send `CVR` with `14.0.8117.0416` and build the `USR S`
  line from `ticket_for`; they pass the version too. The RST2 handler gets one test with the
  captured `User-Agent` and one without the `App` tail.

**Done when:** `cargo test -p tachyon` green, `grep -rn "local.key\|token_secret\|derive_ticket"
crates/tachyon/src` empty, and a fresh data directory starts without creating a key file.

---

## Subtask 3 — docs

**Files:** `CONTEXT.md`, `docs/architecture/tachyon-ports-adapters.md`.

- `CONTEXT.md`: the `BridgeLinkToken` entry says derived from the bridge id, the user id and
  the client version, stable, one per client version per user per bridge, not a credential,
  minted only by core. `BridgeId` and `ClientVersion` get entries.
- Architecture doc: the "opaque, random, expiring" line and the "derived rather than issued;
  expiry pending" migration item say what is true now; the config section drops the secret-key
  path; "Ticket issuance ... are core logic" becomes minting.

**Done when:** `grep -n "random, expiring\|secret-key\|local.key"` over both docs is empty.

---

## Subtask 4 — one live login per token, so one client per version per user per bridge

**Files:** `crates/tachyon-core/src/application/auth_use_case.rs`, `src/application/error.rs`,
`crates/tachyon-core/tests/auth.rs`, `crates/tachyon/src/notification/handlers/auth.rs`,
`crates/tachyon/src/notification/notification_server.rs`, `src/tachyon/global_state.rs`.

The rule is core's, since the registry is core's: `sign_in` on a token that already holds a
login, pending or ready, returns `AuthError::AlreadySignedIn` and touches nothing. The two
`sign_in` arms that replace a pending login and join a ready one go. The USR handler answers
that error with `OUT` and closes the connection; the `msnp` crate already has the bare `OUT`
server command.

Two things have to be true for that rule not to lock the owner out after a network blip.

- **Teardown finishes before the socket is gone.** `ClientDropGuard::drop` spawns `abandon`
  today, so a reconnect can arrive while the old login is still in the slot and be refused.
  The abandon becomes an awaited step at the end of `handle_client`, after the read task
  ends, and the guard keeps only the client-repository removal, which is synchronous.
- **A dead connection is noticed within a minute or two.** The server answers `PNG` with
  `QNG 120` and nothing times a client out, so a half-open socket, the cable-pull case,
  holds its login until the OS gives up on it. Add a server-side deadline on the read side:
  no command for longer than twice the `QNG` interval ends the connection the same way EOF
  does. Both live in `notification_server.rs`.

Tests. In core, written first: a second `sign_in` while a login is pending is refused and the
first login is untouched, its waiter still parked; a second `sign_in` on a ready login is
refused and `restore_calls` stays at one; after `abandon`, `sign_in` succeeds again. The
existing `a_second_sign_in_replaces_a_pending_login` becomes the first of those, and
`two_concurrent_sign_ins_build_only_one_session` changes meaning: the loser is refused and
the winner is ready. In the notification server, `a_sign_in_that_lost_its_login_to_a_second_
connection_leaves_that_one_alone` becomes "a second connection of the same version for a
signed-in user is sent OUT", a new case sends a different version in `CVR` and shows both
clients signed in with two backend logins, and another drops the first client and shows a
same-version reconnect signs in without waiting for the deadline.

**A note for the owner before this subtask.** MSN's own behaviour on a second sign-in is the
opposite: the new connection wins and the old one receives `OUT OTH`. The code before this
subtask matches that minus the `OUT OTH`. Refusing the newcomer is a product choice that
trades the cable-pull case for predictability, and the two prerequisites above are what make it
livable. If the `OUT OTH` semantic is preferred after all, this subtask becomes "send `OUT OTH`
to the connection being replaced", the two prerequisites still apply, and nothing in core
changes.

**Done when:** the core and notification tests green; with the client signed in, a second
client for the same address is sent `OUT`; pulling the first client's cable and reconnecting
within three minutes signs in without a browser step.

---

## Subtask 5 — retire `Attempt`

**Files:** `crates/tachyon-core/src/application/logins.rs`, `src/application/auth_use_case.rs`,
`crates/tachyon-core/tests/auth.rs`, `crates/tachyon/src/notification/handlers/auth.rs`,
`src/tachyon/global_state.rs`, `CONTEXT.md`.

Only after subtask 4. Two facts make it safe. A second sign-in is refused while a login is live,
and the abandon is awaited before the socket closes, so the slot changes hands only after the old
connection's own teardown. The drop guard and the USR failure path therefore only ever hit their
own login or an empty slot, and `abandon(token)` is enough for them. The browser callback and the
parked waiter can still be mid-flight across a teardown and a reconnect, and they hold the
session, so the session is their identity.

- `Logins::replace_if` and `remove_if` take `expected: &Arc<dyn BackendSession>` and compare with
  `Arc::ptr_eq`. The check lives in the registry, so no caller can forget it.
- `Attempt`, `Logins::mint`, the `attempt` field on `Login`, the attempt on `SignIn`, and the
  attempt on `ClientDropGuard` go. `abandon(token, attempt)` becomes `abandon(token)`.
- Tests. `abandoning_with_a_stale_attempt_leaves_the_current_login_alone` goes. The stale
  callback test keeps its meaning but abandons the first login before the second sign-in,
  the only order subtask 4 allows. `a_login_abandoned_while_authenticating_is_logged_out_once_
  the_callback_returns` and `a_callback_for_a_replaced_login_binds_nothing_in_the_store` stay
  as they are; both already work through the session.
- `CONTEXT.md` loses the `Attempt` entry and says the session is the identity for a late
  callback or waiter.

**Done when:** `grep -rn Attempt crates` is empty and the core and notification suites are
green.

---

## Verification

Static, per subtask: the commands under each "done when", then `cargo test --workspace
--exclude msnp` before the PR. The five `msnp` failures predate this branch.

Runtime, after subtask 2, by the owner: wipe the data directory, start the bridge and confirm
no `local.key` appears; sign the MSN client in and confirm it goes through the browser once, then
reconnects and relaunches without the browser. There is no control skill for the MSN client, so
this run is yours.

## Implementation guidance

Read `principle-boundary-discipline` before subtask 1 and
`principle-migrate-callers-then-delete-legacy-apis` before subtask 2. Each subtask is one commit
with its tests, the rename strictly alone. `/deslop` before each commit. No narrating comments.

## Implementation notes (2026-09-14)

- Subtask 4's dead-connection detection is TCP keepalive on every accepted socket (30 s
  idle, 10 s probes) rather than a read deadline over the `QNG` interval. A parked sign-in
  can be silent for the whole five-minute browser window, so a protocol-level deadline
  would have cut legitimate sign-ins short; keepalive notices a vanished peer regardless of
  phase. It cannot be exercised in the harness; the cable-pull run is the owner's.
- The awaited teardown shortens the window in which a reconnect can arrive before the old
  login is gone, it does not close it. A real client needs several round trips before its
  `USR S`, and the harness test that reconnects immediately passes five times out of five,
  but a reconnect inside that window would be sent `OUT` once. If that ever shows up, the
  endpoint GUID in `USR S` is the same-client signal a takeover could key on.
- The RST2 handler has no test of its own; the `User-Agent` parsing is pinned in `msnp`
  and the handler's refusal is one line.
