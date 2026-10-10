# ESCO — European Skills, Competences, Qualifications and Occupations

ESCO is the European Commission's multilingual classification of
occupations, skills, and qualifications: a shared dictionary for the
European labour market and for education and training. This topic records
what ESCO is, how it can be reached and reused, and how WPM should relate to
it. It is a reference, not a dependency: WPM works without ESCO, and nothing
here is yet implemented.

Researched 2026-10-02 against the ESCO portal, its API documentation, and
its copyright notice (sources at the end). Figures are those the portal
published on that date and will change with each release.

## What it is

- **Owner:** the European Commission, Directorate-General for Employment,
  Social Affairs and Inclusion (DG EMPL), which develops and runs it.
- **Cost:** free to consult and to download; reuse terms below.
- **Languages:** 28 — all official EU languages plus Icelandic, Norwegian,
  Ukrainian, and Arabic.
- **Version:** v1.2.1, last updated 2025-12-10. The first full version
  (v1) launched 2017-07-28. Older versions (v1.0.3 onward) stay
  downloadable, so an integration should **pin a version** rather than
  follow "latest".
- **Identifiers:** every concept has a persistent URI, for example
  `http://data.europa.eu/esco/occupation/528f90ed-e250-48bd-aacc-ffb7b1de5654`
  (an occupation) or `http://data.europa.eu/esco/skill/<uuid>` (a skill).
  The URI is the join key; labels vary by language and version.

## The three pillars

| Pillar | What it holds | Size (v1.2.x) |
| --- | --- | --- |
| **Occupations** | Occupation profiles, each with one preferred term plus alternative and hidden terms per language, a description, scope notes, regulatory-aspect information, and the knowledge, skills, and competences that experts consider relevant | 3,039 occupations |
| **Skills and competences** | Four sub-classifications: *knowledge*; *language skills and knowledge*; *skills*; *transversal skills* | about 13,900 concepts (the portal states 13,939 on one page and 13,890 on the skills page; treat as "about 13,900") |
| **Qualifications** | A bridge to national qualification databases and the European Qualifications Framework (EQF), linking formal learning outcomes | not published on the pages consulted |

### Occupations and ISCO-08

The occupations pillar is built on **ISCO-08**, the International Standard
Classification of Occupations. ISCO-08 provides the top four hierarchy
levels; ESCO occupations sit at level 5 and below. **Each ESCO occupation
maps to exactly one ISCO-08 code**, which makes ISCO-08 the natural
cross-walk to national statistics and to other systems.

### Skills

- A skill concept carries: description, formal definition, scope note,
  **skill type** (knowledge *or* skill/competence), **reusability level**,
  the occupations for which it is **essential**, the occupations for which
  it is **optional**, and its URI.
- ESCO does **not** distinguish "skills" from "competences" — both are one
  concept type.
- Reusability level distinguishes transversal skills (relevant across many
  occupations and sectors) from cross-sector and occupation-specific ones.
- **ESCO has no proficiency scale.** It says *which* skills an occupation
  needs and whether each is essential or optional; it does not say how
  well. Proficiency levels must come from elsewhere (see WPM below).

## Access

| Route | Notes |
| --- | --- |
| **Portal** | Browse and search at <https://esco.ec.europa.eu/> |
| **Downloads** | RDF, TTL, ODS, CSV, XML, and JSON-LD; filter by version, content type, language, and format; link delivered by email |
| **Web service API** | A web-based, machine-to-machine API over linked data with version selection (`selectedVersion`); documented at <https://ec.europa.eu/esco/api/doc/esco_api_doc.html>. A resource is fetched by URI, for example `GET /resource/skill?uri=<skill URI>&language=en`, with a bulk form taking repeated `uris` parameters. The ESCO team has announced API updates, so confirm the current documentation before building |

For WPM, **prefer a pinned CSV or JSON-LD download over live API calls**:
it keeps a request path free of third-party calls (the same posture as the
service's other upstream fetches — short timeout, no redirects, nothing
required at boot), and a catalogue pinned to a version is reproducible.

## Licence

