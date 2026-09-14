# Plan — The login ending table: every `log_out` in one match

Status: **proposed** (2026-09-14), **implemented 2026-09-14** (subtasks 0 to 3 on
`refactor/third-times-the-charm`, no test file edited). Branch: `refactor/third-times-the-charm`.
Follows finding 2 of the 2026-09-14 review of `docs/architecture/tachyon-ports-adapters.md`
(pure core, thin shell). Companion to `docs/plans/2026-09-13-auth-use-case-login-identity.md`,
whose outcome rules this makes a value. Subtask 3 folds the outcome back into the docs.

## Context

`AuthUseCase` decides what happens to a session and its store row in five places, each between
awaits: `drop_login` (a 22-line async match), the two failure arms of `settle_authenticated`,
the live and stored branches of `forget`, and the restore branch of `sign_in`. The rule that
matters most, `log_out` ends a device and must run in exactly two cases, is spread over four
functions and pinned only by two `log_out_calls() == 0` assertions that a wrong edit can
satisfy by accident. Every rule is provable only through a `FakeBackendSession` with counters,
a tokio runtime and an `eventually` poll.

Only one of those five sites is a table. The rest are one-liners, duplicated rather than
complex. This plan extracts the table, names the predicate, collapses the one duplication, and
leaves the rest alone.

What this plan does not buy, said plainly because the review implied otherwise: nothing in
`tachyon-testkit` gets deleted, and no async test in `tests/auth.rs` goes. The four race tests
(`two_concurrent_sign_ins_...`, `a_callback_for_a_replaced_login_...`,
`a_stale_callback_...`, `a_login_abandoned_while_authenticating_...`) prove the lifecycle lock
and the `Arc::ptr_eq` identity check. Those are the shell, and the shell needs a runtime. The
win is that the rules become readable on one screen, the `log_out` invariant is one
`assert_eq!`, and a new row costs one `#[test]` line instead of a fixture, a scripted session
and a poll.

### The table

```
Ending                              Effects, in order
Dropped { authenticated: false }    Discard, DeleteRow
Dropped { authenticated: true }     Close
Unsettled                           Close
Orphaned                            LogOut, Discard
Deleted                             LogOut, Discard, DeleteRow
```

- `Dropped`: the client went away or the authorization server refused the flow. Before the
  backend accepted a credential there is no device and nothing to restore.
- `Unsettled`: it authenticated but could not be settled while the token still held it. The
  row stays and the next sign-in restores it.
- `Orphaned`: it authenticated after the token stopped holding it. Nothing points at the device
  it just made, so the device is ended.
- `Deleted`: the user asked for the login to go, for good.

Ordering is load-bearing in one place: `LogOut` before `Discard`, so the device ends before its
on-disk state goes. Every effect is idempotent on its own, so a list that runs twice converges.

## Scope

In: `Effect`, `Ending`, `ending()` and their tests in the domain; `BackendError::ends_the_login`;
`Login::settled` and `Login::authenticated`; `AuthUseCase::end` and `run` replacing
`log_out_and_discard` and the inline matches; the glossary and the architecture doc.

Out: `DeviceVerificationUseCase`, `wait_for_session` and `promote` (registry calls and control
flow, not rules), the `SignIn` and `next_url` matches on `DeviceStatus` (caller-facing shapes,
one line per arm), the fakes, the ports, anything in `crates/tachyon`.

## Constraints

- The public API of `AuthUseCase` and `AuthError` do not change. No caller in `crates/tachyon`
  moves.
- The lifecycle mutex stays. `authenticate` stays outside it. The identity check stays the
  registry's job (`Logins::holds`, `replace_if`, `remove_if`).
- `Effect` carries no session and no `LoginId`. That is what makes it `Copy + PartialEq` and
  the tests `assert_eq!` on values. `run` takes the session and the id beside the list.
- The table lives in `domain/auth.rs`. `tests/architecture.rs::domain_is_pure` already forbids
  `tokio::`, `async fn` and `async_trait` there, so "no runtime, no fakes" is enforced by a
  test that exists. The predicate on `BackendError` cannot follow, since the error type is
  application-layer; it lives next to the type.
- `tests/auth.rs` is not edited in the extraction commit. A test that needs editing means the
  extraction changed behaviour, and the extraction is wrong.
- The two `log_out_calls() == 0` pins stay as written.

## Alternatives

**A. One `Ending` table returning `Vec<Effect>`, a `for` loop in the shell.** Chosen. Five
cells, one screen, every `log_out` visible in one match. The shell is `end` (notify, then run)
and `run` (twelve lines). No interpreter, no trait, no recorder.

