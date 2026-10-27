# Succession, deputies and acting up

Proposed 2026-10-27 ([plan.md](plan.md) section S; spec round WPM-T309, delivery Phase 24, WPM-T338–T344). Requirements
**WPM-R197–R205**, decisions **WPM-D128–D133**. Nothing here is built.

Three ways an organization keeps a role working when its holder cannot do it: a **successor** who could take it on for good, a
**deputy** who holds its authority while the holder is away, and someone **acting up** into it for a while. WPM has the first in a
basic form and nothing for the other two:

| | What exists | What this file adds |
| --- | --- | --- |
| Succession | WPM-R12 ([talent-development.md](talent-development.md)): a plan per key role (free-text title and department, criticality 1–5), ranked successors with readiness `ready_now`, `ready_1y`, `ready_2y`, risk of loss, bench coverage and the single-point-of-failure list; HR-only | Plans on real positions and role profiles, evidence beside readiness, emergency cover, review dates, the successor's own view, slate equality (R197–R200) |
| Cover for the work | WPM-R41 backups: who to ask when someone is out ([people-directory-and-cover.md](people-directory-and-cover.md)) | Unchanged. A backup covers **work**; a deputy holds **authority** |
| Deputies | Nothing; expense claims list "approval delegation" as not done ([expense-claims.md](expense-claims.md)) | Named deputies with scoped, dated authority (R201–R202) |
| Acting up | Nothing | Acting assignments, their pay, limits and fairness (R203–R205) |

## Succession

### WPM-R197 — Succession on positions and roles

- **A plan is for a position** (WPM-R152) **or a role profile** (WPM-T57) rather than a free-text title. A plan for a role profile
  covers every position with that role (for roles held by many, such as ward manager). Existing plans keep their text and are
  listed for HR to link; none is matched by guess.
- **The incumbent is derived** from the position's holding assignment (WPM-R161), not typed, so a plan cannot name someone who
  has left. A plan whose position falls vacant shows `vacant` and its successors move to the top of the list for the
  requisition.
- **Emergency cover** is a successor flag: who would step in tomorrow, even if not ready for good. A role with a `ready_now`
  successor but no emergency cover is shown as such; a role's **deputy** (R201) is offered as its emergency cover and shown
  beside it.
- **Critical roles** come from criticality, as now, and from the key-person register (WPM-D47) where a deployer keeps one; they
  are the same list, not two.

### WPM-R198 — Readiness with its evidence beside it

- **Readiness stays a judgement** recorded by a person (WPM-D128). Beside it the plan shows **evidence** the service derives,
  without turning it into a score: the successor's skill gap and qualification gap against the role's requirements (WPM-R43,
  WPM-R171), acting-up time in this or a similar role (R203), their latest PADR date (WPM-R172) and the development plan items
  linked to this succession.
- **Development for succession** is a development plan item (existing) linked to the plan, so it appears in the person's own
  development plan like any other item. It is phrased as development, not as "you are a successor".
- A readiness that disagrees with the evidence (for example `ready_now` with a critical skill gap) is shown with the evidence;
  nothing changes it automatically.

### WPM-R199 — Review dates, and the successor's own view

- **Each plan has a review date** (configured default in months). A plan not reviewed by then shows `not_reviewed` in the
  coverage view; coverage from a stale plan is reported apart, not as cover.
- **A named successor is told by the software** (WPM-D129). When the entry is saved, the person gets an in-app notification:
  "you have been named as a possible successor for <role>". They are also notified when their readiness or emergency-cover flag
  changes, and when they are removed from the plan. As with every notification, the body carries the role and the date and
  nothing else (WPM-D23).
  - **Nobody can switch this off.** No setting, role or plan marks an entry as confidential.
  - **Their own entry is in self-service** at `/api/me/succession`: the role, their readiness, whether they are emergency cover,
    the date each was set, and the development items linked to it. They do not see their rank, the other candidates, the
    incumbent's risk of loss or the expected vacancy date.
  - **It is in their subject-access export**, which today omits succession entirely, with the same fields.
- **Interest**: a person's mobility interests (existing) may name a role; HR sees, on the plan, people who have said they are
  interested in it and are not on it. Interest is the person's choice and is never shown to their manager (as now).

### WPM-R200 — The equality of slates

