# Statutory pay protections: minimum wage, overtime, workers' compensation, and unemployment insurance

What the law asks of an employer on four protections that touch pay, what WPM holds and does about each today, and what we advise
WPM does next. Researched, assessed and advised on 2026-11-01 (WPM-T410). **Nothing here is built.** Proposed in
[plan.md](plan.md) section T; requirements WPM-R218 to WPM-R223; decisions WPM-D154 to WPM-D159 (**proposed by the AI assistant, not
yet confirmed by the product owner**); tasks in [tasks.md](tasks.md) Phase 26 (WPM-T410 to WPM-T425).

> ⚠️ **Not legal advice, and not a filing.** The facts below come from web search excerpts, many of them from law firms and payroll
> vendors rather than the law itself, and several could not be confirmed at all (see
> [what was checked](#what-was-checked-and-what-was-not)). Every figure is a **dated parameter that a person confirms against the
> primary source** before WPM relies on it, the same rule as [pay-transparency.md](pay-transparency.md) and WPM-T404. WPM shows and
> warns from the data it holds. It decides no entitlement, pays no benefit and files nothing with any authority.

## Summary and advice

| Protection | What the employer must do (where researched) | What WPM holds today | Advice |
| --- | --- | --- | --- |
| **Minimum wage** | Pay at least the legal hourly floor for the worker's age band, counting all hours worked, and keep records (UK: six years) | Salary, currency, FTE, time entries and the date of birth (for pay and pension only, WPM-D114). **No check.** | **Build first** (WPM-R218). Pure arithmetic over data WPM already holds; advisory; unset means "no floor configured", never "compliant". |
| **Overtime** | Federal US: 1.5 times the regular rate over 40 hours in a workweek for non-exempt workers; some states add daily thresholds and double time. UK: no statutory premium, but unpaid hours count towards the minimum wage | Overtime derived as minutes beyond the *contracted day*, priced at a rate setting; the working-time guardrails (WPM-R27). **No weekly or daily statutory threshold, no exempt status.** | **Build second** (WPM-R219). Keep today's derivation, name it *contractual* overtime, and add statutory rules as profile parameters plus a human-recorded exempt status. |
| **Workers' compensation** | US: state schemes, mandatory for private employers in every state except Texas. UK: compulsory Employers' Liability insurance of at least £5 million, certificate on display | Sickness absence (section Q). **No injury record, no coverage record.** | **Record only** (WPM-R220, WPM-R221). Coverage evidence per organization with expiry reminders; a minimal injury notification record. No benefit arithmetic, no claims. |
| **Unemployment insurance** | US: federal and state payroll taxes on the employer. Canada: Employment Insurance premiums, employer pays 1.4 times the employee's. Other countries differ | Stub deductions on the employee side; a leaver record with a reason | **Last, and thin** (WPM-R222, WPM-R223). A separation record from the leaver record, and employer-side cost lines as stubs so workforce cost is honest. WPM never computes or files the tax. |

Five principles hold across all four (WPM-D154, WPM-D155):

1. **WPM checks and warns.** It does not decide an entitlement or file a return. A payroll or tax service does that.
2. **Numbers are data, not code.** Each rate, threshold and date is a dated, sourced parameter in the jurisdiction registry
   ([worldwide.md](worldwide.md), WPM-D144, `rules::jurisdiction`), resolved from the place the work is done, with a status of
   `unverified` until a person has read the primary source.
3. **Unset is not compliant.** With no parameter configured the answer is `no limit configured` (the pattern of the working-time
   flags), never a green tick.
4. **Counts and references, not records,** when anything leaves WPM, and injuries are health data: the fact of an event, never the
   medical detail (WPM-D158).
5. **WPM stays usable on its own.** Nothing here needs PPM or CRM, and no bridge carries a wage, an injury or a separation reason.

## What was found

### 1. Minimum wage

- **United Kingdom.** From 1 April 2026 the National Living Wage (age 21 and over) is **£12.71 an hour**, the rate for ages 18 to 20 is
  **£10.85**, and the rate for under 18s and apprentices is **£8.00** (the GOV.UK press release confirms £12.71; the other rates are
  from a Business Wales summary that cites GOV.UK, and the full GOV.UK table was not read).
  Unpaid overtime counts: total pay across all hours worked must average at least the minimum for the pay reference period, and
  salaried hours work has its own counting rules (National Minimum Wage Regulations 2015, Part 5, Chapter 2). Employers must keep records
  sufficient to show compliance (no set format; failing to keep them is an offence); the minimum period is **six years** since 1 April
  2021. HMRC enforces on behalf of the Fair Work Agency and publishes naming rounds of employers who underpaid.
  Under the Employment Rights Act 2025 a payroll blog says a broader six-year record rule for overtime and holiday pay began on
  6 April 2026: **one secondary source, verify.**
- **European Union.** The Adequate Minimum Wages Directive (2022/2041) does not set a wage. On 11 November 2025 the Court of Justice
  (Case C-19/23, Denmark v Parliament and Council) dismissed the attempt to annul the whole Directive but **annulled Article 5(2)** (the
  four elements a Member State with a statutory minimum must weigh) and two phrases in Article 5(1) and 5(3) (one of them barring
  an indexation mechanism from lowering the minimum), as direct interference with pay, which the EU has no competence over. The rest, including the
  collective bargaining provisions, stands. A national statutory minimum is therefore national data.
- **Germany** (as an example of a Member State with a statutory minimum): **€13.90 an hour from 1 January 2026**, rising to **€14.60 from
  1 January 2027** (Bloomberg Law and WageIndicator reports of the regulation); a collectively agreed sector rate may be higher.
- **United States.** The federal rate and the state and local rates were **not confirmed in this round** (the search returned
  nothing on them). The rule WPM must model is the structure, not a number: the **higher** of the federal, state and local rate applies.
- **Australia.** The Fair Work Commission sets the national minimum wage in an annual review; reports give **$26.44 an hour** from the first
  full pay period on or after 1 July 2026 (a 6 percent rise from $24.95), with modern awards higher for most workers. Sources conflicted
  on the earlier figures, so treat this as moderate confidence.
- **Not researched:** Canada's provincial minimums, the UK accommodation offset and apprentice rules, tips, and piece work.

### 2. Overtime

- **United States, federal (FLSA).** Non-exempt workers get **1.5 times the regular rate for hours over 40 in a workweek**; there is no daily
  trigger. A white-collar exemption needs a salary at or above a federal floor. The 2024 rule that raised that floor was **vacated by a
  Texas federal court on 15 November 2024**, the Fifth Circuit dismissed the appeals in early May 2026, and the Department of Labor
  formally rescinded it in mid-May 2026, restoring the 2019 levels: **$684 a week ($35,568 a year)** and **$107,432 a year** for highly
  compensated employees (law firm and CalChamber sources; one policy wiki that gave $1,128 a week was giving the vacated figure).
  The Department noted that future rulemaking is not barred. State floors can be higher (California is higher).
- **United States, states.** The more protective rule applies. **California:** 1.5 times after 8 hours in a day, double time after 12 hours in
  a day, and rules for the seventh consecutive workday, besides the weekly rule. **Alaska:** daily overtime after 8 hours.
  **Nevada:** daily overtime after 8 hours only for workers paid under 1.5 times the state minimum (one 2026 source; verify).
  **Colorado:** 1.5 times after 12 hours in a day. Sources were payroll blogs, not the statutes.
- **United Kingdom.** There is **no statutory right to overtime pay**; it follows the contract. Unpaid extra hours still count towards the
  minimum wage average above. The Working Time Regulations (48-hour average week, 11 hours of rest) already shape WPM's guardrails
  (WPM-R27, example profile `GB`). How holiday pay treats regular overtime was not researched this round.
- **European Union.** The Working Time Directive sets the same kind of limits; the **premium** for overtime is national and was not
  researched.

### 3. Workers' compensation

- **United States.** A state-by-state system. **Texas is the outlier:** private employers may choose not to carry coverage (a 2024 state
  report put the share without coverage at about 24 percent, down from about 28 percent in an older source). A Texas employer without coverage must file a
  Notice of No Coverage with the Division of Workers' Compensation, must tell its employees, and **loses the common-law defences** if an
  injured worker sues; public employers and contractors on government construction contracts must carry it (Texas Labor Code section 406.096). The
  first-report-of-injury deadlines were **not found**.
- **United Kingdom.** There is no employer-run no-fault scheme of this kind. The duty is **compulsory Employers' Liability insurance**
  (Employers' Liability (Compulsory Insurance) Act 1969): cover of **at least £5 million**, the certificate displayed at each place of business
  (electronic display is accepted if staff can reasonably reach it), and insurers advise keeping certificates for **40 years**. Read from insurers'
  own certificates, not from HSE or the legislation: **verify.** Injury reporting duties (RIDDOR) and state injury benefits were not researched.
- **Germany, Canada, Australia.** Employer-funded statutory accident insurance in Germany (the Berufsgenossenschaften), provincial boards in
  Canada, and state schemes in Australia are the general shape, **but the searches returned nothing on them: unverified.**

### 4. Unemployment insurance

- **United States.** A federal-state system paid by the employer. The federal tax (FUTA) is **6.0 percent of the first $7,000 of each employee's
  pay, with a credit of up to 5.4 percent for state contributions paid on time**, so the usual net rate is 0.6 percent. In a **credit reduction state** the
  credit shrinks: for 2025 the Department of Labor named **California (1.2 percent, an effective 1.8 percent, up to $126 an employee) and the
  Virgin Islands (4.5 percent)**. State wage bases must be at least the federal $7,000. The 2026 credit-reduction states were not confirmed.
- **Canada.** Employment Insurance for 2026: the employee rate is **$1.63 per $100 of insurable earnings** and the employer pays **1.4 times
  that, $2.28**, up to maximum insurable earnings of **$68,900** (Quebec's rates are lower, 1.30 and 1.40 percent), announced 12 September 2025.
- **Germany and the United Kingdom.** Contribution rates and any separate unemployment contribution were **not found / not researched.**
- **Beyond the tax**, an employer usually owes a **separation record** when someone leaves (in Canada a Record of Employment, in the US a
  notice to the state agency). That duty is from general knowledge and **was not verified here.**

## What WPM does today

| Area | Today | Gap |
| --- | --- | --- |
| Minimum wage | `workers.salary_minor`, `salary_currency`, FTE, `employment_type`; time entries; `workers.date_of_birth` held "for pay and pension only" and never scoring anyone (WPM-R169, WPM-D114) | No floor, no age band, no effective hourly rate, no check |
| Overtime | `rules::workforce::overtime_minutes` counts minutes beyond the **contracted day** (`WPM_CONTRACTED_DAY_MINUTES`); the payslip adds an overtime line from a rate setting (WPM-R4, WPM-R13, WPM-T405); working-time flags from a profile (WPM-R27, WPM-D144) | A contractual measure sold as "overtime": no weekly or daily statutory threshold, no double time, no exempt status |
| Workers' compensation | Sickness absence and stress risk assessments (section Q); the ESRS "health and safety" row of [human-capital-reporting.md](human-capital-reporting.md) is shown as a **gap** by design | No injury or illness record, no coverage evidence, no expiry reminder |
| Unemployment insurance | Employee-side stub deductions of kind `social` ([payroll-compensation.md](payroll-compensation.md)); a leaver record with a reason ([joiners-and-leavers.md](joiners-and-leavers.md)); `worker_terminated` webhook (WPM-R207) | No separation record for the authority, no employer-side cost lines |
| The registry | `rules::jurisdiction` holds dated, sourced parameters with a combination rule each (`StrictestMin`, `StrictestMax`, `MostSpecific`, `Bounded`) and resolves them from the place of work | Fits all of this. A parameter is a scalar, so a rate per **age band** needs one key per band, and a currency comes from the profile |

The existing design carries most of the weight. The new work is mostly **parameters and pure cores**, and the one new kind of data is the
injury notification (health data).

## Advice

### Minimum wage: build first (WPM-R218, WPM-D156)

An advisory check over a pay period: **effective hourly pay** (the pay the law counts, for the hours worked) against **the minimum for the
worker's age band on the last day of the period**, with the outcome `below`, `near` (within a margin a deployer sets), `ok`, or `unknown`
(no hours, no date of birth where a band needs one, or no floor configured). HR sees it beside the payroll run; the audit entry holds
counts. It never blocks a run in v1 (see question 2). Why first: it is arithmetic over data WPM already holds, the harm of a miss is
large (back pay, naming, an offence for missing records), and it needs no new personal data. Limits to state on the screen: the check
is **indicative**; it does not model accommodation offsets, tips, piece work, apprentices' special rules or which pay counts, and a
passing check is not a statement of compliance.

Parameters (registry, dated, one key per band, in minor units of the profile's currency): `minimum_wage.hourly_minor.adult`,
`.age_18_to_20`, `.under_18` and `.apprentice`, with the age cut-offs as their own integer parameters, combined **strictest-min** (the
highest floor across the layers wins, so a state or city floor above the federal one governs). The date of birth is used only to place a
worker in a band at the period end and **only the band and outcome are stored**, never a derived age (WPM-D114's purpose is "pay").
Keep the evidence the law asks for: the check's inputs and result are kept for **six years** where a profile sets that retention.

### Overtime: build second (WPM-R219, WPM-D157)

1. **Name today's measure honestly.** What WPM derives is *contractual* overtime (beyond the contracted day). Rename it in the UI, API
   docs and payslip line so nobody reads it as the legal premium.
2. **Statutory rules as parameters:** `overtime.weekly_threshold_minutes`, `overtime.daily_threshold_minutes`,
   `overtime.double_time_threshold_minutes`, and the multipliers in basis points (150 and 200), thresholds **strictest-max** (the lowest
   applicable threshold wins) and multipliers **strictest-min** (the highest wins). An `example` `US` profile (40 hours weekly) and an
   `example` `US-CA` profile (8 hours daily, 12 for double time) show the shape and are marked `unverified`.
3. **Exempt status is a recorded human fact** on the post, `exempt`, `non_exempt` or `unknown`, set by HR, with an *advisory* check of
   the salary against the profile's weekly salary floor (`overtime.exempt_salary_floor_minor_weekly`). WPM never classifies a role as exempt from
   its duties: that is a legal judgement, so the duties test stays outside WPM.
4. **Regular rate** is the hard part in the US (bonuses and other pay count). v1 uses the base hourly rate and **says so**; counting
   other pay is a later task.
5. The UK needs no premium, so the `GB` example sets none. What it does need is that unpaid hours reach the minimum wage check, which
   the time entries already allow.

### Workers' compensation: record only (WPM-R220, WPM-R221, WPM-D158)

- **Coverage evidence (R220).** Per organization and jurisdiction: the kind (statutory scheme, private insurance, self-insured, or
  elective with none), the insurer and policy reference, the minimum cover, valid from and to, and a document reference for the certificate.
  An expiry reminder reuses the existing reminder machinery. Where a rule needs display or retention (the UK certificate), that is a
  reminder, not a copy of the document.
- **Injury and illness notification (R221).** Only **after the gaps in what authorities require are researched** (WPM-T412: RIDDOR, OSHA
  recording, state first-report deadlines): the worker reference, the date, a severity class (for example "lost time" or not), whether it was reported
  to an authority and a reference to that report. **No narrative and no medical detail** (health is special category data). Link to the absence record
  (section Q) by reference, copy nothing. Reports see counts at or above the floor (WPM-D32), which also closes the "health and safety" gap
  row in the human capital report.
- **Not WPM's job:** benefit amounts, claims, adjudication, the insurer's experience rating. Leave those to the insurer.

### Unemployment insurance: last, and thin (WPM-R222, WPM-R223, WPM-D159)

- **Separation record (R222).** Built **from the leaver record WPM already has**: last day worked, the reason from a short standard list,
  the final pay date, and a reference to the pay summary the authority asks for. Produced as an export for HR and a payroll service to
  submit. WPM submits nothing. The reason affects a person's eligibility, so the list is standard codes only and it is read by HR, never by a line manager.
- **Employer-side cost lines (R223).** Payslips and payroll runs gain **cost lines that are not deducted from net** (employer
  unemployment tax, national insurance, EI), driven by parameters `employer_contribution.<name>.rate_bp` and `.wage_base_minor` (a cap per
  year), all labelled stub, so the workforce cost figures stop understating the real cost. No filing, no remittance, no credit-reduction logic.

### Order, and what not to do

1. R218 (minimum wage check), 2. R219 (overtime parameters and exempt status), 3. R220 (coverage evidence), 4. R222 (separation record),
5. R221 (injury notification, after the research task), 6. R223 (employer cost lines).
Do **not**: block payroll on a warning in v1; guess a jurisdiction from an address; store an age; keep injury narratives; compute a
benefit; or ship any default figure marked `read` that a person has not read in its source.

## Requirements (proposed)

- **WPM-R218 Minimum-wage floor check.** For a payroll run, an advisory per-worker result (`below`, `near`, `ok`, `unknown`) from the
  effective hourly pay and the floor of the worker's age band on the last day of the period, resolved from the place of work. HR only; the audit entry holds
  counts, not names. Unset is `no floor configured`.
- **WPM-R219 Overtime rules and exempt status.** Today's contractual overtime is named so. Statutory overtime rules (weekly and daily thresholds, double time,
  multipliers) are profile parameters; a post has a recorded `overtime_status`; an advisory salary-floor check for `exempt` posts.
- **WPM-R220 Workers' compensation and employer's liability coverage evidence.** Per organization and jurisdiction: kind, insurer and policy
  reference, minimum cover, validity dates, certificate document reference, with an expiry reminder.
- **WPM-R221 Work-related injury and illness notification record.** Minimal and without clinical detail: worker reference, date, severity class,
  reported-to-authority flag and reference. Counts to reports at or above the floor.
- **WPM-R222 Separation record.** From the leaver record: last day worked, a standard reason code, the final pay date, a pay summary reference; an export for HR.
- **WPM-R223 Employer-side statutory cost lines.** Parameter-driven stub lines, not deducted from net, shown in payroll cost.

## Decisions (proposed 2026-11-01 by the AI assistant; not confirmed)

- **WPM-D154 WPM checks and warns; it never decides an entitlement, pays a benefit or files.** The authority and the arithmetic of record stay with a payroll,
  tax or insurance service, as WPM-D58 keeps contractors off payroll.
- **WPM-D155 Every statutory figure is a dated, sourced parameter in the jurisdiction registry, `unverified` until a person reads the primary source.**
  Extends WPM-D144 and follows WPM-T404. A figure shipped in an `example` profile is labelled so.
- **WPM-D156 The minimum-wage check is indicative, advisory in v1, and keeps no age.** The date of birth is used under WPM-D114's "pay" purpose to pick a band, and
  only the band and the outcome are stored. A deployer may later ask for approval to be held on `below` (question 2).
- **WPM-D157 Contractual overtime stays; statutory premium is parameters; exempt status is a human-recorded fact.** WPM never infers exemption from duties.
- **WPM-D158 Injury and illness records hold the fact, never the medical detail.** Health is special category data (see the
  [data protection impact assessment](governance/dpia.md)), and [scope.md](scope.md) already refuses clinical and occupational-health records
  (WPM-D17, WPM-D24, WPM-D25): no symptom, diagnosis or condition is storable, and this record keeps to that. A report sees counts at or above the floor.
- **WPM-D159 Employer contributions are cost lines, never deductions from net,** stubbed by parameter, and never remitted or filed by WPM.

## Questions for the product owner

1. **Which jurisdictions first?** We advise `GB` (the worked example already in the product), then `US` federal with `US-CA` as the daily-overtime shape, then `DE` as the
   EU example. `AU` and `CA` follow.
2. **Advisory or blocking?** We advise advisory for v1. A deployer option to hold payroll approval while a worker is `below` is cheap to add later.
3. **Is the injury record wanted at all, and who may see it?** It is the one new health-data record. We advise building it only after WPM-T412, readable by HR and the
   health and safety lead, never by a line manager.
4. **Does a separation record belong in WPM?** It is thin and reuses the leaver record, but some deployers hand all of this to payroll.
5. **Legal review.** Who confirms each profile's figures against the source, and how often? Rates change at least yearly; the minimum wage changed on 1 April 2026 and changes again for
   Germany on 1 January 2027.

## What was checked, and what was not

Method: web searches on 2026-11-01 (standard mode), reading result excerpts, **not** the full texts and not the primary sources. Where sources disagreed, the page says so.

| Fact | Source type | Confidence |
| --- | --- | --- |
| UK National Living Wage £12.71 from 1 April 2026 | GOV.UK press release (title and snippet) | High for £12.71; the other UK rates moderate |
| UK unpaid overtime counts; records six years from 1 April 2021 | HMRC manual excerpt, payroll sources | Moderate |
| UK six-year overtime and holiday pay record rule from 6 April 2026 | One payroll blog | **Low** |
| CJEU judgment 11 November 2025, C-19/23 | Several law firm notes; the Court's press release was named, not read | High on the outcome, moderate on exact wording |
| Germany €13.90 and €14.60 | Bloomberg Law, WageIndicator | Moderate |
| FLSA exempt floors $684 and $107,432; 2024 rule gone | Law firms, CalChamber; one conflicting wiki | Moderate to high |
| State daily overtime (CA, AK, NV, CO) | Payroll blogs, SHRM summary | **Low to moderate** (Nevada especially) |
| Texas elective coverage; Notice of No Coverage | State agency fact sheet (excerpt), law sites | Moderate |
| UK Employers' Liability £5 million and display | Insurers' certificates | Moderate; the legislation and HSE were not read |
| FUTA 6.0 and 5.4 percent, $7,000; 2025 credit reduction | IRS form pages, payroll vendors, Bloomberg Tax | High for the 2025 figures |
| Canada EI 2026 $1.63 and $2.28; $68,900 | Government news release (excerpt), HR trade press | High |
| Australia $26.44 from 1 July 2026 | Several small sites that conflicted on earlier figures | **Low to moderate** |

**Not found or not researched (WPM-T412 closes these):** the US federal and state minimum wage rates; first-report-of-injury deadlines; RIDDOR and OSHA
recording rules; German statutory accident insurance and unemployment contribution rates; Canada's provincial minimum wages and compensation boards; Australia's
state compensation schemes; UK holiday pay and overtime, the accommodation offset, apprentices; the EU overtime premium by country; the 2026 FUTA credit
reduction states; separation-record duties in any country.

## Not built

Everything above. This file adds no code, migration, parameter or test. The only repository change in WPM-T410 is the specification.
