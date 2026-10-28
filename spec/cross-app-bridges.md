# Cross-app bridges: contact-relationship-management and project-portfolio-management

Proposed 2026-10-28 (spec round WPM-T354; delivery WPM-T355 to WPM-T361). Requirements **WPM-R206–R211**, decisions
**WPM-D134–D136**. Builds on [integrations.md](integrations.md) (EntityRef URNs; the family event envelope; WPM-D101, D102, D106) and on
the scheduling boundary of WPM-D55 and WPM-D65.

WPM and its two siblings hold different facts about overlapping things. **contact-relationship-management (CRM)** holds the people and
organizations the employer deals with: a candidate, an agency, a supplier's contact. **project-portfolio-management (PPM)** holds
programmes, their schedules, issues and change requests. WPM holds the workforce. A bridge lets one say something to another **without
copying a record and without carrying a personal field outward**.

> ⚠️ **Demo software.** Synthetic data only. See [regulatory.md](regulatory.md).

> **What was and was not readable when this was written.** CRM is on the same machine and was looked at only as far as its file list
> and its use of `person_ref`. **PPM's code was not available at all.** Every PPM contract below is therefore an *assumed* shape, marked
> **unverified**, and no code is built against it (WPM-D136).

## Decisions

- **WPM-D134 A bridge carries references and aggregates, never records.** A message across a boundary names a thing by its `EntityRef`
  URN (`person:`, `organization:`, `worker:`, `programme:`) and says what happened, or it is a count with the privacy floor applied.
  It never carries a name, a date of birth, a salary, a rating, a note or an address. The receiving app looks a name up from the owner,
  best-effort, as WPM does for person-service. Rejected: syncing a copy, which makes two places to correct and two to erase.
- **WPM-D135 An inbound call acts under the caller's token, and there is no bridge service account.** As WPM-D101 does for the MCP
  server: the API's policy decides, a caller without access is refused exactly as on the endpoint, and every call is audited with
  the bridge's name. Rejected: a shared key with wide rights, which would make a bridge the weakest door.
- **WPM-D136 A contract that was not read is not built against.** Where the other side's API or event shape is not known, WPM specifies
  what *it* will send or accept, marks the rest unverified, and builds the WPM half only. Rejected: coding against a guessed schema and
  finding out in production.

## CRM

### WPM-R206 — Agency and supplier link

- **The link already exists in part.** A contractor's details hold a `supplier_ref`, validated as an `organization:` URN
  (`worker_contractor_details`; see "Contractor details" in [engagements.md](engagements.md)). That *is* the link to the CRM account that supplies them.
- **Add the supplier's contact.** An optional `supplier_contact_ref`, a `person:` URN (CRM keys its contacts by `person_ref`), so a manager
  can reach the agency about this engagement. WPM stores the two refs and nothing about the agency or the contact.
- **Names are looked up, not kept.** Where CRM offers a lookup, WPM resolves the supplier and contact display names best-effort, cached,
  never blocking a write (the person-service pattern). **CRM's lookup route was not read: unverified, not built.**
- **Who sees it.** As the engagement does: the worker, their managers, HR. The supplier is not a data subject here; the contact is, and is
  shown by name only to a caller who may read the engagement.

### WPM-R207 — Hire handoff

- **What WPM says.** The `worker_hired` event gains `data` holding exactly: `person_ref`, `organization_ref` and `effective_on` (the first day).
  Nothing else: no name, no department, no pay, no worker number beyond what the envelope already has. Before this, the event carried a worker pid
  and number but no `person_ref`, so a consumer could not tell *which person* was hired. It is emitted when a candidate is hired from an
  application (`acquisition.rs`) **and when HR creates a worker directly** (`hr_core.rs`, which until now emitted only `created`).
- **What CRM does with it** is CRM's to decide (for example, mark the contact as now an employee and stop campaign contact). That is
  CRM's requirement and lives in CRM's spec; WPM states the contract and no more.
- **Employment ending** is the mirror: `worker_terminated` and `worker_retired` carry the same three fields beside the `from` and `to` they always had,
  so CRM can restore a contact it paused. All are emitted in the transaction that makes the change (WPM-D9).
- **Not done:** any CRM-side behaviour; replay of events a consumer missed (the outbox gives at-least-once, so a consumer must be idempotent
  on `event_id`).

### WPM-R208 — Overlap check

- **The question.** CRM asks, for a batch of `person:` refs it already holds: *which of these are workers of this organization today?*
- **The answer** is, per supplied ref, `worker: true | false`, and nothing more: no pid, no role, no dates, no department. The caller can
  learn only about refs it already has. At most 500 refs a call.
- **Who may ask.** A privileged caller of the organization (HR), under their own token (WPM-D135). Each call is audited with the count, not the refs.
- **Why it is allowed.** It discloses employment status of a person to a caller who already holds that person's ref and is HR of the
  employer; it is the same fact HR reads on the worker list. A person decides any merge of two records (the matcher's rule: [worker-matching.md](worker-matching.md)).
- **Not done:** a bulk export of all workers' refs; a push to CRM.

## PPM

### WPM-R209 — Capacity what-if answer (PPM asks WPM)

This is **WPM-R97** and tasks **WPM-T158 to T160**, unchanged: PPM, under its own caller's token, asks whether a proposed change fits
capacity; WPM reads, computes and stores nothing, names no one, and says what the answer rests on. Registered here so the three bridges
read as one set. **Contract with PPM: unverified.**

### WPM-R210 — Programme demand in (PPM tells WPM)

- **What WPM would accept.** A signed-in caller with the planning right posts a programme's staffing demand: a `programme:` URN, a pool
  or role, FTE per month, and the PPM event id. WPM turns it into demand lines against the programme ref it already holds (WPM-D55).
- **Idempotent** on `(programme_ref, source_event_id)`: a repeat changes nothing. A line for a programme WPM has no planning scope for is
  refused. WPM records the source and never invents a programme.
- **PPM's event shape and delivery are unknown: unverified.** WPM specifies the request it accepts; PPM adapts to it or this changes.
- **Not done:** pulling from PPM; schedule data (dates of tasks, critical path) never enters WPM (WPM-D55).

### WPM-R211 — People-impact out (WPM tells PPM)

- **For a programme, WPM can answer** with counts only: workers by pool who are committed to it, fixed-term and contractor engagements that end
  before its end date, and over-committed pool-months. The privacy floor applies (WPM-D35, `rules::suppression`); a cell under it is hidden
  and no total gives it away. No name, no pid, no pay.
- **Read-only, under the caller's token,** audited (WPM-D135). It is the people-impact answer of WPM-R95 to R97 for an issue or a change
  request that names a programme.
- **Contract with PPM: unverified.** Shares its core with WPM-R209.

## Delivery

| Task | What | Needs |
| --- | --- | --- |
| WPM-T355 | `worker_hired` and `worker_terminated`/`worker_retired` carry `person_ref`, `organization_ref`, `effective_on` and nothing else (R207) | nothing outside WPM |
| WPM-T356 | `supplier_contact_ref` on contractor details (R206) | nothing outside WPM |
| WPM-T357 | `POST /api/bridges/overlap` (R208) | nothing outside WPM |
| WPM-T358 | Best-effort supplier and contact name lookup from CRM (R206) | CRM's lookup route, to be read |
| WPM-T359 | Capacity what-if (R209) | the capacity model (WPM-T104 to T107); is WPM-T158 to T160 |
| WPM-T360 | Programme demand in (R210) | PPM's contract |
| WPM-T361 | People-impact out (R211) | WPM-T359, PPM's contract |