**B. Per-decision plans: `on_drop(Stage) -> DropPlan { notify, session_effect, delete_row }`
and `on_restore_failure(&BackendError) -> RestorePlan`.** Rejected as the base. `DropPlan`
covers `drop_login` only; `settle_authenticated` and `forget` keep their inline `log_out`
calls, so the invariant stays spread over three functions. Its restore rule survives as
`BackendError::ends_the_login()`. Its `Stage` enum was tried and rejected: `VerifyDevice` and
`Ready` have identical rows everywhere, so the enum would exist to be collapsed in its one use.

**C. A whole-lifecycle `Transition` enum returning `(NextLogin, Vec<Effect>)` with the shell
as a driver.** Rejected. It needs `Step`'s payload, the `LoginId` and the registry's identity
answer threaded into a pure mirror of `Login` that cannot hold the session. Most of the diff
would be plumbing, and the decisions it adds (`promote`, `wait_for_session`) are control flow.

**D. `Notify` as an `Effect`.** Rejected. "Fire `changed` when the login being ended is
`Pending`" has no variation to express. It stays in `end`.

## Applicable skills

`tdd` for subtask 0: the table's tests come first. `/deslop` on each diff. `unslop` on the doc
comments and this file. `interrogate` is not needed; two candidates were compared and the
shape has one moving part.

## Subtask 0 — `Effect`, `Ending`, `ending()`

**Files:** `crates/tachyon-core/src/domain/auth.rs`.

```rust
#[derive(Clone, Copy, PartialEq, Eq, Debug)]
pub enum Effect { Close, Discard, LogOut, DeleteRow }

#[derive(Clone, Copy, PartialEq, Eq, Debug)]
pub enum Ending {
    Dropped { authenticated: bool },
    Unsettled,
    Orphaned,
    Deleted,
}

pub fn ending(ending: Ending) -> Vec<Effect> { /* the table above */ }
```

- `Vec` rather than `&'static [Effect]` so `assert_eq!(ending(..), [Close])` compiles without
  `.as_slice()`. Three elements on a login teardown is not a cost.
- Doc comments on the variants say what the table above says, one sentence each. No comment on
  the match arms; the test names are the documentation.
- Tests, written first, under `#[cfg(test)]` in the same file, plain `#[test]`:
  `a_login_dropped_before_it_authenticated_leaves_nothing_behind`,
  `only_an_orphan_and_a_deletion_end_the_device` (loops over the other three endings and
  asserts no `LogOut`), `a_login_that_authenticated_keeps_its_row_unless_the_user_deleted_it`.
  The second is the by-value twin of the two `log_out_calls() == 0` pins.

**Done when:** the three tests green, `cargo test -p tachyon-core --test architecture` green.

---

## Subtask 1 — the predicate and the collapsed constructor

**Files:** `crates/tachyon-core/src/application/error.rs`,
`crates/tachyon-core/src/application/logins.rs`.

- `BackendError::ends_the_login(&self) -> bool`, true for `LoggedOut`, `SoftLoggedOut` and
  `CannotRestoreLogin`. Doc comment: the backend has said the login is over, as opposed to
  being out of reach right now. One test:
  `an_unreachable_backend_does_not_cost_the_user_their_login`.
- `Login::settled(session, login_id, status: DeviceStatus) -> Login`: `Verified` to `ready`,
  `Unverified` to `pending` at `Step::VerifyDevice`. This deletes the second copy of that
  match, not its branchiness. It builds an `Arc<dyn BackendSession>`, so it stays in
  `application`.
- `Login::authenticated(&self) -> bool`: false only at `Step::Authenticate`.

**Done when:** workspace compiles, nothing calls the new items yet.

---

## Subtask 2 — the shell runs the table

**Files:** `crates/tachyon-core/src/application/auth_use_case.rs`.

The two new private functions:

```rust
/// Runs what `ending` decided against the login's session and row, and releases whoever is
/// waiting on it.
async fn end(&self, login: Login, why: Ending) -> Result<(), AuthError> {
    if let Login::Pending { changed, .. } = &login {
        changed.notify_one();
    }
    self.run(login.session(), login.login_id(), ending(why)).await
}

async fn run(
    &self,
    session: &Arc<dyn BackendSession>,
    login_id: &LoginId,
    effects: Vec<Effect>,
) -> Result<(), AuthError> {
    for effect in effects {
        match effect {
            Effect::Close => session.close().await,
            Effect::Discard => session.discard().await,
            Effect::LogOut => {
                if let Err(e) = session.log_out().await {
                    log::warn!("Could not log the device out, ending it anyway: {e:?}");
                }
            }
            Effect::DeleteRow => self.account_repository.delete_login(login_id).await?,
        }
    }
    Ok(())
}
```

