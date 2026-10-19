# Worker matching

How WPM decides that two worker records describe the same person, for import
de-duplication, rehire detection and merging a directory feed. Built as a
standalone crate, [`workforce-planning-management-matcher-with-rust/`](../workforce-planning-management-matcher-with-rust/README.md),
modelled on the sixarm `person-matcher` crate and narrowed to what WPM may hold.

## WPM-R134 — Match worker records, and say what the answer rests on

- **A library, not an endpoint.** The matcher is a pure Rust crate: no database, no clock, no network, no state between calls.
  The service does not call it yet; wiring it to an import or a merge is a later requirement with its own access check.
- **What it compares.** Given name, family name, date of birth, email addresses, phone numbers and employer worker numbers.
  Each is optional. It has **no field for a national identifier** and none for anything about health, because WPM stores neither
  (WPM-D17, WPM-D25). A worker number belongs to the organization that issued it: the same number from two organizations is no evidence.
- **An unknown is never a zero.** A field missing on either side, or one that cannot be normalised (a phone number with no country, an
  email with no `@`), scores `null` and is left out of the average. Fewer than `min_fields` (default 2) comparable fields is
  `insufficient`, with no score: neither a match nor a non-match.
- **The answer says what it rests on.** `basis` is `worker_number` (the same number from the same organization) or `fields` (the weighted
  average), and the breakdown holds every field's score. Four decisions: `match`, `review`, `no_match`, `insufficient`.
- **A stable fact that disagrees is not averaged away.** A different date of birth, or a different worker number inside one
  organization, caps a `match` at `review`: a person decides.
- **Nothing personal in a log line.** A record prints which fields it has, never their values.
- **Symmetric.** Comparing `a` with `b` gives the same result as `b` with `a`. Ranking one against many is best-first with a tiebreak on input order.

## WPM-D85 — A score proposes; it never merges

Two real people share a name and a birth year. A merge cannot be taken back without the audit trail it would erase, so the crate
returns a *decision with its working* and the caller keeps a person in the loop at `review`. The thresholds (match 0.88, review 0.70) are
defaults to be tuned on the deployer's own data; they are not claimed to be accurate, and the crate has been measured on no real data
(this repository holds synthetic data only). A deployment that auto-merges on `match` must say so in its own information-governance record.
