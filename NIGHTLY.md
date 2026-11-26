# Nightly Toolchain Hygiene Policy

This SDK **cannot be built on stable Rust** and is not expected to be for the
foreseeable future. The dependence on nightly is structural, not incidental:

- **Custom JSON target specs** (`ledger_secure_sdk_sys/devices/*/*.json`) have no
  precompiled `core`/`alloc`, so they require `-Z build-std` — nightly only.
- **`relocation-model: "ropi-rwpi"`** (mandatory for runtime-relocated BOLOS apps)
  comes from the custom target specs, or from `-Crelocation-model=ropi-rwpi`
  with a built-in triple; either way `core`/`alloc` must be rebuilt with it,
  which also requires `-Z build-std`.
- The device matrix is selected with the **`ledger_device` cfg**. The build
  scripts derive it from the custom specs' `target_os`; with a built-in triple
  (`target_os = "none"`), `cargo ledger` passes it.
- Several **`#![feature(...)]`** gates (`custom_test_frameworks`,
  `const_option_ops`, `const_trait_impl`) are not stable.

Because stable is off the table, the goal of this policy is **predictable,
low-risk use of nightly**: a single pinned version, an early-warning canary, a
disciplined bump procedure, and a fast rollback path.

---

## 1. Single source of truth: the pin

The toolchain is pinned in [`rust-toolchain.toml`](./rust-toolchain.toml):

```toml
[toolchain]
channel = "nightly-2026-09-15"
components = ["rust-src", "clippy", "rustfmt"]
profile = "minimal"
```

Rules:

- **Exactly one pinned nightly** is used everywhere — CI, the
  `ledger-app-dev-tools` Docker image, and local dev. Never float to bare
  `nightly` outside the canary job (§3).
- The pin **must list `components`** so that fresh/local checkouts get
  `rust-src` (needed for `build-std`), `clippy`, and `rustfmt` automatically
  instead of failing cryptically.
- Changing the channel is a **deliberate, reviewed change** (§4), never a
  drive-by edit.

## 2. Unstable surface we depend on (the watch list)

Every nightly bump must be evaluated against this list. If any item changes
behavior, is renamed, or is removed, the bump is blocked until the code is
adapted.

**Build system / flags**
- `-Z build-std = ["core", "alloc"]`, `build-std-features = ["compiler-builtins-mem"]`
  (`.cargo/config.toml`)
- Custom JSON target specs (`devices/*/*.json`)
- `relocation-model: "ropi-rwpi"`
- `-Zunstable-options`: since nightly-2026-09, `rustc` refuses custom JSON
  target specs without it. The `ledger-app-builder` images pass it through a
  `rustc` wrapper (`ledger-rustc-wrapper`, only for the Ledger targets), the
  canaries set `RUSTFLAGS=-Zunstable-options`, and builds outside the images
  need it too. Note that `RUSTFLAGS` replaces the `rustflags` of the Cargo
  config, including the apps' own.
- `compiler_builtins` target detection: some of its code paths are chosen
  from the target **name** (e.g. Thumb-1 code for names starting with
  `thumbv6m`). Our custom names do not match, so `nanox` (ARMv6-M) can get
  code it cannot run or assemble (rust-lang/compiler-builtins#1374, from
  nightly-2026-09-16).
- `rust-src` component (prerequisite for `build-std`)

**Language feature gates** (`ledger_device_sdk/src/lib.rs`)
- `generic_const_exprs` — incomplete; highest breakage risk on bumps
- `const_trait_impl`
- `const_option_ops`
- `custom_test_frameworks` — the `#![no_std]` test harness

> When a feature on this list is **stabilized**, remove its `#![feature(...)]`
> gate as part of the next bump and note it in the changelog. When one is
> **renamed/replaced**, that is a required follow-up before merging the bump.

## 3. Canary: weekly build against latest nightly

[`.github/workflows/nightly_build_sdk.yml`](./.github/workflows/nightly_build_sdk.yml)
builds and unit-tests the SDK against bare `nightly` across all five targets.
It is the early-warning system: it tells us *in advance* whether the next
nightly will break us.

Policy:
- The weekly `schedule` (Mondays 03:00 UTC) **must be enabled**, not just
  `workflow_dispatch`. It runs only in the private `-dev` repo (the jobs are
  skipped on the public mirror); a failure opens or updates a tracking issue
  there. `workflow_dispatch` works in both repos. The same gate and failure
  issue apply to the apps canary (`nightly_build_all_apps.yml`).
- A canary failure is **triaged within one week** — it is not release-blocking,
  but it predicts pain at the next bump and may reveal a feature that changed
  upstream.
- The canary never modifies the committed pin. It selects its toolchain by
  setting `RUSTUP_TOOLCHAIN` for the job: the CI images set that variable to
  the pinned nightly, and it takes precedence over `rust-toolchain.toml`, so
  overriding the file would have no effect.
- Both canaries take an optional `toolchain` input on `workflow_dispatch`
  (default: latest `nightly`). Dispatch them with a date-pinned nightly to
  build the SDK and every app against a bump target before it ships. This is
  the only check of a new nightly before the image is bumped: PR CI runs in
  the image, so it always builds with the image's nightly.

