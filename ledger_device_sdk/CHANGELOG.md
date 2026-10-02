# Changelog

All notable changes to this project will be documented in this file.

The format is based on [Keep a Changelog](https://keepachangelog.com/en/1.0.0/),
and this project adheres to [Semantic Versioning](https://semver.org/spec/v2.0.0.html).

## [Unreleased]

### Added
- `app_storage` feature and module: the C SDK's application storage (a versioned,
  CRC-checked area at the start of the app data, meant to be kept by the OS across app
  updates), with its size set by the `APP_STORAGE_SIZE` environment variable and its
  header properties by the `app_storage_settings` and `app_storage_data` features.

## [1.38.0] - 2026-09-29

### Changed
- `io_new`: `NbglStreamingReview::start`, `next`, `continue_review` and
  `finish` now take `&mut Comm` as their first argument, like every other
  blocking NBGL widget. Data obtained from a `Command` can no longer be borrowed
  across these calls: copy what is needed first. This is a breaking change for
  `io_new` users.
- `Ed25519Stream`: the `big_r` and `signature` fields are no longer public. The
  signature is read with the new `signature()` method once both message passes
  are complete.

### Fixed
- `io_legacy`: APDUs received while a command is in flight are now handled as
  in `io_new`: BOLOS GET_VERSION is answered inline, anything else is answered
  `CmdNotAccepted`, and the command in flight is left as is.
- `io_legacy`, `io_new`: while a command is in flight, GET_VERSION is the only
  BOLOS APDU answered; the others are answered `CmdNotAccepted`. They are still
  handled as before when no command is in flight.
- `Ed25519Stream`: both message passes must now carry the same message,
  otherwise `sign_finalize` returns an error. Calls are checked against an
  explicit signing phase (no update or finalize before `init` or after
  completion), `init` resets all state, and intermediate state is cleared on
  failure and on drop.

## [1.37.1] - 2026-09-29

### Fixed
- `ìo_new`: reject any incoming APDU when device is locked

## [1.37.0] - 2026-08-24

### Added
- `StatusWords::CmdNotAccepted` (0x6901). An APDU received while a previous
  command is still being processed is now answered with this status word instead
  of being queued or silently dropped. Note that adding a variant to the public
  `StatusWords` enum breaks downstream exhaustive `match` expressions over it.

### Fixed
- `io_new`: BOLOS internal APDUs (CLA 0xB0) arriving while an NBGL screen is
  displayed are handled inline again instead of being answered
  `CmdNotAccepted`. That handling was gated behind the `stack_usage` feature, so
  default builds rejected OS level requests that `next_command` answers.
- `io_new`: a double APDU is now answered on the polling iteration that detects
  it. It used to be answered on the next one, so a screen completing in between
  discarded it with no response at all, leaving the host waiting.
- `io_new`: a malformed APDU received while a screen is displayed is answered
  `BadLen` instead of being ignored, matching `next_command` and `io_legacy`.
- `io_new`: rejecting an APDU no longer leaves `apdu_type` overwritten with the
  rejected APDU's transport, which made the in-flight command reply on the
  wrong channel.

### Deprecated
- `NbglHomeAndSettings::show()` is now marked with the `#[deprecated]` attribute,
  matching what its documentation already stated. Use `show_and_return()`
  instead, which does not force a home screen refresh for every received APDU.

## [1.36.2] - 2026-08-18

### Changed
- updating ref to ledger_secure_sdk_sys to 1.16.4 (stack protector support,
  `.init_array` removed from final link)

## [1.36.1] - 2026-07-02

### Changed

- updating ref to ledger_secure_sdk_sys to 1.16.3

## [1.36.0] - 2026-07-02

### Added
- Build variants: up to 10 per app via the numbered `variant_0` … `variant_9`
  cargo features. The matching `[package.metadata.ledger.variants.<N>]` table is
  overlaid on the base `[package.metadata.ledger]` metadata at build time,
  letting one source tree produce variant apps (e.g. testnet) that differ only in
  name, icon, or derivation path. An app forwards a human-named feature to a slot
  (e.g. `variant_testnet = ["ledger_device_sdk/variant_0"]`) and selects it with
  `--features variant_testnet`. Resolution is fail-closed: a missing selected
  variant table aborts the build rather than falling back to the base values, and
  enabling more than one `variant_<N>` feature is a hard error.

## [1.35.3] - 2026-06-11

### Changed
- Fix app_flags stored in ELF section

## [1.35.2] - 2026-06-04

### Changed
- Silence warning and remove useless cfg_version unstable feature

## [1.35.1] - 2026-04-30

### Changed
    - Fix clippy warnings
    - Embed icon in install_params

## [1.35.0] - 2026-04-24

### Changed
    - Migrate from 2021 to 2024 edition
    - Manage BOLOS stack consumption APDUs
    - Adds ZIP32 (Zcash) derivation support by extending the C-SDK bindings
      and restructuring ECC layer to include new curve families and supporting
      math/BN helpers
    - Fixes Speculos test hangs by ensuring BOLOS APDUs are properly handled

## [1.34.0] - 2026-03-11

### Changed
    - Integrates io_new's version of the Comm object with Nbgl,
      and also with the new libcall module.
    - Ports all SDK examples from the legacy io module (io_legacy) to
      the new io_new module.

## [1.33.1] - 2026-03-03

### Changed
    - Fix unused variable warning in no debug mode (log module)

## [1.33.0] - 2026-02-24

### Changed
    - Enable NBGL use case Generic Review for Nano devices

## [1.32.1] - 2026-02-19

### Changed
    - Reverted: bolos_apdu: do not use os_registry_get_current_app_tag
    - Remove deprecated support of app subtasks

## [1.32.0] - 2026-02-04

### Added
    - log module

### Changed
    - bolos_apdu: do not use os_registry_get_current_app_tag

## [1.31.0] - 2026-01-15

### Changed
    - Manage install parameters and app flags the same way a C apps
    - Fix cargo audit
    - Improve Swap doc
    - Add Genereic Swap error codes


## [1.30.0] - 2026-01-05

### Changed
    - update nightly toolchain version

## [1.29.1] - 2025-11-28

### Changed
    - Bump ledger_secure_sdk_sys to 1.12.1

## [1.29.0] - 2025-11-19

### Changed
    - Rust SDK as a single crate: ledger_device_sdk: include_gif is included as a
      module and ledger_secure_sdk_sys can be accessed by activating the sys feature.

## [1.28.0] - 2025-11-04

### Changed
    - Added Ledger PKI and TLV parsers (Dynamic Token, Trusted Name, Generic) support
    - Add ADDRESS_EXTRA_ID_BUF_SIZE support (swap)
