# Official-language statement (WPM-R92, WPM-R94)

> ⚠️ **The translations are machine-written and have not been reviewed by a person.** A
> deployer who has a duty to offer services in a particular language must have a competent
> translator review that language before relying on it.

## Languages offered

The interface is offered in **13 languages**, each with a general-purpose locale
(`<language>-001`), plus **four regional locales** that override only what differs:

| Language | Locale | Direction |
| --- | --- | --- |
| Arabic | `ar-001` | right to left |
| Bengali | `bn-001` | left to right |
| Welsh | `cy-001` | left to right |
| German | `de-001`, regional `de-de` | left to right |
| English | `en-001` (the source), regional `en-gb`, `en-us` | left to right |
| Spanish | `es-001`, regional `es-es` | left to right |
| French | `fr-001` | left to right |
| Hindi | `hi-001` | left to right |
| Indonesian | `id-001` | left to right |
| Portuguese | `pt-001` | left to right |
| Russian | `ru-001` | left to right |
| Urdu | `ur-001` | right to left |
| Chinese | `zh-001` | left to right |

The language is carried by the web address (for example `/cy-001/workers`), so a link is
always in one language. A bare `/` follows the browser's language, then remembers the choice.

## How the text was produced

English (`en-001`) is the source. Every other language was **written by an AI assistant**
and has not been reviewed. Each locale's review is recorded in
`workforce-planning-management-ui-with-svelte/content/locales/REVIEW.md`; a locale is
"reviewed" only when that record names a reviewer, a date and the scope. At the time of
writing, **none is reviewed**.

## Coverage

- **Every page and component** takes its text from the catalogue in all 13 languages (WPM-R94,
  WPM-T153, T154): headings, labels, buttons, placeholders, page titles and messages; the
  statuses, kinds and categories the server sends as tokens are translated too.
- A test pins that every language holds the same set of keys, that no template holds literal
  visible text, that no language quietly repeats the English (except a short list with a
  reason: brands, acronyms, and words that are the same in that language), and that every token
  the server's vocabularies define has a translation.
- **Not translated**: the server's own error and validation messages (shown as the server sends
  them, in English); names, titles and anything a person typed; a token the catalogue does not
  know yet (shown in English until it is added); the third-party content manager at `/admin`.
- **Not localized**: dates and numbers are shown in ISO form, not in the language's own style,
  and plural forms are written "skill(s)" because the catalogue has no plural rules.
- **Not looked at**: the new text in right-to-left layout (Arabic, Urdu), and every page other
  than the Welsh mentorship page in another language.

## For a deployer with a language duty

1. Have a competent translator review each language you are required to offer, and record
   it in `REVIEW.md`.
2. Decide what must be in that language: the interface, notices, documents, support.
3. Publish the languages you offer and how to ask for another format.
4. Re-review after a release that changes text (the CI test will show which keys changed).
