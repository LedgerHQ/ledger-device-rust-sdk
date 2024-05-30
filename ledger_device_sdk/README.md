# Ledger wallets SDK for Rust Applications

Crate that allows developing Ledger device apps in Rust with a default configuration.

Contains:

- some safe wrappers over common syscalls
- IO abstractions
- signature abstractions
- UI libraries (the `ui` module for Nano (S/SP/X) apps, `nbgl` module for Stax and Flex apps)

## Links

To learn more about using the SDK and what is required to publish an app on the Ledger Live app store, please don't hesitate to check the following resources :

- 📚 [Developer's documentation](https://developers.ledger.com/)
- 🗣️ [Ledger's Discord server](https://discord.gg/Ledger)
- 📦 [Fully featured boilerplate app](https://github.com/LedgerHQ/app-boilerplate-rust)
- 📦 [A Password Manager app](https://github.com/LedgerHQ/rust-app-password-manager)

## Supported devices

|       Nano S       |       Nano X       |    Nano S Plus     |        Stax        |       Flex         |
| ------------------ | ------------------ | ------------------ | ------------------ | ------------------ |
| :heavy_check_mark: | :heavy_check_mark: | :heavy_check_mark: | :heavy_check_mark: | :heavy_check_mark: |

## Usage

Building requires adding `rust-src` to your Rust installation, and both Clang and arm-none-eabi-gcc.
On Ubuntu, `gcc-multilib` might also be required.

Using rustc nightly builds is mandatory as some unstable features are required.

- `rustup default nightly`
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

### Building for Nano S

```
cargo build --release -Z build-std=core --target=./nanos.json
```

### Building for Nano X

```
cargo build --release -Z build-std=core --target=./nanox.json
```

### Building for Nano S+

```
cargo build --release -Z build-std=core --target=./nanosplus.json
```

### Building for Stax

```
cargo build --release -Z build-std=core --target=./stax.json
```

### Building for Flex

```
cargo build --release -Z build-std=core --target=./flex.json
```

## Building with rustc < 1.54

Building before rustc 1.54 should fail with `error[E0635]: unknown feature const_fn_trait_bound`.

This is solved by activating a specific feature: `cargo build --features pre1_54`

## Contributing

You can submit an issue or even a pull request if you wish to contribute, we will check what we can do.

Make sure you've followed the installation steps above. In order for your PR to be accepted, it will have to pass the CI, which performs the following checks:

- Check if the code builds on nightly
- Check that `clippy` does not emit any warnings
- check that your code follows `rustfmt`'s format (using `cargo fmt`)