The call sites, after:

- `drop_login(login)` becomes
  `self.end(login, Ending::Dropped { authenticated: login.authenticated() })`. Its doc comment
  goes; the table says it.
- `settle_authenticated`: the not-ours arm runs `ending(Ending::Orphaned)` then returns
  `LoginNotFound`; the bind-or-status failure arm does `remove_if` then runs
  `ending(Ending::Unsettled)`; the advance uses `Login::settled`. The identity check stays
  first, before `save_login_for_token`. That order is why `Orphaned` has no `DeleteRow`, and
  `a_callback_for_a_replaced_login_binds_nothing_in_the_store` is the guard.
- `forget`: the live branch is `self.end(login, Ending::Deleted)`. The stored branch: when
  `restore` succeeds, `run(&session, &login_id, ending(Ending::Deleted))` then
  `auth_service.forget(&login_id)`; when it fails, `forget_stored(&login_id)` as today. No path
  deletes the row twice. The hoisted `login_id_by_token` moves into the `None` arm.
- `sign_in`: the restore arm matches `Err(e) if e.ends_the_login()`; the device-status block
  inserts `Login::settled(..)` and maps the status to `SignIn` afterwards. The `Err` arm keeps
  its inline `session.close()`: a session that never entered the registry is not a login
  ending, and one line is shorter than a table row.
- Delete `log_out_and_discard`. Its warning moved into the `LogOut` arm.

Expected size: `drop_login` 22 lines to 3, `settle_authenticated` about -8, `forget` about -6,
`log_out_and_discard` -6, `end` and `run` +24. The file goes from 417 to roughly 375 lines.

**Done when:** `cargo test -p tachyon-core` green with `tests/auth.rs`, `tests/device_verification.rs`
and `tachyon-testkit` untouched; `grep -n "log_out()" crates/tachyon-core/src` finds only the
`LogOut` arm and the port.

---

## Subtask 3 — docs

**Files:** `CONTEXT.md`, `docs/architecture/tachyon-ports-adapters.md`.

- `CONTEXT.md`, "Auth flow": the **settle** entry names `Ending::Unsettled` and
  `Ending::Orphaned` instead of describing the two rules in prose. Add an **Ending** entry: why
  a live login ends, and the one function that says what ending it does; the only place a
  device is logged out.
- Architecture doc, "Interface style between bounded contexts": one sentence after "Async only
  where there is I/O": use cases decide in sync code over plain values and run the effects
  afterwards; a rule that needs a fake to test is a rule in the wrong place. "This is the one
  place a device is ever logged out" on `forget`'s doc comment moves to `Ending`.

**Done when:** every name in the "Auth flow" section exists in code, and
`grep -n "log_out_and_discard" CONTEXT.md docs` is empty.

---

## Verification

Static, per subtask: `cargo test -p tachyon-core`, then `cargo test --workspace --exclude msnp`
before the PR. The five `msnp` failures predate this branch. Warnings from the `tachyon` crate
filtered as `AGENT.md` allows.

Runtime: none needed. Subtask 2 is behaviour-preserving by construction; the unchanged
`tests/auth.rs` is the proof, and the pure tests add coverage the runtime cannot.

## Risks

- `run` stops at the first `DeleteRow` store error and skips what follows. Today `DeleteRow` is
  last in every list. If a future row puts it earlier, the loop needs to collect errors instead
  of `?`. Not worth it until then.
- `Ending::Dropped { authenticated }` reads the registry, which is stale at one moment: at
  settle time the slot still says `Step::Authenticate` although the backend accepted the
  credential. `Unsettled` exists as its own row so nobody reaches for `Dropped` there. The
  variant name is the guard; a type-level one is not worth buying.
- A crash between a successful `authenticate` and the `Orphaned` effects leaves a device on the
  account that nothing points at. Pre-existing, unchanged by this plan.

## Implementation guidance

Read `principle-model-the-domain` before subtask 0 and `principle-laziness-protocol` before
subtask 2: the temptation in subtask 2 is to make `run` an interpreter or to add a `Notify`
effect. Each subtask is one commit with its tests. `/deslop` before each commit. No narrating
comments; the assertion names are the documentation.
