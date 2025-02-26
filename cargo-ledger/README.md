# cargo-ledger
![Dynamic TOML Badge](https://img.shields.io/badge/dynamic/toml?url=https%3A%2F%2Fraw.githubusercontent.com%2FLedgerHQ%2Fledger-device-rust-sdk%2Frefs%2Fheads%2Fmaster%2Fcargo-ledger%2FCargo.toml&query=%24.package.version&label=version)

Builds a Ledger device embedded app and outputs a JSON/TOML manifest file that can be used by [ledgerctl](https://github.com/LedgerHQ/ledgerctl) to install an application directly on a device.

In order to build for Nano X, Nano S Plus, Stax and Flex [custom target files](https://docs.rust-embedded.org/embedonomicon/custom-target.html) are used. They can be found at the root of the [ledger_secure_sdk_sys](https://github.com/LedgerHQ/ledger_secure_sdk_sys/) and can be installed automatically with the command `cargo ledger setup`.

## Installation

This program requires:

- `arm-none-eabi-objcopy`
- [`ledgerctl`](https://github.com/LedgerHQ/ledgerctl)

Install this repo with:

```
cargo install --git https://github.com/LedgerHQ/ledger-device-rust-sdk cargo-ledger 
```

or download it manually and install with:

```
cargo install --path cargo-ledger
```

Note that `cargo`'s dependency resolver may behave differently when installing, and you may end up with errors.
In order to fix those and force usage of the versions specified in the tagged `Cargo.lock`, append `--locked` to the above commands.

## Usage

General usage is displayed when invoking `cargo ledger`.

### Setup

This will install custom target files from the SDK directly into your environment.

```
cargo ledger setup
```

### Building

```
cargo ledger build nanox
cargo ledger build nanosplus
cargo ledger build stax
cargo ledger build flex
```

Loading on device can optionally be performed by appending `--load` or `-l` to the command.

By default, this program will attempt to build the current program with in `release` mode (full command: `cargo build --release --target=nanos --message-format=json`)

Arguments can be passed to modify this behaviour after inserting a `--` like so:

```
cargo ledger build nanos --load -- --features one -Z unstable-options --out-dir ./output/
```