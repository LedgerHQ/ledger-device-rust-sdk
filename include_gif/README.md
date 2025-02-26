# include_gif
![Dynamic TOML Badge](https://img.shields.io/badge/dynamic/toml?url=https%3A%2F%2Fraw.githubusercontent.com%2FLedgerHQ%2Fledger-device-rust-sdk%2Frefs%2Fheads%2Fmaster%2Finclude_gif%2FCargo.toml&query=%24.package.version&label=version)

This crate provides a macro `include_gif!("path/to/image.gif")` that packs a gif image into a byte representation that can be understood by the [Rust SDK](https://github.com/LedgerHQ/ledger-device-rust-sdk/tree/master/ledger_device_sdk) and included at compile time to produce icons.
