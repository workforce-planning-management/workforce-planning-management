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

## WPM-R139 — CPD requirements can be corrected, and progress says whether the pace is enough

- **Correct a requirement:** `PUT /api/cpd-requirements/{pid}` changes its name, amount, period or job title, and
  `DELETE /api/cpd-requirements/{pid}` withdraws it (soft delete, audited). The unit cannot change once defined, because entries
  were counted in it. Entries are never touched: they belong to the worker and are counted afresh against whatever the requirement now says.
- **Pace:** each requirement in a worker's `cpd-progress` carries `pace`: `status` (`met`, `not_started`, `on_track`, `behind`,
  `missed`), `days_left`, `needed_per_30_days` (what the remaining amount comes to for each 30 calendar days left, rounded up; absent once the
  period is over or the requirement is met) and `basis`, which says what the status rests on (WPM-D92).
- **It names no one beyond the worker's own page:** the overview (HR's aggregate) is unchanged; pace is shown where the worker's progress
  already is, to whoever may already read it.

## Design decisions

- **WPM-D92 — Pace is a straight line, and says so.** `behind` means the recorded amount is less than the share of the requirement that the
  elapsed share of the period would give (`required × days elapsed ÷ days in the period`), counting both ends of the period. Real CPD is
  lumpy (a conference is two days of hours at once), so `behind` is a prompt to look, not a finding about the person; the payload
  carries the basis and the numbers so nobody has to guess. Verified hours are not used: pace is about what has been done, and
  verification is about evidence. Rejected: a score or a rank across workers (WPM-D32: roll-ups name no one); a prediction of the final total.

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

## Verifiable credentials and a skills crosswalk (WPM-R157, WPM-R158; spec round WPM-T268, built WPM-T284)

### WPM-R157 — Credentials out

- **What.** A completed CPD record or a demonstrated skill can be **issued as a signed credential** in the Open Badges 3.0 format
  (1EdTech; final in May 2024, expressed as a W3C Verifiable Credential in JSON-LD), carrying its ESCO or SFIA code where the skill has
  one. WPM-T284 pins the Open Badges and Verifiable Credentials Data Model revisions it builds to and says which.
- **The worker holds it.** WPM keeps the **issuance record only** (who, which credential kind, when issued, when revoked), not the
  credential's content beyond what is needed to show it was issued, and a **revocation list** the verifier can read. A credential is
  issued only on the worker's request and only for a record that exists and is verified (as CPD entries already are, WPM-T52–T55).
- **Not done.** WPM is not a wallet and does not verify credentials from other issuers. SFIA codes are used as identifiers only
  ([sfia.md](sfia.md), WPM-D69).

### WPM-R158 — A crosswalk between skills frameworks

- **What.** Mappings between ESCO, SFIA and other frameworks so a skill declared under one counts in gap analysis under another
  (`skill_mappings`: from, to, basis, confidence). A mapping is a **statement with a stated basis** (an authority's published
  crosswalk, or a person's judgement, with who and when) and a confidence (exact, close, broader, narrower); gap analysis counts a
  mapped skill with its confidence shown.
- **Unknown stays unknown.** A skill with no mapping is `unknown` under the other framework, not "not held"; the view says how many
  skills were unmapped (WPM-D32). Licensed taxonomies are handled as SFIA is: identifiers and a deployer's own licensed import, no
  shipped text. Delivery: WPM-T284.

### Built (WPM-T284)

Migration `m20261024_000090_credentials_and_crosswalk` (`credential_issuances`, `skill_mappings`), `rules::credential` and
`rules::crosswalk` (pure), `controllers/credentials.rs`, `controllers/skill_mappings.rs`, and gap analysis reading the mappings.
`ed25519-dalek` and `base64` moved from dev-dependencies to dependencies (both were already in the build).

**Credentials (WPM-R157).** Pinned on 2026-10-25: Open Badges 3.0 context `context-3.0.3.json` (the file was read: it defines
`OpenBadgeCredential`, `AchievementSubject`, `Achievement` and `Alignment`), the Verifiable Credentials Data Model 2.0 context, the
**Bitstring Status List v1.0** (W3C Recommendation, 15 May 2025: 131 072 bits minimum, left-most bit first, GZIP, multibase `u`
base64url), and the credential secured as a compact **JWS with EdDSA** (`typ` `vc+jwt`, RFC 7515 and RFC 8037; the RFC 8037 example is
a unit test).
- `POST /api/me/credentials {cpd_entry_pid}` issues from the caller's **own, verified** CPD entry; once while one is live; returned
  once and **not stored**. It carries the activity, the category, when it was verified, the issuer and its place on the revocation list
  and **no name, number, email or identifier of the worker**. `GET /api/me/credentials` lists the issuances (when, place, revoked),
  `POST /api/me/credentials/{pid}/revoke` withdraws one, and HR or an administrator can `POST /api/credentials/{pid}/revoke`.
- The issuance record is: worker, kind, source entry, when, when revoked and by whom, and the SHA-256 of the signed credential. A
  place on the list is the row's id less one (`status_index`), so it is never reused; at 131 072 issuances a new list is needed (issuing
  refuses).
