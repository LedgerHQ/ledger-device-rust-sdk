# Changelog

All notable changes to this project will be documented in this file.

The format is based on [Keep a Changelog](https://keepachangelog.com/en/1.0.0/),
and this project adheres to [Semantic Versioning](https://semver.org/spec/v2.0.0.html).

## [Unreleased]

### Fixed
    - NBGL: choose the bpp and grey levels like the C SDK's icon2glyph.py: 1 bpp for up to 2
      colors, else 4 bpp with 16 levels. Images with 1, 5 to 8 or 256 colors made the macro
      panic, and 3 or 4 color images used only levels 0 to 3 (white was stored as 3, not 15).

## [1.3.3] - 2026-11-25

### Changed
    - Fix clippy warnings of nightly-2026-09-15

## [1.3.2] - 2026-06-26

### Changed
    - Remove image dependency

## [1.3.1] - 2026-04-30

### Changed
    - Fix clippy warnings

## [1.3.0] - 2026-04-24

### Changed
    - Migrate from 2021 to 2024 edition

## [1.2.6] - 2026-03-11

### Changed
    - Reduce the dependency surface of the include_gif proc-macro

## [1.2.5] - 2026-01-20

### Added
    - CHANGELOG.md file to comply with reusable_crates_deployment.yml
