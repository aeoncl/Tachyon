# Plan — Login identity in `AuthUseCase` (fix the by-token races, shrink the file)

Status: **proposed** (2026-09-13), **implemented 2026-09-13** (subtasks 0 to 6 and 8 on
`refactor/third-times-the-charm`; subtask 7 skipped, see below; the end-to-end run against
the MAS homeserver is still to do). Branch: `refactor/third-times-the-charm`.
Follows the 2026-09-13 review of `crates/tachyon-core/src/application/auth_use_case.rs`
(three independent critics plus a runtime check). Companion to
`docs/architecture/tachyon-ports-adapters.md`; subtask 8 folds the outcome back into the docs.

## Implementation notes (2026-09-13)

- Subtask 5a and 5b landed as one commit: the callers cannot compile against the old
  `abandon` once `SignIn` carries the attempt, so one wave.
- The 5b harness test is the two-connections case, not the reconnect-after-`Ready` case.
  A `Ready` sign-in cannot complete in the notification harness because the USR handler
  still downcasts the session to `BackendSessionMatrix` and the fake is not one. The
  reconnect ordering is covered by the core test `abandoning_with_a_stale_attempt_leaves_the_current_login_alone`
  plus the drop guard reading its own attempt.
- Subtask 7 skipped. A `Step` variant that exists only to refuse a double-click adds an arm
  to every match on `Step`, and the harm it prevents is one extra device from a rare
  double-submit. Revisit if it shows up in practice.

## Context

`AuthUseCase` is one state machine over one invariant: a token names at most one live login, and
every transition is either serialized by the `lifecycle` mutex or identity-checked against the
session the caller was looking at. The two identity-checked sites (`promote`,
`settle_authenticated`) are correct. The two removal sites that skip the check are the bugs the
review found:

1. `finish_login`'s failure path calls `abandon(&token)` and discards whatever login holds the
   token now, which after a replacement is somebody else's fresh sign-in.
2. `ClientDropGuard` spawns `abandon(&token)` from `Drop`; a fast reconnect can win the lock, be
   handed the old `Ready` session, and then have it closed underneath.
3. A login that authenticated on the homeserver but never settled is dropped through the
   `Step::Authenticate` arm, which `discard`s without `log_out`. The device stays on the account
   with no keys anywhere. No race is needed for this one; any store or `device_status` failure
   right after a successful OAuth does it.
4. `settle_authenticated` writes `save_login_for_token` before checking the login is still its own.

The review also declined splitting the use case by caller. All three halves (MSN sign-in, browser
completion, maintenance) mutate the same registry under the same mutex, and the tests in
`tests/auth.rs` drive the halves against each other. What shrinks the file is moving the
transitions down into `Logins` and `WebUrls` out to its own module.

### Lifecycle of one login, after this plan

`Logins` mints an `Attempt` every time a login is inserted under a token. It is the identity of
one occupancy of the token slot. A restored login and a fresh interactive one get different
attempts even when they share a `LoginId`. Every path that removes or replaces a login names the
attempt it is acting on, and `Logins` refuses when the slot holds another. `Arc::ptr_eq` on the
session goes away; the attempt is the one identity.

```
sign_in ─────────────┐
                     ▼
              Pending{Authenticate} ──finish_login (authenticate on the wire, no lock)──┐
                     │                                                                  │
      abandon(attempt)│                                     settle under the lock, if still ours
                     ▼                                                                  ▼
                  Absent                                            Pending{VerifyDevice} / Ready
                     ▲                                                                  │
                     └──────── abandon(attempt) / forget ◄──────────────────────────────┘
```

Outcomes for a login that authenticated but could not settle:

- still ours, `device_status` or the store failed: `close`, keep the row. The next `sign_in`
  restores it. Same rule `sign_in` already applies when a restored session's status cannot be read.
- no longer ours (abandoned or replaced while `authenticate` was on the wire): `log_out` then
  `discard`. Nothing points at it, so the device must not outlive it.

## Scope

In: the four findings above, the `Attempt` identity in `Logins`, `WebUrls` moved out,
`FinishedLogin` carrying the confirm-device URL, the password double-submit guard, the glossary.

Out: splitting `AuthUseCase` into several use cases (declined, see Alternatives), per-token
locks, a caller for `forget`, `sweep`, any change to `DeviceVerificationUseCase`.

## Constraints

- One client per instance. The global `lifecycle: Mutex<()>` stays. It is what makes two
  concurrent sign-ins build one session (`two_concurrent_sign_ins_build_only_one_session`).
- `authenticate` stays outside the lock. It is a homeserver round trip.
- Bridges never see an unready session (`CONTEXT.md`, "Backend seam"). Whatever identity a caller
  passes back to `abandon` must not be the session.
- `log_out` remains the only call that ends a device, and `abandon` of a never-authenticated login
  still never logs out. The two `log_out_calls() == 0` pins in `tests/auth.rs` stay as written.
- Tests over the real `Logins` and the testkit fakes, as `tests/auth.rs` does today. No tests of
  the fakes themselves.

