# Absence, sickness absence, work-related stress, and help for line managers

Proposed 2026-10-27 ([plan.md](plan.md) section Q; spec round WPM-T309, delivery Phase 24, WPM-T326–T333). Requirements
**WPM-R179–R189**, decisions **WPM-D121–D124**. Nothing here is built.

Absence management in WPM today is entitlements, requests and balances ([workforce-management.md](workforce-management.md)); a
sick-leave request records that it is sick leave and its dates, never why (WPM-R118, WPM-D73). What is missing is the
**management**: knowing an episode has ended, holding the return-to-work conversation, noticing a pattern early enough to help,
keeping in touch through a long absence, planning a phased return, involving occupational health, and treating work-related
stress as a problem **in the work**. Most of that falls to line managers, who are also given the least help, so the second half of
this file is theirs.

> The posture throughout: WPM records **that** and **when**, never **what is wrong** (WPM-D73, WPM-D123). A conversation is held
> by people; the software says it is due and keeps the date it happened.

## Absence and sickness absence

### WPM-R179 — Sickness episodes and fit notes

- **An episode** is one continuous sickness absence: the first day, the last day (open while the person is off) and the date
  they returned. Episodes are derived from approved sick-leave requests, so a request that is extended extends the episode, and
  two requests with no working day between them make one episode.
- **Self-certified or certified.** Each episode records whether evidence was needed and received. The period after which a
  medical certificate is needed is configuration (in Great Britain a fit note is needed after seven calendar days; not shipped as
  a default, WPM-D118 spirit). A missing certificate is shown to the manager and HR as **due**, never as a disciplinary fact.
- **A fit note is recorded as dates and one answer**: the period it covers, and whether it says the person **may be fit for work
  with adjustments** (yes or no). Its text, the condition and the clinician's notes get no column (WPM-D123). Where it says the
  person may be fit with support, the person is offered a link to request an adjustment (WPM-R33), in their own words.
- **Who sees it.** The person; their managers up the chain (dates and status); HR. Exported with the worker; erased with them.

### WPM-R180 — Return-to-work conversations

- **Due after every episode**, on the return date, between the person and their line manager (or someone the manager delegates
  to, recorded).
- **Recorded as**: held on (date), held by, the **support offered** from a closed list (`adjustment`, `phased_return`,
  `occupational_health_referral`, `employee_assistance`, `keeping_in_touch_plan`, `none_needed`), and **agreed actions** as
  short text. The form says, beside the field, that health details do not belong in it (WPM-D73: discouraged by design where a
  field cannot refuse it).
- **The person sees the record and may add a comment**; a correction is a new version with the previous kept. Not held within a
  configured number of working days is shown to the manager as overdue, and to HR as a count.

### WPM-R181 — Absence review points

- **A review point** is a configured threshold over a rolling window: a number of episodes, or a number of working days, or
  both (for example "three episodes or ten days in twelve months"). The rule is configuration; none is shipped.
- **Reaching one means a supportive conversation is due**, nothing else (WPM-D122). The manager sees "review point reached" for
  their report; a person decides what follows, and the outcome (`no_action`, `support_agreed`, `referred`, `formal_stage`) is
  recorded with a date. A formal stage is a person's decision, never the software's.
- **Disability-related absence is adjusted for, without recording disability** (WPM-R193, WPM-D127): an agreed adjustment can
  disregard named episodes or raise the threshold for one person.
- **No Bradford factor.** WPM does not compute it (WPM-D122).

### WPM-R182 — Long-term absence, keeping in touch and phased return

- **Long-term** is an episode longer than a configured number of calendar days; the episode then shows as long-term to the
  manager and HR.
- **A keeping-in-touch plan** agreed with the person: how often and how (`call`, `video`, `message`, `in_person`, `through_hr`),
  and a log of contacts as dates only. The person may ask for contact to pause; that is recorded and respected, and the plan
  shows "paused at the person's request".
- **A phased return plan**: a start date and, per week, the working pattern as FTE percent (or hours), with the end date
  derived; how a phased return is paid is a configured rule (pay as normal, pay for hours worked, or annual leave used), stated
  on the plan. The plan's weeks are pure-core arithmetic; the plan is agreed by the person and the manager and dated.

### WPM-R183 — Occupational health referrals

- **A referral** is a status and dates: `drafted`, `consented`, `sent`, `appointment_booked`, `advice_received`, `closed`. It
  cannot be sent without the person's **recorded consent**, and the person may withdraw consent before it is sent.
- **The referral questions** are about the work (what the role involves, what the manager wants advice on), from a closed list
  plus short text; the form says health details do not belong in it.
