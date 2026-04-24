# Changelog

All notable changes to this project will be documented in this file.

The format is based on [Keep a Changelog](https://keepachangelog.com/en/1.0.0/),
and this project adheres to [Semantic Versioning](https://semver.org/spec/v2.0.0.html).

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