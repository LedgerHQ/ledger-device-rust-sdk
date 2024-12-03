# Ledger Device Rust SDK
This workspace contains the 5 crates members of Ledger Device Rust SDK

* [ledger_device_sdk](./ledger_device_sdk): main Rust SDK crate used to build an application that runs on BOLOS OS,
* [ledger_secure_sdk_sys](./ledger_secure_sdk_sys): bindings to [ledger_secure_sdk](https://github.com/LedgerHQ/ledger-secure-sdk)
* [include_gif](./include_gif): procedural macro used to manage GIF
* [testmacro](./testmacro): procedural macro used by unit and integrations tests
* [cargo-ledger](./cargo-ledger): tool to build Ledger device applications developped in Rust
