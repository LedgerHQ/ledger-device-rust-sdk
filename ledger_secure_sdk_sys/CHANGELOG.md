# Changelog

All notable changes to this project will be documented in this file.

The format is based on [Keep a Changelog](https://keepachangelog.com/en/1.0.0/),
and this project adheres to [Semantic Versioning](https://semver.org/spec/v2.0.0.html).

## [1.19.0] - 2026-11-26

### Added
- Builds with the built-in Rust triples (`thumbv6m-none-eabi` for Nano X,
  `thumbv8m.main-none-eabi` for the other devices). The device comes from the
  `ledger_device` cfg passed in rustflags; the build script fails with an
  explanation when it is missing, when the relocation model rustc will use is
  not `ropi-rwpi`, or when `link_wrap.sh`, `link.ld` or the device's own
  `<device>_layout.ld` are not all in rustflags.

### Changed
- With the custom target specs, the build script derives the `ledger_device`
  cfg from `target_os`.

## [1.18.3] - 2026-11-26

### Changed
- `link_wrap.sh` prints nothing when linking succeeds: since nightly-2026-09-15,
  rustc reports any linker output as a warning (`linker_messages`). Set
  `LINK_WRAP_TRACE` to trace its commands. The generated ELFs are unchanged.

## [1.18.2] - 2026-11-25

### Fixed
- No warnings with `nightly-2026-09-15`: the bindings of `memcpy`, `memset`,
  `strlen`... declare `unsigned int` sizes (`usize` on our targets), which
  rustc now reports as `suspicious_runtime_symbol_definitions`; the lint is
  allowed on the generated bindings.

## [1.18.1] - 2026-11-06

### Fixed
- `c_main` is declared as `int c_main(int arg0)` and receives the argument the OS
  passes to the entry point (0 for a normal start, a `libargs_t` pointer for a
  library call).
- `c_main` runs `sample_main` once: an exception, `EXCEPTION_IO_RESET` included,
  ends the app instead of booting it and running `sample_main` again in the same
  memory.
- `seph::io_tx` sends only the first `length` bytes of the buffer. When `length` is
  larger than the buffer or than 65535, it sends nothing and returns -1, as a
  failed transmission does.

## [1.18.0] - 2026-10-16

### Added
- `app_io_stack` feature: build without `USE_OS_IO_STACK`, so the app runs the C SDK's
  USB/BLE IO stack itself (the C SDK's `DISABLE_OS_IO_STACK_USE`); `os_io.o` is always
  linked in that mode instead of the weak syscall stubs
- `debug_over_usb` feature: PRINTF over a USB CDC interface, which only an app-side IO 
  stack exposes 

### Changed
- `exit_app` stops the app's IO stack (`os_io_stop`) before `os_sched_exit` with
  `app_io_stack`

## [1.17.0] - 2026-10-06

### Added
- `app_storage` feature: compile the C SDK's `lib_standard_app/app_storage.c` with its size
  from the `APP_STORAGE_SIZE` environment variable (default 480) and the SETTINGS/DATA
  header properties from the `app_storage_settings` / `app_storage_data` features, bind
  `include/app_storage.h`, and initialize the storage at startup as `common_app_init()`
  does: for a standalone start and for Exchange's `SIGN_TRANSACTION`, before the
  transaction parameters are copied rather than after
- `.storage_section` in link.ld, at the start of the app data region as in the C SDK;
  empty without the feature

## [1.16.6] - 2026-10-05

### Changed
- `build.rs` reports the offending path when the per-device `.defines` file
  cannot be opened or read, instead of panicking with a bare
  `called 'Result::unwrap()' on an 'Err' value: Os { code: 2, ... }`

## [1.16.5] - 2026-10-05

### Changed
- Nano X: with `mlkem` or `mldsa` enabled, the heap is 2 KB by default and a
  larger `HEAP_SIZE` fails the build.

## [1.16.4] - 2026-08-18

### Added
- Enable stack protector: compile the C SDK with `-fstack-protector-strong` and
  wrap `__stack_chk_fail`/`__stack_chk_init` in each device's linker spec

### Changed
- Remove `.init_array` from the final link (discarded in link.ld)

## [1.16.3] - 2026-08-03

### Added
- added `_ram` symbol in link.ld following update of BSS wipe function in C SDK

## [1.16.2] - 2026-06-04

### Changed
- Refactor build.rs

## [1.16.1] - 2026-04-30

### Changed
    - Fix clippy warnings
    - Update linker script

## [1.16.0] - 2026-04-24

### Changed
    - Migrate from 2021 to 2024 edition
    - Sort C SDK glyphs to be in the same order in ELF (reproducible build)

## [1.15.0] - 2026-02-24

### Changed
    - Initialize and check the stack canary at every io_tx/io_rx call

## [1.14.1] - 2026-02-04

### Changed
    - Make the link_pass_nvram resilient to the nvram being wiped
    - C SDK: Remove usage of checks.c and checks.h
    -

## [1.14.0] - 2026-01-15

### Changed
    - Updated linker script link.ld (add install_parameters and app_flags sections)
    - Fix cargo-audit
    -

## [1.13.0] - 2026-01-05

### Changed
    - update nightly toolchain version
    - update custom target files

## [1.12.1] - 2025-11-28

### Changed
    -  update C SDK compilation flags

## [1.12.0] - 2025-11-19

### Changed
    -  Can be accessed by activating the sys feature from ledger_device_sdk package.

## [1.11.6] - 2025-11-04

### Changed
    - Bind to Ledger PKI syscalls
