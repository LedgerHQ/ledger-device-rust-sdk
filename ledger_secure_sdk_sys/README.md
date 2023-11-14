# Rust Low-level bindings to the Ledger C SDK

Provides access to low-level APIs to the operating system of Ledger devices.

## Build

Depending on the target (`--target nanos`, `--target nanox`, ...), this crate will `git clone` the appropriate branch (`API_LEVEL_x`) of the [C SDK](https://github.com/LedgerHQ/ledger-secure-sdk/) and compile the subset of files necessary for the [Rust SDK](https://github.com/LedgerHQ/ledger-nanos-sdk/) to work.

To use an already-cloned C SDK, you can pass its path through the environment variable `LEDGER_SDK_PATH=/path/to/c_sdk` or through `cargo`'s `--config` flag:

```sh
cargo build --target nanosplus --config env.LEDGER_SDK_PATH="../ledger-secure-sdk/"
```
