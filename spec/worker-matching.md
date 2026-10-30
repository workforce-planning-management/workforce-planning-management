# Worker matching

How WPM decides that two worker records describe the same person, for import
de-duplication, rehire detection and merging a directory feed. Built as a
standalone crate, [`workforce-planning-management-matcher-with-rust/`](../workforce-planning-management-matcher-with-rust/README.md),
modelled on the sixarm `person-matcher` crate and narrowed to what WPM may hold.

## WPM-R134 — Match worker records, and say what the answer rests on

- **A library, not an endpoint.** The matcher is a pure Rust crate: no database, no clock, no network, no state between calls.
  The service calls it for exactly one read-only question (WPM-R136); an import or a merge is still a later requirement with its own access check.
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

## WPM-R136 — Ask which existing workers a new hire might already be

- **One read-only endpoint.** `POST /api/workers/possible-duplicates` takes `display_name`, `organization_ref` and an optional
  `worker_number`, and returns at most 10 existing workers that might be the same person, best first, each with `worker_pid`,
  `display_name`, `decision`, `basis`, `score` and the field `breakdown`. It writes nothing, merges nothing and records no audit row.
- **Only what the caller can already read.** Candidates are the workers in the caller's organizations that pass the per-record `Read`
  check; `organization_ref` outside the caller's organizations is `403`. The answer names no one the directory would not.
- **Evidence is a name and a worker number, nothing else.** WPM holds one `display_name` and no date of birth; the name is compared whole, never split
  into a first and a last word (WPM-T381, below). Email and phone are not compared (WPM-D87), so the breakdown has no email, phone or birth score.
- **A name alone is `review`.** Only the same worker number from the same organization is `match`. A different organization's number is
  no evidence. A blank, a name with no letters, or one word against a name of several words ("Ada" against "Ada Lovelace") is too
  little and lists no one; two one-word names are compared like any other. Invalid input is `422`.
- **The name is compared whole, in any script and any order (WPM-T381, amended 2026-10-30).** The name is folded into tokens: NFKC,
  lower case, punctuation and spaces as boundaries, diacritics dropped in Latin, Greek and Cyrillic only (in Devanagari, Bengali and
  other scripts a combining mark is part of the letter and kept; Arabic and Hebrew optional vowel marks are dropped). Two names with the
  same letters once spaces are ignored ("山田 太郎", "山田太郎") score 1. Otherwise each token is paired with its most alike token on the
  other side (the matcher's 0.7 Jaro-Winkler + 0.3 Levenshtein), best pairs first, each used once, and the score is the pairs'
  similarity weighted by their letters over all the letters on both sides; an unpaired token (a middle name, a particle) scores 0. Order
  is ignored, so a family name written first ("Lovelace Ada", "YAMADA Taro") is the same name, and by the same rule "Ivan Petrov" and
  "Petr Ivanov" can be `review`. A score at or above the matcher's review threshold (0.70) is `review`. The breakdown is `name` and
  `worker_number`; the given-name and family-name scores, and the matcher's English nickname table, are no longer used for this check.
  **Not done:** no transliteration, so one person written in two scripts ("Ivan Petrov", "Иван Петров") is not found; no name parts a
  person gave are held (WPM-D143's "optional parts"), so none is compared.

## WPM-D87 — Compare names and numbers only, and never let a name decide

Two reasons narrow the comparison. First, the contact details (`worker_contact_details`) sit behind a narrower access check than
the directory; "the email you typed matches worker X" would tell a caller the very thing that check withholds, so email and phone are
left out even though the matcher can score them. Second, two real people share a name, and WPM has no date of birth to separate them, so
a name-only `match` is reported as `review` and a person decides (WPM-D85). The thresholds are the matcher's defaults, tuned on no data.
