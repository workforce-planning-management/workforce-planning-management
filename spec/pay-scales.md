# Pay scales: NHS Agenda for Change, Wales (WPM-R51)

Wales runs the NHS **Agenda for Change** (AfC) pay and grading system, and
issues its own pay circular each year. WPM carries the Wales scale as
**reference data** and answers "where does this salary sit, and is a step up
due?". Delivered as WPM-T92 (see [tasks.md](tasks.md)).

## Research (2026-10-06)

Primary source: Welsh Government Health and Social Services Group, **pay letter
AfC(W) 02/2026**, 12 February 2026 (copies on the NHS Wales Employers site;
enquiries `HSSWorkforceOD@gov.wales`), Annex 1. It states:

- the 2026/27 scales apply **from 1 April 2026** and are the previous year's
  (AfC(W) 02/2025) **plus 3.3%**; the sleeping-in and on-call allowances rise 3.3%;
- Wales is **not** England's scale: the structure differs (below), and the
  separate advance payment in AfC(W) 01/2026 is excluded from the uplift;
- **bank workers** engaged under the All-Wales Terms of Engagement are paid the
  Welsh scales automatically (not modelled — WPM has no bank-worker concept).

| Band | Entry | years | Intermediate | years | Top |
| --- | ---: | :-: | ---: | :-: | ---: |
| 1 (closed) | £26,300 | | | | |
| 2 | £26,300 | | | | |
| 3 | £26,300 | 2 | | | £27,890 |
| 4 | £28,819 | 3 | | | £31,626 |
| 5 | £32,557 | 2 | £35,114 | 2 | £39,631 |
| 6 | £40,559 | 2 | £42,805 | 3 | £48,841 |
| 7 | £50,129 | 2 | £52,712 | 3 | £57,365 |
| 8a | £58,379 | 2 | £61,317 | 3 | £65,723 |
| 8b | £67,583 | 2 | £71,952 | 3 | £78,530 |
| 8c | £80,698 | 2 | £85,611 | 3 | £92,984 |
| 8d | £95,773 | 2 | £101,643 | 3 | £110,448 |
| 9 | £114,475 | 2 | £121,377 | 3 | £131,732 |

"years" is the circular's *years until eligible for pay progression* from that
step. Band 1 is closed to new entrants; band 2 is a single rate. Allowances:
sleeping-in £44.82; Wales on-call £26.05 (weekday or weekend) and £52.08 (public
holiday). Annual figures are **full-time, 37.5 hours a week**.

Secondary sources (pay-calculator sites) were used only to find the circular;
their band minimum/maximum agree with it, but **their structure does not** (they
omit the intermediate step), so the circular is the only source of the figures.

## WPM-R51 — Pay scales

*As anyone signed in I can see the AfC scale for Wales, and check where a
salary sits on a band and whether a step up is due.*

- `GET /api/pay-scales` lists the scales (id, nation, effective date, uplift,
  source circular, band codes). `GET /api/pay-scales/{id}` returns the whole
  scale: every band, step (pence) and years-to-progression, and the allowances.
  Id today: `afc-wales-2026-27`. An unknown id is **404**.
- `GET /api/pay-scales/{id}/position?band=&salary_minor=[&step=&months_on_step=]`
  places a **full-time-equivalent annual** salary on a band: `on_step`,
  `between_steps`, `below_entry` (with the shortfall) or `above_top` (with the
  excess). With `step` and `months_on_step` it also says whether progression is
  `due`, `not_yet` (months remaining) or `at_top`, with the next step's pay.
  Band codes are case-insensitive. Missing band, unknown band, nothing asked,
  negative salary, `step` without `months_on_step` (or the reverse) and a step
  not on the band are **422**.
- Money is minor units; the week is whole minutes (2250), never a float.
- UI: `/pay-scales` — the table (entry, intermediate, top, with years; band 1
  marked closed), the allowances, and the lookup form.
- **Reference data.** A new year is a new scale added in
  `src/rules/pay_scale.rs` with its circular named in `source`; the old scale
  stays so past figures remain checkable.

## WPM-D38 — A scale is transcribed, dated and sourced; a lookup stores nothing

- The figures are **transcribed from the circular**, never derived by applying a
  percentage to last year's, so a rounding rule cannot make WPM disagree with the
  employer. Each scale carries `effective_from` and `source`.
- The lookup is **stateless**: nothing is stored against a person, so it adds no
  table, no export or erasure wiring and no salary masking surface. A caller who
  asks about a worker has already read the salary through the audited, maskable
  `GET /api/workers/{pid}`.
- A salary **between** steps is reported as such, not rejected: protected pay and
  pro-rata figures are real. The caller normalises part-time pay to full-time
  equivalent first; the response states the basis.
- **Eligibility is only the circular's years on the step.** Whether the
  employer's own conditions for progression are met is not modelled, and the
  wording says "eligible", never "will be paid".

## Not done

- Assigning a band and step to a worker, with a progression date and a reminder
  — it would store a pay position per person (masking, export, erasure) and is a
  separate decision.
- Linking roles or requisitions to a band; flagging a salary that is off-scale.
- England, Scotland and Northern Ireland scales (each has its own circular).
- Bank-worker hourly rates; hourly rates from the annual figure (the divisor is a
  contractual choice).
- Applying the on-call allowance to the rota ([people-directory-and-cover.md](people-directory-and-cover.md)).