- **Advice stays with occupational health** ([scope.md](scope.md)). WPM records that advice was received and whether it was
  shared with the manager (the person's choice). Recommended changes come back **through the adjustment flow** (WPM-R33), so
  they are in the person's words and follow its rules.
- Uses the existing occupational-health token attribute ([auth.md](auth.md)) for the steps occupational health owns.

## Work-related stress

### WPM-R184 — Stress risk assessments, about the work

- **An individual stress risk assessment** looks at the six areas of the Health and Safety Executive's Management Standards:
  **demands, control, support, relationships, role and change**. For each, the person and their manager record the level of
  concern (`none`, `some`, `significant`) and the actions agreed, with an owner and a date (WPM-D121).
- **Anyone can start one**: the person, their manager, or HR; it does not need an absence. It is **offered** after any
  episode the person says was related to work, and after a return-to-work conversation that offered support.
- **"Related to work"** is an answer the person may give about an episode: `yes`, `no` or `not_said` (the default). It is not a
  reason and names nothing; the person, their managers and HR see it, and it is counted only in aggregate with the floor.
- **No symptom, no diagnosis, no "stress" label** on the person or the episode. The assessment's text is content-tier (as
  adjustment words are): the person, their managers up the chain and HR; reads audited; exported; scrubbed on erasure.
- **Team-level indicators.** The anonymous pulse (WPM-R26) may carry questions mapped to the six areas; results are aggregate
  with the floor, as every pulse result is, and name no one.

## Absence information for HR

### WPM-R185 — Absence measures

- **Absence rate**: working days lost to sickness over working days available, by organization and department, per month,
  with the floor; a department with no available days is `null`. Split short-term and long-term (R182). **Never by reason**:
  there is none to split by.
- **The work-related share** (R184) and **return-to-work conversations held on time** as counts and rates with the floor.
- The human capital report's absence measure (WPM-R147) reads this definition, which closes the "needs a definition first" note
  in [plan.md](plan.md) section B.

## Help for line managers

### WPM-R186 — The manager's action list

- **One screen of what is due** for the manager's direct reports (and, for a manager of managers, a count per team below):
  return-to-work conversations due or overdue, review points reached, keeping-in-touch contacts due, certificates due, phased
  returns starting or ending, leave to approve, flexible working decisions and their statutory dates (WPM-R126), resignations
  to acknowledge, PADRs due (WPM-R172), statutory and mandatory learning expiring (WPM-R170), step readiness (WPM-R173),
  adjustments due for review (WPM-R192), engagements and probation ending, and expense claims to decide.
- **Derived, not stored.** Each item is computed from the record that owns it and links to the screen where it is done; doing
  it there clears it. Nothing is a separate to-do record.
- **Named for one's own reports only**, and each item says **what is due, never why**: "a return-to-work conversation is due",
  not anything about the absence. The pure core takes the as-of date.
- **The same list for HR** across an organization, as counts per manager with the floor, so HR can see where managers need
  help without a league table of people.

### WPM-R187 — Guidance at the point of need

- **Every action links to guidance**: the organization's policy (WPM-R156) and a short page in the reader's locale on how to do
  it well ("holding a return-to-work conversation", "talking about adjustments", "a stress risk assessment", "keeping in touch
  without pressure", "when someone tells you they are neurodivergent").
- **Guidance is content** ([localization.md](../AGENTS/localization.md)): pages in the CMS per locale, which a deployer replaces
  with their own. The repo ships short, generic pages written for this demo, labelled as such and not legal advice.
- **Conversation guides** give prompts and what not to ask (for example, never ask for a diagnosis), as content, not as a
  script the software records.

### WPM-R188 — Changing manager

- **When a worker's line manager changes** (a transfer, a restructure, a new manager), the new manager's action list shows a
  dated "getting to know your team" item for each new report with what is open: absences and their plans, review points,
  PADR due date, learning expiring, and **whether the person has an adjustments passport they may choose to share**
  (WPM-R190). The words of adjustments are not handed over; the person shares them (WPM-D126).
- **No manager notes about people** (WPM-D124): there is no free-text "notes on this person" field for a manager to hand on.

### WPM-R189 — Managers' own learning and support

- **Being a people manager is an audience** for statutory and mandatory requirements (WPM-R170): a requirement can target
  workers who have at least one direct report, so manager training (absence, adjustments, wellbeing conversations) is tracked
  like any other.
- **A manager can ask for help** on an item: a one-click request to HR from the action list ("I'd like support with this"),
  which HR sees with the item's kind and the manager, and closes. It carries no detail about the report beyond the item.

## Design decisions

- **WPM-D121 Stress is assessed in the work, not recorded in the person.** The Management Standards look at demands, control,
  support, relationships, role and change, all properties of the job. Recording them gives a manager things to change; a
  "stress" label on a person or an absence would be health data that changes nothing. Rejected: an absence reason of `stress`
  (it reopens the reason column WPM-D73 closed, and the rest of mental health would follow).
- **WPM-D122 A review point starts a conversation and never acts; no Bradford factor.** Thresholds are configuration and the
  outcome is a person's decision. The Bradford factor weights frequent short absences heavily, which is the pattern of many
  disabilities and long-term conditions; computing it would put a number on the discrimination the adjustment duty exists to
  prevent.
- **WPM-D123 A certificate and an occupational health report are evidence held elsewhere.** WPM records that one exists, the
  dates it covers and the one answer the workflow needs; its contents stay with the person and occupational health.
- **WPM-D124 A manager gets a to-do list, not a file.** Every item is derived from a record with its own purpose, rules and
  retention. A free-text notes field about a person would be a shadow file with none of those, so it does not exist.
