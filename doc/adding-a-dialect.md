# Adding a dialect

The contributor contract for Tachyon's MSNP dialect structure (shipped by
stage D5 of `doc/architecture.md` §8; acceptance per §9 item 1).

What exists today: exactly **one** dialect, `Dialect::Msnp18`. What ships is
the *structure*, never a second dialect in any form — no stubs, no dormant
variants, no speculative arms (`doc/architecture.md` §6 D). MSNP21 itself is
post-release phase **M1** (§7), contributor-picked-up; pre-MSNP18 dialects
are **M2**. This document is the enforced part of that promise: the seam
inventory the compiler checks for you, the fixture discipline, the
scratch-variant exercise that proves the inventory is real, the honest
regression gate, and the one sanctioned auth shape.

Everything below describes what is **demonstrated at HEAD**, not what is
hoped: the seam inventory was verified by running the scratch-variant
exercise (§3 here, §9 item 1 in the plan), and the chunker's
profile-driven cap is anchored by the e2e flows and the `msg_handler` chunk
table tests.

---

## 1. The seam inventory

A `Dialect` is negotiated once per NS connection (VER), stored once in
`SessionState`, copied once into `TachyonClientInner` at auth, and matched
**exhaustively** at every seam: no `_` arms, no `#[non_exhaustive]` — either
one defeats the compiler enumeration the whole structure exists for. Every
site names its concrete anticipated difference in a comment; a site that
cannot name one is not a seam (inline v18 and delete it).

**Budget: ≤6 sites** (`doc/architecture.md` §3.3). Four exist today; a
seventh requires deleting one or stating why the budget moved.

| # | Site | The decision it owns | Named anticipated difference |
|---|------|----------------------|------------------------------|
| 1 | `crates/msnp/src/msnp/notification/models/dialect.rs` — `Dialect::profile()` (+ the `dialect_for` mapping beside it) | Negotiation / profile lookup, reached from the VER and CVR arms in `notification/handlers/negotiation.rs` | MSNP21 serves SDG/UUM over the NS and has **no switchboard** (§7 M1); pre-MSNP18 clients need the **browser-auth trigger** (§7 M2). Both land as new arms with their own records. |
| 2 | `crates/tachyon/src/notification/handlers/ready/chg_handler.rs:152` — `render()` | Presence wire shape | The MSNP21 presence payload / UBX endpoint-data shape differs, and MSNP21 has no switchboard (§7 M1). |
| 3 | `crates/tachyon/src/notification/handlers/auth.rs:191` — `render_auth_trigger()` | The web-login **trigger** (§5 here) | v1 renders the in-protocol NOT alert; M2's arm launches the host browser for pre-MSNP18 clients that cannot act on the alert (§7 M2). The fork is the trigger only, never the login. |
| 4 | `crates/tachyon/src/matrix/handlers/message_handlers.rs:351` — `deliver()` | SB-vs-NS delivery (§3.4) | The classic arm dispatches the decided ops through the room's switchboard handle (today's SB/XFR flow); MSNP21 has no switchboard — the M1 arm delivers the same ops as SDG/UUM over the NS writer and **refuses XFR** instead. The fork is the transport only, never the translation/dedup/room resolution shared above the arms. |

Site 1 lives in `msnp`; sites 2–4 are the exactly-three `match dialect`
matches in `crates/tachyon/src` (`grep -rn 'match dialect' crates/tachyon/src
--include=*.rs`). The profile *record* — `DialectProfile` in `dialect.rs` —
is plain data, not a seam match, and is the sanctioned home of MSNP18's wire
literals: the CVR strings and the message-size cap (`max_msg_body_size:
1311`, consumed by `SwitchboardHandle::receive_msg`, which stores it at
construction; the pre-D5 file-local `MAX_MSG_BODY_SIZE` const is retired).
Behavior forks (like the auth trigger) are match arms, **not** profile
fields: a data field would let a new dialect compile with the wrong behavior
silently constructed; a match arm is a compiler error = a work-list item
(§3.7).

**Erosion anchors** (§10 — single-variant erosion is the most likely 1.0
failure):

