# Job levels: Google technical levels (WPM-R52)

A job-level framework is a published ladder of levels: what each level is, the
experience it usually takes, and the management-track role at the same level.
WPM carries Google's technical (individual-contributor) ladder, L3–L11, as
**reference data**. Delivered as WPM-T93 (see [tasks.md](tasks.md)).

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

## Not done

- Assigning a level to a worker or a role, mapping WPM's role profiles or the
  UK Government capability framework to levels, or level-based promotion paths.
- Other ladders (the non-technical, sales or management tracks) and other
  companies; a verified or official source.