- **From voluntary equality monitoring only** (WPM-R125, WPM-D74), the composition of succession slates across critical roles
  compared with the workforce they are drawn from, per category, with the floor; `unknown` without monitoring. Never per plan:
  a plan's slate is too small.
- Succession readiness and risk of loss are scorings in the equality impact register ([equality-impact](equality-impact/register.md),
  WPM-R99); this view is the evidence that register asks for.

## Deputies

### WPM-R201 — Naming a deputy

- **A deputy is named for a position** (or, where a deployer does not use positions, for a worker's primary assignment): one or
  more, ranked, either **standing** (whenever the holder is away) or for a **dated window**.
- **A deputy must** be someone else, currently employed, in the same organization (or one the organization's confederation
  allows), and not report to the holder through the role being deputised (a deputy who approves their own manager's decisions
  for them is allowed; one who approves their own requests is not, R202).
- **Who names them**: the holder or HR; the deputy is told and **accepts** before the deputyship starts. Visible to anyone who
  can see the position's holder, like backups.

### WPM-R202 — Authority while deputising

- **Scopes** are a closed list a deputy is given, each on or off: `approve_leave`, `approve_time`, `decide_expenses`,
  `approve_training`, `sign_off_return_to_work`, `see_action_list`. Others are added by code with this spec, not by
  configuration.
- **When it applies**: a standing deputyship is active while the holder is on approved leave or absent (derived from leave and
  sickness episodes, never from a reason); a dated one in its window. The holder can also hand over for a day ("I am
  unavailable"), recorded and audited.
- **While active**, items in scope that would go to the holder go to the first-ranked available deputy (not themselves on leave,
  as backups resolve), and appear on their action list (WPM-R186) under the holder's name. Every decision is recorded **as
  deputy for** the holder, in the audit trail and on the record.
- **Never delegated** (WPM-D130): the words of adjustments, a passport or "working with me" (WPM-D126), stress risk assessments,
  occupational health referrals, PADR content and pay. A deputy sees that such an item is due and that it waits for the holder
  or HR.
- **No self-dealing**: a deputy never decides a request of their own, as a claimant never decides their own claim.
- **Deciding flexible working requests is an ABAC operation the holder may grant** (WPM-D133). It is not one of the scopes
  above. The operation is `flexible_working:decide`, and the policy evaluates it like any other decision, so a deployer
  configures who may perform it. For a deputy, the request carries the record-level attribute
  `resource.deputy_may_decide_flexible_working`, and the policy permits a deputy only when that attribute is `true`.
  - **The primary holder sets it**, per deputy, and may withdraw it at any time. The deputy can never set it for themself.
  - **HR may grant it on the holder's behalf while the holder is unavailable.** "Unavailable" means the holder is on approved
    leave, in an open sickness episode, or has recorded that they are unavailable that day. This is derived from those records,
    and never from a reason. The grant is recorded as made by HR on behalf of the holder, and the holder is notified.
    - **It lapses when the holder is available again.** On return, the holder sees it on their action list and either confirms
      it, which makes it their own grant, or lets it lapse. Requests the deputy decided in the meantime stay decided.
    - **HR cannot grant it while the holder is available**, and may withdraw a grant they made at any time.
  - **Every grant, confirmation, lapse and withdrawal** is recorded with who and when, and audited.
  - **The default is `null`, read as no.** A missing attribute, `null` and `false` all deny. Without the grant, a deputy sees
    that a request is due and its decide-by date, and the request waits for the holder, a manager up the chain or HR.
  - **The rules of the decision are unchanged.** The decide-by date is the same whoever decides, and a decision by a deputy is
    recorded as deputy for the holder. An appeal against a deputy's refusal is decided by someone other than that deputy.

## Acting up

### WPM-R203 — Acting assignments

- **Acting up** is an assignment (WPM-R161) of kind `acting`, into a position or a role at a higher band, with a start, a planned
  end and whether it is `full` (all the duties) or `partial` (named duties, as short text).
- **The substantive assignment is kept** and marked as on hold for the period, so the person returns to it; the FTE rule
  counts the acting assignment instead of the held one while it runs.
- **Into a position.** The position shows its acting holder beside its substantive holder (if any). **Acting cover does not fill a
  vacancy** (WPM-D131): a vacant position covered by acting up stays vacant in the establishment, shown as `vacant, acting cover`.
- **Ending.** On the planned end the acting assignment ends and the substantive one resumes. Extending is a recorded decision
  with a new end date; the history of extensions is kept (the engagement-extension pattern).

### WPM-R204 — Pay while acting up

- **The pay treatment** is configuration, stated on the acting assignment: placed on a step of the higher band (by the deployer's
  rule, as pay scale placement already computes), an allowance as a percentage of current pay, a fixed allowance, or none (for
  example below a qualifying period). The demo ships a synthetic rule, labelled synthetic; national rules (in the NHS, Agenda for
  Change's provisions for higher duties) are the deployer's to configure (WPM-D118).
- **On the payslip** the acting pay is its own earnings line, so gross shows it apart and it is pensionable or not by the scheme's
  stated rule (WPM-R176).
- **Back pay**: an acting assignment recorded late is paid from its start in the next run, shown as arrears, never silently.

### WPM-R205 — Limits and fairness

- **A configured maximum duration**: an acting assignment reaching it is flagged on HR's and the manager's action lists for a
  decision (`extend`, `end`, `recruit`). It is never made permanent by the software or by the date passing; making it permanent is
  a fair recruitment decision recorded by a person.
- **Who gets to act up** is counted for fairness (WPM-D132): from voluntary equality monitoring only, the share of acting-up
  opportunities by category against the eligible workforce, with the floor; `unknown` without monitoring. Also, as plain counts,
  how many acting assignments were filled with an expression of interest (open) and how many by direct appointment.
- **An expression of interest** can be opened for an acting opportunity: visible to the eligible audience (by band and area),
  applications recorded, the choice made by a person with a short recorded basis.

## Design decisions

- **WPM-D128 Readiness is a judgement shown with its evidence, never a computed score.** The service shows the gaps, the acting
  experience and the reviews beside the rating; a person sets the rating. A computed readiness would be a scoring of people that
  the equality impact register would then have to justify, and it would hide the judgement behind arithmetic.
- **WPM-D129 A successor is told, by the software, every time.** Being named is a judgement about the person's future. They
  are told when it is made, changed or ended, can see their own entry, and find it in their export. A conversation with their
  manager is still expected, but WPM does not wait for one, and there is no confidential plan. The plan's other parts stay
  HR-only: the rank, the other candidates, and the incumbent's risk of loss and vacancy date. Those are about other people, and
  the incumbent's are sensitive. Rejected: telling the person only through the subject-access export (they would have to ask
  in order to find out), and a per-plan "confidential" switch (it would become the default).
- **WPM-D130 A deputy holds named authority, not the manager's confidences.** Deputising is for decisions that cannot wait:
  leave, time, expenses, training, and signing a return to work. What a person shared with their manager in confidence, and
  pay, are never in scope, so deputising never widens who knows them.
- **WPM-D131 Acting cover does not fill a vacancy.** A covered post is still vacant; counting it as filled would hide the
  recruitment the organization still needs and the budget it is still carrying.
- **WPM-D132 Acting up is a door to promotion, so who walks through it is counted.** Acting opportunities given by direct
  appointment are a common route to unequal promotion; the counts of open and direct appointments, and the equality share from
  voluntary monitoring, make that visible without naming anyone.
- **WPM-D133 The holder, or HR while the holder is unavailable, lets a deputy decide flexible working, and the default is no.**
  A flexible working request is a statutory decision about someone's contract, with a deadline.
  - **The holder decides normally.** Whether a deputy may take the decision is the holder's choice, made per deputy, and the
    deployer's ABAC policy evaluates it as the operation `flexible_working:decide`.
  - **HR decides when the holder cannot.** A holder who is away cannot grant anything, and the deadline keeps running, so HR may
    grant on their behalf. HR's grant lapses on the holder's return, so it never outlasts the absence that justified it.
  - **The default denies.** A `null` or missing grant means a deputy who was never given it cannot decide one.
  - **Rejected:** letting the deputy turn it on (no one grants themself authority); letting HR grant it while the holder is
    available (the holder answers for decisions taken in their name); and putting it among the closed scopes (those are on
    whenever a deputy is given them, and this one needs an explicit choice).
