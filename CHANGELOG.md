# Changelog

All notable changes to this project are documented here. Format: [Keep a
Changelog](https://keepachangelog.com/) — versions follow [semver](https://semver.org).

## [Unreleased]

## [0.1.2] - 2026-10-24

### Changed

- Tracks `ap-kit 0.1.3` and `invoice-kit 1.0.2` — the estate's graph-wide convergence on `double-entry 0.1.1`. No behavioural change.


## [0.1.1] - 2026-10-24

### Changed

- Tracks the invoice-kit 1.0.0 API-stability release (identical surface to
  0.2.1). No behavioural change: the pin move keeps one Invoice type in a
  graph whose consumer already binds 1.0.0.


## [0.1.1] - 2026-10-24

### Changed

- Tracks  (the API-stability release; identical surface to
  0.2.1). No behavioural change: the pin move keeps one  type in a
  graph whose consumer already binds 1.0.0.

## [0.1.0] - 2026-10-07

Initial release.

### Added

- `VatReturn::build` — Boxes 1, 4, 6, 7 from AR invoices and AP bills, Box 5
  derived. Input tax is the deductible portion only; the non-deductible
  portion is expense and its net still counts in Box 7.
- Per-currency refusal, and tax-point filtering that fails loudly on a
  document outside the period.
- Reverse-charge documents contribute net to Box 6 and no tax to Box 1,
  matching what EN 16931 puts on the document; domestic reverse-charge filing
  rules are documented out of scope.
