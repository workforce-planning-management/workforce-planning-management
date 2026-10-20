# Tolerances

How much a workforce plan may drift before someone has to decide. A **tolerance** is the largest deviation a plan is
allowed on one dimension; the plan shows **red, amber or green** for each dimension and for the plan as a whole. The idea
is the project-management one (a plan has a permissible variation in time, cost, risk and resources, and a breach goes to a
person with the authority to accept it or change course); WPM measures the deviation and says where it stands. It does not
decide, escalate or block anything.

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