The ESCO classification may be downloaded, used, reproduced, and reused
**for any purpose and by any party, free of charge**, under the Commission
Decision of 12 December 2011 on the reuse of Commission documents
(2011/833/EU), with the source acknowledged. Europa-owned content is
generally CC BY 4.0. **Obligation for WPM:** attribute ESCO wherever its
labels or URIs are republished, and record the ESCO version used. Re-read
the portal's copyright notice before shipping, since this summary is not
legal advice.

## How WPM should relate to it

WPM keeps its own skills catalogue (WPM-T20) and is adding role profiles
(WPM-R34, [strategic-workforce-planning.md](../strategic-workforce-planning.md)).
ESCO fits as an **optional external reference**, never a second skills model:

1. **Skill reference.** Add an optional external reference to a catalogue
   skill (framework id `esco`, concept URI, ESCO version). A skill without
   one is valid; the catalogue stays WPM's own. Matching is by URI, never
   by label.
2. **Occupation reference.** A role profile may carry an ESCO occupation URI
   and, through it, an ISCO-08 code. The mapping is declared by the
   planner, not inferred — an auto-matched occupation is a guess about a
   person's job.
3. **Seeding role profiles.** An occupation's *essential* skills are a
   sensible first draft of a role profile's required skills, and its
   *optional* skills a draft of `useful` importance (WPM-R34's
   `critical` / `important` / `useful`). It is a draft for a human to edit:
   ESCO says a skill is relevant, not that a given employer needs it.
4. **Proficiency stays WPM's.** Because ESCO has none, every required-skill
   level in a role profile is set by the planner on WPM's own 1–5 scale
   (or imported from a framework that has levels, such as
   [uk-gdad-pcf](../uk-gdad-pcf/index.md)).
5. **Qualifications and EQF.** Out of scope until a concrete need appears
   (for example, regulated professions); the qualifications pillar's size
   and structure were not confirmed in this research.
6. **Multilingual labels.** ESCO's 28-language labels could back the
   locale pickers, but only as display text next to the URI — the same
   best-effort rule as upstream display names (WPM-T3): never copied as
   truth, never required.

## Implementation in WPM (WPM-T60)

ESCO is a **pinned local reference copy plus links**, never a second skills
model. Decided from the open questions below: a *table* (not a file read at
request time), because seeding and search need the occupation–skill relations
queryable.

```sh
cargo loco task import_esco dir:/path/to/esco-csv [lang:en] [version:v1.2.1]
```

- **Input:** the ESCO classification download in CSV — `skills_<lang>.csv`,
  `occupations_<lang>.csv`, `occupationSkillRelations_<lang>.csv`. Columns are
  resolved **by header name** (`conceptUri`, `preferredLabel`, `skillType`,
  `reuseLevel`, `iscoGroup`, `description`, `occupationUri`, `relationType`,
  `skillUri`), so a different column order between versions does not matter.
  A small RFC 4180 parser handles ESCO's quoted commas and newlines.
- **Replaced wholesale** on every import, so the local copy is exactly one
  ESCO version; the version is recorded on the framework row beside the
  attribution (*© European Union … Decision 2011/833/EU*).
- **Linking never guesses.** A catalogue skill is linked to an ESCO skill
  only when exactly one ESCO skill has the same *normalised* label (trimmed,
  lower-cased, whitespace collapsed — never fuzzy), the ESCO skill is not
  already linked elsewhere, and the skill has no ESCO reference yet.
  Ambiguous labels are counted and left for a planner.
- **API:** `GET /api/esco/occupations?q=` and `/esco/skills?q=` (literal,
  case-insensitive search, at least 2 characters, at most 100 results),
  `GET /api/esco/occupation?uri=` (essential then optional skills, each with
  its draft category and catalogue link), and `POST /api/skills/{pid}/refs`
  for a manual link — a URI must name a real ESCO skill in the pinned copy.
- **Seeding a role profile:** `POST /api/role-profiles/from-esco`
  `{occupation_uri, default_min_proficiency, include_optional?, job_title?}`.
  **ESCO has no proficiency scale, so the planner supplies the starting
  level** (1–5) — never invented. Essential skills draft importance
  `important`, optional ones `useful` (never `critical`: ESCO does not say any
  skill is). Skills are matched by ESCO reference, then exact label, else
  created with a **draft category** (knowledge → `domain`; transversal →
  `other`; otherwise `technical`) and linked. One profile per ESCO occupation;
  the ISCO-08 code is kept in the profile's profession line. The result is a
  draft for a human to edit.
