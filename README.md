# Ledger Nano SDK for Rust Applications

Crate that allows developing Ledger Nano apps in Rust with a default configuration.

Contains:

- low-level pre-generated bindings to the C SDK version 2.0
- some safe wrappers over common syscalls
- IO abstractions

This SDK is still in BETA as wrappers are currently missing, but two apps were made using it:

- [A demo application with a signature UI workflow](https://github.com/LedgerHQ/rust-app)
- [A Password Manager](https://github.com/LedgerHQ/rust-app-password-manager)

All issues and PRs are welcome ! 

## Usage

Building requires adding a toolchain to your Rust installation, and both Clang and arm-none-eabi-gcc.
On Ubuntu, `gcc-multilib` might also be required.

Using rustc nightly builds is recommanded as some unstable features are
required.

- `rustup default nightly`
- `rustup target add thumbv6m-none-eabi` for Nano S and Nano X builds
- `rustup target add thumbv8m.main-none-eabi` for Nano S+ builds
- install [Clang](http://releases.llvm.org/download.html).
- install an [ARM GCC toolchain](https://developer.arm.com/tools-and-software/open-source-software/developer-tools/gnu-toolchain/gnu-rm/downloads)

This SDK provides three [custom target](https://doc.rust-lang.org/rustc/targets/custom.html) files for Nano S, Nano X and Nano S+.

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


## Building with rustc < 1.54

Building before rustc 1.54 should fail with `error[E0635]: unknown feature const_fn_trait_bound`.

This is solved by activating a specific feature: `cargo build --features pre1_54`

## Contributing

Make sure you've followed the installation steps above. In order for your PR to be accepted, it will have to pass the CI, which performs the following checks:

- Check if the code builds on nightly
- Check that `clippy` does not emit any warnings
- check that your code follows `rustfmt`'s format (using `cargo fmt`)
