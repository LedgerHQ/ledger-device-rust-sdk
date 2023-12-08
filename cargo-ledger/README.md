# Cargo-ledger

Builds a Nano App and outputs a JSON manifest file that can be used by [ledgerctl](https://github.com/LedgerHQ/ledgerctl) to install an application directly.

In order to build for Nano S, Nano X, and Nano S Plus, [custom target files](https://docs.rust-embedded.org/embedonomicon/custom-target.html) are used. They can be found at the root of the [Rust SDK](https://github.com/LedgerHQ/ledger-nanos-sdk/) and can be installed automatically with the command `setup`.

## Installation

This program requires:

- `arm-none-eabi-objcopy`
- [`ledgerctl`](https://github.com/LedgerHQ/ledgerctl)

Install this repo with:

```
cargo install --git https://github.com/LedgerHQ/cargo-ledger
```

or download it manually and install with:

```
cargo install --path .
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
cargo ledger build nanos
cargo ledger build nanox
cargo ledger build nanosplus
```

Loading on device can optionally be performed by appending `--load` or `-l` to the command.

By default, this program will attempt to build the current program with in `release` mode (full command: `cargo build --release --target=nanos --message-format=json`)

Arguments can be passed to modify this behaviour after inserting a `--` like so:

```
cargo ledger build nanos --load -- --features one -Z unstable-options --out-dir ./output/
```