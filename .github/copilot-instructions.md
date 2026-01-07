# Ledger Device Rust SDK - AI Coding Agent Instructions

## Architecture Overview

This is a **Cargo workspace** (resolver 2) for developing Ledger hardware wallet applications. The SDK wraps the underlying C SDK and provides safe Rust abstractions for embedded development on ARM Cortex-M devices.

### Key Crates

- **`ledger_device_sdk/`**: High-level Rust SDK with safe abstractions for IO, crypto, UI, NVM storage
- **`ledger_secure_sdk_sys/`**: Low-level FFI bindings to Ledger's C SDK (auto-generated via `build.rs`)
- **`include_gif/`**: Procedural macro for embedding images at compile-time (GIF/PNG to NBGL/BAGL format)
- **`testmacro/`**: Test harness for `#![no_std]` environments

### Device Architecture

The SDK targets **5 custom Rust targets** (not standard Rust platforms):
- `nanosplus` / `nanox`: Older Nano devices (buttons + small screen, use `ui` module)
- `stax` / `flex` / `apex_p`: Newer touchscreen devices (use `nbgl` module)

**Critical**: Use conditional compilation with `#[cfg(target_os = "...")]` throughout. Device names are:
- `nanosplus` (NOT `nanosp`)
- `nanox`  
- `stax`
- `flex`
- `apex_p`

See [nbgl_home_and_settings.rs](ledger_device_sdk/examples/nbgl_home_and_settings.rs) for device-specific image size patterns.

## Build System (Essential Knowledge)

### Custom Build Process via `build.rs`

The `ledger_secure_sdk_sys/build.rs` (~1100 lines) performs complex setup:
1. Detects target device from `CARGO_CFG_TARGET_OS` 
2. Clones/retrieves Ledger C SDK branch via git (or uses `LEDGER_SDK_PATH` env var)
3. Compiles subset of C SDK files with arm-none-eabi-gcc
4. Generates Rust bindings with device-specific defines
5. Exports metadata to ELF sections (`ledger.target`, `ledger.api_level`, etc.)

**Key point**: Each device has different API levels, defines, linker scripts in `ledger_secure_sdk_sys/devices/*/`.

### Building

```bash
# Standard build - requires specific nightly toolchain (see rust-toolchain.toml)
cargo build --target nanosplus --release

# Using Docker (RECOMMENDED for reproducibility)
docker run --rm -v "$(pwd):/app" ghcr.io/ledgerhq/ledger-app-builder/ledger-app-builder:latest \
  cargo build --target stax --release

# Using cargo-ledger tool (abstracts device selection)
cargo ledger build nanosplus
```

**Environment variables**:
- `LEDGER_SDK_PATH`: Use local C SDK instead of git clone
- `CARGO_CFG_TARGET_OS`: Automatically set by cargo (nanosplus/nanox/stax/flex/apex_p)

## Critical Development Patterns

### 1. Device-Specific UI Modules

- **Nano S+/X**: Use `ledger_device_sdk::ui` module (BAGL library) OR `nbgl` with `nano_nbgl` feature
- **Stax/Flex/Apex P**: Use `ledger_device_sdk::nbgl` module (mandatory)

Example from [lib.rs](ledger_device_sdk/src/lib.rs):
```rust
#[cfg(any(target_os = "stax", target_os = "flex", target_os = "apex_p"))]
pub mod nbgl;

#[cfg(all(any(target_os = "nanosplus", target_os = "nanox"), not(feature = "nano_nbgl")))]
pub mod ui;
```

### 2. No Standard Library (`#![no_std]`)

All code uses `#![no_std]` + `alloc` crate. Common restrictions:
- Use `core::` instead of `std::` (e.g., `core::convert::TryFrom`, `core::fmt`)
- Use `alloc::` types for heap allocation: `alloc::vec::Vec`, `alloc::string::String`
- No file I/O, threads, networking, or OS primitives
- Format with `write!()` to preallocated buffers instead of `format!()`
- No panics with formatted messages (limited panic handler support)

