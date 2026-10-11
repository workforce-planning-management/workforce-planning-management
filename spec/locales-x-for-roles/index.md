# Locales × for roles (WPM-R131, WPM-D79)

One page per role for each locale, at `locales/<locale>/for/<role>/index.md`, saying what Workforce Planning
Management gives that role, how to read what it shows, what it will not do, and what is still open. The first
locale is `en-gb-oxendict` (British English with Oxford spelling): see
[its index](../../locales/en-gb-oxendict/for/index.md).

| File                                                     | Role                       |
| -------------------------------------------------------- | -------------------------- |
| locales/<locale>/for/chief-executive-officer/index.md    | Chief Executive Officer    |
| locales/<locale>/for/chief-operating-officer/index.md    | Chief Operating Officer    |
| locales/<locale>/for/chief-technology-officer/index.md   | Chief Technology Officer   |
| locales/<locale>/for/chief-information-officer/index.md  | Chief Information Officer  |
| locales/<locale>/for/chief-risk-officer/index.md         | Chief Risk Officer         |
| locales/<locale>/for/chief-legal-officer/index.md        | Chief Legal Officer        |
| locales/<locale>/for/chief-people-officer/index.md       | Chief People Officer       |
| locales/<locale>/for/chief-strategy-officer/index.md     | Chief Strategy Officer     |
| locales/<locale>/for/chief-security-officer/index.md     | Chief Security Officer     |
| locales/<locale>/for/governance-risk-compliance/index.md | Governance, Risk, Compliance |
| locales/<locale>/for/independent-security-tester/index.md | Independent security testing (a brief for commissioning it, not a role) |

For each `index.md`, `README.md` is a symlink to it (the same convention as the rest of this repository).

## WPM-R131 — A page for each role

*As an executive deciding whether and how to use WPM, I read one page written for my role, in my locale.*

- Each page has the same parts: the question the role is likely asking, what WPM gives (a table, each row linking to
  the specification), how to read the numbers, what it will not do, the open limits, the decisions that are the
  reader's, and a dated statement of how current it is.
- Each page begins from the [readiness assessment](../readiness-assessment.md) verdict where it matters: the software
  is demonstration software and is not ready for production use in a high-assurance organization.
- A page states only what the specification or the code shows. A thing planned or not built is said to be so.
- No organization is named. No legal advice is given.
- A further locale is a translation of these pages under its own `locales/<locale>/for/`, with the same folder names.

## Locales

| Locale | How it was made |
| --- | --- |
| `en-gb-oxendict` | The source: British English with Oxford `-ize` spelling |
| `en-001` | A copy of the source (international English; no change was needed) |
| `en-us` | The source with American spelling (`license`, `program`) outside code, links and identifiers |
| `en-gb` | The source with the `-ise` family (`organisation`, `recognised`) outside code, links and identifiers |
| `zh-cn`, `zh-001` | Translated into Simplified Chinese; `zh-001` is a copy of `zh-cn` |
| `hi-in`, `hi-001` | Translated into Hindi; `hi-001` is a copy of `hi-in` |
| `cy-gb`, `cy-001` | Translated into Welsh; `cy-001` is a copy of `cy-gb` |
| `de-de`, `de-001` | Translated into German; `de-001` is a copy of `de-de` |
| `es-es`, `es-001` | Translated into Spanish; `es-001` is a copy of `es-es` |
| `fr-001`, `pt-001`, `id-001`, `ru-001` | Translated into French, Portuguese, Indonesian and Russian |
| `ar-001`, `bn-001`, `ur-001` | Translated into Arabic, Bengali and Urdu (the Arabic and Urdu pages are right-to-left and were not rendered or checked on screen) |

Each folder under `for/` (and `for/` itself) holds `index.md`, a `README.md` symlink to it, and a `.locale-peer-id` file:
32 lowercase hexadecimal characters and a newline, **byte-identical in every locale** for the same page. Folder names
(slugs) are **per locale**, following [locales-for-global-sharing-with-svelte](../locales-for-global-sharing-with-svelte/index.md):
the English locales (`en-gb-oxendict`, `en-001`, `en-us`, `en-gb`) keep the English names in the table above, and every
translated locale names each page folder in its own language and script (for example `首席执行官`, `geschäftsführung`,
`الرئيس-التنفيذي`), lowercase, joined by hyphens, with no spaces. The `<role>` names in the table are therefore the English
ones and the key of the page; the same page in another locale is found by its `.locale-peer-id`, not by its name. The folder
`for/` is not translated. A copy locale (`zh-001`, `hi-001`, `cy-001`, `de-001`, `es-001`) has the same slugs as its source.

Each translated page keeps every link target, table row and heading of the source, and ends with a note that an AI wrote
the translation and no native speaker has reviewed it. Code, paths, requirement ids and API routes are not translated.

## WPM-D79 — The pages link to the specification and copy no figures

The specification stays the single source of truth (rule 1 of [AGENTS.md](../../AGENTS.md)). A role page summarizes
and links; it does not restate a requirement or copy a count that changes, so it cannot drift from a number. Counts
live in [implementation-status.md](../implementation-status.md), which a script generates. The `locales/` folder holds
reader-facing pages, one folder per locale; the `spec/` folder holds the requirement.

**Not verified:** the translations (Chinese, Hindi, Welsh) were not reviewed by a native speaker, and the English pages were written from the specification and were not reviewed by a person in any of these
roles. A claim in one that the specification no longer supports is a defect to fix there and here.
