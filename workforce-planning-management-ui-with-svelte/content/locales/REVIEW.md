# Translation review record (WPM-R94, WPM-D64)

English (`en-001`) is the source. Every other locale was **written by an AI assistant** and
**has not been reviewed by a person**. A locale is "reviewed" only when its row below names a
reviewer, a date and a scope, and says the outcome. Until then, treat it as a draft: do not
present it to users as a professional translation, and do not rely on it to meet a legal duty
to offer a service in that language.

| Locale | Language | Written by | Reviewer | Reviewed on | Scope | Outcome |
| --- | --- | --- | --- | --- | --- | --- |
| `ar-001` | Arabic | AI assistant, 2026-10-10 | none | not reviewed | not reviewed | not reviewed |
| `bn-001` | Bengali | AI assistant, 2026-10-10 | none | not reviewed | not reviewed | not reviewed |
| `cy-001` | Welsh | AI assistant, 2026-10-10 | none | not reviewed | not reviewed | not reviewed |
| `de-001` | German | AI assistant, 2026-10-10 | none | not reviewed | not reviewed | not reviewed |
| `es-001` | Spanish | AI assistant, 2026-10-10 | none | not reviewed | not reviewed | not reviewed |
| `fr-001` | French | AI assistant, 2026-10-10 | none | not reviewed | not reviewed | not reviewed |
| `hi-001` | Hindi | AI assistant, 2026-10-10 | none | not reviewed | not reviewed | not reviewed |
| `id-001` | Indonesian | AI assistant, 2026-10-10 | none | not reviewed | not reviewed | not reviewed |
| `pt-001` | Portuguese | AI assistant, 2026-10-10 | none | not reviewed | not reviewed | not reviewed |
| `ru-001` | Russian | AI assistant, 2026-10-10 | none | not reviewed | not reviewed | not reviewed |
| `ur-001` | Urdu | AI assistant, 2026-10-10 | none | not reviewed | not reviewed | not reviewed |
| `zh-001` | Chinese | AI assistant, 2026-10-10 | none | not reviewed | not reviewed | not reviewed |

The 2026-10-13 additions (88 keys for the capacity, resilience, oversight, cutover and early-warning pages, including the long
"how this is worked out" texts and the twelve cutover reason sentences) were written by the same assistant in one pass, also unreviewed.
Read the long texts first: they carry meaning (an unknown is never a pass; the go/no-go is a human decision).

The 2026-10-21 additions (86 keys for the My details page and eight status words) were written by the same assistant, also unreviewed. The equality and health notices on that page are the service's English text.

The 2026-10-23 additions (the `errors.*` keys: eight sentences saying what a failed call means) were written by the same assistant, also unreviewed.

The 2026-10-28 additions (nine value words in all 12 translated locales: the leave kind `public_holiday`, and `site`, `area_of_work`, `job_role`, `staff_group`, `occupation_code`, `mandatory`, `performance`, `padr`) were written by the same assistant, also unreviewed. `padr` ("personal development review") and the three staff-record words (`job_role`, `staff_group`, `occupation_code`) are terms of art that differ between health, government and commercial employers; a reviewer should check them against the employer's own vocabulary. Two French words equal the English and are allow-listed (`site`, `performance`) in `tests/unit/untranslated.allow.json`.

The 2026-10-24 additions (the `labels.*` keys: eight accessible names for form controls; three `values.*` words for the outcome of a joint pay assessment; five `values.*` words for the Annex III categories of the AI register; two `values.*` words for position states, 2026-10-24) were written by the same assistant, also unreviewed.

The 2026-10-25 additions (`values.*` words for the kinds of equipment and how an issue ends, the three audiences of a policy, the four confidences and two bases of a skill mapping, written by the same assistant, also unreviewed): laptop, phone, pass, key, returned, written_off, all, department, job_title, exact, close, broader, narrower, authority, judgement. The German "Notebook", the Indonesian "komputer jinjing" and the French "précis" were chosen so as not to repeat the English word.

The 2026-10-30 additions (four keys for money input, WPM-T373 and WPM-T376: `expenses.badAmount`, `expenses.currencyNeeded`,
`pages.planning.budget_needs_currency` and `pages.planning.bad_budget`) were machine-translated by the same assistant, also unreviewed.
The currency codes in them (EUR, JPY, INR) are ISO 4217 codes and stay in Latin letters in every locale.

The 2026-10-30 additions for typed numbers (WPM-T375: `common.badNumber` and `common.badNumberAtLeast`) were machine-translated by the
same assistant, also unreviewed.

The regional locales (`de-de`, `en-gb`, `en-us`, `es-es`) hold only overrides of their language
and have the same status as their language.

The text from 2026-10-10 (710 keys: every page and component, the page titles, and the closed
vocabularies under `values.*`) was written in one pass, without a glossary; earlier keys were
written earlier, also by the assistant. Expect inconsistent terminology between the two.

## How to review a locale

1. Open the locale's `ui.json` beside `en-001/ui.json` (or use the content manager at `/admin`).
2. Read it as a user would, in the app, not only as a file: the same word can need different
   forms in a heading, a button, a table column and a sentence.
3. Check that placeholders such as `{name}` are untouched, that plurals read correctly for the
   numbers that fill them (the interface writes forms like "skill(s)" because it has no plural
   rules), and that right-to-left languages read correctly with Latin names and numbers.
4. Check the terms the pack depends on: the HR terms (headcount, attrition, succession),
   the framework names (ESCO, ISCO, UK GDAD PCF) and the statuses under `values.*`.
5. Review the entries on the allow-list (`tests/unit/untranslated.allow.json`): a word the
   assistant left the same as English. Translate any a native speaker would not accept, and
   delete it from the list.
6. Fill in the row above (reviewer, date, scope, outcome) in the same change.

A review that finds problems is a success. Record what was fixed in the outcome column.
