# Figure declarations

The declarations [RCA-DECL-1](rust-checked-arithmetic/index.md#numeric-figure-declarations) asks for, for the figures whose code
names a **fallback** (RCA-DECL-4, RCA-NONE-2). A fallback is the answer the specification gives when a computed result cannot be
held, so the code writes it as `checked_*` or `try_from` followed by `unwrap_or`, and cites this file. Every fallback here is a
valid value of its figure, and none stands in for an input that is out of range or malformed: an input outside its range is
refused before the computation (RCA-IN-2, RCA-DECL-6). Where a result has no correct answer, it is `None` or an error, and no
fallback is declared.

Each figure below has the seven fields of RCA-DECL-1. "Why" says why the fallback is the correct answer.

## Lists and pages

### `page_limit`

| Field | Declaration |
| --- | --- |
| Unit | rows in a page |
| Type | `u64` from the request; `usize` or `i64` where a collection or a SQL `LIMIT` takes it |
| Range | 1 to the list's maximum (`MAX_LIMIT`); anything else is a `422` before this figure is used |
| Unknown | not applicable: the limit is always supplied or defaulted |
| Rounding | none |
| Overflow | a limit that does not fit the narrower type |
| Fallback | the largest value of that type (`usize::MAX`, `i64::MAX`) |

Why: a validated limit is far below `i64::MAX`, so the conversion cannot fail on a 64-bit target. If it ever did, a limit past the
type already asks for every row there is, and the largest value of the type takes every row. `rows_to_read` adds one to the limit
to learn whether another page follows; at the top of `usize` the limit already takes the whole list, so the sum is `usize::MAX`
and no row is lost.

### `page_offset`

| Field | Declaration |
| --- | --- |
| Unit | rows skipped |
| Type | `u64` from the cursor; `usize` where a collection skips |
| Range | 0 to the largest offset a cursor can carry; a bad cursor is a `422` before this figure is used |
| Unknown | not applicable |
| Rounding | none |
| Overflow | an offset that does not fit `usize` |
| Fallback | `usize::MAX` |

Why: an offset past `usize` is past every row of any list in memory, so skipping `usize::MAX` rows leaves the page empty, which
is what an offset past the end is.

## Counts against a limit

### `people_count`

| Field | Declaration |
| --- | --- |
| Unit | workers |
| Type | `usize` (a collection's length) compared as `u64` |
| Range | 0 to the ceiling (`WORKFORCE_PLANNING_MANAGEMENT_MAX_ORG_WORKERS`, WPM-R120) |
| Unknown | not applicable: the count is of rows already loaded |
| Rounding | none |
| Overflow | a count that does not fit `u64` |
| Fallback | `u64::MAX` |

Why: the figure is only compared with the ceiling. A count past `u64` is over any ceiling, so `u64::MAX` is refused exactly as
the real count would be.

### `status_index`

| Field | Declaration |
| --- | --- |
| Unit | position in a credential revocation list |
| Type | `usize` |
| Range | 0 up to the list's size (`STATUS_BITS`) |
| Unknown | an id that does not give an index |
| Rounding | none |
| Overflow | an id too large for `usize` |
| Fallback | `usize::MAX` |

Why: the index is only compared with the list's size. An index that does not fit is past the end, so the list is full and a new
list is needed, which is the answer the code gives (`422`).

### `cache_entry_bytes`

| Field | Declaration |
| --- | --- |
| Unit | bytes |
| Type | `usize` |
| Range | 0 to the cache's capacity for an entry that is held |
| Unknown | not applicable |
| Rounding | none |
| Overflow | a key and value together too large for `usize`, or a running total too large |
| Fallback | `usize::MAX` (`OVER_CAPACITY`) |

Why: the figure is only compared with the capacity. A size no integer holds is over every capacity, so the entry is not held and
the cache is cleared first, as for any entry that does not fit.

## Tolerances, economics and succession

### `cost_deviation_bp`

| Field | Declaration |
| --- | --- |
| Unit | basis points of the plan's budget |
| Type | `i64` |
| Range | 0 to `i64::MAX`; no budget, or a budget of zero, is unknown |
| Unknown | `None`: a plan with no budget has not shown it is within tolerance (WPM-R140) |
| Rounding | down, once, at the end |
| Overflow | an overspend so large that its share of the budget does not fit `i64` |
| Fallback | `i64::MAX` |

Why: the figure is only compared with the plan's tolerance and its amber point. A share too large for `i64` is above any
tolerance, so `i64::MAX` is red, which is what the true figure would be.

### `investment_minor` and `benefit_minor`

| Field | Declaration |
| --- | --- |
| Unit | minor units of the plan's currency |
| Type | `i64`, as echoed in the viability answer; `i128` while it is summed |
| Range | 0 to `i64::MAX` |
| Unknown | not applicable: a total of no flows is `0` and the verdict is `Unmeasured` (`NoInvestment` or `NoBenefit`) |
| Rounding | none |
| Overflow | a total of absolute flows past `i64` |
| Fallback | `i64::MAX` in the echoed field only |

Why: a total past `i64` makes the verdict `Unmeasured` with the reason `OutOfRange`, so the figure takes no part in a verdict or a
net present value. The echo says "at least this much" and is never used to compute.

### `lowest_bar`

| Field | Declaration |
| --- | --- |
| Unit | proficiency level |
| Type | `i32` |
| Range | the proficiency scale, from the lowest minimum any critical requirement declares |
| Unknown | not applicable |
| Rounding | none |
| Overflow | not applicable |
| Fallback | `i32::MAX` when no critical requirement declares a minimum (`NO_HOLDER_BAR`) |

Why: the bar is a lower bound on who can hold a skill. With no bar declared there is no skill to hold, so the bound is one no
proficiency reaches and nobody is a holder, which is the correct answer for a plan with no critical requirement.
