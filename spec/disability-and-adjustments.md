# Disability, non-visible disability, neurodivergence and reasonable adjustments

Proposed 2026-10-27 ([plan.md](plan.md) section R; spec round WPM-T309, delivery Phase 24, WPM-T334–T337). Requirements
**WPM-R190–R196**, decisions **WPM-D125–D127**. Nothing here is built. It extends reasonable adjustments (WPM-R33,
[requirements.md](requirements.md)) and its decision WPM-D25 ([design.md](design.md)), which stay as they are.

WPM's starting point is already the right one: an adjustment request records **the barrier, its impact on the work and the change
that would reduce it**, in the person's own words, with **no condition field, no neurodiversity label and no medical-evidence
flag**, so nobody needs a diagnosis to ask and WPM has nowhere to put one (WPM-D25). What is missing is everything around the
request: adjustments that **follow the person** when they change role, manager or employer; a way to say **how I work best**
without saying why; adjustments that are **reviewed** rather than forgotten; absence rules that **do not penalise** disability;
adjustments for **candidates**; and **aggregate** disability equality measures that come only from voluntary monitoring.

**Non-visible disabilities** (long-term conditions, mental health, neurodivergence, chronic pain, sensory and cognitive
differences) are the case this design is shaped around: the person decides who knows anything, piece by piece, and WPM never
shows that a person has an adjustment, a passport or a preference anywhere they have not shared it (the directory never shows it,
the action list says only that something is due).

