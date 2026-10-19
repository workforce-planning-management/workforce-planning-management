# Changelog

## 0.1.0 - 2026-10-19

- First version: `WorkerRecord`, `MatchingEngine`, `MatchConfig` (strict, default,
  lenient), `Decision`, `Basis`, `Breakdown`, `NicknameTable`, normalizer and scorer.
- Modelled on the sixarm `person-matcher` crate, narrowed to what WPM may hold:
  no national identifiers, nothing about health; employer worker numbers are
  scoped to their organization.
- Unknown fields score `None`; fewer than `min_fields` comparable fields is
  `Decision::Insufficient`; a differing date of birth or a different worker number
  in the same organization caps a match at `Review`.
