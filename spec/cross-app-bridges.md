# Cross-app bridges: contact-relationship-management and project-portfolio-management

Proposed 2026-10-28 (spec round WPM-T354); built 2026-10-28 to 2026-10-30. Requirements **WPM-R206–R211, WPM-R213, WPM-R215
and WPM-R216**, decisions **WPM-D134–D136, WPM-D138–D141 and WPM-D145–D147**, tasks **WPM-T354 to WPM-T366, WPM-T372, WPM-T387
and WPM-T388** (all done; see "Delivery" for what each leaves undone). The ids between them belong elsewhere: WPM-R212 and WPM-D137
are the MHRA ([mhra.md](mhra.md)), WPM-R214, WPM-D143 and WPM-D144 the worldwide target ([worldwide.md](worldwide.md)), WPM-D142
the what-if's audience ([ppm-issues-and-changes.md](ppm-issues-and-changes.md)), and WPM-T367 to T386 the worldwide tasks.
WPM-D145 is written in [integrations.md](integrations.md). Builds on [integrations.md](integrations.md) (EntityRef URNs; the family event envelope; WPM-D101, D102, D106) and on
the scheduling boundary of WPM-D55 and WPM-D65.

WPM and its two siblings hold different facts about overlapping things. **contact-relationship-management (CRM)** holds the people and
organizations the employer deals with: a candidate, an agency, a supplier's contact. **project-portfolio-management (PPM)** holds
programmes, their schedules, issues and change requests. WPM holds the workforce. A bridge lets one say something to another **without
copying a record and without carrying a personal field outward**.

> ⚠️ **Demo software.** Synthetic data only. See [regulatory.md](regulatory.md).

