# Pay transparency

What the EU Pay Transparency Directive (2023/970) asks of an employer, and what WPM does about it: the pay gap
indicators an employer must report (WPM-R144), what a worker or a candidate may ask about pay (WPM-R145), and the signal
that a joint pay assessment is due (WPM-R146). Proposed in [plan.md](plan.md) section N; the facts below were checked
against the Directive's text and the Commission's guidance in the spec round (WPM-T268, 2026-10-24: see
[what was checked](#what-was-checked-and-what-was-not)).

> ⚠️ **Not legal advice, and not a filing.** WPM computes and shows the indicators from the data it holds. It submits
> nothing to any authority, and a figure here is not a statement that an employer meets a national law. The Directive
> reached national law unevenly (see [where it stands](#where-the-directive-stands)), so every date, size band and
> threshold below is **configuration whose default is the Directive's own value**, and a deployer sets what its
> national law says.

## Where the Directive stands

- **Transposition was due 7 June 2026** (Article 34). On 30 June 2026 only four of the 27 Member States were reported to
  have transposed it in full (Slovakia, Italy, Lithuania, Malta, with the sources disagreeing on Lithuania); others
  transposed in part (for example recruitment transparency only) or said they would phase it. This was read from trackers
  and law-firm notes, not from each national law, and it moves; **re-check the deployer's own country before relying on any date.**
- **First reports** (Article 9): employers with 250 or more workers report **every year**, from 7 June 2027; employers with
  150 to 249 workers report **every three years**, from 7 June 2027; employers with 100 to 149 workers report **every three
  years**, from 7 June 2031. Smaller employers have no reporting duty under the Directive (a Member State may add one). The
  report covers the previous calendar year. A Member State may require more.
- The Commission services say their FAQ states preliminary views and that only the Court of Justice interprets the Directive.
  That caveat applies to this file as well.

## WPM-R144 — Pay gap indicators, overall and by category of workers

The seven items of Article 9(1), as the Directive words them (point letters as in the text):

| Article 9(1) | Indicator | Level |
| --- | --- | --- |
| (a) | The gender pay gap | The whole workforce |
| (b) | The gender pay gap in complementary or variable components | The whole workforce |
| (c) | The median gender pay gap | The whole workforce |
| (d) | The median gender pay gap in complementary or variable components | The whole workforce |
| (e) | The proportion of female and of male workers receiving complementary or variable components | The whole workforce |
| (f) | The proportion of female and of male workers in each quartile pay band | The whole workforce |
| (g) | The gender pay gap between workers, by **category of workers**, in ordinary basic salary and in complementary or variable components | Each category |

- **"Pay" includes the variable part.** The Directive defines pay (Article 3(1)(a)) as the ordinary basic or minimum wage or
  salary and any other consideration, in cash or in kind, that a worker receives from the employer for the employment. So
  *complementary or variable components* (a bonus, an allowance, a payment in kind) belong to the indicators above. **There
  is no separate "share receiving benefits" indicator**: the first draft of WPM-R144 listed one, and the spec round took it out.
- **The gap** is the difference between the average pay of female and male workers, as a share of the average pay of male
  workers. WPM shows the two averages (or medians) and the gap, and the number of workers behind each, so a reader can redo it.
- **Computed in the pure core** (`rules::pay_gap`, WPM-T269, `rules::pay_category`, WPM-T270) from what WPM already holds: the salary on the worker record, the
  components on payslips ([payroll-compensation.md](payroll-compensation.md)) and the sex the worker has chosen to declare under equality monitoring (WPM-R125,
  WPM-D74, off until a deployer records a lawful basis). **WPM does not hold what it is not given**: a pay component it has no
  record of is `unknown` in that indicator, not zero (WPM-D99).
- **Who counts.** Only workers with a declared sex of female or male enter the indicators, because that is what the Directive
  compares. Everyone else is **excluded and counted** (WPM-T269, `rules::pay_gap::Exclusions`), each worker under the first reason
  that applies: no sex declared, a declaration other than female or male, or no basic pay recorded; and, still in the basic-pay
  figures, a variable component not recorded, and no category. Nothing is inferred from a name or a title.
- **Total pay is basic plus variable**, so a worker whose variable component is unknown is in the basic-pay figures and the
  per-category basic gap only, and is counted as such. **A gap in variable components** is taken, by the deployer's choice
  (`VariableBasis`), over every worker whose variable pay is known with none counting as zero (the default), or over recipients only;
  the Directive's text for this was not read (open item). The share receiving a component does not depend on that choice. The
  per-category gap (g) is the mean gap, and a median per category is not computed.
- **The quartile bands** are four equal groups of workers ranked by total pay, lowest first; a tie across a boundary is split by the
  worker key, so the same data gives the same bands in any input order.
- **Categories of workers** are the employer's own grouping of roles that do the same work or work of equal value. They are
  **configuration with a stated basis** (`pay_categories`, WPM-T270): a name, the **basis** in the employer's words (required, 10 to 2,000
  characters: the criteria used, such as skills, effort, responsibility and working conditions, which the Directive requires to be
  objective and gender-neutral), the **job titles** in it (1 to 50, matched to the worker record's `job_title` ignoring case and
  spacing, a title in one category per organization), and who last set it. `POST`, `PUT` and `DELETE /api/pay-categories` write them
  (audited, no personal data), `GET /api/pay-categories?organization=` reads them. **WPM never builds a category from titles**:
  the employer lists them. *(The first draft named role profiles; a worker has no role profile link yet, so titles are what can be
  matched, as benchmarking does.)* A worker whose title is in no category is counted as such, not guessed.
- **The floor.** A cell with fewer workers than the floor on either side (female or male) is **withheld**: no figure and no count.
  A cell with nobody of one sex is `unknown` (a gap needs two sides), and a cell with nobody at all says so. **No figure in the
  report is a head count**, and a band's composition is shown only when both sexes reach the floor in it, so a withheld cell cannot be
  worked out from the others. An exclusion count (below) under the floor shows only as "under the floor". The floor can never be
  below the equality monitoring minimum of five (`equality_monitoring::MIN_FLOOR`), and its default is that module's default of ten
  (WPM-D32, WPM-D99).
- **Period and basis.** A report is for a stated reporting period (the previous calendar year by default) and says what its pay
  is made of and how a part-time worker's pay is put on a comparable basis (the basis is recorded with the report). *The
  Directive's own rule on that basis (for example gross hourly pay) was not read in the spec round and is an open item for
  WPM-T269.*
- **Reporting tier.** From the organization's headcount WPM says which tier of Article 9 applies and when the next report is due,
  using the configured dates. It reminds; it does not file.
- **Read** (WPM-T270): `GET /api/pay-transparency/gap-report?organization=&as_of=&currency=&variable=` and the same as CSV at
  `/gap-report/csv`. Both are audited (`pay_gap_report_read`, `pay_gap_report_exported`: the day and how many categories, no
  figure) and name no one. **Who:** a caller who may read salaries unmasked, as for the plan cost view (payroll, administrators);
  **HR, who reads salaries masked, is refused with a reason**, because a mean of salaries is made of them. (The first draft said
  "HR only"; the policy decides it this way.) A caller outside the organization's scope gets a `404`.
- **Where the data comes from.** Pay is the **annual full-time-equivalent basic salary** on the worker record (not scaled down for
  part-time, standard hours taken as equal), in one currency (the `currency` asked for, else the most common among workers with a
  salary); pay in another currency counts as "no pay recorded". Sex is the answer to the equality category named by
  `WPM_PAY_GAP_SEX_CATEGORY` (default `sex`; values `female` and `male`, anything else is left out), so **the report does not exist
  (`422`) where equality monitoring is off**. **WPM holds no bonus, allowance or payment in kind**, so every variable component is
  unknown and (a) and (c) (total pay), (b), (d) and (e) are `unknown` until a source exists; (g) in basic pay is the one gap that
  can be shown today. The report says so in its `basis`. A category the employer set up that has nobody eligible is listed as
  `unknown`, not dropped.
- **The privacy guard moves, narrowly.** The equality answers were readable only by the worker and an aggregate summary; the pay gap
  report is the one other reader (the test `no_other_code_reads_the_declarations` names it), because comparing two groups is the
  point of the Directive, and it decides nothing about a person.

## WPM-R145 — Pay information for the worker, and none asked of the candidate

- **A worker may ask** for their own pay level and for the average pay level, broken down by sex, of the category of workers
  that does the same work or work of equal value (Article 7). `POST /api/me/pay-information-requests` records the request with its
  date and a **computed due date**: the Directive's period for the answer is "a reasonable period and in any event within
  **two months**" of the request, kept as a configured number of calendar months (default 2) so a national law can change it.
  The due date is derived, shown to the worker and to HR, and a request past it is flagged, never silently closed.
- **HR answers on a screen** that shows the worker's own figure and the category average by sex, with the floor: where fewer than
  the floor of one sex are in the category, the average for that sex is withheld and the answer says so (WPM-D99). The answer is
  recorded (who, when); the figures are not stored in the request, only that they were given.
- **A worker is told they may ask**, in the sense of a notification at the start of each year (the Directive has the employer
  inform workers of the right each year). It is a notice, not a request. *(Proposed; WPM-T271 decides whether the notice is in or
  out of its first slice.)*
- **A candidate gets the pay, not asked for theirs.** Article 5 gives an applicant the right to the initial pay or its range,
  based on objective gender-neutral criteria, in the vacancy notice or before the interview, and forbids the employer to ask
  about their pay history. In WPM: a **requisition cannot move to open** without a pay range (minimum and maximum, or a
  stated single figure); the candidate record has **no pay-history column** and a guard test fails the build if one is added
  (WPM-R145; compare "what must not be stored gets no column"). WPM publishes no advert itself, so the range is what an
  outbound feed or posting reads.

## WPM-R146 — A joint pay assessment is due

- **When.** Article 10(1): where the report shows an average pay gap of **at least 5%** between female and male workers in a
  category of workers, **and** the employer has not justified the gap on objective, gender-neutral criteria, **and** has not
  remedied it **within six months** of the date of reporting, the employer carries out a joint pay assessment with the workers'
  representatives. Both the 5% and the six months are configuration with these defaults; the Commission is to review the 5%
  trigger by 7 June 2033, as the Directive's review clause is summarised on EUR-Lex.
- **What WPM does.** The pure rule (WPM-T272) takes the indicators, the threshold, the period and the recorded justification
  and reports `due`, `not_due` or `unknown` per category; `unknown` where the gap itself is withheld or unknown. It reports; **a
  person records** whether the gap was justified, whether it was remedied, and the assessment's outcome, with the date. It links to
  the recognised body for that organization ([worker-unions.md](worker-unions.md), WPM-R102–R108) so the workers'
  representatives are the ones the organization already recognises. It never concludes that anyone was paid unfairly.
- **Not done.** No individual pay is compared, no remedy is calculated and no worker is told they are in an affected category by
  the service.

## Design decisions

- **WPM-D99 — A pay gap is withheld, never guessed.** A cell under the floor shows "withheld"; a worker with no recorded
  female or male sex is excluded and the exclusion counted; an indicator with too few workers on either side, or a pay component
  WPM does not hold, is `unknown`. Categories of equal value are configuration with a stated basis, never inferred from titles.
  Rejected: filling a missing component with zero (it would invent a gap), inferring sex from a name, and showing a figure for a
  group of one "because the law asks for it" (the Directive's own text leaves protection of personal data in place).

## What was checked, and what was not

Checked on 2026-10-24 (WPM-T268): the seven indicators and the reporting sizes and dates (Article 9, from the Directive's EUR-Lex
text as search snippets and the Commission's FAQ); the 5% and six months (Article 10(1)); the two months (Article 7); the ban on
asking pay history and the pay range for applicants (Article 5, from the Commission's FAQ); the transposition date (Article 34);
and the definition of pay (Article 3(1)(a), from the FAQ).

**Not checked from the full primary text:** EUR-Lex refused scripted access (a bot challenge, not bypassed), so the Directive was
read as search-engine excerpts of its EUR-Lex text plus the Commission's FAQ PDF, not as a whole document. Not confirmed: the exact
paragraph numbers inside Articles 5 and 7; the Directive's rule on how pay is put on a comparable basis for the indicators; any
national transposition (none was read); and how a country defines "category of workers" in its own law. WPM-T269 and WPM-T270
start by reading those from the Official Journal.

## Sources

- Directive (EU) 2023/970, [EUR-Lex](https://eur-lex.europa.eu/eli/dir/2023/970/oj/eng).
- European Commission, *Frequently asked questions on the Pay Transparency Directive (2023/970)* (preliminary views of the
  Commission services).
- Council of the EU, [pay transparency](https://www.consilium.europa.eu/en/policies/pay-transparency/).