## Alternatives

**A. Split by caller** (`SignInUseCase`, `BrowserLoginUseCase`, maintenance). Rejected. Every
half mutates the registry and needs the mutex and `drop_login`, so it is three structs over one
`Arc<Logins>`, one `Arc<Mutex<()>>` and a shared helper. The invariant would live in three files.

**B. `Arc::ptr_eq` everywhere, callers hold the session.** Rejected. The MSN failure path and the
drop guard would need the session before it is ready, which the seam forbids.

**C. An `Attempt` id minted by `Logins`, global mutex kept.** Chosen. One `Copy` value that any
caller can hold, one guarded `replace` and one guarded `remove` on the registry, and the two
existing `ptr_eq` sites collapse onto them.

**D. Per-token locks.** Rejected. Hypothetical contention on a one-client instance.

## Applicable skills

`how` is done (this review). `tdd` for subtasks 2 to 5 and 7: each bug gets its failing test
first. `interrogate` on the subtask 4 outcome rules and the subtask 5 API before they ship.
`/deslop` on every diff, `unslop` on doc comments and this file, `babysit` after the PR opens.

## Subtask 0 — `Attempt` and the guarded transitions in `Logins`

**Files:** `crates/tachyon-core/src/application/logins.rs`, `src/application/auth_use_case.rs`.

- `Login` gains `attempt: Attempt`, a `Copy` newtype over a counter `Logins` owns. `Logins::insert`
  mints it and returns it.
- `Logins::replace_if(token, attempt, login) -> Option<Attempt>` and
  `Logins::remove_if(token, attempt) -> Option<Login>` on the `DashMap` entry API. Both keep the
  flow index in step the way `insert` and `remove` do today.
- `promote` and `settle_authenticated` use `replace_if` with the attempt of the snapshot they
  took. Their `Arc::ptr_eq` matches go.
- Behaviour-preserving. No public signature changes yet.

**Done when:** `cargo test -p tachyon-core --test auth --test device_verification` green with no
test edited.

---

## Subtask 1 — testkit: a parkable `authenticate`

**Files:** `crates/tachyon-testkit/src/fakes.rs`.

- `FakeBackendSession` gets a gate on `authenticate`: open by default, `park_authenticate()`
  makes the next call wait until `release_authenticate()`. This is how subtasks 2 to 5 put a
  replacement or an abandon between the snapshot and the settle.
- `log_out` on the fake works after `close` or `discard` (it counts the call). Subtask 4 needs
  the same from the adapter.

**Done when:** workspace compiles, `tests/auth.rs` green unchanged.

---

## Subtask 2 — settle checks identity before it writes the store

**Files:** `src/application/auth_use_case.rs`, `crates/tachyon-core/tests/auth.rs`.

- `settle_authenticated` reads the registry and compares the attempt first. Only then
  `save_login_for_token`, `device_status`, `replace_if`. The lock is held for the whole body, so
  the check at the top makes the function atomic.
- Test: a callback whose login was replaced while `authenticate` was parked returns
  `LoginNotFound` and the store still has no row for the token.

**Done when:** the new test is red before the reorder and green after; the rest unchanged.

---

## Subtask 3 — `finish_login` cleans up only its own attempt

**Files:** `src/application/auth_use_case.rs`, `crates/tachyon-core/tests/auth.rs`.

- The failure path after `authenticate` uses `remove_if(token, attempt)` under the lock and drops
  what it removed. It never calls `abandon(&token)`.
- Test: a stale callback fails while a second sign-in's login sits under the token. The second
  login is still there, its waiter is still parked, and its session has no `discard` or `close`.

**Done when:** the new test is red on the current code and green after.

---

## Subtask 4 — a login that authenticated but did not settle

**Files:** `src/application/auth_use_case.rs`,
`crates/tachyon-backend-matrix/src/infrastructure/backend/session.rs`,
`crates/tachyon-core/tests/auth.rs`.

- After a successful `authenticate`, `finish_login` applies the two outcome rules from the
  lifecycle section. Still ours and settle failed: `close`, keep the row, release the waiter. Not
  ours any more: `log_out`, then `discard`, and delete the row `drop_login` would have deleted.
- `drop_login`'s `Authenticate` arm is now provably only reached for a login that never
  authenticated. Its doc comment says so.
- Adapter: `BackendSessionMatrix::log_out` no longer requires the session to be open. It only
  needs the HTTP client. Update the `log_out` doc in `ports.rs` to name the one new caller.
- Tests: "a login that authenticated but whose device status cannot be read is kept for the next
  sign-in" (`close_calls == 1`, `discard_calls == 0`, `log_out_calls == 0`, row present, a second
  `sign_in` restores it). "A login abandoned while authenticating is logged out once the
  callback returns" (`log_out_calls == 1`, `discard_calls == 1`, no row, no login).
- Run `interrogate` on the two rules before merging. The "keep the row" rule is a product choice.

**Done when:** both new tests green, the two existing `log_out_calls() == 0` pins untouched.

