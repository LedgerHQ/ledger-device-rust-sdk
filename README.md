# Ledger Device Rust SDK
[![DEV SUPPORT](https://img.shields.io/badge/Dev_Support-red?logo=discord
)](https://discord.com/channels/885256081289379850/912241185677000704)

Rust SDK for developing applications on Ledger hardware wallets. This workspace provides safe, idiomatic Rust abstractions over the Ledger C SDK for building embedded applications on Nano X, Nano S Plus, Stax, Flex, and Apex P devices.

## Supported Devices

| Nano X | Nano S Plus | Stax | Flex | Apex P |
| :----: | :---------: | :--: | :--: | :----: |
| ✅ | ✅ | ✅ | ✅ | ✅ |

## Crates

| Crate                                            | Description                                                     | Documentation  | Latest Release | Changelog |
| ------------------------------------------------ | --------------------------------------------------------------- | -------------- | -------------- | --------- |
| [ledger_device_sdk](./ledger_device_sdk)         | High-level Rust SDK with safe abstractions                       | [Link](https://ledgerhq.github.io/ledger-device-rust-sdk/)               | ![Dynamic TOML Badge](https://img.shields.io/badge/dynamic/toml?url=https%3A%2F%2Fraw.githubusercontent.com%2FLedgerHQ%2Fledger-device-rust-sdk%2Frefs%2Fheads%2Fmaster%2Fledger_device_sdk%2FCargo.toml&query=%24.package.version&label=version) | [Link](./ledger_device_sdk/CHANGELOG.md) |
| [ledger_secure_sdk_sys](./ledger_secure_sdk_sys) | Low-level FFI bindings to [C SDK](https://github.com/LedgerHQ/ledger-secure-sdk) |                | ![Dynamic TOML Badge](https://img.shields.io/badge/dynamic/toml?url=https%3A%2F%2Fraw.githubusercontent.com%2FLedgerHQ%2Fledger-device-rust-sdk%2Frefs%2Fheads%2Fmaster%2Fledger_secure_sdk_sys%2FCargo.toml&query=%24.package.version&label=version) |  [Link](./ledger_secure_sdk_sys/CHANGELOG.md) |
| [include_gif](./include_gif)                     | Proc macro for embedding images (GIF/PNG → NBGL/BAGL)           |                | ![Dynamic TOML Badge](https://img.shields.io/badge/dynamic/toml?url=https%3A%2F%2Fraw.githubusercontent.com%2FLedgerHQ%2Fledger-device-rust-sdk%2Frefs%2Fheads%2Fmaster%2Finclude_gif%2FCargo.toml&query=%24.package.version&label=version) | |
| [testmacro](./testmacro)                         | Test harness for `#![no_std]` environments                       |                | ![Dynamic TOML Badge](https://img.shields.io/badge/dynamic/toml?url=https%3A%2F%2Fraw.githubusercontent.com%2FLedgerHQ%2Fledger-device-rust-sdk%2Frefs%2Fheads%2Fmaster%2Ftestmacro%2FCargo.toml&query=%24.package.version&label=version) | |

## Docker builder

Docker images are available and shall be used to build and test Rust applications for Ledger devices.
 
||`full` | `dev-tools` |
| ---- | ----- | ----------- |
| Rust nightly-2025-12-05 toolchain | :white_check_mark: | :white_check_mark: |
| cargo-ledger | :white_check_mark: | :white_check_mark: |
| Rust Ledger devices custom targets | :white_check_mark: | :white_check_mark: |
| ARM & LLVM toolchains | :white_check_mark: | :white_check_mark: |
| [Speculos](https://github.com/LedgerHQ/speculos) | :x: | :white_check_mark: |
| [Ragger](https://github.com/LedgerHQ/ragger)  | :x: | :white_check_mark: |

Please check [here](https://github.com/LedgerHQ/ledger-app-builder) for more details.

### Quick Start with Docker

```bash
# Pull the Docker image
docker pull ghcr.io/ledgerhq/ledger-app-builder/ledger-app-builder:latest

# Build your app (from your project directory) using cargo-ledger
docker run --rm -v "$(pwd):/app" \
  ghcr.io/ledgerhq/ledger-app-builder/ledger-app-builder:latest \
  cargo ledger build stax

# Run examples with Speculos (dev-tools image)
docker run --rm -v "$(pwd):/app" \
  ghcr.io/ledgerhq/ledger-app-builder/ledger-app-dev-tools:latest \
  cargo run --example nbgl_home_and_settings --target stax --release
```

## Getting Started

For detailed setup instructions, see the [ledger_device_sdk README](./ledger_device_sdk/README.md).

To start building your own app:

1. **Use the boilerplate**: Clone [app-boilerplate-rust](https://github.com/LedgerHQ/app-boilerplate-rust) for a complete working example
2. **Explore examples**: Check the [examples/](./ledger_device_sdk/examples/) directory for UI patterns, APDU handling, and more
3. **Read the docs**: See [developers.ledger.com](https://developers.ledger.com/) for comprehensive guides

### Key Requirements

- Rust nightly toolchain (see [rust-toolchain.toml](./rust-toolchain.toml))
- [cargo-ledger](https://github.com/LedgerHQ/cargo-ledger) - CLI tool for building Ledger apps (`cargo install --git https://github.com/LedgerHQ/cargo-ledger`)
- `arm-none-eabi-gcc` (ARM toolchain)
- Clang
- For testing: [Speculos](https://github.com/LedgerHQ/speculos) emulator

## Resources

- 📚 **[Developer Documentation](https://developers.ledger.com/)** - Complete guides for building and publishing apps
- 📦 **[Rust Boilerplate App](https://github.com/LedgerHQ/app-boilerplate-rust)** - Full reference implementation
- 🔧 **[API Documentation](https://ledgerhq.github.io/ledger-device-rust-sdk/)** - Generated SDK docs
- 🗣️ **[Discord Support](https://discord.gg/Ledger)** - Community and developer support
- 🐳 **[Docker Builder](https://github.com/LedgerHQ/ledger-app-builder)** - Build environment images

## Contributing

Contributions are welcome! Please ensure your PR:
- Builds successfully with `cargo ledger build <device>` (or `cargo build --release --target <device>`)
- Passes `cargo clippy` without warnings
- Is formatted with `cargo fmt`
- Includes tests where applicable

See individual crate READMEs for specific contribution guidelines.