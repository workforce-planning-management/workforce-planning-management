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
