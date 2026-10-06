# Job levels: Google technical levels (WPM-R52)

A job-level framework is a published ladder of levels: what each level is, the
experience it usually takes, and the management-track role at the same level.
WPM carries Google's technical (individual-contributor) ladder, L3–L11, as
**reference data**. Delivered as WPM-T93 and WPM-T94 (see [tasks.md](tasks.md)).

## Source and its limits

Supplied by the maintainer on 2026-10-06 as a pasted summary of Google's job
levels. It is a **secondary, unofficial** source: Google does not publish its
ladder, titles differ by function, and the summary mentions a `T` technical-track
prefix that it does not define, so only the `L` ladder is modelled. It is not
independently verified, and the service says so in each framework's `source`.

| Level | Title | Typical experience | Management equivalent |
| --- | --- | --- | --- |
| L3 | Software Engineer I | 0–1 year | — |
| L4 | Software Engineer II | roughly 2–4 years, or a completed PhD | — |
| L5 | Senior Software Engineer | 5–10+ years | Engineering Manager |
| L6 | Staff Software Engineer | roughly 10+ years | Engineering Manager |
| L7 | Senior Staff / Principal Software Engineer | not stated | Engineering Manager |
| L8 | Distinguished / Principal Engineer | 15+ years | Director |
| L9 | Senior Distinguished Engineer | not stated | Senior Director |
| L10 | Google Fellow | not stated | Vice President (VP) |
| L11 | Senior Google Fellow | not stated | Vice President II (VP II) |

Engineering Managers at L5–L7 oversee teams of roughly 5 to 40+ people. The
summary's L8 label is "Distinguished / Principal Engineer" and L7's is "Senior
Staff / Principal"; "Principal" appears at both, and WPM keeps the source's
wording rather than reconcile it.

## WPM-R52 — Job levels

*As anyone signed in I can see a published job-level ladder and read each level.*

- `GET /api/job-levels` lists the frameworks (id, organization, track, source,
  level codes). `GET /api/job-levels/{id}` returns every level.
  `GET /api/job-levels/{id}/levels/{code}` returns one level and the next one up
  (`null` at the top); `L5`, `l5` and `5` name the same level. An unknown
  framework or level is **404**. Id today: `google-levels`.
- UI: `/job-levels` — the table, with a dash for anything the source does not state.

## WPM-D39 — A level is not a salary, and an unstated fact is unknown

- **No pay.** The source gives none; a level carries no salary, band or range, so
  none can be mistaken for Google's or anyone's actual pay. Linking a level to a
  pay scale ([pay-scales.md](pay-scales.md)) would be a separate decision.
- An experience or management equivalent the source does not state is `null`,
  rendered "—": **unknown is never a guess** (WPM-D32).
- Reference data only: no table, no worker field, nothing stored against a person.

## WPM-R53 — Grades: a level for a worker, a level and a pay band for a role

*As a person I can record my own job level; as someone who edits a role I can
say which job level and pay band the role sits at.*

- **Worker** — `GET|PUT|DELETE /api/workers/{pid}/job-level`. One current level per
  worker: `{framework, level, effective_on?}` (the level may be spelt `L5`, `l5` or
  `5`; `effective_on` defaults to today and cannot be in the future). The framework
  and level are validated against the reference ladder (**422** otherwise). `GET`
  returns `{"job_level": null}` until one is recorded; `DELETE` of none is **404**.
  Readable and writable by **the person and HR only** (a manager cannot, by
  default); edits record who made them and whether on the person's behalf.
- **Role profile** — `GET|PUT|DELETE /api/role-profiles/{pid}/grade`. A role has an
  optional job level and an optional pay band (`{scale, band}`, validated against
  the pay scale). `GET` shows the level's title and the band's entry and top pay and
  steps. `PUT` replaces the grade — what is absent is cleared — and sending neither
  is **422** (`DELETE` clears).
- UI: a **Job level** panel on `/me` and on a worker's page (hidden for anyone who
  may not see it), and a **Grade** panel on each role at `/roles`.
- Privacy wiring: `worker_job_levels` is in the **subject-access export** and
  **deleted on erasure** (hard-deleted; no `deleted_at`, so not in the retention
  sweep's list). Migration `m20261006_000047_grades`.

## WPM-D40 — A level is sensitive, and a link is an editor's statement, not a derivation

- **A worker's level is high-sensitivity**, like the emergency contacts: seniority
  tracks pay closely. The person and HR only. The audit entry (`job_level_set`,
  `job_level_cleared`) **names no level** — that the level changed is the record,
  not what it changed to — so only the current level is kept and there is **no
  level history** (a deliberate limit; see below).
- **No equivalence is derived.** No source says which Google level matches which
  NHS band, and the two systems measure different things, so WPM neither ships a
  mapping nor computes one. A role carrying both a level and a band is a statement
  **by whoever edits that role**, recorded as that (`grade_set`, with the pair in the
  audit snapshot — a role is not a person). Nothing reads one to infer the other.
- Roles are reference data, not personal data; a role's grade is readable by anyone
  who can read role profiles.

## Not done

- Level **history** and promotion dates (deliberately not kept, WPM-D40); a roll-up
  of headcount by level (a count of one at a high level would name someone);
  assigning a level through the joiner/leaver flow; mapping the UK Government
  capability framework or ESCO levels to ladders; level-based promotion paths.
- A suggested level for a role from its skills, or a pay band for a level.
- Other ladders (the non-technical, sales or management tracks) and other
  companies; a verified or official source.
