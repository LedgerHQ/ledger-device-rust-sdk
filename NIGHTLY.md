# Nightly Toolchain Hygiene Policy

This SDK **cannot be built on stable Rust** and is not expected to be for the
foreseeable future. The dependence on nightly is structural, not incidental:

- **Custom JSON target specs** (`ledger_secure_sdk_sys/devices/*/*.json`) have no
  precompiled `core`/`alloc`, so they require `-Z build-std` — nightly only.
- **`relocation-model: "ropi-rwpi"`** (mandatory for runtime-relocated BOLOS apps)
  is only reachable through a custom target spec.
- **Custom `target_os` / `target-family = ["bolos"]`** drive the entire
  `#[cfg(target_os = "…")]` device matrix; stock targets report `target_os = "none"`.
- Several **`#![feature(...)]`** gates, including `generic_const_exprs`
  (permanently "incomplete"), have no stabilization path.

Because stable is off the table, the goal of this policy is **predictable,
low-risk use of nightly**: a single pinned version, an early-warning canary, a
disciplined bump procedure, and a fast rollback path.

---

## 1. Single source of truth: the pin

The toolchain is pinned in [`rust-toolchain.toml`](./rust-toolchain.toml):

```toml
[toolchain]
channel = "nightly-2025-12-05"
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
- `RUSTFLAGS = -Zunstable-options` (used by the canary build)
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
  `workflow_dispatch`.
- A canary failure is **triaged within one week** — it is not release-blocking,
  but it predicts pain at the next bump and may reveal a feature that changed
  upstream.
- The canary never modifies the committed pin; it overrides
  `rust-toolchain.toml` only inside the CI job.

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

Bump cadence: **quarterly by default**, or sooner when driven by a needed
feature fix, a stabilization we want to adopt, or a security advisory.

1. **Pick a date-pinned nightly** (`nightly-YYYY-MM-DD`), ideally one the canary
   has already exercised green.
2. Update `rust-toolchain.toml` (channel + components).
3. Run the **full CI matrix locally or on the branch**: `clippy -D warnings`,
   `fmt --check`, `build`, and `test` for **all five targets × all crates**
   (mirror `.github/workflows/ci.yml`). A bump that builds for one target can
   break another.
4. Review the §2 watch list: remove any now-stabilized `#![feature]` gates;
   adapt to any renamed/removed unstable items.
5. **Confirm the `ledger-app-dev-tools:latest` Docker image** ships (or can
   install) the chosen nightly + `rust-src`. The pin and the image must agree.
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
