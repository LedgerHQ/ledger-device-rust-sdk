# Ledger device SDK for Rust Applications
![Dynamic TOML Badge](https://img.shields.io/badge/dynamic/toml?url=https%3A%2F%2Fraw.githubusercontent.com%2FLedgerHQ%2Fledger-device-rust-sdk%2Frefs%2Fheads%2Fmaster%2Fledger_device_sdk%2FCargo.toml&query=%24.package.version&label=version)

Crate that allows developing Ledger device applications in Rust.

Contains:

- Safe wrappers over common syscalls and C SDK functions
- IO abstractions (`io` and `seph` modules)
- Cryptographic abstractions (`ecc`, `hash` and `hmac` modules)
- Aritmethic (simple and modular) abstraction (`math` module)
- Persistent data storgae (`nvm` module)
- UI/UX libraries (`nbgl` module)
- Swap support (`libcall`module)

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

We also provide a [Docker container](https://github.com/LedgerHQ/ledger-app-builder) to build Rust applications for Ledger devices:

```bash
docker pull ghcr.io/ledgerhq/ledger-app-builder/ledger-app-builder:latest
````

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

## Examples
### Nano S+
#### Build 
```
cargo build -p ledger_device_sdk --example nbgl_home_and_settings --target nanosplus --profile release --features nano_nbgl
```
#### Build + Run
```
cargo run -p ledger_device_sdk --example nbgl_home_and_settings --target nanosplus --profile release --features nano_nbgl --config ledger_device_sdk/examples/config.toml -- --api-port 5001
```
### Nano X
#### Build
```
cargo build -p ledger_device_sdk --example nbgl_home_and_settings --target nanox --profile release --features nano_nbgl
```
#### Build + Run
```
cargo run -p ledger_device_sdk --example nbgl_home_and_settings --target nanox --profile release --features nano_nbgl --config ledger_device_sdk/examples/config.toml -- --api-port 5001
```
### Stax
#### Build
```
cargo build -p ledger_device_sdk --example nbgl_home_and_settings --target stax --profile release
```
#### Build + Run
```
cargo run -p ledger_device_sdk --example nbgl_home_and_settings --target stax --profile release --config ledger_device_sdk/examples/config.toml
```
### Flex
#### Build
```
cargo build -p ledger_device_sdk --example nbgl_home_and_settings --target flex --profile release
```
#### Build + Run
```
cargo run -p ledger_device_sdk --example nbgl_home_and_settings --target flex --profile release --config ledger_device_sdk/examples/config.toml
```
### Apex P
#### Build
```
cargo build -p ledger_device_sdk --example nbgl_home_and_settings --target apex_p --profile release
```
#### Build + Run
```
cargo run -p ledger_device_sdk --example nbgl_home_and_settings --target apex_p --profile release --config ledger_device_sdk/examples/config.toml
```

## Contributing

You can submit an issue or even a pull request if you wish to contribute.

Make sure you've followed the installation steps above. In order for your PR to be accepted, it will have to pass the CI, which performs the following checks:

- Check if the code builds on nightly
- Check that `clippy` does not emit any warnings
- check that your code follows `rustfmt`'s format (using `cargo fmt`)
