# Changelog

## Unreleased

- `rust-version` is 1.97, the N-2 MSRV (spec/rust-msrv-n-minus-2; stable is 1.99.1), down from 1.99: the crate
  compiles on 1.97 and CI checks it there (WPM-T389). Not published: 0.1.0 on crates.io still declares 1.99 (WPM-T390).
  `cargo update` had nothing to update.

## 0.1.0 - 2026-10-19

- First version: `WorkerRecord`, `MatchingEngine`, `MatchConfig` (strict, default,
  lenient), `Decision`, `Basis`, `Breakdown`, `NicknameTable`, normalizer and scorer.
- Modelled on the sixarm `person-matcher` crate, narrowed to what WPM may hold:
  no national identifiers, nothing about health; employer worker numbers are
  scoped to their organization.
- Unknown fields score `None`; fewer than `min_fields` comparable fields is
  `Decision::Insufficient`; a differing date of birth or a different worker number
  in the same organization caps a match at `Review`.
