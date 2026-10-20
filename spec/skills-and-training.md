# Skills gaps and training time (WPM-R43–R44)

What people need against what they have declared, and how long it would take
to close the gap. Builds on skills and learning paths
([talent-development.md](talent-development.md), WPM-T20), capability
analysis (WPM-T41) and the role gap (WPM-T51). Delivered as WPM-T87–T88.

## WPM-R43 — Skills gap analysis

*As a person I can see where my declared skills fall short of what I need; as
HR or a leader I can see where the workforce is thin — without anyone being
named.*

- **Needs** come from three sources, each with a level and an importance
  (`critical` weight 3 · `important` 2 · `useful` 1): the worker's **current
  UK GDAD PCF role profile's requirements** (role), the person's **own skill
  target** (target), and their **skill aspirations** (aspiration). Needs merge
  per skill at the highest level and strongest importance, keeping every
  source. (ESCO roles carry no requirements, so give none.)
- A need is graded `met` / `below` / `undeclared` against the **declared**
  proficiency. **Priority = importance weight × levels short.** `undeclared`
  is *unknown* (WPM-D32): no shortfall, no priority, ranked after the real
  gaps — a prompt to assess.
- `GET /api/workers/{pid}/skill-gaps` — one person, ranked. **Aspirations are
  included only for the person themself or HR** (the caller who may write the
  record); the payload says so (`includes_aspirations`).
- `GET /api/workforce-intelligence/skill-gaps?department=&limit=` — per skill
  over employed workers in the caller's organizations: needed by, met, below,
  undeclared, levels short, critical-below, score, departments. **Counts only;
  nobody is named; aspirations never included.**
- UI: "My skill gaps" on `/me` and `/workers/{pid}`; `/skill-gaps`.

## WPM-R44 — Training time recommendations

*As a person I can see which courses would close my gaps, how many hours that
takes, and when I would finish at a pace I choose.*

- For each skill the person is **below**, in priority order: catalogue courses
  they have **not completed**, cheapest hours-per-level first until the gap is
  covered; for any levels the courses leave, the skill's own `hours_per_level`
  (or the service default, **30**). Each recommendation says what it **rests
  on** — `courses`, `mixed`, `estimate` (WPM-D33).
- **A skill the person has not declared gets no hours** — it is listed under
  `assess_first`: unknown needs assessing, not training.
- **Schedule:** items run one after another in whole weeks at `weekly_hours`
  (1–40; default 4 scaled by FTE, at least 1) from a start day (default today).
- `GET /api/workers/{pid}/training-plan?weekly_hours=&start=`;
  `GET /api/workforce-intelligence/training-demand?department=` (hours and
  counts only, no one named, no aspirations).
- **Catalogue:** `GET|POST /api/skills/{pid}/courses` (course reference,
  title, hours 1–1000, levels added 1–4), `DELETE /api/skill-courses/{pid}`,
  `PUT /api/skills/{pid}/training-hours` (1–500; null = default). Migration
  `skills.hours_per_level`, `skill_courses`.
- UI: a training plan under "My skill gaps"; workforce training time and a
  catalogue editor on `/skill-gaps`.

## WPM-R138 — Links to the learning management system

The LMS owns courses and learners (WPM-D10); WPM holds a course only as a `course:` or `courseinstance:` URN. So that a person can
get from WPM to the course, every place that names a course also carries a link to it.

- **One setting, the deployer's:** `WPM_LMS_COURSE_URL`, a template such as `https://lms.example.org/course/{course}`. Unset means no
  links, which is the default; a template that is not valid (WPM-D91) is ignored with a warning at boot, never half-used.
- **Where a link appears** (`lms_url`, `null` when there is no template): a CPD entry that came from the LMS, a worker's training
  enrolment, a learning-path step, a skill's catalogue course, a cutover role's course. Nowhere else.
- **A CPD entry remembers its course:** migration `000075` adds `cpd_entries.course_ref` (set by the completion sync; filled from the
  entry's text for those already there), because a link from a sentence of text would be a guess.
- **A link is not a disclosure:** it carries the course id only, never the worker, and it is built from data the caller could
  already read in the same response.

## Design decisions

- **WPM-D91 — A link is built from the deployer's template, never from a caller's text.** Following a link out of WPM is a trust
  decision, so the destination is fixed by whoever runs the service: `https` only, the placeholder once, a real host with no user-info,
  and the course id percent-encoded so it cannot add a path, a query or a second host. A caller supplies only the URN, which was already
  validated as one. Rejected: an `lms_url` field on each record (a caller could point it anywhere); a link built in the browser (the template
  would have to ship to every client, and the rule would be written twice).

- **WPM-D32 — Unknown is not a number, and aggregates name no one.** Extends
  WPM-D16: an undeclared skill never becomes a zero or a shortfall;
  workforce views are counts and sums; private aspirations are used only in
  the owner's own view.
- **WPM-D33 — A recommendation states its basis.** Hours are planning
  estimates, not promises: catalogue hours where a course exists, a per-level
  figure otherwise, and the payload says which.
