# Changelog

All notable changes to this project are documented here. Format: [Keep a
Changelog](https://keepachangelog.com/) — versions follow [semver](https://semver.org).

## [Unreleased]

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
