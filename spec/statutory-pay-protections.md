# Statutory pay protections: minimum wage, overtime, workers' compensation, and unemployment insurance

What the law asks of an employer on four protections that touch pay, what WPM holds and does about each today, and what we advise
WPM does next. Researched, assessed and advised on 2026-11-01 (WPM-T410). **Nothing here is built.** Proposed in
[plan.md](plan.md) section T; requirements WPM-R218 to WPM-R223; decisions WPM-D154 to WPM-D160 (**proposed by the AI assistant, not
yet confirmed by the product owner**); tasks in [tasks.md](tasks.md) Phase 26 (WPM-T410 to WPM-T426).

**The jurisdictions in scope** were named by the product owner on 2026-11-01 (answer to question 1): **the United States, the United Kingdom, Germany, Spain,
France, Sweden, Denmark, Japan, South Korea, China and India.** The first-pass research for all eleven is in [section 5](#5-the-eleven-jurisdictions), and
what it changes in the design is in [what the eleven change](#what-the-eleven-jurisdictions-change).

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

Five principles hold across all four (WPM-D154, WPM-D155), and WPM-D160 adds a sixth for the eleven jurisdictions below:

1. **WPM checks and warns.** It does not decide an entitlement or file a return. A payroll or tax service does that.
2. **Numbers are data, not code.** Each rate, threshold and date is a dated, sourced parameter in the jurisdiction registry
   ([worldwide.md](worldwide.md), WPM-D144, `rules::jurisdiction`), resolved from the place the work is done, with a status of
   `unverified` until a person has read the primary source.
3. **Unset is not compliant.** With no parameter configured the answer is `no limit configured` (the pattern of the working-time
   flags), never a green tick.
4. **Counts and references, not records,** when anything leaves WPM, and injuries are health data: the fact of an event, never the
   medical detail (WPM-D158).
5. **WPM stays usable on its own.** Nothing here needs PPM or CRM, and no bridge carries a wage, an injury or a separation reason.
6. **"None by law" is not "not configured"** (WPM-D160): some jurisdictions set no statutory floor, and the screen says so.

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

### 5. The eleven jurisdictions

First-pass research on 2026-11-01, from search excerpts (see [what was checked](#what-was-checked-and-what-was-not)). A cell says what was **found**;
"none" means the law was found to set none; "?" means the search returned nothing and nothing is claimed.

| Jurisdiction | Minimum wage | Overtime | Workers' compensation | Unemployment insurance |
| --- | --- | --- | --- | --- |
| **US** | Federal rate not confirmed; states and cities higher | 1.5 times over 40 hours a week; some states daily (CA, AK, NV, CO); exempt salary floor $684 a week | State schemes; Texas elective | FUTA 6.0 percent of the first $7,000, credit up to 5.4; state schemes |
| **GB** | NLW £12.71 from 1 April 2026; age bands | No statutory premium; unpaid hours count to the minimum | Employers' Liability insurance, at least £5 million | ? (not researched) |
| **Germany** | €13.90 from 1 January 2026; €14.60 from 1 January 2027 | ? | ? (statutory accident insurance, unverified) | ? |
| **Spain** | SMI €1,221 a month in 14 payments (€17,094 a year; €40.70 a day), Real Decreto 126/2026 | ? | ? | ? |
| **France** | SMIC €12.02 an hour from 1 January 2026, **€12.31 from 1 June 2026** | ? (25 and 50 percent bands are from general knowledge, unverified) | ? | ? |
| **Sweden** | **None by law**; collective agreements, about 90 percent of workers covered | Working Hours Act: 40 hours a week normal, overtime capped at 48 hours over four weeks and 200 hours a year; **no statutory premium** (agreements often 150 to 200 percent) | ? | ? (employer social contribution 31.42 percent in 2025, one source) |
| **Denmark** | **None by law**; collective agreements | Set by agreements (typically a 37-hour week); no statute found | One commercial source says compulsory: **unverified** | a-kasse is voluntary for the worker and funded by members and the state (one source): **unverified** |
| **Japan** | Set **by prefecture**; fiscal 2025 average ¥1,121; a panel proposed +¥55 (4.9 percent) to an average ¥1,176 for fiscal 2026 | ? (25 percent, and 50 percent over 60 hours a month, are from general knowledge, unverified) | ? | ? |
| **South Korea** | ₩10,320 an hour from 1 January 2026 (+2.9 percent); about ₩2,156,880 a month on 209 paid hours | ? (50 percent premium: general knowledge, unverified) | ? | ? |
| **China** | **No national minimum; set by province** in tiers (reports: about RMB 1,750 to 2,740 a month; Shanghai 2,740) | 150 percent on a weekday, **200 percent on a rest day, 300 percent on a statutory holiday**; capped at 36 hours a month | ? | ? (employer social insurance about 26 to 28 percent in all, by city; no split by type) |
| **India** | Minimum wages are the **states'**; the Code on Wages national **floor wage is not yet notified** (the Central Rules were notified 8 May 2026); an unofficial 2019 reference of Rs 375 a day | ? (a double-rate rule is from general knowledge, not confirmed) | ? | ? |

Notes the research added:

- **Source conflicts.** Japan's rise is "from next April" in one report and "from around October 2026" in another (the panel's figure is a recommendation;
  each prefecture decides, and at least 17 were reported likely to exceed it). A Beijing and Shanghai figure differs across sites. A Spanish source mentions a
  further 3 percent rise to about €1,258 that was **not confirmed**. Korea's rate is from the Official Gazette notice as reported by Bloomberg Law.
- **France changes twice in 2026**, so the rate depends on the date, which is why WPM's floor is a **dated** parameter.
- **A statutory floor can be monthly, not hourly** (Spain, Korea, China, and most Indian states): the check must convert, and Spain's 14 payments change what a "monthly" figure means.

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

### What the eleven jurisdictions change

The eleven were not in view when the advice above was first written. The research changes the design in six ways (WPM-D160 and edits to WPM-R218 and WPM-R219):

1. **"None by law" is a value, not an absence.** Sweden and Denmark set no statutory minimum wage or overtime premium; collective agreements do. A parameter that is
   unset means *not configured* and must never read as *no rule*. The registry gains an explicit `none_by_law` state per parameter and jurisdiction, with a
   source, so the screen says "no statutory floor here; check the collective agreement" and never a green tick (WPM-D160).
2. **A collective-agreement floor is an organization-layer parameter sourced to an agreement.** A deployer may enter one; WPM never reads or interprets the
   agreement (union and agreement rule engines stay out of scope: [scope.md](scope.md), [worker-unions.md](worker-unions.md)).
3. **Floors are set below the nation.** US states and cities, Japan's 47 prefectures, China's provinces and tiers, and India's states: the floor is resolved from the
   place the work is done through the existing chain (WPM-D144, `Combine::StrictestMin`), so the subnational layer is the normal case, not the exception.
4. **A floor has a basis.** Hourly (US, GB, Germany, France, Japan, Korea), monthly (Spain, China, many Indian states) or daily (Spain also states a daily figure).
   The parameter carries `basis` and the pure core converts using the worker's contracted hours, with the number of payments a year as a parameter (Spain's 14).
5. **Overtime depends on the kind of day, and has caps.** China's 150, 200 and 300 percent follow weekday, rest day and statutory holiday; its 36 hours a month, Sweden's
   48 hours over four weeks and 200 hours a year, and similar caps are advisory flags beside the working-time guardrails (WPM-R27). The overtime parameters gain
   `rest_day_multiplier_bp`, `holiday_multiplier_bp` and `max_monthly_minutes`; the day kinds come from the holiday and weekend data WPM already resolves (WPM-D149).
6. **Social insurance is a bundle in half of these countries.** Germany, France, Spain, Japan, Korea and China fold accident, unemployment and more into
   statutory contributions, often by city or prefecture. Coverage evidence therefore gets a kind `statutory_scheme` (the duty is registration and contribution, not buying a policy),
   and the employer cost lines (R223) take the contribution as named lines where the split is known and one `social` line where it is not.

### Order, and what not to do

1. R218 (minimum wage check), 2. R219 (overtime parameters and exempt status), 3. R220 (coverage evidence), 4. R222 (separation record),
5. R221 (injury notification, after the research task), 6. R223 (employer cost lines).
Do **not**: block payroll on a warning in v1; guess a jurisdiction from an address; store an age; keep injury narratives; compute a
benefit; or ship any default figure marked `read` that a person has not read in its source.

## Requirements (proposed)

- **WPM-R218 Minimum-wage floor check.** For a payroll run, an advisory per-worker result (`below`, `near`, `ok`, `unknown`) from the
  effective hourly pay and the floor of the worker's age band on the last day of the period, resolved from the place of work. HR only; the audit entry holds
  counts, not names. Unset is `no floor configured`; a jurisdiction found to have no statutory floor says `none by law` (WPM-D160). The floor has a basis (hourly, daily or monthly) and the
  check converts using the worker's contracted hours.
- **WPM-R219 Overtime rules and exempt status.** Today's contractual overtime is named so. Statutory overtime rules (weekly and daily thresholds, double time,
  multipliers, rest-day and public-holiday multipliers, and monthly and annual caps as advisory flags) are profile parameters; a post has a recorded `overtime_status`; an advisory salary-floor check for `exempt` posts.
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
- **WPM-D160 "None by law" is a recorded value, and a floor has a basis.** A jurisdiction found to set no statutory minimum wage or overtime premium (Sweden and Denmark, from the first-pass research) is recorded as
  `none_by_law` with a source, distinct from *not configured*; a collective-agreement floor is an organization-layer parameter sourced to the agreement; a floor states its basis (hourly, daily or monthly) and the check converts.

## Questions for the product owner

1. **Which jurisdictions?** **Answered 2026-11-01 by the product owner: the United States, the United Kingdom, Germany, Spain, France, Sweden, Denmark, Japan, South Korea, China and India.**
   **The order was answered the same day: by population size**, as the product owner chose: India, China, the United States, Japan, Germany, the United Kingdom, France, South Korea, Spain, Sweden, Denmark
   (the populations are from general knowledge, and the UK and France, and South Korea and Spain, are close). The arithmetic is built once and is generic, so the order decides which example profile is written and verified first. **Cost to know:** the
   first two are the jurisdictions with the weakest research so far (India has no notified national floor and minimums are the states'; China's are provincial tiers, with conflicting figures), and the `none_by_law`
   case (Sweden, Denmark) comes last, so WPM-T426 still lands before any profile. Verifying the first profiles needs reviewers who read Indian and Chinese primary sources.
2. **Advisory or blocking?** We advise advisory for v1. A deployer option to hold payroll approval while a worker is `below` is cheap to add later.
3. **Is the injury record wanted at all, and who may see it?** It is the one new health-data record. We advise building it only after WPM-T412, readable by HR and the
   health and safety lead, never by a line manager.
4. **Does a separation record belong in WPM?** It is thin and reuses the leaver record, but some deployers hand all of this to payroll.
5. **Legal review.** Who confirms each profile's figures against the source, and how often? Rates change at least yearly; France changed twice in 2026, the UK on 1 April 2026, and Germany changes again on 1 January 2027.
   Eleven jurisdictions in several languages and scripts need reviewers who can read the primary source in the original language (the Official Gazette notice, BOE, Légifrance, the e-Gov and MHLW pages, the provincial notices).
6. **Collective-agreement floors.** For Sweden and Denmark the only floor is in agreements. May a deployer enter an agreement's floor as a parameter (we advise yes, sourced to the agreement, never interpreted by WPM), or is that a union-rule engine and out of scope?

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
| Spain SMI €1,221 in 14 payments, Real Decreto 126/2026 | Garrigues, Fieldfisher, PwC excerpts; the BOE text was not read | Moderate to high |
| France SMIC €12.02 then €12.31 from 1 June 2026 | Payroll and legal-publisher sites | Moderate |
| Sweden and Denmark: no statutory minimum wage | Union, government and WageIndicator pages | Moderate to high |
| Sweden Working Hours Act limits | Union and advisory sites | Moderate |
| Denmark workers' compensation compulsory; a-kasse voluntary | One commercial blog; Danish labour market guides | **Low** |
| Japan minimum wage proposal and prefectural decisions | Nippon.com, Jiji and Adnkronos reports | Moderate (a recommendation, dates conflict) |
| South Korea ₩10,320 | Bloomberg Law, WageIndicator, ActivPayroll | Moderate to high |
| China overtime 150, 200, 300 percent and the 36-hour cap; provincial tiers | Consultancy sites and L&E Global | Moderate (Beijing and Shanghai figures conflict) |
| India: national floor wage not notified; Central Rules 8 May 2026 | Law-firm and tax blogs, no gazette | **Low to moderate** |

**Not found or not researched (WPM-T412 closes these):** for the eleven jurisdictions, **overtime premiums in Germany, Spain, France, Japan, South Korea and India; workers' compensation in every one
except the US and the UK; unemployment insurance in every one except the US**, and the rest of the table's "?" cells; also the US federal and state minimum wage rates; first-report-of-injury deadlines; RIDDOR and OSHA recording rules;
UK holiday pay and overtime, the accommodation offset and apprentices; the 2026 FUTA credit reduction states; separation-record duties in any country. Canada and Australia were researched only in part and are **not** in the eleven (they are
kept as findings, not as scope).

## Not built

Everything above. This file adds no code, migration, parameter or test. The only repository change in WPM-T410 is the specification.