**Neurodivergence** (autism, ADHD, dyslexia, dyspraxia, dyscalculia, Tourette's and others) is never a field. The useful things,
written instructions, agendas in advance, a quieter space, clear priorities, predictable change, a different interview format,
are adjustments and preferences anyone may ask for, labelled by **what they change**, not by who needs them (WPM-D125).

## WPM-R190 — An adjustments passport the person owns

- **A passport** is the person's own living record of the adjustments that help them: for each, the barrier, the change and
  what works, in their words, with the agreed adjustments (WPM-R33) linked in. It is written by the person, not HR.
- **It follows the person** across roles, managers and organizations through the person-centric record (WPM-R167): it belongs to
  the `person_ref`, not to one worker row.
- **Sharing is the person's act** (WPM-D126): they share it, or chosen entries of it, with a named manager, HR or a new
  organization, and can withdraw a share. A share is dated and audited (who, when; not the words). A new manager learns only
  that a passport **may** be shared, and only if the person has set it to say so.
- **Agreeing it with a new manager.** When a shared entry is accepted by the new manager, the adjustment is agreed afresh in that
  role (a new WPM-R33 request in the agreed state, linked to the entry), so the person does not have to make the case again.
- Exported to the person; erased with the person.

## WPM-R191 — "Working with me"

- **A short profile of how a person works best**: how they like to receive information (`written`, `spoken`, `both`), meeting
  preferences (agenda in advance, camera optional, time to process before deciding), when they focus best, how they prefer
  feedback, and a few lines in their own words. Closed options are suggestions, not categories of people.
- **Visibility is the person's** (the aspiration pattern, WPM-T65): `private`, `manager` or `team`. Default `private`.
- **Anyone can have one**; it is offered to everyone, which is what keeps it from signalling anything. It names no condition and
  has no field for one.

## WPM-R192 — Reviewing adjustments, and keeping them through change

- **An agreed adjustment has a review date** (configured default, changeable per adjustment) and the review is recorded as
  `continue`, `change` or `end`, by the person with their manager. An adjustment is never ended by the date passing.
- **Through a movement** (transfer, change of role or manager, WPM-R45), the movement checklist carries an item "adjustments
  carried over or re-agreed", done or skipped, with no words; equipment issued as an adjustment (WPM-R155) moves with the
  person unless they say otherwise.
- **Through a change of work** (a new system, an office move, a restructure: a change initiative, WPM-T55), the change's
  impact view counts how many people in scope have in-place adjustments, **only from the floor and only to HR**, so the change
  can be assessed; it names no one and shows no words. This is the one aggregate over adjustments, and it exists because the
  duty to adjust applies to a change before it lands (WPM-D127 explains why it does not breach WPM-D25).

## WPM-R193 — Absence rules that adjust

- **An agreed adjustment of kind `absence_review_points`** (WPM-R181) may disregard named sickness episodes, or raise one
  person's threshold, with a review date like any adjustment.
- **It records the effect, never the reason** (WPM-D127): the episodes disregarded or the new threshold. Nothing says
  "disability"; the manager sees that a review point is not reached, and HR sees an adjustment is in place.
- **Disability leave** (planned time off for treatment, assessment or adjustment to a new condition) is a configurable leave
  kind like any other, recorded like sick leave (no reason, WPM-D73) and **not counted** toward review points.

## WPM-R194 — Adjustments for candidates

- **A candidate may ask for adjustments** to the selection process (format of an assessment or interview, extra time, questions
  in advance, a quiet room, a break, a different way to apply), at any stage, in their own words: the same three fields as
  WPM-R33.
- **Kept apart from scoring.** The request and its words are seen by the recruiter who arranges it and HR, **never by a
  scorer**, and never feed a score or a match (the same test as equality monitoring: other code naming the table fails).
  Assessors see only what they must do differently ("allow 25% extra time"), not why.
- **Retention follows the candidate**: deleted with the application under the candidate retention rule; carried into the
  person's passport only if they choose to when hired.
- **A guaranteed interview scheme** (disabled candidates who meet the essential criteria are offered an interview), where a
  deployer runs one, is a candidate's own opt-in answer, seen by the recruiter only, and reported only as counts with the floor.

## WPM-R195 — Government support schemes for adjustments

- **Where a jurisdiction funds adjustments** (in Great Britain, the Access to Work scheme), the person may record an
  application against an adjustment: status (`applied`, `assessed`, `awarded`, `declined`, `renewal_due`), dates and a
  reference. The award's amount and the assessor's report get no column.
- The person owns the record and may share it with HR (who often handles the invoices); the renewal date appears on the
  person's and, if shared, HR's action list.
- Configuration names the scheme and its link; none is shipped outside the demo's labelled example.

## WPM-R196 — Disability equality measures, from voluntary monitoring only

- **Computed only from equality monitoring** (WPM-R125, voluntary, off by default, deployer-defined categories, WPM-D74), never
  from adjustments, passports, absences or anything else (WPM-D125). A deployer that does not run monitoring gets `unknown`.
- **The measures WPM can compute**: representation by pay band and by organization level, the declaration rate (how many have
  answered at all, so a low figure is read for what it is), relative likelihood of appointment from shortlisting (when
  candidates are monitored), and the share of people who answered who report having asked for an adjustment through monitoring's
  own question, if the deployer adds one. Every cell has the floor; a measure without data is `unknown`.
- **National frameworks are mapped, not copied** (WPM-D100): in NHS England, the Workforce Disability Equality Standard (WDES)
  sets metrics an employer publishes; a deployer may map these measures to its metric numbers. The repo holds no metric text, and
  metrics needing data WPM does not hold (staff survey results, board composition) are `unknown`.

## Design decisions

- **WPM-D125 No label, ever.** WPM-D25 extended to the passport, "working with me", candidate adjustments and absence
  adjustments: no field anywhere holds a condition, a diagnosis, a neurodivergence label, a disability flag or evidence of one.
  Disability as a characteristic exists only as a voluntary equality-monitoring answer, which feeds aggregates and nothing else.
  Rejected: a "disabled: yes/no" on the worker to drive adjustments (it would make the protected characteristic the gate, which
  WPM-D25 refused, and leak into every view that touches the worker).
- **WPM-D126 The person owns disclosure.** For a non-visible disability, disclosure is a series of choices about who knows
  what. The passport and the profile are shared by the person, entry by entry and person by person, and a share can be
  withdrawn. A new manager is told only that a passport may be shared. Nothing is handed over between managers or organizations
  without the person's act.
- **WPM-D127 Adjust the rule, not the record.** Disability-related absence is handled by an adjustment that changes how a rule
  applies to one person (episodes disregarded, a threshold raised), not by tagging absences as disability-related. The one
  aggregate over adjustments, the count in a change's impact view, is HR-only and from the floor, and is allowed because a
  change that removes an in-place adjustment would breach the duty to adjust; it still names no one and shows no words, so the
  inference WPM-D25 guarded against (which departments have disabled people) needs a group under the floor, which is withheld.
