# Ledger Device Rust SDK
[![DEV SUPPORT](https://img.shields.io/badge/Dev_Support-red?logo=discord
)](https://discord.com/channels/885256081289379850/912241185677000704)

## Crates

| Crate                                            | Description                                                     | Documentation  | Latest Release | Changelog |
| ------------------------------------------------ | --------------------------------------------------------------- | -------------- | -------------- | --------- |
| [ledger_device_sdk](./ledger_device_sdk)         | Rust SDK                                                        | [Link](https://ledgerhq.github.io/ledger-device-rust-sdk/)               | ![Dynamic TOML Badge](https://img.shields.io/badge/dynamic/toml?url=https%3A%2F%2Fraw.githubusercontent.com%2FLedgerHQ%2Fledger-device-rust-sdk%2Frefs%2Fheads%2Fmaster%2Fledger_device_sdk%2FCargo.toml&query=%24.package.version&label=version) | [Link](./ledger_device_sdk/CHANGELOG.md) |
| [ledger_secure_sdk_sys](./ledger_secure_sdk_sys) | [C SDK](https://github.com/LedgerHQ/ledger-secure-sdk) bindings |                | ![Dynamic TOML Badge](https://img.shields.io/badge/dynamic/toml?url=https%3A%2F%2Fraw.githubusercontent.com%2FLedgerHQ%2Fledger-device-rust-sdk%2Frefs%2Fheads%2Fmaster%2Fledger_secure_sdk_sys%2FCargo.toml&query=%24.package.version&label=version) |  [Link](./ledger_secure_sdk_sys/CHANGELOG.md) |
| [include_gif](./include_gif)                     | Procedural macro to integrate logo in the UI/UX                 |                | ![Dynamic TOML Badge](https://img.shields.io/badge/dynamic/toml?url=https%3A%2F%2Fraw.githubusercontent.com%2FLedgerHQ%2Fledger-device-rust-sdk%2Frefs%2Fheads%2Fmaster%2Finclude_gif%2FCargo.toml&query=%24.package.version&label=version) | |

## Docker builder

Docker images are available and shall be used to build and test Rust applications for Ledger devices.
 
||`full` | `dev-tools` |
| ---- | ----- | ----------- |
| Rust nightly-2024-12-01 toolchain | :white_check_mark: | :white_check_mark: |
| cargo-ledger | :white_check_mark: | :white_check_mark: |
| Rust Ledger devices custom targets | :white_check_mark: | :white_check_mark: |
| ARM & LLVM toolchains | :white_check_mark: | :white_check_mark: |
| [Speculos](https://github.com/LedgerHQ/speculos) | :x: | :white_check_mark: |
| [Ragger](https://github.com/LedgerHQ/ragger)  | :x: | :white_check_mark: |

Please check [here](https://github.com/LedgerHQ/ledger-app-builder) for more details.

## Links

To learn more about using the SDK and what is required to publish an app on the Ledger Live app store, please don't hesitate to check the following resources:

- 📚 [Developer's documentation](https://developers.ledger.com/)
- 🗣️ [Ledger's Discord server](https://discord.gg/Ledger)
- 📦 [Fully featured boilerplate app](https://github.com/LedgerHQ/app-boilerplate-rust)