Every app must define panic handler with `set_panic!` macro:
```rust
ledger_device_sdk::set_panic!(ledger_device_sdk::exiting_panic);
```

### 3. NVM Storage Patterns

Data **must** be stored in flash with `#[link_section=".nvm_data"]`:

```rust
#[link_section=".nvm_data"]
static mut COUNTER: NVMData<AtomicStorage<i32>> = NVMData::new(AtomicStorage::new(&3));

// Always use PIC-aware access
let mut counter = unsafe { COUNTER.get_mut() };
counter.update(&42);
```

See [nvm.rs](ledger_device_sdk/src/nvm.rs) for `AlignedStorage` / `AtomicStorage` / `SafeStorage` types.

### 4. Position-Independent Code (PIC)

Apps are **relocated at runtime**. Always wrap pointers/references with `pic_rs()` / `pic_rs_mut()`:
```rust
use ledger_secure_sdk_sys::{pic_rs, pic_rs_mut};
let relocated_ref = unsafe { pic_rs(&STATIC_DATA) };
```

### 5. Dual IO Modules (Legacy vs New)

Two IO implementations exist (selected at compile time via `io` module re-export in [lib.rs](ledger_device_sdk/src/lib.rs)):
- `io_legacy` (default): Original blocking event loop with `Comm` struct for APDU handling
- `io_new` (opt-in via `io_new` feature): Refactored with callback-based architecture

Both handle APDU communication and NBGL events. Use `Comm::new()` and `comm.reply()` for APDU replies. The `ApduHeader` struct contains CLA/INS/P1/P2 fields - implement `TryFrom<ApduHeader>` for your instruction enum.

## Testing & Debugging

### Running with Speculos Emulator

Examples use [Speculos](https://github.com/LedgerHQ/speculos) for testing. The [config.toml](ledger_device_sdk/examples/config.toml) defines per-target runners:

```bash
# Build + run via cargo (uses config.toml runner automatically)
cargo run --example nbgl_home_and_settings --target stax --release \
  --config ledger_device_sdk/examples/config.toml

# Unit tests with speculos feature
cargo test --target nanosplus --features speculos,debug --tests

# Manual speculos invocation
speculos --model stax target/stax/release/examples/nbgl_home_and_settings
```

### Debug Feature

Enable `debug` feature for ARM semihosting output via `debug_print()` in [testing.rs](ledger_device_sdk/src/testing.rs). Output appears in Speculos console.

## Cargo Features

- `heap`: Enable heap allocator (default)
- `nano_nbgl`: Use NBGL UI on Nano S+/X instead of BAGL
- `io_new`: Switch to refactored IO module
- `sys`: Re-export `ledger_secure_sdk_sys` for low-level FFI access
- `speculos`: Enable emulator-specific code for testing
- `debug_csdk`: Enable C SDK debug output

## Common Gotchas

1. **Wrong target_os checks**: Always verify `#[cfg(target_os = "...")]` matches device. `nanosplus` ≠ `nanosp`.

2. **Custom panic handler required**: Every app must define panic with `set_panic!` macro or implement `#[panic_handler]`.

3. **ARM toolchain required**: `arm-none-eabi-gcc` must be installed even for Rust code (C SDK compilation).

4. **Rust nightly version pinned**: Use exact version in [rust-toolchain.toml](rust-toolchain.toml), currently `nightly-2025-12-05`.

5. **Examples are documentation**: The [examples/](ledger_device_sdk/examples/) directory contains the most up-to-date usage patterns for NBGL widgets, APDU handling, and device-specific conditional compilation.

## Documentation & Resources

- [Main README](README.md): Docker images, supported devices
- [Developer docs](https://developers.ledger.com/): Official Ledger documentation
- [Boilerplate app](https://github.com/LedgerHQ/app-boilerplate-rust): Full reference implementation
- API docs: Generated from code, see inline rustdoc comments
