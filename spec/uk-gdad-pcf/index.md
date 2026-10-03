# UK GDAD PCF — Government Digital and Data Profession Capability Framework

The UK Government Digital and Data (GDAD) Profession Capability Framework
(PCF) is the framework UK government uses to describe digital and data jobs:
which roles exist, what each level of a role is accountable for, and which
skills it needs. This topic records what it contains, what the community
repository at `~/git/uk-gdad` adds to it, and how WPM can use it. It is a
reference and a candidate import source; nothing here is implemented yet.

Researched 2026-10-02 from the local clone `~/git/uk-gdad/uk-gdad`
(specification, manual, licence, and a sample of its documents). The
repository's own figures were current at its own last update; the
authoritative source is the framework itself,
<https://ddat-capability-framework.service.gov.uk/>.

> The repository is a community project, **not a government service**, and
> everything in it beyond the role summaries is AI-assisted with human review
> and says so. WPM must carry the same caution: nothing derived from it is a
> validated instrument, and none of it should be the sole basis for pay,
> grading, recruitment, or promotion.

## The framework's shape

Four nouns, in this order:

| Term | Meaning | Count |
| --- | --- | ---: |
| **Profession** | Broadest grouping, e.g. `software-development` | 8 |
| **Role** | A job in a profession, e.g. `software-developer` | 52 |
| **Role level** | A seniority step in a role, e.g. `4-senior-developer` | 205 (201 in use, 4 retired) |
| **Skill** | A named capability a level requires | 183 used (the framework's catalogue lists 185) |

The eight professions: architecture; chief digital and data; data; IT
operations; product and delivery; quality assurance and testing; software
development; user-centred design.

- A **role level** is identified by a **slug** — `<profession>/<role>/<number>-<level>`
  — and the number is a display order, not a grade. The four
  chief-digital-and-data roles have no levels and use `<profession>/<role>`.
  Where a level has both a technical and a management track, the management
  one is suffixed `-management`. The four retired levels are marked
  `Role level: NOT IN USE`.
- A **skill name is the join key** between levels. Two levels naming the same
  skill describe the same capability at different depths. A qualified name
  such as `User focus (content design)` is a **different** skill from
  `User focus`: match exactly, never by prefix.
- **Role summaries are line-oriented plain text**, one file per level:
  `<Profession> role: <Role>`, optional duties, `Role level: <Level>`,
  optional accountabilities, then one or more `Skill: <name>` blocks, each a
  bulleted description of the skill *at that level*. A skill may appear more
  than once in one file; consumers merge the bullets.

### The proficiency scale

The framework's own scale has **four** points (the repository's competency
assessments use it unchanged):

| Value | Rating | Meaning |
| ---: | --- | --- |
| 1 | Awareness | Can describe skill fundamentals and show basic knowledge of related tools |
| 2 | Working | Can apply the skill with some support and use appropriate tools |
| 3 | Practitioner | Can apply the skill independently, determine the best tools, and share experience |
| 4 | Expert | Can lead organisational best practice and teach advanced techniques |

It is a **proficiency** scale, not a frequency scale. WPM's declared
proficiency scale is **1–5** (`rules::learning::valid_proficiency`), so the
two are not interchangeable — see "Mapping to WPM".

## What the community repository adds

