# Ledger device SDK for Rust Applications
![Dynamic TOML Badge](https://img.shields.io/badge/dynamic/toml?url=https%3A%2F%2Fraw.githubusercontent.com%2FLedgerHQ%2Fledger-device-rust-sdk%2Frefs%2Fheads%2Fmaster%2Fledger_device_sdk%2FCargo.toml&query=%24.package.version&label=version)

Crate that allows developing Ledger device applications in Rust.

Contains:

- Safe wrappers over common syscalls and C SDK functions
- IO abstractions (`io` and `seph` modules)
- Cryptographic abstractions (`ecc`, `hash` and `hmac` modules)
- Arithmetic (simple and modular) abstraction (`math` module)
- Persistent data storage (`nvm` module)
- UI/UX libraries (`nbgl` module)
- Swap support (`libcall`module)

## Supported devices

|       Nano X       |    Nano S Plus     |        Stax        |       Flex         |      Apex P        |
| ------------------ | ------------------ | ------------------ | ------------------ | ------------------ |
| :white_check_mark: | :white_check_mark: | :white_check_mark: | :white_check_mark: | :white_check_mark: |

## Usage

Building requires adding `rust-src` to your Rust installation, and both Clang and arm-none-eabi-gcc.
On Ubuntu, `gcc-multilib` might also be required.

Using rustc nightly builds is mandatory as some unstable features are required.

- `rustup default nightly-2025-12-05` (or use the version specified in `rust-toolchain.toml`)
- `rustup component add rust-src`
- install [Clang](http://releases.llvm.org/download.html).
- install an [ARM gcc toolchain](https://developer.arm.com/tools-and-software/open-source-software/developer-tools/gnu-toolchain/gnu-rm/downloads)

If you wish to install the ARM gcc toolchain using your distribution's packages, these commands should work:

```bash
# On Debian and Ubuntu
sudo apt install clang gcc-arm-none-eabi gcc-multilib

# On Fedora or Red Hat Entreprise Linux
sudo dnf install clang arm-none-eabi-gcc arm-none-eabi-newlib

# On ArchLinux
sudo pacman -S clang arm-none-eabi-gcc arm-none-eabi-newlib
```

This SDK provides [custom target](https://doc.rust-lang.org/rustc/targets/custom.html) files. One for each supported device.

We also provide a [Docker container](https://github.com/LedgerHQ/ledger-app-builder) to build Rust applications for Ledger devices (recommended for reproducibility):

```bash
docker pull ghcr.io/ledgerhq/ledger-app-builder/ledger-app-builder:latest

# Build using Docker
docker run --rm -v "$(pwd):/app" ghcr.io/ledgerhq/ledger-app-builder/ledger-app-builder:latest \
  cargo ledger build stax
````

### Building your app

Using [cargo-ledger](https://github.com/LedgerHQ/cargo-ledger) (recommended):

```bash
# Install cargo-ledger
cargo install cargo-ledger

# Setup custom targets (one-time)
cargo ledger setup

# Build for your target device
cargo ledger build nanox       # Nano X
cargo ledger build nanosplus   # Nano S+
cargo ledger build stax        # Stax
cargo ledger build flex        # Flex
cargo ledger build apex_p      # Apex P

# Build and load to device
cargo ledger build nanosplus --load
```

Alternatively, using plain cargo:

```bash
cargo build --release --target=nanox       # Nano X
cargo build --release --target=nanosplus   # Nano S+
cargo build --release --target=stax        # Stax
cargo build --release --target=flex        # Flex
cargo build --release --target=apex_p      # Apex P
```

### App metadata and build variants

`cargo-ledger` and the SDK build script read your app's install parameters
(name, icon, flags, allowed curves and derivation paths) from the
`[package.metadata.ledger]` table of your app's `Cargo.toml`:

```toml
[package.metadata.ledger]
curve = ["secp256k1"]            # curves the app is allowed to use
flags = "0x000"                  # application flags (hex string)
path  = ["44'/0'"]               # allowed BIP32 derivation-path prefixes
name  = "MyApp"                  # name shown on the device dashboard
# one icon table per supported device
nanox     = { icon = "icons/app_nanox.gif" }
nanosplus = { icon = "icons/app_nanosplus.gif" }
stax      = { icon = "icons/app_stax.gif" }
flex      = { icon = "icons/app_flex.gif" }
apex_p    = { icon = "icons/app_apexp.png" }
```

#### Build variants (e.g. testnet)

A single source tree can produce several installable apps that differ only in a
few metadata fields — typically a testnet build with a different name, icon, and
derivation path. Declare just the differing keys under
`[package.metadata.ledger.variants.<name>]`; every key you omit is inherited
from the base table:

```toml
[package.metadata.ledger.variants.testnet]
name = "MyApp Testnet"
path = ["44'/1'"]                # standard testnet coin type
nanox = { icon = "icons/app_testnet_nanox.gif" }
# flags, curve, and any non-overridden icon are inherited from the base table
```

Select a variant at build time in either of two ways:

```bash
# 1. Cargo feature — forward the SDK's `testnet` feature from your app:
#      [features]
#      testnet = ["ledger_device_sdk/testnet"]
cargo ledger build nanosplus -- --features testnet

# 2. Generic env var — works for any variant name, no feature wiring needed:
LEDGER_APP_VARIANT=testnet cargo ledger build nanosplus
```

Notes:
- Variant resolution is **fail-closed**: selecting a variant whose
  `[package.metadata.ledger.variants.<name>]` table is absent aborts the build
  instead of silently using the base values. This prevents shipping a
  "Testnet"-labelled binary that carries mainnet paths or curves.
- Unspecified per-device icons fall back to the base table's icon (icons are
  cosmetic).
- `LEDGER_APP_VARIANT` takes precedence over the `testnet` feature.

## Getting Started

For a complete application example, see the [Rust Boilerplate App](https://github.com/LedgerHQ/app-boilerplate-rust).

Key concepts for Ledger app development:
- **`#![no_std]` environment**: No standard library, use `core::` and `alloc::` types
- **Panic handler required**: Every app must define a panic handler with `set_panic!` macro
- **Device-specific UI**: Use `nbgl` module for touchscreen devices (Stax/Flex/Apex P), `ui` module or `nbgl` with `nano_nbgl` feature for Nano devices
- **Testing**: Examples can be run with [Speculos](https://github.com/LedgerHQ/speculos) emulator

## Examples

The [`examples/`](examples/) directory contains various demonstrations. Build and run with:

```bash
# Touchscreen devices (Stax, Flex, Apex P)
cargo run --example nbgl_home_and_settings --target stax --release \
  --config examples/config.toml

# Nano devices (S+, X) - requires nano_nbgl feature for NBGL UI
cargo run --example nbgl_home_and_settings --target nanosplus --release \
  --features nano_nbgl --config examples/config.toml

# View all available examples
ls examples/*.rs
```

**Note**: Running examples requires Speculos emulator. The `config.toml` automatically invokes Speculos as the target runner.

## Contributing

You can submit an issue or even a pull request if you wish to contribute.

Make sure you've followed the installation steps above. In order for your PR to be accepted, it will have to pass the CI, which performs the following checks:

- Check if the code builds on nightly
- Check that `clippy` does not emit any warnings
- check that your code follows `rustfmt`'s format (using `cargo fmt`)