### Tracking a bump-preparation branch

When a canary failure is being fixed ahead of a bump (§4), the adaptations land
on a dedicated **bump-preparation branch** rather than the default branch. To
have the canary continuously validate those staged fixes against the moving
`nightly` target, point it at that branch via the **`NIGHTLY_PREP_REF`
repository variable**:

- The checkout ref resolves as
  `workflow_dispatch input → vars.NIGHTLY_PREP_REF → default branch`.
- **Set** `NIGHTLY_PREP_REF` to the prep branch when a bump cycle starts;
  **clear** it once the bump PR lands (§4.7). While unset, the canary tracks the
  default branch as before.
- A single `workflow_dispatch` run can override the ref ad hoc via the
  `rust_sdk_ref` input without touching the variable.

Note: while `NIGHTLY_PREP_REF` is set, the canary builds *only* the prep branch,
so it no longer reports whether latest `nightly` breaks the shipped SDK on the
default branch. That signal returns automatically when the variable is cleared.

The same `NIGHTLY_PREP_REF` variable is also consumed by
[`.github/workflows/nightly_build_all_apps.yml`](./.github/workflows/nightly_build_all_apps.yml),
which builds the downstream Rust apps against bare `nightly`. When it is set,
that workflow overrides each app's published SDK crates (via a
`[patch.crates-io]` pointing at a checkout of the prep branch) so the apps are
exercised against the staged fixes too; when unset, apps build against their
pinned (crates.io) SDK. A single shared variable therefore steers both canaries
through one bump-preparation branch.

## 4. Bump procedure

Bump cadence: **every second Rust stable release** (about every 12 weeks), or
sooner when driven by a needed feature fix, a stabilization we want to adopt,
or a security advisory.

- Rust ships a stable release every six weeks, on a fixed schedule. Bumping
  every release would force an image release and an app-wide pin change each
  time; every second release keeps the pin at most ~3 months behind.
- [`.github/workflows/nightly_bump_reminder.yml`](./.github/workflows/nightly_bump_reminder.yml)
  checks weekly (in the `-dev` repo) and opens a `Nightly bump due:` issue as
  soon as the pin is two or more stable releases behind. It keeps flagging a
  skipped bump, so the pin cannot silently age.

1. **Pick a date-pinned nightly** (`nightly-YYYY-MM-DD`): by default, the
   nightly the **current stable release branched from**, i.e. the last
   nightly whose `rustc` still reports the stable's version
   (`1.N.0-nightly`). Its code became the current stable and has had six
   weeks of beta testing, so its regressions are known: check the beta
   backports of that release, and if one affects us, pick a later nightly
   that has the fix. The reminder issue names this nightly
   (`.github/workflows/check_nightly_bump.sh` computes it).

   The recommendation only looks at release dates, not at whether the SDK
   builds. Before taking it, check the open canary issues and the §2 watch
   list for known blockers: for example, no nightly from 2026-09-16 on builds
   `nanox` until rust-lang/compiler-builtins#1374 is fixed. If the
   recommended nightly is affected, pick the latest nightly before the
   blocker, and re-check at the next reminder.
2. Update `rust-toolchain.toml` (channel + components).
3. Run the **full CI matrix locally or on the branch**: `clippy -D warnings`,
   `fmt --check`, `build`, and `test` for **all five targets × all crates**
   (mirror `.github/workflows/ci.yml`). A bump that builds for one target can
   break another.
4. Review the §2 watch list: remove any now-stabilized `#![feature]` gates;
   adapt to any renamed/removed unstable items.
5. **Confirm the `ledger-app-dev-tools:latest` Docker image** ships (or can
   install) the chosen nightly + `rust-src`. The pin and the image must agree:
   bump `RUSTUP_TOOLCHAIN` in the `ledger-app-builder` image in the same
   cycle, and tell app maintainers the new pin ahead of time (apps pinning a
   different nightly are rejected by the app workflows).
6. Update version references in [`README.md`](./README.md) and the relevant
   crate `CHANGELOG.md` files.
7. Land via a dedicated PR titled `chore: bump nightly to YYYY-MM-DD`, ideally
   the only change in the PR so it is trivially revertable.

## 5. Rollback

Because each bump is an isolated PR touching the pin (+ docs):

- Revert the bump PR. The committed date-pin guarantees every consumer returns
  to the exact prior toolchain — no "it works on my machine" drift.
- Downstream apps consuming the SDK should likewise pin via their own
  `rust-toolchain.toml` and bump in lockstep with SDK releases.

## 6. Supply-chain / reproducibility notes

- The date-pin makes builds **reproducible**: a given SDK commit always builds
  with one known toolchain.
- Prefer the `ghcr.io/ledgerhq/ledger-app-builder/ledger-app-dev-tools` image
  for release builds so the toolchain, `cargo-ledger`, and ARM/LLVM tooling are
  fixed together rather than assembled ad hoc.
- Treat the toolchain pin as part of the release artifact's provenance: record
  it alongside the C SDK ref already captured in CI inputs.