Per role level, besides the canonical summary, eight derived documents
(nine in total, 1,846 documents across 205 levels, published as a static
site at <https://uk-gdad.github.io>):

| Document | Use | Relevance to WPM |
| --- | --- | --- |
| Role summary (**canonical**) | What the level requires | Source for role profiles (WPM-R34) |
| Start here | Orientation and a learning pathway | Onboarding content |
| Upskilling resources | Courses, posts, research, videos, books | Candidate source for development-plan items |
| **CPD checklist** | A per-level, skill-by-skill checklist of development activities | The seed vocabulary for the CPD ledger |
| Psychometric assessment (by assessor / by individual) | Reasoning-and-judgement practice and administration | Out of scope (see caution below) |
| Competency assessment (by assessor / by individual) | A skill-by-skill matrix rated on the 1–4 scale | A model for skill-rating forms |
| **Skills gap form** | A form HR sends and an employee fills in, per level | A model for the gap-analysis input |

Every derived document states the AI-assisted advisory. The repository
requires a document to be produced from the canonical summary, never from
another derived one, and enforces its contracts with an executable check.

## Licence

- **Role summaries:** © Crown copyright, adapted from the framework, under
  the **Open Government Licence v3.0** (SPDX `OGL-UK-3.0`).
  **Attribution is required wherever they are republished**, including in a
  quote on a slide.
- **Everything else in the repository** (resources, checklists,
  assessments, forms, specifications, scripts, site) is offered under the
  same OGL v3.0, so one licence covers the whole tree.
- Third-party courses, articles, and books are referenced by title and link
  only; nothing is reproduced.

**Obligation for WPM:** attribute the framework and the licence wherever
role summaries are imported, displayed, or republished, and keep the
repository's caution that it is not an official service.

## Mapping to WPM

| PCF | WPM | Notes |
| --- | --- | --- |
| Role level slug | Role profile (WPM-R34), keyed by job title | Import as a *source* with the slug retained for provenance; a WPM job title is the benchmark key, the PCF slug is a different identifier |
| `Skill:` name | Catalogue skill (WPM-T20) | Match by exact name; qualified names are distinct skills. 183 skills would seed the catalogue; categories need assigning |
| Skill description at a level | Role-profile requirement note | Keep verbatim, attributed |
| Level of a skill | Required minimum proficiency | **Scale mismatch** — below |
| CPD checklist items | CPD ledger activities (planned) | Items are `- [ ] Title: description` under `Role Level Focus` and per `Skill:` headings |
| Skills gap form | Gap-analysis input | A model, not a data source |

### The scale mismatch (decided: identity default, source kept)

The PCF's role summaries describe each skill's behaviour at a *level of the
role*, and its assessments rate on 1–4; WPM declares 1–5. Options:

1. **Record the source scale** on each imported requirement (`scale: pcf-4`)
   and convert only for comparison, with a stated mapping the planner can
   see and change. *Recommended*: it loses no information and never
   pretends two scales are one.
2. Widen WPM to a framework-neutral scale — too disruptive for the data
   already declared.
3. A fixed mapping (1→1, 2→2, 3→3, 4→5) — simple, but it manufactures a
   precision the source does not have, and "Expert" is not obviously "5".

Whichever is chosen, an imported requirement is a **draft for a human to
edit**, exactly as for ESCO seeding (see [../esco/index.md](../esco/index.md)).

### Cautions

- **Not validated instruments.** The repository says its assessments have
  not been piloted, normed, or checked for adverse impact. WPM already
  refuses to treat cognitive/psychometric results as ranking tools (see the
  assessment rules in [talent-development.md](../talent-development.md));
  importing the *assessment* documents would not be consistent with that.
  Import the **role summaries and CPD checklist vocabulary** only.
- **Drift.** The repository's own `spec/skills.md` records that its summaries
  name 183 of the framework's 185 skills, seven catalogue skills no level
  names, and five skill names not in the catalogue (a content-design
  vocabulary the framework has since reorganised). An importer must tolerate
  unmatched skills rather than fail.
- **Pin and attribute.** Record the commit or fetch date of the import; the
  framework changes upstream and the repository re-fetches periodically.

## Implementation in WPM (WPM-T57)

Role levels import as **role profiles** (`spec/strategic-workforce-planning.md`
WPM-R34), so every role-based feature — gap against a role, matching,
mobility, plans, change tracking — works on the framework's roles.

```sh
cargo loco task import_framework dir:/path/to/uk-gdad/uk-gdad \
    [scale:identity|linear] [overwrite_levels:true]
```

- **Reads** `uk-gdad-pcf-role-summaries/roles/**` (structure and wording;
  Crown copyright, OGL v3.0) and, for each skill's required level, the
  `Baseline: N — Rating` line in
  `uk-gdad-pcf-competency-assessments-by-assessor/roles/**`. Nothing from the
  clone is copied into this repository.
- **A skill with no baseline is skipped and counted** — never given an
  invented level. Retired (`NOT IN USE`) levels are left out.
- **Titles** are the level names exactly as the framework writes them
  (`Senior developer - management` keeps its own suffix); a name shared by
  two roles is qualified with the role. The management track is recognised
  from the level *name*, not the slug (`Head of IT service management` is
  not a track).
- **Skills** are matched to the catalogue by **exact name** (qualified names
  are distinct skills) and created with category `other` when new.
- **Requirements** default to importance `important` — the framework does not
  grade importance — and keep the framework's wording in `note` and the
  **source level and scale** beside WPM's 1–5 `min_proficiency`.
- **Level mapping — `scale:identity` (default):** PCF 1–4 become WPM 1–4
  (Practitioner = 3). **`scale:linear`** rescales so the ends meet (1, 2, 4,
  5), which leaves WPM 3 unused and makes a worker who declared 3 fall
  *below* a Practitioner requirement — stricter and surprising, hence not the
  default. The source level is always kept, so the choice is reversible:
  `overwrite_levels:true` re-derives every minimum from it.
- **Idempotent.** A re-import refreshes provenance, titles, wording and the
  source level, but never overwrites a planner's `min_proficiency` or
  `importance`, and never removes a requirement.
- **Attribution** lives on the `capability_frameworks` row and is shown on
  every imported profile in `/roles`: *Contains public sector information
  licensed under the Open Government Licence v3.0. © Crown copyright.*
  The framework row also notes that required levels come from the community
  assessments (AI-assisted, human-reviewed) — **a draft for a human to
  edit**, not an official statement.
- **API:** `GET /api/capability-frameworks` (attribution, licence, scale,
  counts), `GET /api/role-profiles?framework=`, and `GET
  /api/role-profiles/{pid}/progression` — what changes going up a level in the
  same role (skills added, raised, unchanged), the career-ladder view.

First real import (2026-10-03, from the local clone): 205 levels read, 4
retired skipped, **201 profiles**, **161 skills** created (the retired roles'
skills are not imported), **1,654 requirements**, 21 skill lines skipped for
lack of a baseline; a second run created nothing. The competency-assessment
files are the only source of numeric levels, so the import is only as good as
that AI-assisted data.

## Open decisions

- Import the PCF as role profiles directly, or only as a reference list a
  planner chooses from.
- Which ESCO occupation, if any, each PCF role maps to (ISCO-08 is the likely
  bridge; the mapping is manual — see [../esco/index.md](../esco/index.md)).
- Whether the CPD checklist items become suggested ledger activities
  (WPM's CPD ledger is not yet scoped; see
  [../strategic-workforce-planning.md](../strategic-workforce-planning.md)).

## Sources

- The framework — <https://ddat-capability-framework.service.gov.uk/>
- Community repository (local clone, researched 2026-10-02): `~/git/uk-gdad/uk-gdad` —
  `spec/index.md` (taxonomy, document contracts, licensing), `spec/skills.md`
  (catalogue drift), `LICENSE.md`, `uk-gdad-pcf-role-summaries/`,
  `uk-gdad-pcf-continuing-professional-development-checklists/`,
  `uk-gdad-pcf-competency-assessments-by-assessor/spec/index.md` (rating scale)
- Published site — <https://uk-gdad.github.io>
- Open Government Licence v3.0 — <https://www.nationalarchives.gov.uk/doc/open-government-licence/version/3/>
