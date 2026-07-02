# Changelog

All notable changes to this project will be documented in this file.

The format is based on [Keep a Changelog](https://keepachangelog.com/en/1.0.0/),
and this project adheres to [Semantic Versioning](https://semver.org/spec/v2.0.0.html).

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
