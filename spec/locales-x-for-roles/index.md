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

## WPM-D79 — The pages link to the specification and copy no figures

The specification stays the single source of truth (rule 1 of [AGENTS.md](../../AGENTS.md)). A role page summarizes
and links; it does not restate a requirement or copy a count that changes, so it cannot drift from a number. Counts
live in [implementation-status.md](../implementation-status.md), which a script generates. The `locales/` folder holds
reader-facing pages, one folder per locale; the `spec/` folder holds the requirement.

**Not verified:** the pages were written from the specification and were not reviewed by a person in any of these
roles. A claim in one that the specification no longer supports is a defect to fix there and here.
