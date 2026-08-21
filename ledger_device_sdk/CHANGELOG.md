# Changelog

All notable changes to this project will be documented in this file.

The format is based on [Keep a Changelog](https://keepachangelog.com/en/1.0.0/),
and this project adheres to [Semantic Versioning](https://semver.org/spec/v2.0.0.html).

## [1.38.0] - 2026-08-25

### Added
- `NbglHomeAndSettings::info_list(&[(&str, &str)])` sets an arbitrary number of
  information fields on the home screen's info page, and
  `NbglHomeAndSettings::app_name(&str)` sets the application name on its own.
  The info list was previously frozen to the two `Version` / `Developer` fields.
- Tag/value pairs can carry a value extension (alias), exposing
  `nbgl_contentValueExt_t` and `nbgl_contentValueAliasType_t`. New `TagValue`
  type (`Field` plus an optional extension, with `From<&Field>`) and
  `FieldExtension`, built with one constructor per alias kind —
  `full_value`, `ens`, `address_book`, `qr_code`, `info_list`,
  `tag_value_list` — and the chainable `alias_sub_name`, `explanation`,
  `title`, `back_text` setters. NBGL draws a `>` next to an aliased value and
  opens a modal built from the extension.
- Extension-aware variants of the review entry points, taking `&[TagValue]`
  where the existing ones take `&[Field]`: `NbglReview::show_ext`,
  `NbglAdvanceReview::show_ext`, `NbglReviewExtended::show_ext`,
  `NbglStreamingReview::next_ext`, `NbglAddressReview::set_tag_value_list_ext`
  and `TagValueList::new_ext`.
- `nbgl_tag_value_alias` example, covering all six alias kinds on one review
  screen.
- Full coverage of `nbgl_warning_t` through the new `NbglWarning` builder.
  `predefined(&[WarningType])` raises any combination of the six pre-defined
  warnings (`W3cIssue`, `W3cRiskDetected`, `W3cThreatDetected`, `W3cNoThreat`,
  `BlindSigning`, `GatedSigning`); the set was previously frozen to
  `W3cRiskDetected` alone. The manual path is also exposed: `info`,
  `intro_details`, `review_details`, `intro_top_right_icon`,
  `review_top_right_icon` and `prelude`, with the supporting `CenterInfo`,
  `QrCode`, `Prelude`, `WarningBar` and `WarningDetails` types. Bar lists nest
  to arbitrary depth.
- `NbglAdvanceReview::warning(&NbglWarning)` and
  `NbglStreamingReview::warning(&NbglWarning)`.
- `nbgl_warning` example, showing both the pre-defined and the manual paths.
- `NbglGenericConfiguration`, wrapping `nbgl_useCaseGenericConfiguration` — a
  configuration screen built from arbitrary `NbglPageContent`, paginated by
  NBGL, ended through the header. It is the general form of
  `NbglGenericSettings`, which is fixed to one switch list backed by NVM.
- `NbglGenericConfiguration::on_action` and `NbglGenericReview::on_action`,
  reporting touches on a switch, choice or bar back to the app through
  `nbgl_content_t.contentActionCallback`. That field was previously always NULL,
  so the interactive content types added earlier in this release were drawn but
  reported nothing. `TagValueConfirm`, `InfoLongPress` and `InfoButton` keep the
  SDK's own callback either way, since that is how a review detects approval.
- `NbglChoice::show_with_details` and `NbglChoice::show_advanced_with_details`,
  wrapping `nbgl_useCaseChoiceWithDetails` and
  `nbgl_useCaseAdvancedChoiceWithDetails`. Both take a `WarningDetails`, the
  same details tree `NbglWarning` uses, and return the user's choice; the
  advanced variant adds a header icon and title.
- `NbglConfirm`, wrapping `nbgl_useCaseConfirm`, previously reachable only
  indirectly through `NbglChoice::ask_confirmation`. It is a modal, so it must
  be raised over a screen that is already drawn, and `show_and_return` does not
  block: the C API reports the button being touched and says nothing about
  dismissal, since dismissing simply reveals what was underneath. The
  `nbgl_home_and_settings` example raises one from the home action button.
- `NbglNavigableContent`, wrapping `nbgl_useCaseNavigableContent` — a flow of
  pages under a touchable header, for content that is not a review. The module
  existed but was commented out and unusable: its navigation callback ignored
  the page index and returned four hard-coded choices through a pointer to a
  temporary array, using non-NUL-terminated `&str`s. Pages are now declared up
  front with `add_page` / `add_titled_page`, one `NbglPageContent` each, and
  `on_control` receives control touches. Declaring pages up front is what makes
  it sound: content built inside the callback would be dropped before NBGL drew
  it. On Nano the per-page title and top-right icon are ignored, and
  `ExtendedCenter` / `InfoLongPress` pages are rejected by `show`, that device's
  page union having eight members against the touchscreen devices' eleven.
- Four more `NbglGenericReview` content types, taking it from 6 to 10 of the 11
  members of the C content union: `SwitchesList` (`SWITCHES_LIST`),
  `ChoicesList` (`CHOICES_LIST`), `BarsList` (`BARS_LIST`) and `ExtendedCenter`
  (`EXTENDED_CENTER`, a centered info block with an inline tip box). They are
  added through the corresponding new `NbglPageContent` variants.
  `TAG_VALUE_DETAILS` is deliberately not offered: the C dispatch for
  app-supplied content rejects it (`nbgl_use_case.c`, `default:` arm returning
  false), NBGL only producing it internally when a pair is too long to fit, so
  a page built from it renders empty.
- Tip box on a review's first page, exposing `nbgl_tipBox_t` through the new
  `TipBox` type: `NbglAdvanceReview::tip_box()`. Touching it opens a modal
  listing `[type, content]` rows; the parameter was previously always NULL.
  Only `INFOS_LIST` is offered, the sole member of the C union. It applies only
  to a review whose warning set raises no tip box of its own: any `W3c*`
  warning, or `BlindSigning`, makes NBGL draw its own tip box and route the
  touch to the security report instead. That is also why `NbglReview` has no
  equivalent — the only use case it wraps that takes a tip box is the
  blind-signing one, which always raises `BlindSigning`. The
  `nbgl_advance_review` example shows one on a review with no warning.
- Home screen action button, exposing `nbgl_homeAction_t` through the new
  `HomeAction` and `HomeActionStyle` types, set with
  `NbglHomeAndSettings::action()`. The button's function is supplied by the app
  and NBGL runs it on touch, so it can start any use case; the parameter was
  previously always NULL. The `nbgl_home_and_settings` example shows one
  displaying a status page.

### Changed
- `NbglHomeAndSettings::infos(app_name, version, author)` is unchanged and now a
  shortcut over `app_name()` + `info_list()`.
- `NbglHomeAndSettings` passes a NULL `infosList` to the C use case when no
  information field is set, instead of announcing two fields backed by an empty
  array.
- The `&[Field]` review methods are unchanged and now lift their fields into
  extension-less `TagValue`s, so every use case builds its C tag/value array
  through a single path.
- `NbglAddressReview::set_tag_value_list` no longer ties the borrowed fields to
  the builder's lifetime (it copies them), which only relaxes what callers may
  pass.
- `warning_details(...)` on `NbglAdvanceReview` and `NbglStreamingReview` is
  unchanged and still raises exactly `W3cRiskDetected`; it now delegates to
  `warning()`, so both use cases build the C warning through one path.
- `QrCode` and `WarningDetails::QrCode` are compiled only for Stax, Flex and
  Apex. `NBGL_QRCODE` is not defined for Nano, so the C bindings there have
  neither `nbgl_layoutQRCode_t` nor the union member.
- updating ref to ledger_secure_sdk_sys to 1.16.5

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
