# Joiners and leavers (WPM-R45–R46)

People joining and leaving the organisation: a record with a dated checklist,
and — for a leaver — a last-day handover with an audit trail. Delivered as
WPM-T89. (The legal gate for activating a worker is the existing onboarding
checklist, WPM-R3; this is the dated, owned schedule around it.)

## WPM-R45 — Joiner and leaver records with dated checklists

*As HR I can open a record for someone joining or leaving and see, item by
item, what is due, what is late and who is responsible.*

- `POST /api/workers/{pid}/movements`: a **joiner** (start day, default the
  hire date) or a **leaver** (last day required, and a reason — resignation,
  redundancy, retirement, end of contract, dismissal, other; the person must
  still be employed). At most one open record of each kind per person.
- The **checklist** is built from a standard template with every item dated
  relative to the effective day. Joiner: −7 contract and right-to-work,
  −5 equipment, −2 accounts, 0 welcome, 0 buddy, +5 first-week check-in,
  +30 review. Leaver: −28 notice, −14 handover plan, −7 knowledge transfer,
  −3 exit interview, 0 equipment returned, 0 access revoked, 0 tasks
  reassigned, +5 final pay. People items go to the manager; any item can be
  assigned, added, reopened.
- Each item is `done`, `skipped` (with a reason), `overdue`, `due_today` or
  `upcoming`; the list shows progress and the overdue count.
- A record **cannot be completed** while checklist items are open or (for a
  leaver) anything they hold is unassigned; the 422 names what is left. A
  completed record takes no more changes.
- `GET /api/movements?kind=&status=`, `GET /api/movements/{pid}`,
  `POST …/items`, `POST /api/movement-items/{pid}/done|skip|reopen|assign`,
  `POST …/complete|cancel`; `GET /api/workers/{pid}/movements`.

## WPM-R46 — Last-day handover

*As HR, on a leaver's last day I can list everything they still hold and
reassign it, with a record of who did what.*

- `GET /api/movements/{pid}/handover` lists what the leaver holds as of the
  last day. **Ownerships:** direct reports, dotted-line reports, groups they
  lead, their seat in an on-call rota, mentorships they give. **Bookings:**
  future shifts, on-call swaps, being named as someone's backup. **Tasks:**
  checklist items assigned to them on others' records. **Access:** their
  organization roles.
- `POST …/handover` reassigns or closes one item. A direct report **needs** a
  new manager (the org-chart cycle check applies); the new holder must be an
  employed worker of the same organization and not the leaver; **access can
  only be revoked** (ended on the last day), never handed to a person
  (WPM-D34). `POST …/handover/all` hands everything handable to one
  successor and revokes access, reporting anything that could not move.
- **Audit trail:** each action is one transaction — the change, a
  `handover_actions` row (kind, thing, from, to, action, note, who, when), an
  audit entry and a `handover_received` notification to the new holder.
  `GET …/handover/actions` lists them oldest first.
- Records and handover actions are in the subject-access export; erasure
  scrubs notes and unassigns the person's tasks but **keeps the audit rows**.
- UI: `/movements`, `/movements/{pid}`.

## Design decisions

- **WPM-D34 — A handover is a person's act, recorded.** Nothing moves by
  itself on the last day; HR (or whoever may write the record) reassigns, and
  every action is audited. Access is a role, not a possession: it is revoked.
  Completion is gated on nothing being left, so a leaver cannot be "closed"
  with live ownerships behind them.