- `GET /api/credentials/issuer` (the public key as a JWK, the issuer, the list's address) and `GET /api/credentials/status-list` (the
  list as a signed credential) are **public**: see WPM-D109.
- Subject access lists the issuances. Erasure revokes a live credential and cuts the row from the person (worker and entry set to the
  nil id), keeping its place on the list.
- Settings: `WPM_CREDENTIAL_ISSUER`, `WPM_CREDENTIAL_SIGNING_KEY`, `WPM_CREDENTIAL_ISSUER_NAME` (deployment.md).
- **Skill alignment (WPM-T285, 2026-10-25).** A CPD entry may name the skill it developed (`skill_pid` on `POST
  /api/workers/{pid}/cpd-entries`, checked against the catalogue; migration `m20261025_000091_cpd_entry_skill`; optional). A
  credential issued from such an entry carries an Open Badges `alignment` for **every external reference the catalogue holds for that
  skill**, ESCO first (its URI is also given as `targetUrl`), then SFIA (a code, so no link), then any other; the name is the
  reference's label, else the skill's name. An entry with no skill has no `alignment`: none is guessed. SFIA codes are carried as
  identifiers only, as elsewhere (WPM-D69). A skill that is only **self-declared** (a `worker_skills` row) is deliberately not
  issued as a credential: nobody has verified it, and an employer's credential is a statement that someone checked.
- **Not built:** a credential from a verified skill assessment (there is none to issue from); the
  Data Integrity proof form (`eddsa-rdfc-2022`, which needs RDF canonicalization; JWT is a form Open Badges 3.0 allows); a key
  rotation (one `kid`, `#key-1`); a separate key per organization; a wallet or a presentation; verifying credentials from other issuers
  (by design); a real verifier tried (the unit and enforcement tests verify with the published key; **no third-party OB or VC verifier
  was run against it**); a screen.

**Crosswalk (WPM-R158).** `GET|POST /api/skill-mappings`, `DELETE /api/skill-mappings/{pid}`: a mapping runs from the skill a person
declares to the skill that is needed, with a confidence (`exact`, `close`, `broader`, `narrower`), a basis (`authority` or
`judgement`), a **source** that says which published crosswalk or whose judgement (required), and who stated it and when. Any signed-in
caller reads; HR writes; every write is audited. Mappings are between WPM skill records (which carry their ESCO or SFIA references),
so no licensed text is shipped.
- **How gap analysis uses it** (`GET /api/workers/{pid}/skill-gaps`): a need the person has not declared is graded through an `exact`
  or `close` mapping from a skill they did declare, and says so (`through_mapping`: the declared skill and the confidence). `broader`
  and `narrower` are listed beside it (`related_not_counted`) and never counted. A direct declaration is never overridden. Of several
  mappings the most confident wins, then the highest declared level. A need no mapping reaches stays **undeclared (unknown)**, and the
  view counts them (`unmapped_undeclared`).
- **The workforce roll-up reads it too** (WPM-T298): `GET /api/workforce-intelligence/skill-gaps` and the training-hours view grade in SQL
  (`controllers/gap_sql.rs`) with the same rule: a need the person did not declare is graded through the best `exact` or `close`
  mapping from a skill they did declare (the most confident, then the highest declared level); a direct declaration is never
  overridden; `broader` and `narrower` are never counted. A request test grades the same people both ways, including a close
  mapping outranked by an exact one at a lower level and a broader one counted by nobody.
- **Not built:** the roll-up does not say how many of its figures went through a mapping (the per-person view does); a mapping
  import; transitive mappings (A to B to C is not followed). The screen is a panel on `/skills` (WPM-T288).

### Decision

- **WPM-D109 — A verifier needs no account, and learns nothing about a person.** The issuer's public key and the revocation list are
  the only unauthenticated routes added for this (`is_public_path`). They hold keys and bit positions: no worker, no entry, no
  credential. A credential names no person, so a verifier learns that an employer vouches for an achievement, not for whom. The signing
  key lives outside the database. A mapping counts only when it is `exact` or `close`, and a skill no mapping reaches is unknown, never
  "not held" (WPM-D32).