- no `_` wildcard in any `match dialect`;
- no `#[non_exhaustive]` on `Dialect` (the tree's one live attribute of that
  name sits on an unrelated, pre-existing msnp p2p TLV type — `p2p/v2/tlv.rs:43`,
  untouched by D0–D5; within the dialect machinery the only mention is
  dialect.rs's comment forbidding it);
- `Dialect` keeps exactly one variant in every commit until a real dialect
  lands;
- MSNP18 literal confinement: in `crates/msnp/src` the profile record in
  `dialect.rs` is the sanctioned home; in `crates/tachyon/src` the literal
  appears only in negotiation's profile lookup and in tests — any appearance
  inside a handler body is a defect, not a style nit.

## 2. The fixture rule

Wire round-trips in `msnp` are asserted as **exact byte strings**
(`AGENTS.md`). If your dialect work changes a renderer, you must update its
fixture **deliberately, named in the same commit** (`doc/architecture.md`
§2) — the H8 fix (`26a6604`) is the precedent: the new usertile fixture is
named in the commit body, and the commit states which fixtures are
untouched.

The four **known-red** fixtures are red at baseline and are **never fixed**
on a dialect branch (or any branch) — they are pre-existing msnobj
URL-encoding failures, pinned as the repo's honest red baseline
(`AGENTS.md`: "Don't chase them"). Verbatim:

```
msnp::notification::command::iln::tests::test_iln_ser_msn_obj
msnp::notification::command::iln::tests::test_nln_via_ser_msn_obj
msnp::notification::command::nln::tests::test_nln_ser_msn_obj
msnp::notification::command::nln::tests::test_nln_via_ser_msn_obj
```

Your change must not add a fifth failure, and must not touch these four
fixtures' files.

## 3. The scratch-variant exercise (the seam test)

The inventory is **enforced, not asserted** (§9 item 1). Run this whenever
you touch a seam, and as the first step of adding a dialect:

1. In your working tree (on a branch, **never committed**), add a scratch
   variant: `Dialect::Msnp21,` in `dialect.rs`.
2. `cargo check --workspace`. Cargo stops at `msnp`, so the exercise runs in
   two passes:
   - **Pass 1** (variant only): the compiler errors at `dialect.rs`'s
     `profile()` match — `error[E0004]: non-exhaustive patterns:
     Dialect::Msnp21 not covered`.
   - **Pass 2**: give `profile()` a scratch arm with placeholder data so
     `msnp` compiles; the compiler then errors at **exactly** the three
     `match dialect` seams in `crates/tachyon/src`.
3. Every inventoried site must light up, and **no non-inventoried site may**.
   A silent inventoried site or an unexpected error site means the inventory
   in §1 (and `doc/architecture.md` §3.3) is wrong — fix the inventory in the
   same commit as the seam change, or stop and report; do not patch around
   it.
4. `git restore crates/msnp/src/msnp/notification/models/dialect.rs` and
   confirm `git status` is clean. `git log -p` on `dialect.rs` must never
   show `Msnp21` before M1 actually lands.

**Executed at D5 (HEAD `cc5570b`, recorded for honesty — this is a real
run, not a claim):**

```
pass 1: error[E0004] crates\msnp\src\msnp\notification\models\dialect.rs:39  (profile()'s `match self`)
pass 2: error[E0004] crates\tachyon\src\matrix\handlers\message_handlers.rs:351   (deliver)
        error[E0004] crates\tachyon\src\notification\handlers\ready\chg_handler.rs:152 (render)
        error[E0004] crates\tachyon\src\notification\handlers\auth.rs:191      (render_auth_trigger)
        error: could not compile `tachyon` (lib) due to 3 previous errors
```

All four inventoried sites errored; zero non-inventoried sites did; the file
was restored and the tree verified clean.

## 4. The regression gate (honest current numbers)

The plan's "212 passed / 4 known-red" (§9 item 4, §11) is **stale text** —
the baseline grew by deliberate, attributed fixture additions:

- **212/4** — plan baseline;
- **+2 → 214/4** — D0 (`44e722f`): the `ver.rs` / `cvr.rs` negotiation pins;
- **+1 → 215/4** — D3 (`26a6604`): the H8 usertile-URL fixture;
- **+2 → 217/4** — D5 C1 (`515edc3`): `dialect.rs`'s first test mod (the
  profile-record pin and the `dialect_for` mapping pin).

The four failures are the §2 known-reds at every step — invariant.

Before **every** commit of dialect work, all of these must hold:

```sh
cargo check --workspace                                # green
cargo test -p tachyon --lib                            # 114 passed / 0 failed / 1 ignored
cargo test -p msnp                                     # 217 passed / exactly 4 failed (§2 names)
cargo clippy --workspace                               # 1816 warnings at D5 — tee to a file and
                                                       # set-diff against the previous baseline:
                                                       # zero new warnings from your code
# e2e ALONE (never concurrent with other cargo; Docker):
export DOCKER_HOST=tcp://localhost:2375
cargo test -p tachyon --test e2e_msn_homeserver -- --ignored            # 4 passed
cargo test -p tachyon --test e2e_msn_homeserver -- --ignored msnp18_dialect_conformance_script
```

Run the lib suite at least **3×** per commit and keep fixtures
parallel-safe: every file/store/dir a test creates gets a **unique per-test
name** (the `test_credentials_store(name)` pattern in
`switchboard/handlers/bootstrap_handlers.rs`) — a shared-temp-dir race is exactly what the D4
addendum (`b951cc7`) fixed. The four e2e tests mutate process-global
`TACHYON_*` env and serialize on a static mutex — run them `--ignored` and
alone.

## 5. The sanctioned auth approach (§3.7)

**The ticket is the boundary.** Everything downstream of `TicketToken` is
dialect-independent shared machinery (`MatrixLoginService`,
`Backend::Pending→Ready`, watcher, sync, presence). Auth is the one place a
dialect may fork, and what forks is **the trigger only** — in-protocol NOT
alert (MSNP18, today) vs host-launched browser (pre-MSNP18 clients that
cannot act on the alert, M2). The recorded product decision: the pre-MSNP18
client gets the **same web password page**, with Tachyon opening the browser
itself; the flow after the page is identical.

Hard rules:

- `msnp` **never** handles passwords: the server never receives, stores, or
  verifies an MSN password in-protocol; challenge-hash *verification* is off
  the table (tickets-are-not-secret localhost trust model). Never add
  password/hash handling to `usr.rs` or a future `syn.rs` — credentials are
  collected only by the web page (§10).
- The trigger fork is a **match arm at the already-budgeted auth seam**
  (site 3), never a `DialectProfile` field and never `if dialect == …`
  sprinkled through `auth.rs`.
- Browser-launching is host IO → shell only (§3.6): decide fns emit the
  semantic `ClientOut::DirectToWebLogin { url }` (the URL carries the ticket
  token); the renderer arm decides NOT-alert vs browser. Acceptable only
  because listeners bind 127.0.0.1.
- The auth seam **renders** the trigger; it never owns the login. If the
  site starts holding session state, the fork point has drifted (§10).

## 6. Where to go from here

- **The conformance script is your template:**
  `crates/tachyon/tests/e2e_msn_homeserver.rs` →
  `msnp18_dialect_conformance_script`, under the `PER-DIALECT CONFORMANCE
  SCRIPT (D5)` banner. Copy the test fn; change exactly three things — the
  dialect variant (the compiler then prints your seam work list), the
  negotiation offer, and the steps whose contract differs per §1's named
  differences. Never re-invent the harness (`Run`/`Stack`/`MsnClient` and
  the flow helpers are reused; `msnp` is a sync lib — no new harness
  needed, §11). Expectations that are profile data get read from
  `Dialect::profile()` — the script pins the record-to-wire mapping, not a
  copy of the record. Your dialect's script must pass green, alone, under
  the same `--ignored` discipline.
- **M1 = MSNP21** (§7): post-release, contributor-triggered. Entry
  requirements: verb-semantics verification against captures/Escargot
  (§1.3) + the §5.1 paper exercise. M1 adds the variant, the profile record,
  one arm per inventoried seam, the new `msnp` verb files, and its
  conformance script — and the §9 item 3 measurement: if the first
  contributor dialect touches twelve files, the inventory was wrong;
  falsifying the ≤6-site budget is M1's stated purpose, not a surprise.
  Don't delete `SwitchboardServer` when MSNP21 lands — classic stays the
  default; v21 is purely additive (§10).
- **M2 = pre-MSNP18 dialects** (§7): same web password page, browser-launch
  trigger arm (§5 here); entry requirement is capture verification of the
  target client's actual handshake.
- **When a seam changes**, update `doc/architecture.md` §3.3's inventory and
  the decisions record **in the same commit** (§10) — the inventory above
  mirrors it.
