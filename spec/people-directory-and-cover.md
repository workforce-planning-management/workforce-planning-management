# People, cover and on-call (WPM-R39–R42)

Who works where, who to call, who covers when someone is out, and who is on
call. Delivered as WPM-T74–T81 (see [tasks.md](tasks.md)); stack detail in
each subproject's `spec/`.

## WPM-R39 — Employee directory

*As anyone who can read an organization I can find a colleague by name,
title, department, location, manager or rota — and see who is away and who
covers for them.*

- `GET /api/directory?q=&department=&limit=&offset=`: **employed today**
  ([`is_employed_on`](glossary.md#employed), the one definition) workers in the
  caller's organizations, ordered by name then pid so pages are stable,
  `x-total-count` set. Every whitespace-separated term of `q` must match
  (case-insensitively) the name, title, department, location, manager's name
  or the name of a rota the person is on call for.
- **Nothing sensitive**: no salary, employment dates or person reference
  (WPM-D29). A manager is named only if in the same readable set.
- A worker on **approved leave today** is `away_today` — never the *kind* of
  leave or any reason — with `covered_by`: their best-ranked available backup
  (WPM-R41), or null when nobody can (said plainly, not guessed). Anyone on
  call today is marked with the rota's name.
- UI: `/directory` (debounced search, department picker, chips for away /
  covered by / on call).

## WPM-R40 — Emergency contacts

*As a person I can say who to reach if something happens to me.*

- Up to **5** contacts per person, ranked (1 = first to call): name,
  relationship, a dialable phone (5–15 digits), optional alternate phone,
  email and note.
- **Third-party personal data** (WPM-D30): only the person and whoever may
  write their record (HR) can read or change them — a manager cannot — and
  audit entries record *that* a contact changed, never its details.
- In the subject-access export; deleted on erasure.
- `GET|POST /api/workers/{pid}/emergency-contacts`,
  `PUT|DELETE /api/emergency-contacts/{pid}`. UI panel on `/me` and
  `/workers/{pid}` (renders nothing for anyone who may not see it).

## WPM-R41 — Backups (cover)

*As a person I can name the colleague(s) who cover for me when I am out sick
or on leave.*

- Up to **3** backups, ranked (1 = first to ask), optionally for a dated
  window; a backup must be someone else, **currently employed**, in an
  organization the caller can read, and not named twice. Visible to anyone
  who can see the person; edited by the person or HR.
- `GET /api/workers/{pid}/cover?on=` resolves who covers on a day: the
  best-ranked backup whose window includes it, who is employed and **not on
  approved leave**; ties go to the lower pid; `covered_by` is null when nobody
  can (WPM-D31). Pure `rules::cover`.
- `GET|POST /api/workers/{pid}/backups`, `PUT|DELETE /api/backups/{pid}`.

## WPM-R42 — On-call rota

*As a team lead I can run an on-call rotation; as a team member I can see my
turns, swap them, and be reminded.*

- A **rota** is a named rotation in one organization: members in order, a
  period of 1–31 calendar days, a start date; the duty passes to the next member every
  period, wrapping. Not the shift day-view (`GET /api/shifts`, WPM-R6).
- **Overrides** (swaps): a worker on call for a date window regardless of the
  rotation; a later override wins on shared days.
- **Leave-aware:** a scheduled member on approved leave (or no longer
  employed) is **skipped** to the next available member in order; the stretch
  says so (`source`: `rotation` / `skipped` / `override`). If nobody can take
  a day it is unassigned (WPM-D31).
- **Swap requests:** a person on call asks a colleague to take their days in a
  window; the colleague accepts or declines, the requester can cancel
  (decided once). On acceptance **only the requester's own on-call stretches**
  inside the window become overrides for the colleague.
- **Notifications** (in-app, reference-only, WPM-D23): `rota_added`,
  `on_call_swap`, `swap_requested`, `swap_decided`, and `on_call_reminder`.
  The reminder is the loco task `rota_reminders [days_ahead:0–14]
  [as_of:YYYY-MM-DD]` — idempotent, tells whoever's turn **starts** that day
  (on call then, not the day before); **schedule it daily**.
- Members must be employed workers of the rota's organization; rotas outside
  the caller's organizations do not exist to them. Membership, swaps and swap
  requests are in the subject-access export and removed on erasure.
- API: `POST|GET /api/rotas`, `GET|PUT|DELETE /api/rotas/{pid}`
  (`?from=&to=`, at most 92 calendar days), `GET …/on-call?on=`, `POST …/overrides`,
  `DELETE /api/rota-overrides/{pid}`, `POST|GET …/swap-requests`,
  `POST /api/rota-swap-requests/{pid}/accept|decline|cancel`,
  `GET /api/workers/{pid}/on-call`. UI: `/rota`, "My on-call" on `/me`.

## Design decisions

- **WPM-D29 — The directory is deliberately narrow.** A listing anyone in an
  organization can open must carry nothing that is only the person's or HR's:
  name, title, department, location, organization, manager's name, away /
  covered-by / on-call flags. Being away is shown; *why* is not (sick leave is
  health data).
- **WPM-D30 — Emergency contacts are other people's data.** They are visible
  only to the person and HR, audited without detail, exported with the person
  and deleted with them. A contact has no field for anything beyond how to
  reach them.
- **WPM-D31 — Cover and on-call say "nobody" rather than guess.** Resolution is
  pure and deterministic; an unavailable member is skipped *visibly*; a day
  with no available person is reported as such.