---

## Subtask 5a — `abandon` takes the attempt, `SignIn` hands it out

**Files:** `src/application/auth_use_case.rs`, `crates/tachyon-core/tests/auth.rs`.

- `SignIn::Ready` and `SignIn::Pending` carry the `Attempt`. `abandon(token, attempt)` replaces
  `abandon(token)`. `abandon_flow` resolves the attempt under the lock from the flow's login and
  goes through the same `remove_if`.
- Test: abandoning with a stale attempt after a second sign-in leaves the second login alone.
- Existing tests pass the attempt they got from `sign_in_pending`.

**Done when:** `tests/auth.rs` green, no by-token removal left in the file.

---

## Subtask 5b — callers: the drop guard and the USR handler carry their attempt

**Files:** `crates/tachyon/src/tachyon/global_state.rs`,
`crates/tachyon/src/notification/handlers/auth.rs`,
`crates/tachyon/src/notification/notification_server.rs` (tests only).

- `ClientDropGuard` holds the attempt and passes it to `abandon`. It removes the `TachyonClient`
  from the repository only when the repository still holds the one it inserted; today a late
  drop removes a reconnected client.
- The USR handler's failure path passes the attempt it got from `sign_in`.
- Test in the notification server module, over `TestClient`: a client that reconnects right after
  a disconnect ends up `Ready` on a session with `close_calls == 0`, repeated enough times to
  cover both orderings of the spawned abandon.
- Deployment note: with concurrent serving, a second connection for the same account replaces the
  first's pending login. The first's timeout must not take the second down. The test above covers
  the reconnect ordering; add a second case for that overlap if `TestClient` can hold two
  connections at once.

**Done when:** `cargo test -p tachyon --lib notification` green including the new cases.

---

## Subtask 6 — `WebUrls` out, `FinishedLogin` carries its next URL

**Files:** `crates/tachyon-core/src/application/web_urls.rs` (new),
`src/application/auth_use_case.rs`, `crates/tachyon/src/web/tachyon/matrix_auth.rs`, plus the
`use` lines in `infrastructure/app_state.rs`, `tests/auth.rs`, `tests/device_verification.rs`.

- Move `WebUrls` verbatim. Migrate every importer in the same commit, no re-export.
- `FinishedLogin` gains `next_url: Option<String>`, the confirm-device URL when the device is
  unverified. `login_finished` in `matrix_auth.rs` redirects to it and the hardcoded
  `/tachyon/confirm_device?t=` goes.

**Done when:** workspace green, `grep confirm_device crates/tachyon/src` finds only the route.

---

## Subtask 7 — one `authenticate` on the wire per login (optional)

**Files:** `src/application/logins.rs`, `src/application/auth_use_case.rs`,
`crates/tachyon-core/tests/auth.rs`.

- A second `finish_login` for a flow whose `authenticate` is in flight creates a second device on
  the homeserver for a password login. Represent the in-flight state in `Step` (an
  `Authenticating` variant, or a flag on `Authenticate`) set through `replace_if` before the
  network call and cleared on a rejected credential. `wait_for_session` and `drop_login` treat
  it as `Authenticate`; `prompt` returns `None` for it.
- Test: a second submission while the first is parked is refused and `authenticate_calls == 1`.
- Optional. Do it last, and skip it if the `Step` change reads worse than the bug.

**Done when:** the new test green, existing pins untouched.

---

## Subtask 8 — docs

**Files:** `CONTEXT.md`, `docs/architecture/tachyon-ports-adapters.md`.

- The glossary still describes `Readiness`, `SessionRepository` and `LoginOutcome`. Replace with
  `Login`, `Step`, `Logins`, `Attempt`, and the two outcome rules for an authenticated login that
  did not settle. "If a name here and a name in code disagree, one of them is wrong" applies.
- The architecture doc's login-flow section gets the same names.

**Done when:** every term in the "Auth flow" section of `CONTEXT.md` exists in code.

---

## Verification

Static, per subtask: `cargo test -p tachyon-core`, `cargo test -p tachyon-testkit`,
`cargo test -p tachyon --lib notification`, then `cargo test --workspace` before the PR.
Filter warnings from the `tachyon` crate as `AGENT.md` allows.

Runtime, after subtask 5b: there is no control skill for the MSN client. The owner runs the
patched client against the MAS homeserver and does, in order: sign in fresh through the browser,
pull the network cable for ten seconds during the "signing in" screen and reconnect, kill and
relaunch the client within two seconds of a working session. After each, the Element device list
shows one bridge device and the client is signed in. The unit tests show each branch behaves; only
this run shows the ghost devices are gone.

## Implementation guidance

Read `principle-model-the-domain` and `principle-migrate-callers-then-delete-legacy-apis` before
subtasks 0 and 5. Each subtask is one commit with its test. `/deslop` before each commit. No
narrating comments; the assertion strings are the documentation. Keep a local decision trail via
`show-me-your-work` only if the subtask 4 rules change during implementation. `babysit` the PR.
