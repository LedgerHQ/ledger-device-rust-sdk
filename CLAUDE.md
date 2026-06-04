# CLAUDE.md

This file provides guidance to Claude Code (claude.ai/code) when working with code in this repository.

## Repository Layout

Cargo workspace (resolver 2) with four member crates:

- `ledger_device_sdk/` — High-level safe Rust SDK (IO, crypto, NVM, UI, swap). This is the crate apps depend on.
- `ledger_secure_sdk_sys/` — Low-level FFI bindings to Ledger's C SDK. The `build.rs` here (~1100 lines) is the heart of the build system: it clones or locates the C SDK, compiles a subset of it via `cc::Build` (typically using `clang` by default), uses `arm-none-eabi-gcc` to locate the GCC sysroot and prebuilt libraries, generates bindings via `bindgen`, and emits ELF metadata sections (`ledger.target`, `ledger.api_level`, …). Per-device defines, linker scripts, and SDK config live under `ledger_secure_sdk_sys/devices/{apex_p,flex,nanosplus,nanox,stax}/`.
- `include_gif/` — Proc macro for embedding images (GIF/PNG → NBGL/BAGL) at compile time.
- `testmacro/` — Test harness for `#![no_std]` environments.

## Targets and Conditional Compilation

The SDK targets **5 custom Rust targets** (not stock Rust platforms). Always gate device-specific code with `#[cfg(target_os = "...")]`:

- `nanox`, `nanosplus` — Button devices (NOT `nanosp`). Default UI is the `ui` module (BAGL); opt into NBGL with the `nano_nbgl` feature.
- `stax`, `flex`, `apex_p` — Touchscreen devices. The `nbgl` module is mandatory.

Re-export logic in `ledger_device_sdk/src/lib.rs` shows the exact `#[cfg]` matrix for `ui` vs `nbgl`.

## Build, Test, Lint

The pinned toolchain is `nightly-2025-12-05` (see `rust-toolchain.toml`). Requires `arm-none-eabi-gcc` and `clang` on the host even for pure-Rust changes (the C SDK is compiled by `ledger_secure_sdk_sys/build.rs`).

```bash
# Build for a specific device (plain cargo)
cargo build --release --target nanosplus
# …or via cargo-ledger (handles target setup + packaging)
cargo ledger build stax

# Unit + integration tests (CI runs this per-target)
cd ledger_device_sdk && cargo test --target nanosplus --features unit_test --tests

# Run a single example under Speculos (config.toml supplies the runner)
cargo run --example nbgl_home_and_settings --target stax --release \
  --features io_new --config ledger_device_sdk/examples/config.toml

# Lint and format — CI rejects warnings
cargo clippy --target <device> -- -D warnings   # run per-crate, per-target
cargo fmt --all --check
```

CI matrix (`.github/workflows/ci.yml`) runs `cargo clippy` and `cargo test` for every crate × every device target. A change that compiles for one target can easily break another — match the matrix when in doubt.

The recommended way to build/test reproducibly is the `ghcr.io/ledgerhq/ledger-app-builder/ledger-app-dev-tools:latest` Docker image (it has the toolchain, `cargo-ledger`, Speculos, and Ragger preinstalled).

Set `LEDGER_SDK_PATH` to point `build.rs` at a local C SDK checkout instead of having it git-clone one.

## Critical Patterns

- **`#![no_std]` + `alloc`.** Use `core::` / `alloc::`; no `std::`. Heap requires the `heap` feature (on by default).
- **Panic handler is mandatory.** Every app calls `ledger_device_sdk::set_panic!(ledger_device_sdk::exiting_panic);`.
- **Position-independent code.** Apps are relocated at runtime. Wrap static accesses with `pic_rs` / `pic_rs_mut` (or the `Pic<T>` helper in `lib.rs`).
- **NVM storage.** Persistent data goes in `#[link_section=".nvm_data"]` with `NVMData<T>` and `AtomicStorage` / `AlignedStorage` / `SafeStorage` (see `ledger_device_sdk/src/nvm.rs`). The `NVMData` accessors compute `.nvm_data` offsets from `r9` — do not bypass them.
- **Dual IO modules.** `io_legacy` (default, blocking event loop with `Comm`) vs `io_new` (callback-based, opt-in via the `io_new` feature). `pub mod io` in `lib.rs` re-exports whichever is selected; most NBGL examples require `io_new`.
- **Examples are the canonical usage docs.** `ledger_device_sdk/examples/` is where current NBGL widget, APDU, and PKI/TLV patterns live; prefer reading those over inferring from the API.

## Common Gotchas

1. Device name is `nanosplus`, not `nanosp` (the Speculos `--model` flag uses `nanosp`, but `target_os` and the custom target name are `nanosplus`).
2. The `sys` feature on `ledger_device_sdk` re-exports `ledger_secure_sdk_sys` under `ledger_device_sdk::sys` — only enable it when you genuinely need raw FFI; prefer the safe wrappers.
3. The `speculos` feature is a backward-compat alias for `unit_test` (kept for the Sui app).