- **Front-end:** `/roles` "Start from an ESCO occupation" (search, preview the
  skills, choose the level, create) and `/skills` "Link ESCO" per skill.

**Verified against the real ESCO v1.2.1 (2026-10-03).** The emailed download
could not be obtained here, so `workforce-planning-management-service-with-rust/scripts/esco-fetch.py`
builds the same three CSVs from the public web-service API — it crawls the
occupation taxonomy (ISCO groups → occupations, closing over nested
occupations) and the skills hierarchy, bulk-fetches skills, and translates the
API's vocabulary into the CSV's ("skill" → "skill/competence", "sector specific
skills and competences" → "sector-specific"). It makes about 6,000 polite
requests (a few threads, cached so it resumes) and uses `curl`, because this
Python build could not complete the server's certificate chain. Real result:

| | Loaded | Published by ESCO |
|---|---:|---:|
| Occupations | **3,039** | 3,039 ✓ |
| Skills | 13,653 | 13,939 (what-is page) / 13,890 (skills page) — **not reconciled**; the skills hierarchy reaches 13,653 skills plus 640 skill groups |
| Occupation–skill relations | 126,051 (0 dangling) | — |

Import takes a few seconds; search answers in milliseconds. Seeding the real
*software developer* occupation (ISCO 2512; 24 essential, 84 optional skills)
took 0.37 s and drafted 108 requirements (24 `important`, 84 `useful`).

What the real data showed:

- **Only 2 of the 161 PCF skills link by exact label** (`Data engineering`,
  `Financial management`): the PCF's names ("Information security") and ESCO's
  labels differ, and exact matching is deliberately all that is done. Linking
  the rest is a planner decision (`/skills` → *Link ESCO*).
- **The draft category mapping is crude.** It makes every ESCO *knowledge*
  skill `domain`: 79 of the 108 skills drafted for *software developer* —
  including programming languages, which are really technical. ESCO's skills
  hierarchy (not in the CSV) would classify them far better; until that is
  used, treat the category as a draft and correct it in `/skills`.
- **Nested occupations** (1,071 narrower occupations of other occupations) are
  easy to miss and carry no ISCO link of their own; the script takes the ISCO
  unit group from their ESCO code (`2512.4.1` → `2512`).
- A search or crawl that stops at the API's 200-result paging cap silently
  returns a fraction of ESCO; the loaded counts above are the check.
- **Starting the app in the `test` environment recreates the database** on boot
  (`dangerously_recreate`); load reference data into a development database.

The CSVs here are API-derived, not the official package, so confirm against
the official download if exact parity matters (e.g. `altLabels`, which the
importer ignores). Qualifications (EQF) remain out of scope.

### Open decisions

- Which languages to import (all 28 is large; start with the locales the UI
  already ships).
- Whether to store ESCO data in a WPM table or resolve it from a pinned
  file at read time. A file is simpler and avoids owning a copy of a
  classification that changes each release.
- How to treat a skill whose ESCO concept is deprecated in a later version
  (keep the pinned URI and flag it, rather than silently re-pointing).

## Sources

- ESCO, "What is ESCO" — <https://esco.ec.europa.eu/en/about-esco/what-esco>
- ESCO, download — <https://esco.ec.europa.eu/en/use-esco/download>
- ESCO, occupations pillar — <https://esco.ec.europa.eu/en/classification/occupation_main>
- ESCO, skills pillar (ESCOpedia) — <https://esco.ec.europa.eu/en/about-esco/escopedia/escopedia/skills-pillar>
- ESCO, web service API — <https://esco.ec.europa.eu/en/use-esco/use-esco-services-api/esco-web-service-api> and
  <https://ec.europa.eu/esco/api/doc/esco_api_doc.html>
- ESCO, copyright notice — <https://esco.ec.europa.eu/en/copyright-notice-esco-skills-competences>
- European Commission, DG EMPL — <https://employment-social-affairs.ec.europa.eu/policies-and-activities/skills-and-qualifications/skills-jobs/european-skillscompetences-qualifications-and-occupations-esco_en>
