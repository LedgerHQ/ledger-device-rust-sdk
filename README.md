# Ledger Nano S SDK for Rust Applications

Crate that allows developing Ledger Nano S apps in Rust with a default configuration.

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
- `rustup target add thumbv6m-none-eabi`
- install [Clang](http://releases.llvm.org/download.html).
- install an [ARM GCC toolchain](https://developer.arm.com/tools-and-software/open-source-software/developer-tools/gnu-toolchain/gnu-rm/downloads)

## Generating bindings for the C sdk

The Rust SDK uses [FFI](https://doc.rust-lang.org/nomicon/ffi.html)s to call functions defined in the C SDK. The C function signatures and types are automatically "translated" to Rust by using [bindgen](https://rust-lang.github.io/rust-bindgen/introduction.html), which generates the appropriate bindings.

The bindings are split into two different files:
- The `bindings.rs` which contains all the os, io, and seproxyhal bindings.
- The `usbbindings.rs` which contains basically all the bindings needed to interact with `lib_stusb`.

Two corresponding header files exist in `binding_headers` directory: `bindings.h` and `usbbindings.h` which are provided in order for you to be able to re-generate those bindings easily. They contain all the `#define`s and `#include`s necessary to generate the appropriate bindings.

Here are the steps to follow in order to generate those bindings by yourself:
1. Make sure you have [bindgen setup](https://rust-lang.github.io/rust-bindgen/requirements.html)
2. Run this command to generate the `bindings.rs` file (you might need to adapt `-I/usr/arm-linux/gnueabihf/include` to your configuration):
```
bindgen ./binding_headers/bindings.h --use-core --no-prepend-enum-name --no-doc-comments --no-derive-debug --ctypes-prefix=cty --no-layout-tests --with-derive-default -- --target=thumbv7m-none-eabi -fshort-enums -I/usr/arm-linux-gnueabihf/include -Inanos-secure-sdk/include -Inanos-secure-sdk/lib_cxng/include > ./src/bindings.rs
```
3. Run this command to generate the `usbbindings.rs` file (you might need to adapt `-I/usr/arm-linux-gnueabihf/include` to your configuration):
```
bindgen ./binding_headers/usbbindings.h --use-core --no-prepend-enum-name --no-doc-comments --no-derive-debug --ctypes-prefix=cty --no-layout-tests --with-derive-default -- --target=thumbv7m-none-eabi -fshort-enums -Inanos-secure-sdk/lib_stusb/STM32_USB_Device_Library/Core/Inc -I/usr/arm-linux-gnueabihf/include -Inanos-secure-sdk/lib_stusb > ./src/usbbindings.rs
```
4. Modify the generated `.rs` files and add those lines at the BEGINNING of both files:
```
#![allow(non_snake_case)]
#![allow(non_upper_case_globals)]
#![allow(non_camel_case_types)]
#![allow(clippy::upper_case_acronyms)]
#![allow(clippy::too_many_arguments)]
```
5. Remove the line containing `IO_USB_MAX_ENDPOINTS` in the `bindings.rs` file.
6. Profit!

Note that for step 4, you might need to change `/usr/arm-linux/gnueabihf/include` to another directory that includes the `stdio.h` header for the arm target.

## Building with rustc < 1.54

Building before rustc 1.54 should fail with `error[E0635]: unknown feature const_fn_trait_bound`.

This is solved by activating a specific feature: `cargo build --features pre1_54`

## Contributing

Make sure you've followed the installation steps above. In order for your PR to be accepted, it will have to pass the CI, which performs the following checks:

- Check if the code builds on nightly
- Check that `clippy` does not emit any warnings
- check that your code follows `rustfmt`'s format (using `cargo fmt`)
