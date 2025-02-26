# Ledger device SDK for Rust Applications
![Dynamic TOML Badge](https://img.shields.io/badge/dynamic/toml?url=https%3A%2F%2Fraw.githubusercontent.com%2FLedgerHQ%2Fledger-device-rust-sdk%2Frefs%2Fheads%2Fmaster%2Fledger_device_sdk%2FCargo.toml&query=%24.package.version&label=version)

Crate that allows developing Ledger device apps in Rust with a default configuration.

Contains:

- some safe wrappers over common syscalls
- IO abstractions
- signature abstractions
- UI libraries (the `ui` module for Nano (S/SP/X) apps, `nbgl` module for Stax and Flex apps)

## Supported devices

|       Nano X       |    Nano S Plus     |        Stax        |       Flex         |
| ------------------ | ------------------ | ------------------ | ------------------ |
| :white_check_mark: | :white_check_mark: | :white_check_mark: | :white_check_mark: |

## Usage

Building requires adding `rust-src` to your Rust installation, and both Clang and arm-none-eabi-gcc.
On Ubuntu, `gcc-multilib` might also be required.

Using rustc nightly builds is mandatory as some unstable features are required.

- `rustup default nightly-2024-12-01`
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

### Building for Nano X

```
cargo build --release --target=nanox
```

### Building for Nano S+

```
cargo build --release --target=nanosplus
```

### Building for Stax

```
cargo build --release --target=stax
```

### Building for Flex

```
cargo build --release --target=flex
```

## Contributing

You can submit an issue or even a pull request if you wish to contribute.

Make sure you've followed the installation steps above. In order for your PR to be accepted, it will have to pass the CI, which performs the following checks:

- Check if the code builds on nightly
- Check that `clippy` does not emit any warnings
- check that your code follows `rustfmt`'s format (using `cargo fmt`)
