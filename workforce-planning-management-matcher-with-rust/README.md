# workforce-planning-management-matcher

Decide whether two worker records describe the same person: for import
de-duplication, rehire detection and merging a directory feed.

Pure (no database, no clock, no network), stateless, `Send + Sync`, and
**explainable**: every result carries each field's score and what the decision
rests on. Part of [workforce planning management](../README.md); the
requirement is [WPM-R134](../spec/worker-matching.md). Modelled on the sixarm
`person-matcher` crate and narrowed to what WPM may hold: **no national
identifiers and nothing about health.**

> ⚠️ Demo software. The thresholds are defaults, tuned on no data. A score
> proposes; a person decides ([WPM-D85](../spec/worker-matching.md)).

## Use

```toml
[dependencies]
workforce-planning-management-matcher = "0.1"
```

```rust
use workforce_planning_management_matcher::{Decision, MatchingEngine, WorkerRecord};

let held = WorkerRecord::new()
    .with_given_name("Katherine")
    .with_family_name("Johnson")
    .with_email("k.johnson@example.org");
let incoming = WorkerRecord::new()
    .with_given_name("Kathy")
    .with_family_name("Johnson")
    .with_email("K.Johnson@Example.org");

let result = MatchingEngine::default().match_workers(&held, &incoming);
assert_eq!(result.decision, Decision::Match);
assert_eq!(result.breakdown.phone, None); // unknown, not zero
```

Screen one row against many, best first: `engine.rank_one_to_many(&query, &held)`.
Run `cargo run --example dedupe_import`.

## What it compares

| Field | Score |
| --- | --- |
| Given name | 1 equal after folding; 0.95 a nickname (Mike ↔ Michael); else Jaro-Winkler/Levenshtein, at least 0.85 if it sounds the same |
| Family name | 1 equal; 0.9 one is a part of the other (`Smith-Jones` / `Jones`); else similarity |
| Date of birth | 1 same; 0.6 the two numbers of day and month transposed; 0.5 exactly one part of the date differs; else 0 |
| Email | 1 a shared address; 0.5 only the local part; else 0 |
| Phone | 1 a shared number once normalised to `+digits`; else 0 |
| Worker number | the same number from the **same organization** settles it; a number is never compared across organizations |

## Rules it keeps

1. **An unknown is never a zero.** A missing or unparseable field is `None` and left out of the average.
2. **Too little is not a verdict.** Fewer than `min_fields` (2) comparable fields is `Decision::Insufficient`, with no score.
3. **It says what it rests on.** `Basis::WorkerNumber` or `Basis::Fields`, plus the breakdown.
4. **A stable fact that disagrees is not averaged away.** A different date of birth, or a different worker number in one organization, caps `Match` at `Review`.
5. **No values in logs.** `{:?}` of a `WorkerRecord` shows which fields are present, not what they hold.
6. **Symmetric.** `match_workers(a, b)` equals `match_workers(b, a)`.

## Configuration

`MatchConfig::default()`, `strict()` and `lenient()`; it serialises to JSON, so
it can be loaded from a file, and `MatchingEngine::new` refuses an invalid one
(negative weight, review above match, `min_fields` of 0, a bad country code).
`default_phone_country_code` (digits, for example `"44"`) lets a number written
with a leading `0` be compared; without it such a number is unknown.

## Not done

Address matching, transliteration across scripts, non-English nicknames, a
fuzz or property test, a benchmark, wiring into the service, and tuning on data.
See [WPM-T226](../spec/tasks.md).

## Licence

`MIT OR Apache-2.0 OR BSD-3-Clause OR GPL-2.0-only OR GPL-3.0-only`, as the rest of the repository.
