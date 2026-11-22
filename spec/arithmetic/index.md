# Arithmetic must not panic

Arithmetic on values that come from outside the code (a count, an amount, a duration, a date, a size, a page number) never
panics and never wraps. Where the result does not fit, there is no result: the operation returns `None`, and the caller turns
that into an error value, a refusal or a stated "unknown". An overflow is never turned into a zero, a maximum or a wrapped
number, because an unknown is never a zero ([privacy-and-data-rules](../../AGENTS/privacy-and-data-rules.md)).

## The rule

- Use the **checked** operations of the integer types, which return `Option<T>`: `checked_add`, `checked_sub`, `checked_mul`,
  `checked_div`, `checked_rem`, `checked_pow`, `checked_neg`, `checked_abs`, `checked_shl` and `checked_shr`.
- A result that does not fit is `None`:

  ```rust
  assert_eq!(255_u8.checked_add(1), None);   // not 0 (wrapped), and not a panic
  assert_eq!(250_u8.checked_add(5), Some(255));
  assert_eq!(i32::MIN.checked_abs(), None);
  assert_eq!(7_u32.checked_div(0), None);    // not a panic
  ```

- **Never unwrap the result of a `checked_*` call.** `checked_add(..).unwrap()`, `.expect(..)`, `.unwrap_or_default()` and
  `.unwrap_or(<number>)` turn the `None` back into a panic or an invented number, which is the failure the rule exists to
  prevent. Handle the `None`: return it, or turn it into an error with `ok_or` / `ok_or_else`, or match on it.
- Chain with `?` in a function that returns `Option`, or with `ok_or(...)` / `ok_or_else(...)` in one that returns `Result`, so
  the error says which figure overflowed.
- Converting between integer types uses `try_from` / `try_into`, never `as`, which truncates without a sound.
- Do not use the plain operators (`+`, `-`, `*`, `/`, `%`, unary `-`) on values from outside the code. They panic in a debug
  build and wrap in a release build, unless overflow checks are on. Do not use `wrapping_*` or `saturating_*` to hide an
  overflow, and do not use `unwrap_or(0)` or `unwrap_or(u64::MAX)` on a checked result; both invent a number.

## Where a plain operator is fine

- A counter that cannot exceed a bound the code itself sets, for example the index of a loop over a fixed-size array. Say why in a
  comment when it is not obvious.
- Tests.

## How it is checked

- The service and domain crates already deny panics in production code: `unwrap`, `expect`, `panic!`, indexing and string slicing
  (WPM-T633). `clippy::arithmetic_side_effects` is the lint for this rule. It flags every plain operator, so it is turned on one
  module at a time, as that module is converted, until the crate roots deny it.
- A rule tests the edges: the largest and smallest value of the type, zero as a divisor, and the first value that overflows.
