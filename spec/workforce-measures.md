# Derived workforce measures (WPM-R67, WPM-T114)

A **derived** oversight measure is computed by the service from WPM data, one reading per calendar
month, every time the status is read. Nothing is stored, so a reading is never stale and never
needs evidence. A **reported** measure is the alternative: entered by a person with an evidence note
and a source document ([oversight](plan.md), WPM-R66). A derived measure names its computation in
`derivation`, and takes that computation's unit, so a threshold cannot be read in another.

`GET /api/oversight/derivations` returns the list below with each definition, so a consumer reads
the definition beside the number.

## Rules every derivation follows

- **An unknown is never a zero.** When the data cannot speak for a month the reading is `null`. An
  oversight criterion treats a `null` as not met (it stops the run), so an unknown is never a pass.
- **Aggregates name no one.** Turnover and the contingent share are withheld below the aggregate
  floor (`DEFAULT_FLOOR`, 10), the same floor as the equality and contingent views.
- **Dated or not at all.** A derivation exists only if its inputs carry a date, so a past month can
  be computed. Figures that are only true today are not filed under an earlier month.
- **Whole calendar months.** The period of every reading is one calendar month, read as of the last
  complete month; the current month is never counted.

## Built (WPM-T114)

| Key | Unit | Derivation |
| --- | --- | --- |
| `turnover_rate` | basis points | Leavers in the calendar month, doubled and divided by the sum of the headcount the day before the month and on its last day (the metrics layer's turnover rate for one month). Employed workers of the organization; soft-deleted records left out. Absent below the floor, or with nothing to divide by. |
| `contingent_share` | basis points | Contractor plus fixed-term FTE as a share of all FTE, from the last headcount snapshot dated in the month. Absent with no snapshot in the month, with any department row from before the split by basis, below the floor, or with no FTE. |
| `overcommitted_pools` | pools | How many skill pools are `over` in the month: short of the demand from active programme claims even with firm partner capacity ([delivery-capacity](delivery-capacity.md)). Absent with no pool, or when any pool-month cannot be said because a partner has not answered (a count of the pools known to be over would be a lower bound, and a lower bound passes an at-most threshold). Computed from the supply, claims and leave on file now, so a later correction changes a past month. |
| `time_to_report` | calendar days | The longest time from the first sweep that saw an early-warning condition to a person acknowledging it, over the warnings first seen in the month ([early warning](production-readiness.md), WPM-R68, WPM-D82). Absent with no warning first seen in the month, and absent while any of them is unacknowledged. |
## Not yet derivable: single points of failure and governance hours

Both are needed by WPM-R67, and both are computed today
(`/api/workforce-intelligence/key-person`, `/api/governance-commitments/load`). Neither is dated: a
holder or a commitment has no "as of" month, so last month's figure cannot be computed, and a figure
computed today and filed under last month would be an invention. They become derivable when the
monthly pack freezes them as of a date (WPM-T116). Until then a criterion that needs them uses a
reported measure.

## Candidate measures: defined here, not built

Each needs a decision before it is computed. The definitions are fixed now so the decision is about
whether to build, not about what is meant.

1. **Sickness absence rate** (`sickness_absence_rate`, basis points). Business days of **approved**
   sick leave falling in the month, divided by the business days in the month summed over the workers
   employed on each day (counted per person-day, not weighted by FTE). Requested, rejected and cancelled leave is left out. The service holds that leave is sick
   and its dates, never why (WPM-D73), so the rate says nothing about cause. Health-adjacent: shown
   only as an organization-wide aggregate above the floor, never by department or team below it, and
   absent when fewer people than the floor were employed. Open question: whether sick leave of
   part-time workers is counted in their scheduled days only.
2. **Anonymous pulse** (`pulse_favourable`, basis points). The share of responses to a named pulse
   question in the month that are on the favourable side of the scale, using the pulse's existing
   k-floor ([`rules::pulse`](../workforce-planning-management-service-with-rust/src/rules/pulse.rs),
   WPM-D20). Absent when the responses are below the floor, which discloses nothing, not even the
   count. Open question: which question and which side is favourable is a choice for the
   organization, so the measure must name the question rather than assume one.
3. **Training completion for required courses** (`required_training_completion`, basis points).
   Of the enrollments in courses marked required whose due date falls in the month, the share
   completed on or before the due date. Absent with no such enrollment. **Blocked:** no course
   carries a "required" mark and enrollments carry no due date today, so this needs a data change
   (and a decision on who marks a course required) before it can be computed.

## Not verified

- No measure was checked against real data; the request test uses synthetic workers, one snapshot
  pair and one pool.
- The capacity derivation was checked for an `over` pool and for no pool, not for a pool with an
  unanswered partner at the request level (the unit test covers the rule).
- The first-of-month period convention means a worker who leaves on the last day of a month is a
  leaver of that month and absent from its closing headcount; that follows the metrics layer.
