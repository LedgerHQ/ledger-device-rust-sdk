# Ledger Nano S SDK for Rust Applications

Crate that allows developing Ledger Nano S apps in Rust with a default configuration.

Contains:

- low-level pre-generated bindings to the C SDK version 1.6.0
- some safe wrappers over common syscalls
- IO abstractions

This SDK is still in BETA as wrappers are currently missing, but two apps were made using it:

- [A demo application with a signature UI workflow](https://github.com/LedgerHQ/rust-app)
- [A Password Manager](https://github.com/LedgerHQ/rust-app-password-manager)

All issues and PRs are welcome ! 

## Usage

Building requires adding a toolchain to your Rust installation, and both Clang and arm-none-eabi-gcc.

Using rustc nightly builds is recommanded as some unstable features are
required.

- `rustup target add thumbv6m-none-eabi`
- install [Clang](http://releases.llvm.org/download.html).
- install an [ARM GCC toolchain](https://developer.arm.com/tools-and-software/open-source-software/developer-tools/gnu-toolchain/gnu-rm/downloads)
