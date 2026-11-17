# Changelog

## Unreleased

- Coverage (WPM-T539): 98.75% of lines (317 of 321) with `cargo tarpaulin --engine llvm`; CI holds it at 95%.

## 0.2.0 - 2026-11-17

- **Breaking:** `WorkerRecord::date_of_birth` is `Option<jiff::civil::Date>`, was `Option<chrono::NaiveDate>` (WPM-T528); the crate
  depends on `jiff` 0.2 (with `serde`) in place of `chrono`, which RUSTSEC-2026-0543 marks unmaintained. `with_date_of_birth(year,
  month, day)` keeps its signature and still gives `None` for a day that does not exist; a date of birth serialises as before,
  `YYYY-MM-DD`.
- Tests only (WPM-T526): nine tests from a cargo-mutants run, so every viable mutant is caught (192 of 194; one equivalent
  mutant is left out in `.cargo/mutants.toml`). No change to the code or the API.
- `rust-version` is 1.98, the N-2 MSRV (spec/rust-msrv-n-minus-2), down from 0.1.0's 1.99: first 1.97 (WPM-T389), then 1.98
  when stable reached 1.100 (WPM-T492). CI checks it there.

## 0.1.0 - 2026-10-19

- First version: `WorkerRecord`, `MatchingEngine`, `MatchConfig` (strict, default,
  lenient), `Decision`, `Basis`, `Breakdown`, `NicknameTable`, normalizer and scorer.
- Modelled on the sixarm `person-matcher` crate, narrowed to what WPM may hold:
  no national identifiers, nothing about health; employer worker numbers are
  scoped to their organization.
- Unknown fields score `None`; fewer than `min_fields` comparable fields is
  `Decision::Insufficient`; a differing date of birth or a different worker number
  in the same organization caps a match at `Review`.
