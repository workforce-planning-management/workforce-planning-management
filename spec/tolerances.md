# Tolerances

How much a workforce plan may drift before someone has to decide. A **tolerance** is the largest deviation a plan is
allowed on one dimension; the plan shows **red, amber or green** for each dimension and for the plan as a whole. The idea
is the project-management one (a plan has a permissible variation in time, cost, risk and resources, and a breach goes to a
person with the authority to accept it or change course); WPM measures the deviation and says where it stands. It does not
decide or block anything. Taking a breach to a person who can decide is [escalation](#escalations) (WPM-R141), which is
raised by a person, never by the measurement.

## WPM-R140 — A plan has tolerances, and the plan shows where it stands against each

- **Four dimensions**, each measured from what WPM already works out for the plan (WPM-D93):

  | Dimension | Deviation | Tolerance unit |
  | --- | --- | --- |
  | `time` | the most calendar days any demand line with an unmet headcount gap is past its target date | calendar days |
  | `cost` | the projected annual cost of closing the gaps above the plan's budget, as a share of the budget | basis points |
  | `resources` | the largest headcount gap on any demand line, as a share of that line's planned demand | basis points |
  | `risk` | the critical roles in the plan's departments with no successor ready (the shared single-point-of-failure rule) | roles |

- **Set per plan:** `PUT /api/workforce-plans/{pid}/tolerances/{dimension}` with `limit` (a whole number, `0` or more; `0` means
  any deviation is a breach) and optionally `amber_percent` (1 to 99, default 75). `DELETE` takes it away. Audited.
- **Read:** `GET /api/workforce-plans/{pid}/tolerances` lists every dimension, set or not, with the deviation, the limit, the
  status and the basis, and an `overall` that is the worst status.
- **Status** (`rag`), per dimension: `green` (the deviation uses less than `amber_percent` of the tolerance, or there is none),
  `amber` (it uses `amber_percent` or more, up to and including the limit), `red` (it is over the limit), `unknown` (the deviation
  cannot be worked out: no budget, no attrition assumption, no succession plans, salaries not visible to this caller), `not_set`
  (no tolerance has been set for this dimension).
- **Overall:** `red` if any dimension is red; else `amber` if any is amber; else `unknown` if any set dimension is unknown;
  else `green` if at least one is set; else `not_set`. **An unknown is never green** (WPM-D93).
- **Colour is not the only signal.** The page shows the word beside the colour and the numbers behind it.
- **Cost is salary-derived**, so a caller who cannot read salaries sees `unknown` for it, with the reason, not a hidden number.

## Design decisions

- **WPM-D93 — A status says what it rests on, and an unknown is not a green.** The deviation is read off the plan's own
  forecast, cost and alignment views (one definition each, not recomputed), the payload carries the number, the limit and a
  sentence of basis, and a deviation WPM cannot work out is `unknown` and ranks above green in the overall: a plan with no budget
  has not shown it is within its cost tolerance. Amber is a share of the tolerance used, so a looser tolerance does not
  move the warning point in absolute terms by accident. Rejected: a score or traffic light across plans (roll-ups of plans are for a later
  requirement; WPM-D32 keeps aggregates from naming people, and a plan's gaps are by department, not by person); automatic escalation
  (WPM does not know who has the authority); tolerance on dimensions with no measure here (quality, benefits) until there is something to measure.

## Escalations

A tolerance says a plan has drifted; an **escalation** is a person taking that to someone who can decide. It is a record of the
question and the answer: what was breached, how far, who raised it, how far up it went, and what was decided, by whom and when.

## WPM-R141 — A breach can be escalated, and the decision is recorded

- **Raise:** `POST /api/workforce-plans/{pid}/escalations` with `dimension` and an optional `note`. It is refused (`422`, naming
  the status) unless that dimension has a tolerance and its status is **amber or red** now: amber is a plan forecast to breach, which is when to
  ask, and a green, unknown or unset dimension has nothing a measure can show. One escalation can be open per plan and dimension.
  It starts at level 1, `plan_owner`; the deviation, limit and status at that moment are stored with it.
- **Raise the level:** `POST /api/escalations/{pid}/raise-level` moves it up one: `plan_owner`, `board`, `executive`. It stops at the top.
- **Decide:** `POST /api/escalations/{pid}/decide` with `decision` (`accept_deviation`, `change_plan` or `stop`) and a `note` (required,
  at most 500 characters). Only an administrator of the plan's organization may decide, as for a frozen oversight pack. The decision, who made it,
  when, and the plan's status and deviation at that moment are stored; the escalation is closed. A decision does not change the tolerance or the
  plan: the person does that, and the audit trail shows both.
- **Read:** `GET /api/workforce-plans/{pid}/escalations` (open first, then decided, newest first), and `GET /api/escalations?organization=`
  for the open ones in an organization, oldest first, each with its **calendar days open**: an administrator's inbox. Both are limited to
  the organizations the caller can read.
- **The tolerances view says what is unanswered:** each dimension carries its open `escalation` (or `null`), and the plan
  carries `breaches_without_escalation`: red dimensions nobody has taken to anyone.
- **Told, not chased:** raising or raising the level sends an in-app notification to the organization's HR and organization administrators.
  Nothing is sent again, and nothing is raised on a schedule.
- Every step is audited (`escalation_raised`, `escalation_level_raised`, `escalation_decided`).

## WPM-D94 — An escalation is a person's act with the measure attached, and it keeps no one's name but the actor's

- The measure is stored at each step (deviation, limit, status) so the record says what the decider was looking at, even after the plan
  changes. It is not recomputed when read.
- **No automatic escalation.** WPM does not know who in a given organization has authority at which level, and a system that escalates by itself
  teaches people to ignore it. The level names are generic (`plan_owner`, `board`, `executive`); who sits at each is the organization's.
- **The note is short free text and can hold a name.** It is capped at 500 characters and the form says not to put personal details in it; it is
  never copied into a notification (the notification says which plan, dimension and level).
- **Deciding is for an administrator, raising is not.** Anyone who may write to the plan can say "this needs a decision"; closing the question is
  the authority's act. Rejected: a decision by role per level (the organization's levels are not modelled); a score of how late an answer is
  (days open is shown and left alone).
