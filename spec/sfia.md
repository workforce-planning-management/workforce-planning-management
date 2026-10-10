# SFIA and SFIAplus (WPM-R109–R113)

Status: **research and proposal, 2026-10-10.** Nothing here is built. Where a statement comes
from my own knowledge rather than a source I read, it is marked *(unverified)*.

> ⚠️ **Not legal advice.** SFIA and SFIAplus are licensed content. Read the **current** terms of
> the SFIA Foundation and of BCS before importing or showing any of it. This note records what the
> sources said on 2026-10-10; it can be out of date.

## 1. What they are

**SFIA** (Skills Framework for the Information Age) is a global, technology-neutral reference model
of the skills and levels of responsibility in digital, technology and related business roles,
maintained by the SFIA Foundation ([SFIA](https://sfia-online.org/en/about-sfia)). Its structure,
as the maintainer described it:

- **Seven levels of responsibility**, from entry (follow) to strategy (set strategy, inspire,
  mobilize). SFIA defines a level by increasing *responsibility*, not just depth of expertise.
- **Five generic attributes** that describe each level: autonomy, influence, complexity, knowledge,
  and business skills.
- **A set of professional skills**, each with a code, a name, a category, a description, and a
  description **at each level at which the skill exists** (a skill is not defined at every level).
  Version 9 has 147 skills (the maintainer's figure, as supplied; version 8 had fewer).
- **Uses**: individuals assess skills and plan a career; employers write role profiles, manage
  performance and plan capability; educators align curricula.

**SFIAplus** is BCS (The Chartered Institute for IT)'s extension of SFIA. It adds, for each SFIA
skill at each level, practical detail: typical background, the work done, supporting knowledge and
skills, suggested training, and relevant certifications and professional memberships; BCS
qualifications and professional standards are aligned to it
([BCS SFIAplus](https://www.bcs.org/develop-your-people/develop-your-team-or-organisation/sfiaplus-it-skills-framework/)).
Full access is for BCS members through its browsing tool; non-members get a wall chart. The counts
BCS publishes differ between pages and versions, so WPM takes any count from the import, never from
this note.

## 2. The licence decides the design

From the SFIA Foundation's licensing pages
([choosing a licence](https://sfia-online.org/en/licensing-sfia/choosing-a-licence);
[corporate user licence](https://sfia-online.org/en/licensing-sfia/corporate-user-licence)), as
summarized by the search (the 2021 general terms were the latest text found; check the current one):

- Most individuals and organizations use SFIA under a **free-of-charge licence**. A free **corporate
  user licence** covers **internal use** as a management resource by a single organization.
- Large organizations are expected to take an **extended corporate user licence** (a modest annual
  fee); **commercial use** (consulting, products) needs a **partner licence**.
- The free corporate licence **excludes distribution or publication of SFIA material in any form**,
  even to other organizations in the same group or to other government departments; it excludes
  use of the SFIA trademark and mapping training or certification offerings for external use.
- **Downloads need registration** (Excel, PDF and RDF were listed). I found no JSON download.

BCS's SFIAplus is a separate product with its own terms, mostly for BCS members.

**Therefore (WPM-D69): WPM ships no SFIA or SFIAplus content.** This repository is public, so
including the framework would be distribution. WPM provides an **importer** that reads a file the
**deployer** downloaded under **their own licence**, stores it in **their own database**, and
shows it only to **their own signed-in users**. The repository carries only a tiny **synthetic
fixture of invented text**, labelled as not SFIA, for tests. This is the opposite of the existing
frameworks, which are open data shipped or fetched openly (UK GDAD PCF under the Open Government
Licence; ESCO).

## 3. How it fits what WPM already has

| WPM already has | For SFIA |
| --- | --- |
| `capability_frameworks` (slug, name, source, licence, attribution, scale max and labels, note) | One row, scale max **7**, plus the new licence fields below |
| `role_profiles` and `role_skill_requirements` with the framework's own level kept (`source_level`, `source_scale_max`) | SFIA role profiles and required skills at SFIA levels |
| `skills` and `skill_external_refs` | The SFIA code attached to a WPM skill, so one skill can be known in SFIA, ESCO and UK GDAD PCF at once |
| `rules::framework::map_level` (`identity` or `linear`) | `linear`: 1, 2, 2, 3, 4, 4, 5 for SFIA levels 1 to 7, with the source level always kept |
| Job levels (a published ladder, with a level on a worker and a role) | SFIA's seven levels of responsibility as a job-level framework, **imported**, not coded |
| `import_framework` and `import_esco` tasks | `import_sfia` and `import_sfiaplus` |
| Training recommendations, skill courses, development plans, CPD | SFIAplus training, certification and membership suggestions, as pointers |

## 4. Proposed requirements

- **WPM-R109 Import SFIA from the deployer's licensed download.** A task `import_sfia
  file:<path>` reads a documented **CSV contract** (a deployer saves the licensed Excel download as
  CSV; the contract names each column: skill code, name, category, subcategory, description, level,
  level description, and the level-of-responsibility attributes). Idempotent; records the SFIA
  **version** (8 or 9); a skill or level removed in a newer version is kept and marked retired, not
  deleted, so declared skills do not vanish; refuses a file whose columns do not match, and never
  invents a level or text. Reading Excel or RDF directly is a later option.
- **WPM-R110 Levels of responsibility as a job-level framework.** The seven levels, with the five
  generic attributes and each level's description, as an **imported** job-level framework (job
  levels today are coded reference data; this needs the same shape in the database). A worker's and
  a role's job level can be an SFIA level, separate from their skill levels.
- **WPM-R111 SFIA skills in role profiles and declared skills, native level kept.** A role profile
  can require SFIA skills at SFIA levels, and a worker can declare an SFIA skill at an SFIA level.
  The level is stored on the **framework's own scale** and shown on it, with its description;
  comparisons use the documented `linear` mapping onto WPM's 1 to 5 (WPM-D70), which the interface
  shows. A role can be started from SFIA skills in the way ESCO occupations seed a role today.
- **WPM-R112 SFIAplus development resources (optional).** A second import, `import_sfiaplus
  file:<path>`, attaches to an SFIA skill at a level the SFIAplus detail the deployer is licensed
  for: typical tasks, suggested training, certifications, memberships. They appear as **pointers**
  in training recommendations, development plans and the skill-gap view, with the source named. WPM
  never presents them as its own, and nothing is bought or booked from them. The deployer's BCS
  licence is recorded. The import format is to be agreed with the licence holder; WPM defines a
  CSV contract the same way as for R109.
- **WPM-R113 The licence guard and record.** Each framework records its **version**, its
  **redistribution** status (`open`, `internal_only`, `unknown`), the licence **holder**, a
  reference, the **date the licence was last checked**, and the required **attribution**; SFIA and
  SFIAplus are `internal_only`. A pure rule, `publishable(framework)`, is true only for `open`;
  every surface that leaves the signed-in organization (public job posts, the sitemap, unauthenticated
  routes, anything sent to a third party) uses it and never includes imported wording of a framework
  that is not publishable. The interface shows the attribution and no SFIA trademark or logo. A
  re-import warns when the recorded licence check is older than a configured age.

## 5. Decisions

- **WPM-D69 Licensed content is imported by the licensee, never shipped.** The repository holds
  code, a contract and an invented fixture; the deployer holds the content and the licence.
- **WPM-D70 The framework's own scale is kept, and the mapping is shown.** SFIA's seven levels
  are stored and displayed as SFIA writes them. WPM's 1 to 5 is derived by a stated rule for
  comparison, never the other way round, and the interface says which rule it used. (This repeats
  the decision already made for the UK GDAD PCF's four levels.)

## 6. What to watch

- **Translation.** Do not machine-translate SFIA text into the interface languages: translation
  was a restricted activity under the partner licence, and the free licence forbids distribution.
  Framework text is shown as imported; SFIA's own translations are a licence matter for the
  deployer.
- **Subject access.** A worker's own declared skills include SFIA names and levels. Giving a person
  their own record is not distribution to another organization, but the deployer should confirm it
  against their licence.
- **Exports and reports** that include framework wording leave the screen: apply the guard to them
  too, and keep to codes and names where wording is not needed.
- **Equality.** A level of responsibility or a skill level is a score. The equality impact
  assessment for it ([equality-impact/template.md](equality-impact/template.md)) applies like any
  other (WPM-R99).
- **Not modelled**: SFIA's own accreditation or assessment services, and any SFIA membership data.
- **Open question**: how much of the SFIA content a role-profile builder may copy into a role
  (wording) versus reference (code and level). Copy is internal use; it still must not reach a
  public surface.

## Sources

- [SFIA: about](https://sfia-online.org/en/about-sfia), [documentation](https://sfia-online.org/en/sfia-9/documentation)
- [SFIA licensing: choosing a licence](https://sfia-online.org/en/licensing-sfia/choosing-a-licence) and [corporate user licence](https://sfia-online.org/en/licensing-sfia/corporate-user-licence)
- [BCS: SFIAplus](https://www.bcs.org/develop-your-people/develop-your-team-or-organisation/sfiaplus-it-skills-framework/)
- The maintainer's summary of SFIA (seven levels, five attributes, 147 skills in version 9, SFIAplus from BCS), supplied 2026-10-10.