> **What was and was not readable when this was written (2026-10-28).** CRM was then looked at only as far as its file list and its
> use of `person_ref`, and PPM's code was not found at first; every PPM contract was an *assumed* shape, marked **unverified**, and no
> code was built against it (WPM-D136). Both were read later the same day (see "What CRM was read to say" and "What PPM was read to
> say"). **On 2026-10-30 the product owner fixed the family contract** (next section); where it applies it replaces the "unverified"
> notes below. What the contract does not cover (the what-if call, WPM-R209) is still unverified.

## Decisions

- **WPM-D134 A bridge carries references and aggregates, never records.** A message across a boundary names a thing by its `EntityRef`
  URN (`person:`, `organization:`, `worker:`, and `thing:` for a programme, as the family contract fixes) and says what happened, or it is a count with the privacy floor applied.
  It never carries a name, a date of birth, a salary, a rating, a note or an address. The receiving app looks a name up from the owner,
  best-effort, as WPM does for person-service. Rejected: syncing a copy, which makes two places to correct and two to erase.
- **WPM-D135 An inbound call acts under the caller's token, and there is no bridge service account.** As WPM-D101 does for the MCP
  server: the API's policy decides, a caller without access is refused exactly as on the endpoint, and every call is audited with
  the bridge's name. Rejected: a shared key with wide rights, which would make a bridge the weakest door.
- **WPM-D136 A contract that was not read is not built against.** Where the other side's API or event shape is not known, WPM specifies
  what *it* will send or accept, marks the rest unverified, and builds the WPM half only. Rejected: coding against a guessed schema and
  finding out in production.

## The family contract (decided 2026-10-30, at the product owner's direction)

The three apps (WPM, PPM, CRM) are being harmonized together. The product owner fixed these points for all three; WPM records them
here and builds its half. They sit inside WPM-D134 to D141: references and counts only, the caller's own token, one owner per fact.

1. **Programme identity.** A PPM plan's `pid` (a UUID) is named in WPM as `programme_ref = "thing:<ppm plan pid>"`. WPM keeps no copy
   of the plan; it holds the reference where it already held a programme's `thing:` URN (demand claims, post funding, positions).
2. **The pool link.** WPM's `skill_pools.peer_pool_ref` holds the **PPM pool's pid**; PPM holds the **WPM pool's pid** in its own
   `peer_pool_ref`. Each side sets its half by hand (WPM-D139).
3. **PPM → WPM demand and supply.** PPM calls the existing `POST /api/capacity/peer-demand` with `{pool_pid: <WPM pool pid>,
   programme_ref, month: YYYY-MM-01, fte_centi, peer_event_id}`, **under the forwarded caller's bearer token** (WPM-D135). PPM pulls
   the existing `GET /api/capacity/peer-supply`. Neither route changes for this. **Amended 2026-10-30 (at the product owner's
   direction, WPM-R215, WPM-D146):** `fte_centi: 0` withdraws the peer's claim, and PPM sends an optional `sent_at` (RFC 3339) so an
   older event cannot overwrite a newer one; see "Withdrawing a figure, and the order of events" below.
4. **WPM → PPM people-impact is a pull.** PPM asks `GET /api/capacity/people-impact?programme=thing:<uuid>` under the caller's token;
   WPM answers counts only (WPM-R211, WPM-T361). WPM pushes nothing to PPM.
5. **WPM → CRM handoff is over webhooks.** A `worker_hired`, `worker_terminated` or `worker_retired` delivery carries the handoff
   references so a subscriber can tell who was hired or left (WPM-R207; the payload change is a decision of its own). CRM
   builds the receiver.

**What this does not verify.** No PPM or CRM code that implements its half had been run against WPM when this was written: PPM's
sender and puller and CRM's webhook receiver were being built in parallel by their own owners. Each WPM route named here is exercised
by WPM's request tests against WPM alone.

## CRM

### WPM-R206 — Agency and supplier link

- **The link already exists in part.** A contractor's details hold a `supplier_ref`, validated as an `organization:` URN
  (`worker_contractor_details`; see "Contractor details" in [engagements.md](engagements.md)). That *is* the link to the CRM account that supplies them.
- **Add the supplier's contact.** An optional `supplier_contact_ref`, a `person:` URN (CRM keys its contacts by `person_ref`), so a manager
  can reach the agency about this engagement. WPM stores the two refs and nothing about the agency or the contact.
- **Names are looked up, not kept.** Where CRM offers a lookup, WPM resolves the supplier and contact display names best-effort, cached,
  never blocking a write (the person-service pattern). **CRM was read and has no lookup by reference** (see "What CRM was read to
  say"), so both names come through `clients::display_name` from the owning service instead (WPM-T358):
  `GET /api/workers/{pid}/contractor-details` adds `supplier_name` and `supplier_contact_name`, `null` when unresolved. Only stub
  mode has been run.
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
- **Over webhooks (family contract point 5; proposed WPM-D145, WPM-T372).** A webhook subscriber to these three kinds now receives the
  same handoff in the payload's `data`, so CRM can build its receiver against `POST`s it is sent rather than read WPM's API:

  ```json
  { "id": "<event uuid>", "kind": "worker_hired", "subject": "worker:<pid>", "occurred_at": "2026-10-30T12:00:00Z",
    "data": { "person_ref": "person:<uuid>", "organization_ref": "organization:<uuid>", "effective_on": "2026-10-30" } }
  ```

  `worker_terminated` and `worker_retired` add `"from"` and `"to"` (the status change, for example `"offboarding"` to
  `"terminated"`) inside `data`. `worker_activated` and `leave_approved` have no `data`. Headers: `x-wpm-event`, `x-wpm-delivery`,
  `x-wpm-signature: t=<unix>,v1=<hex HMAC-SHA256 of "<t>.<body>">`, unchanged. **CRM's receiver has not been run against it.**

### WPM-R208 — Overlap check

- **The question.** CRM asks, for a batch of `person:` refs it already holds: *which of these are workers of this organization today?*
- **The answer** is, per supplied ref, `worker: true | false`, and nothing more: no pid, no role, no dates, no department. The caller can
  learn only about refs it already has. At most 500 refs a call.
- **Who may ask.** A privileged caller of the organization (HR), under their own token (WPM-D135). Each call is audited with the count, not the refs.
- **Why it is allowed.** It discloses employment status of a person to a caller who already holds that person's ref and is HR of the
  employer; it is the same fact HR reads on the worker list. A person decides any merge of two records (the matcher's rule: [worker-matching.md](worker-matching.md)).
- **Built** as `POST /api/bridges/overlap` with `{"organization_ref": "organization:<uuid>", "person_refs": [...]}` (WPM-T357).
- **Not done:** a bulk export of all workers' refs; a push to CRM; a rate limit on the route.

## PPM

### WPM-R209 — Capacity what-if answer (PPM asks WPM)

This is **WPM-R97** and tasks **WPM-T158 to T160**, unchanged: PPM, under its own caller's token, asks whether a proposed change fits
capacity; WPM reads, computes and stores nothing, names no one, and says what the answer rests on. Registered here so the three bridges
read as one set. **Contract with PPM: unverified** (the family contract of 2026-10-30 does not cover the what-if call).

### WPM-R210 — Programme demand in (PPM tells WPM)

- **What WPM accepts.** A caller the policy lets write capacity posts one figure: a linked WPM pool, a programme's `thing:` URN, a
  month and centi-FTE, with the PPM event id. WPM stores it as a demand claim with `source = ppm` (WPM-T364), against the programme
  reference, never a copy of the programme (WPM-D55).
- **Idempotent** on the `peer_event_id` of a claim: a repeat changes nothing and answers `duplicate: true`. A pool that is not
  linked is refused. WPM records the source and never invents a programme.
- **The shape is fixed (2026-10-30, family contract point 3):** PPM calls `POST /api/capacity/peer-demand` with
  `{pool_pid, programme_ref, month, fte_centi, peer_event_id}` under the forwarded caller's token, for a pool linked on both sides.
  The route was built for WPM-R213 (WPM-T364); PPM's sender is being built against it and has not been run against WPM.
- **Not done:** pulling from PPM; schedule data (dates of tasks, critical path) never enters WPM (WPM-D55).

### WPM-R215 — Withdrawing a figure, and the order of events (PPM tells WPM)

Decided 2026-10-30 as part of the family contract, at the product owner's direction; built as **WPM-T387**. Before this the route
refused `fte_centi: 0`, so a figure PPM dropped to zero stayed in WPM for good, and only the latest event id of a claim was remembered,
so an older event replayed after a newer one overwrote it (recorded in WPM-T360 and T364).

**WPM-R215** The peer withdraws its claim for a (pool, programme, month) by sending `fte_centi: 0` with a new `peer_event_id`. A
withdrawal is idempotent on the event id; zero for a claim that does not exist changes nothing and says so. An event the peer says it
sent (`sent_at`) before the last event applied to the claim is ignored and says so.

- **WPM-D146 A withdrawn claim is closed and marked withdrawn by the peer; the start decision that activated it is left as it was.**
  The claim's status becomes `closed` (the existing "finished or withdrawn" state, ignored by the view, the start check and the
  what-if) and `peer_withdrawn_at` records when. Its last figure is kept, since the figure column refuses zero and the figure is what
  was withdrawn. **A claim a start decision had made `active` stops counting at once**; the decision row (who, why, the fit it saw) is
  history and is not changed or undone, and the audit entry of the withdrawal says the claim had been active (no figure). A later
  figure above zero **reopens** the claim as `proposed`, so it counts again only after a new start decision (WPM-D50): the peer can
  withdraw a figure, never activate one. A pool whose only claims are withdrawn has a *known* demand of zero (the peer has spoken), not
  the unknown of a peer that has said nothing (WPM-D141). Rejected: deleting the row (the trail of what the peer said goes with it);
  undoing the start decision (a person's recorded decision is not the peer's to change).
- **Order.** `sent_at` is optional. An event is stale only when **both** it and the last applied event carry `sent_at` and it is
  strictly older; the same instant, or an event without `sent_at`, is applied in arrival order, as before. A stale event changes
  nothing, is not audited, and is not remembered (a repeat of it is stale again).

**The contract** (`POST /api/capacity/peer-demand`, under the forwarded caller's token, for a pool linked on both sides):

```json
{ "pool_pid": "<WPM pool pid>", "programme_ref": "thing:<ppm plan pid>", "month": "YYYY-MM-01",
  "fte_centi": 0..1000000, "peer_event_id": "<1 to 128 characters>", "sent_at": "2026-10-30T12:00:00Z" }
```

`sent_at` may be omitted. The answer is always `200` with `{ "pid", "duplicate", "outcome", "status" }`:

| `outcome` | When | `pid` | `status` | Written and audited |
| --- | --- | --- | --- | --- |
| `recorded` | a figure above zero, no claim yet | the new claim | `proposed` | yes, `peer_demand_received` |
| `updated` | a figure above zero for a live claim | the claim | unchanged (`proposed` or `active`) | yes, `peer_demand_received` |
| `reopened` | a figure above zero for a withdrawn claim | the claim | `proposed` | yes, `peer_demand_received` |
| `withdrawn` | zero for a live claim | the claim | `closed` | yes, `peer_demand_withdrawn` with `was_active` |
| `already_withdrawn` | zero for a withdrawn claim | the claim | `closed` | the event id and `sent_at` are kept; `peer_demand_withdrawn` |
| `nothing_to_withdraw` | zero and no claim | `null` | `null` | no |
| `stale` | `sent_at` older than the last applied event's | the claim | unchanged | no |
| `duplicate` (`duplicate: true`) | a `peer_event_id` already applied | the claim | its status | no |

Refused with `422`: a negative figure or one above 1 000 000, a month that is not its first day, a `programme_ref` that is not a
`thing:` URN, an empty or over-long event id, a pool that is not linked. A `sent_at` that is not RFC 3339 is refused by the body
parser (a `4xx`). The claim list (`GET /api/programme-demands`) shows `peer_sent_at` and `peer_withdrawn_at`.

**Not done:** zero for a claim that does not exist stores nothing, so if that withdrawal overtakes the figure it withdraws (the figure
arrives second), the figure is recorded; PPM's ordering on its side, or sending the figure again as zero, closes it. An event without
`sent_at` can still overwrite a newer one, as before. **Not verified:** PPM's sender (being changed by PPM's owner in parallel against
this contract) has not been run against WPM.

### WPM-R211 — People-impact out (WPM tells PPM)

- **For a programme, WPM can answer** with counts only: workers by pool who are committed to it, fixed-term and contractor engagements that end
  before its end date, and over-committed pool-months. The privacy floor applies (WPM-D35, `rules::suppression`); a cell under it is hidden
  and no total gives it away. No name, no pid, no pay.
- **Read-only, under the caller's token,** audited (WPM-D135). It is the people-impact answer of WPM-R95 to R97 for an issue or a change
  request that names a programme.
- **A pull (2026-10-30, family contract point 4):** `GET /api/capacity/people-impact?programme=thing:<uuid>`, **built as WPM-T361**.
  Where a sub-count cannot be computed from what WPM holds, it is `unknown` with the reason, never zero.
- **What each count rests on** (`rules::people_impact`):
  - *Workers committed* are the people employed today whose post is recorded as funded by the programme (`post_fundings.programme_ref`,
    funding not ended): WPM links a person to a programme only that way (WPM-D59). Each is counted in every pool they count towards
    (the pool's role profiles or skills, as the capacity view counts them), and in `in_no_pool` when in none. **Unknown** when no
    such post is recorded, since WPM cannot tell "nobody" from "not recorded".
  - *Engagements ending* are, of those people, the fixed-term and contract engagements that end on or after today and before the
    programme's end. **WPM holds no end date for a programme**, so the count is **unknown** unless the caller passes
    `ends_on` (the date the programme's owner gives; the answer says `programme_ends_on_source: "caller"`). WPM never takes an end
    from its own data, such as the last month with a claim. A contingent engagement with no end recorded is counted in
    `end_not_recorded`, not as ending.
  - *Over-committed pool-months* are, of the pool-months in which the programme holds a claim the pool counts (a linked pool the
    peer's, an unlinked one its own, WPM-D140), those `over` in the capacity view and those that cannot be said. **Unknown** when
    the programme holds no such claim. Over is worked out from active claims; a proposed one (as every claim PPM sends arrives) is
    shown but not counted until a start decision (WPM-D50). A pool-month is not a person and is not floored.
- **The floor** is ten for every count of people (the floor of the HR-only programme roll-up over the same funding data, WPM-R83),
  since this answer reaches a caller who need not be HR. A count under it is `null` with `hidden: true`; zero is shown. **There is
  no total across pools**: a person can count in more than one, and a total beside the cells could give a hidden one away.
- **Who may ask:** any signed-in caller who is not partner-only (an aggregate, as the capacity view), and only about the
  organizations in the caller's scope; a caller with none in scope is told every part is unknown. Each answer is audited
  (`people_impact_asked` with the programme reference, the number of organizations and whether `ends_on` was given; no count).
- **Answer** (`months` defaults to 12, from the current month):

  ```json
  {
    "programme_ref": "thing:<uuid>", "source": "wpm", "as_of": "YYYY-MM-DD",
    "programme_ends_on": "YYYY-MM-DD" | null, "programme_ends_on_source": "caller" | null,
    "months": ["YYYY-MM-01", ...], "floor": 10,
    "workers_committed": {
      "known": true, "reason": null,
      "pools": [{ "pool_pid": "<uuid>", "pool": "<name>", "count": 12 | null, "hidden": false | true }],
      "in_no_pool": { "count": 0 | null, "hidden": false | true }
    },
    "contingent_ending_before_end": {
      "known": true, "reason": null,
      "pools": [ ...as above... ], "in_no_pool": { ... }, "end_not_recorded": { "count": ..., "hidden": ... }
    },
    "over_committed_pool_months": {
      "known": true, "reason": null, "considered": 1, "over": 1, "unknown": 0,
      "pools": [{ "pool_pid": "<uuid>", "pool": "<name>", "over_months": ["YYYY-MM-01"], "unknown_months": [] }]
    },
    "derivation": "..."
  }
  ```

  An unknown part is `{"known": false, "reason": "<why>"}` with every figure `null`.
- **Not done:** a push to PPM; a per-month breakdown of the people counts; a defence against differencing two answers for
  overlapping programmes or two pools that share people.

## Working alone, and working together (WPM-R213, WPM-D138 to D141)

Added 2026-10-28 after PPM was read. PPM and WPM each keep a capacity model (see "What PPM was read to say"). The aim is to **harmonize**
them without making either depend on the other.

- **WPM-D138 Each app owns its model and is complete without the other.** With no link configured, WPM's pools, demand and what-if work
  exactly as today, and nothing calls out. A peer being down, absent or slow never fails a WPM request and never changes a number
  silently. Rejected: making WPM read PPM's pools, which would make PPM a dependency.
- **WPM-D139 A link is an explicit mapping set by a person, never matched by name.** A pool carries an optional `peer_pool_ref`, the
  other app's identifier for the pool that means the same skills. It is set on both sides by someone who can see both, shown to them, and
  removed in one step. An unmapped pool stays local. Rejected: matching pool names or tags, because a name is not an identity and a wrong
  match would move real numbers.
- **WPM-D140 For a linked pool each fact has one owner.** *(Built in part: the count follows the one owner,
  `rules::capacity::counted_source`; the capacity view does not yet show the ignored local figure as "entered here" beside it, though
  the claim list shows every claim with its source. WPM-T364.)* **Supply is WPM's** (it holds the workers, their leave and their FTE); **demand
  is PPM's** (it holds the plans). Each side shows the owner's figure for a linked pool, labelled with its source and the time it was
  taken. The receiving side's own entry of that kind is kept but not added: it is shown as "entered here" beside the owner's figure, so
  two numbers for one pool are visible, named and never summed.
- **WPM-D141 When the peer cannot be reached, the last figure received is shown with its age; with none, the answer is `unknown`.** Never
  zero, never a silent fall back to the local entry. Unlinking returns the pool to local figures at once.

**WPM-R213** A pool can be linked to its peer's pool by an explicit mapping; linked or not, WPM answers every capacity request itself;
a linked pool takes demand from PPM and gives supply to PPM, each with its source and age; the two apps' figures are never added
together. This extends WPM-R209 and WPM-R210.

What this needs on WPM's side, in order, **all built 2026-10-28** (WPM-T363 to T366; see "Tasks for the harmonization"):

1. A pool's `peer_pool_ref` (nullable), set and cleared by HR with the pool's own audit, no name or figure in the entry.
2. Demand that arrives from the peer is stored as claims with `source = ppm`, the peer's event id, and the time received (idempotent on
   the event id), and counted in the view with its source named; a local claim is `source = local`.
3. The view and the what-if read demand by source for a linked pool as D140 says.
4. An outbound supply figure for a linked pool: counts by pool and month, floored, to the caller's token (WPM-D135).

What it needs on PPM's side is for its owners to say: a nullable mapping to a WPM pool, the same one-owner rule, and an inbound route for
people-impact. WPM does not change PPM.

## Delivery

| Task | What | Needs |
| --- | --- | --- |
| WPM-T355 | `worker_hired` and `worker_terminated`/`worker_retired` carry `person_ref`, `organization_ref`, `effective_on` and nothing else (R207) | nothing outside WPM |
| WPM-T356 | `supplier_contact_ref` on contractor details (R206) | nothing outside WPM |
| WPM-T357 | `POST /api/bridges/overlap` (R208) | nothing outside WPM |
| WPM-T358 | Supplier and contact names (R206) | **Read 2026-10-28: CRM has no lookup by reference**, and holds no person's name; see "What CRM was read to say". Re-scoped to person-service |
| WPM-T359 | Capacity what-if (R209) | the capacity model (WPM-T104 to T107); is WPM-T158 to T160, **all done** (`POST /api/capacity/what-if`, the contingent effect, the persona test). PPM's side unverified |
| WPM-T360 | Programme demand in (R210, R213) | **Closed 2026-10-30.** The receive route exists (T364); the family contract fixes PPM as its caller. PPM's sender is being built against it; not verified end to end |
| WPM-T361 | People-impact out (R211) | **Built 2026-10-30:** `GET /api/capacity/people-impact`, a pull by PPM, fixed by the family contract |
| WPM-T363 to T366 | Pool link, sourced demand, outbound supply, standalone tests (R213) | **Built 2026-10-28**; see "Tasks for the harmonization" |
| WPM-T372 | Hire and ending handoff in the webhook payload (R207, D145) | **Built 2026-10-30.** CRM's receiver not run against it |
| WPM-T387 | Withdrawal with zero, and `sent_at` ordering (R215, D146) | **Built 2026-10-30.** PPM's sender not run against it |
| WPM-T388 | The lean worker list, `?fields=`, and salary-read audit on the list (R216, D147) | **Built 2026-10-30.** CRM's import not changed or run |

Every route above is in the OpenAPI document.

## What CRM was read to say (2026-10-28)

CRM's source is on the same machine (`contact-relationship-management-service-with-rust`) and was read, not run.

1. **CRM keeps no name for a person and has no lookup by reference.** A contact holds a `person:` URN; the name is resolved from
   person-service at the moment of need (`clients::resolve_person`), and the list routes (`/api/contacts`, `/api/accounts`) are paged
   lists, not lookups by ref. So the plan of WPM-T358, a supplier contact's name from CRM, cannot be done and should not be: the owner of
   a `person:` name is person-service, which WPM already reads best-effort. **T358 is re-scoped** to resolving `supplier_contact_ref`
   through that client. An `organization:` name has no confirmed source; CRM accounts carry a `display_name` but no by-ref route was found.
2. **CRM already calls WPM, and the call does not match WPM-D135.** `POST /api/bridges/wpm/teams-import` (admin only, stub-first) pages
   `GET {CRM_WORKER_SERVICE_URL}/api/workers?status=active&limit=200&offset=`, **sending no token**, and keeps each worker's `pid` and
   `department` only, as `worker:<pid>` references. Two things follow. WPM's list is for a caller with access: an unauthenticated
   call is refused when authentication is required, and an open one would be reading every active worker's record to use two fields.
   And the reference is `worker:` where this document and WPM's events use `person:` (the `worker_hired` payload carries both). Neither is
   fixed here; both are for CRM's owner and for WPM-T358's successor to settle. The call was read, not run against WPM.
   **Update 2026-10-30 (CRM's spec read again):** the import now sends a token (CRM-R87, CRM-T153): when `CRM_WORKER_SERVICE_TOKEN` is
   set, the call carries it as `Authorization: Bearer`, only over `https` or to this machine, never logged. It is **one deployer-held
   token for every caller**, not the caller's own token forwarded, so WPM's policy and scope decide on whatever identity that token
   carries and the import sees the workers that identity may see. That is closer to WPM-D135 than no token, and not the same as it.
   **The `worker:` against `person:` question is settled on CRM's side (2026-10-30, CRM-T161, CRM-D17, read from CRM's spec):** the import
   now keeps each worker's `person_ref` and stores team members by it, skipping (and counting) a worker without one; it never falls back
   to `worker:`. Two things remain for WPM's owner, found by a live end-to-end run of the three apps on 2026-10-30: `GET /api/workers`
   returns the whole worker record (name, job title, manager, salary unless the policy masks it) to the import's token, which keeps three
   fields of it; and the list does not audit salary reads as the single-worker read does. Neither is changed here. **Both settled
   on WPM's side the same day (WPM-R216, WPM-D147, WPM-T388):** the list takes `?fields=`, so the import can ask for
   `GET /api/workers?status=active&fields=pid,department,person_ref&limit=200&offset=` and receive those three fields of each worker
   and nothing else, under the same policy and scope; and a list that carries an unmasked salary now audits a `salary_read` for each
   such worker. **Whether and when CRM asks for the lean list is CRM's change**; nothing here has been run from CRM.
3. **PPM's code was not found at first.** On 2026-10-28 it was at `~/git/project-porfolio-management/project-porfolio-management`
   (the directory then spelled `porfolio`); as of 2026-10-30 it is at `~/git/project-portfolio-management/project-portfolio-management`,
   spelled as the crate and docs spell it. PPM itself was read next (below).

### WPM-R216 — A lean worker list for a bridge

**WPM-R216** `GET /api/workers` takes an optional `fields`: a comma-separated list of the record's field names (`pid`, `person_ref`,
`upstream_worker_ref`, `organization_ref`, `worker_number`, `display_name`, `status`, `employment_type`, `fte_percent`, `department`,
`job_title`, `location`, `manager_pid`, `salary_minor`, `salary_currency`, `hired_on`, `terminated_on`, `engagement_ends_on`,
`created_at`, `updated_at`); each row then holds those fields and no others. An unknown name, an empty list or an empty name is `422`
with the allowed names; the serial `id` and `deleted_at` are not offered. Filters, paging (`x-total-count`), the policy, the
organization scope and the per-row salary mask are unchanged: a salary asked for by a caller the policy masks is `null`.

- **WPM-D147 A list is a salary read whenever it carries a salary, and is audited as the single read is.** For each listed worker whose
  salary the answer carries unmasked (no `fields`, or `fields` naming `salary_minor` or `salary_currency`), a `salary_read` row is
  written with the department and `"via": "list"`, no figure, all in one transaction before the answer is sent (WPM-D107: if the audit
  fails, the list is not given). A list that does not carry salary, or masks it, records none. Rejected: one summary row per list
  (it cannot answer "who read this person's salary" for one person); ignoring an unknown field name (a typo would silently widen the
  answer to the whole record or narrow it to nothing).

## What PPM was read to say (2026-10-28)

PPM's service (`project-portfolio-management-service-with-rust`) was read, not run.

1. **PPM has its own capacity model, parallel to WPM's.** `PUT /api/capacity-commitments` sets, for one of **PPM's** pools (`pool_pid`), a plan
   (`plan_pid`), an optional partner organization, a `kind` (supply or demand), a month (`YYYY-MM`) and `committed_fte` (a float, with an
   optional `delivered_fte`). `GET /api/capacity-heatmap` computes the pool-by-month view from them. PPM's pools are `skill_pools` with
   `roles` and `skills` as text tags, one role or skill in at most one pool. WPM's pools are different rows with different ids, built from
   role profiles and skills at a minimum proficiency, with FTE in centi-FTE and months as `YYYY-MM-01`.
2. **So "programme demand in" is not plumbing; it needs a decision.** Either WPM reads PPM's demand commitments and maps PPM's pool to its
   own (by what? a name is not an identity), or PPM stops keeping supply and asks WPM, or the two views are shown side by side and
   labelled. Nothing is built (WPM-D136); the choice is the owners'. The risk to name is two numbers for one pool.
3. **PPM calls a worker-service route that WPM does not have.** `GET {base}/api/workers/{id}/skills` expecting `{"skills": ["tag", ...]}`
   (`src/workers_client.rs`, for its skill-gap view). PPM's own notes say the route did not exist; it still does not on WPM (WPM has
   `/api/workers/{pid}/skills-as-of`, a different shape). Without it PPM reports `unknown`, which is honest. A route would have to return
   skills as tags WPM never uses (WPM skills are catalogue entries with proficiency), so adding one is another mapping decision.
4. **PPM has no inbound route for people-impact, and nothing that calls WPM's what-if.** Only `POST /api/ideas` was found as an inbound
   contract (title, pitch, tags). T361 has nowhere to send to. *(Settled 2026-10-30: people-impact is a pull PPM makes of WPM; see the
   family contract.)*
5. **PPM names its plans `plan_pid` and its programmes by `programme`-labelled plans**; WPM's `programme_ref` is a `thing:` URN. No mapping
   between the two was found. *(Settled 2026-10-30: `thing:<ppm plan pid>`; see the family contract.)*

## Tasks for the harmonization (WPM-R213)

- [x] WPM-T363 `skill_pools.peer_pool_ref` and its set/clear endpoint (HR, audited without names or figures); the link is shown with the pool.
- [x] WPM-T364 `programme_demands.source` (`local` default, `ppm`), the peer event id and received time; idempotent receive; the view and what-if show demand by source and never sum across sources.
- [x] WPM-T365 Outbound supply for a linked pool: floored counts by month, to the caller's token.
- [x] WPM-T366 A test that WPM answers every capacity request with the peer unset, down and slow, and that a link removed returns the local figures.
- [x] WPM-T387 The peer withdraws a figure with `fte_centi: 0`, and an older event (`sent_at`) does not overwrite a newer one (WPM-R215, WPM-D146).

