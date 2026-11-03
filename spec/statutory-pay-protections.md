# Statutory pay protections: minimum wage, overtime, workers' compensation, and unemployment insurance

What the law asks of an employer on four protections that touch pay, what WPM holds and does about each today, and what we advise
WPM does next. Researched, assessed and advised on 2026-11-01 (WPM-T410). **Built so far (2026-11-01): the minimum-wage floor check, in part** (see [what is built](#what-is-built-and-what-is-not)). Everything else here is not built. Proposed in
[plan.md](plan.md) section T; requirements WPM-R218 to WPM-R223; decisions WPM-D154 to WPM-D162 (**drafted by the AI assistant and confirmed by the product owner on 2026-11-01**); tasks in [tasks.md](tasks.md) Phase 26 (WPM-T410 to WPM-T427).

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
| **Germany** | €13.90 from 1 January 2026; €14.60 from 1 January 2027 | A daily limit of 8 hours, up to 10 on a 24-week average (Arbeitszeitgesetz); "overtime" (beyond the contract) is not "more work" (beyond the legal maximum); **no statutory premium found** (contract or agreement) | **The employer alone pays**; the contribution is the pay times the risk class of the insurer's tariff (formula from a calculator site: unverified) | **2.6 percent of pay, employer 1.3 percent** (2026, unchanged), up to a ceiling of €8,450 a month; the employer's statutory contributions are about 21 percent in all |
| **Spain** | SMI €1,221 a month in 14 payments (€17,094 a year; €40.70 a day), Real Decreto 126/2026 | **At most 80 hours a year** (Estatuto de los Trabajadores art. 35); voluntary unless agreed; compensated in pay or by rest within four months; the premium comes from agreement or contract | The employer pays the whole premium; **the rate follows the activity's tariff** (no single rate) | Indefinite contracts: **employer 5.50 percent**, worker 1.55 percent (2026) |
| **France** | SMIC €12.02 an hour from 1 January 2026, **€12.31 from 1 June 2026** | **25 percent for the first eight overtime hours (the 36th to 43rd hour in a week), 50 percent from the 44th**; an agreement may set another rate, never below 10 percent (Code du travail L3121-36) | **Average net rate 2.08 percent in 2026** (order of 30 December 2025); the actual rate is **set per establishment**; from 2027 it is calculated by risk groups | **Employer 4.05 percent, worker none**, on pay up to four times the social security ceiling (2026) |
| **Sweden** | **None by law**; collective agreements, about 90 percent of workers covered | Working Hours Act: 40 hours a week normal, overtime capped at 48 hours over four weeks and 200 hours a year; **no statutory premium** (agreements often 150 to 200 percent) | Work injury contribution **0.10 percent** of pay in 2026 (0.20 before), inside the 31.42 percent employer total (Skatteverket) | Labour market contribution **2.64 percent** of pay; how the a-kassa is funded was not found |
| **Denmark** | **None by law**; collective agreements | Set by agreements (typically a 37-hour week); no statute found | Compulsory occupational injury insurance **bought from a private insurer** (premium estimates DKK 1,150 to 4,000 a year, conflicting) and an occupational disease levy by industry (DKK 284 to 2,911 a year per full-time worker): payroll guides, **not the statute** | The a-kassa is voluntary for the worker, funded by members and the state (one source); the employer pays two thirds of the ATP pension contribution: **unverified** |
| **Japan** | Set **by prefecture**; fiscal 2025 average ¥1,121; a panel proposed +¥55 (4.9 percent) to an average ¥1,176 for fiscal 2026 | **25 percent; 50 percent beyond 60 hours a month**, for small and medium employers too from April 2023 (one guide says 2025: conflict); time off may replace the 50 percent portion by agreement | Employer-paid, **by industry class**; the Labour Bureau says the rates are unchanged for fiscal 2026; the table was not read | **Employment insurance, fiscal 2026** (1 April 2026 to 31 March 2027): general business **13.5 per 1,000 of pay, employer 8.5 and worker 5**; construction 16.5 (10.5 and 6); agriculture, forestry and sake brewing 15.5 (9.5 and 6); 0.1 point lower than the year before. The 3.5 per 1,000 for business activities is the employer's alone |
| **South Korea** | ₩10,320 an hour from 1 January 2026 (+2.9 percent); about ₩2,156,880 a month on 209 paid hours | **At least 50 percent extra for overtime, night work (10 p.m. to 6 a.m.) and holiday work** (Labor Standards Act art. 56), **and the premiums stack** (overtime at night is 200 percent; holiday work beyond 8 hours 200 percent); 40 regular plus 12 overtime hours a week, with sources disagreeing whether holiday hours count towards the cap; **not applied to a workplace of fewer than five employees** | The employer pays the whole premium, **0.56 to 18.56 percent by industry** (a 2026 source) | Employment insurance rate not confirmed (an older source: 0.9 percent each); wage claim guarantee fund 0.09 percent; pension 9.5 percent split 4.75 each; health 7.19 percent split |
| **China** | **No national minimum; set by province** in tiers (reports: about RMB 1,750 to 2,740 a month; Shanghai 2,740) | 150 percent on a weekday, **200 percent on a rest day, 300 percent on a statutory holiday**; capped at 36 hours a month | The employer pays; **the rate follows the industry's risk, in eight tiers** since 2015, and is adjusted locally by the employer's record (a secondary source: 0.2 to 1.9 percent) | **Usually 0.5 percent employer and 0.5 percent worker; some places 0.5 to 1 percent**; pension about 16 percent employer and 8 worker; medical about 9 to 10 percent by city; **local bureaux set rates and the contribution base, reset each July** |
| **India** | Minimum wages are the **states'**; the Code on Wages national **floor wage is not yet notified** (the Central Rules were notified 8 May 2026); an unofficial 2019 reference of Rs 375 a day | **At least twice the normal rate of wages for excess hours** (Code on Wages section 14); the normal rate takes in basic pay and certain allowances, not bonus or overtime (text from a third-party site, not the gazette) | State insurance (ESIC, which covers employment injury and sickness): **employer 3.25 percent, worker 0.75 percent** since 1 July 2019, for pay up to Rs 21,000 a month; the Employees' Compensation Act was not confirmed | ? |

Notes the research added:

- **Source conflicts.** Japan's rise is "from next April" in one report and "from around October 2026" in another (the panel's figure is a recommendation;
  each prefecture decides, and at least 17 were reported likely to exceed it). A Beijing and Shanghai figure differs across sites. A Spanish source mentions a
  further 3 percent rise to about €1,258 that was **not confirmed**. Korea's rate is from the Official Gazette notice as reported by Bloomberg Law.
- **Second and third research rounds (2026-11-01, same day).** They filled most of the "?" cells above. What is still missing: a statutory premium in Germany (none found), India's unemployment cover, Denmark's and Sweden's unemployment funding, Korea's employment insurance rate, Japan's accident rate table, and the US federal and state rates. Several rates are from payroll guides, not the authority; Japan's employment insurance rates are from Labour Bureau notices and France's from Legisocial's reading of the order.
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

The eleven were not in view when the advice above was first written, and the second round of research (below the table) added items 7 and 8. The research changes the design in eight ways (WPM-D160, WPM-D162 and edits to WPM-R218, WPM-R219 and WPM-R223):

1. **"None by law" is a value, not an absence.** Sweden and Denmark set no statutory minimum wage or overtime premium; collective agreements do. A parameter that is
   unset means *not configured* and must never read as *no rule*. The registry gains an explicit `none_by_law` state per parameter and jurisdiction, with a
   source, so the screen says "no statutory floor here; check the collective agreement" and never a green tick (WPM-D160).
2. **A collective-agreement floor is an organization-layer parameter sourced to an agreement** (WPM-D161, confirmed). A deployer may enter one; WPM never reads or interprets the
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
7. **Overtime is a list of bands, not two thresholds.** France pays 25 percent for the first eight overtime hours of a week and 50 percent after; Japan pays 25 percent and 50 percent beyond 60 hours a month; the US adds a daily
   band in some states; China prices by kind of day. The rule is therefore a **sequence of bands**, each with a period (day, week, month), a threshold and a multiplier, plus the kind-of-day multipliers. **Premiums add where they overlap** (Korea: overtime 50 percent plus night 50 percent is 200 percent of the ordinary wage), and **a rule may apply only above an establishment's size** (Korea: not under five employees) (WPM-D162).
8. **Some employer rates follow a class, or the company.** Korea's accident rate runs from 0.56 to 18.56 percent by industry, China's follows eight risk tiers, Spain's follows the activity tariff, and France's is set per establishment, Germany's follows the risk class of the insurer's tariff, and Japan's follows an industry class. The employer cost parameter may therefore be
   **set by the deployer for the organization** (one rate, a class and its rate, or a rate table keyed by a class the deployer sets), never per worker and never inferred from the organization's name (WPM-D162).

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

## Decisions (drafted 2026-11-01 by the AI assistant; confirmed by the product owner the same day)

**Confirmed 2026-11-01.** The product owner answered "confirm all" to the list of D154 to D160 and D162 (D161 had been confirmed earlier, as option (a) of question 6). The requirements WPM-R218 to WPM-R223 stay proposed; questions 3 and 4 (the injury record, the separation record) are still open, so WPM-R221 and WPM-R222 are not started.

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
- **WPM-D162 Overtime is a sequence of bands, and an employer rate may follow a class the deployer sets.** Drafted 2026-11-01 from the second research round. A band has a period (day, week or month), a threshold in minutes and a multiplier in basis points; premiums that overlap add (Korea: overtime plus night is 200 percent); a rule may apply only above an establishment size (Korea: five employees); kind-of-day multipliers (rest day, public holiday) and
  caps (monthly, annual) sit beside it as parameters. An employer contribution rate may be one rate, or a table keyed by a class (industry, risk tier, activity) that the deployer sets on the organization. WPM never picks a class from what the organization is called.
- **WPM-D169 The statutory overtime premium is added to gross pay, as its own line.** *Decided by the product owner, 2026-11-03.* A payroll calculation derives each salaried worker's statutory overtime with the same pure function the advisory check uses, so the two cannot disagree, and adds the premium (the extra over the base rate) to gross; the stub tax sees it. The payslip carries it as `statutory_overtime_minor`, already inside `gross_minor`, so it can be shown without subtracting. Contractual overtime stays as it is and pays minutes at the base rate, so the two add and never pay one minute twice. Nothing is added unless every input is recorded: a post a person marked non-exempt, a place of work, and bands configured for it. An exempt post, an unrecorded status, no seat, no bands, a monthly band and a "none by law" place each add zero, and the check says which. A person is never paid a guess. Not modelled: a regular rate that counts bonuses, rest days and public holidays, and a monthly band.
- **WPM-D161 A collective-agreement floor may be entered, narrowly. *Confirmed by the product owner, 2026-11-01* (option (a) of question 6).** An organization-layer parameter holds one floor figure (or a few, each with its basis) with the agreement's name, a clause or table reference, and
  valid-from and valid-to dates. WPM compares pay to the figure and **never reads the agreement**; the screen says "the floor you entered, not a statement that it is correct", and warns when the entry is past its valid-to date or missing. Grade and step tables, allowances,
  overtime tiers and any rule that interprets an agreement stay out ([scope.md](scope.md)): that would be a union-rules engine. A longer table is a pay scale ([pay-scales.md](pay-scales.md)), not a floor.
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
6. **Collective-agreement floors.** **Answered 2026-11-01 by the product owner: option (a), yes**: a deployer may enter an agreement's floor as a parameter, sourced to the agreement and never interpreted by WPM (WPM-D161).

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
| Germany unemployment 2.6 percent, employer 1.3; ceilings | Several payroll guides and a health insurer | Moderate |
| France overtime 25 and 50 percent bands (L3121-36) | Legal publishers, parliamentary question, the Code's page named | Moderate to high |
| Spain overtime 80 hours a year; unemployment 5.50 and 1.55 percent | Spanish legal and payroll sites, a 2026 labour handbook | Moderate |
| Sweden work injury 0.10 percent, labour market 2.64 percent, total 31.42 percent | Skatteverket's table as reported, Business Sweden | Moderate to high (third-party guides conflict) |
| Denmark injury insurance and levies | Payroll guides; no statute cited | **Low to moderate** |
| Japan 25 and 50 percent overtime; 60-hour band | Law-firm and payroll guides (one conflicting date) | Moderate |
| South Korea accident insurance 0.56 to 18.56 percent; pension 9.5 percent | One 2026 guide and others | **Low to moderate** |
| China work injury insurance, eight risk tiers | Law and consultancy pages, a municipal notice | Moderate |
| India ESIC 3.25 and 0.75 percent, Rs 21,000 | Several payroll and law sites (older sources conflict) | Moderate |
| Korea overtime, night and holiday premiums stack; under five employees exempt | Law-firm and HR sources; holiday-hours counting disputed | Moderate |
| India overtime at least twice the normal rate (Code on Wages s. 14) | A third-party text and commentary; not the gazette | Moderate |
| France employer unemployment 4.05 percent; average accident rate 2.08 percent | Legisocial, citing the order of 30 December 2025 | Moderate to high |
| Japan employment insurance fiscal 2026 rates | Labour Bureau notices (MHLW) | **High** for those rates |
| China unemployment 0.5 percent each; pension 16 and 8 percent | Payroll guides, no official table | **Low to moderate** |
| Germany accident insurance by risk class; working-hours limits | Insurer and chamber pages, a calculator site | Moderate |

**Not found or not researched (WPM-T412 closes these):** after the third round, for the eleven: **a statutory overtime premium in Germany (none found; understood to come from contract or agreement); unemployment cover in India; the a-kassa funding in Sweden and Denmark; Korea's employment insurance rate; Japan's accident rate table; the Employees' Compensation Act in India; official (not payroll-guide) schedules for China**; also the US federal and state minimum wage rates; first-report-of-injury deadlines; RIDDOR and OSHA recording rules;
UK holiday pay and overtime, the accommodation offset and apprentices; the 2026 FUTA credit reduction states; separation-record duties in any country. Canada and Australia were researched only in part and are **not** in the eleven (they are kept as findings, not as scope).

## What is built and what is not

**Built (2026-11-01, WPM-T413 to WPM-T415 in part):** the minimum-wage floor check (WPM-R218). Twelve registry parameters (floors by band on an hourly or a monthly basis, the age cut-offs, the currency, the payments
a year, a near margin, and the `none_by_law` flag); the pure core `rules::minimum_wage`; `GET /api/payroll-runs/{pid}/minimum-wage-check` (HR and payroll, organization-scoped, advisory, audited with counts); the example `GB` rows for the 2025
and 2026 rates; the OpenAPI path and the UI's types; a request test. The answers are `below`, `near`, `ok`, `none_by_law` and `unknown` with a reason, and `unknown` is never `ok`.

**Limits of what is built:**

- **The date of birth is not stored** (it is not a column yet: WPM-T319). It is read from the person service, only where a profile has age bands. With no person service a banded profile such as the `GB` example answers
  `no_date_of_birth`, except that pay under the lowest band's floor is still `below` (WPM-T427).
- **Hours** are the recorded time entries in the period, except on-call; **pay** is the payslip's gross, overtime included. Both are indicative. Apprentices, accommodation, tips and piece work are not modelled.
- **No result is kept** beyond the audit entry's counts, so the retention a profile might set (UK: six years) is not met by WPM. That is the rest of WPM-T415.
- **No screen yet** (WPM-T416). The figures in the `GB` example are `example`, and none has been read in its source (WPM-T411).

**Not built:** WPM-R219 to WPM-R223 (statutory overtime, coverage evidence, the injury record, the separation record, employer cost lines), WPM-T417 to WPM-T426, and the research tasks WPM-T411 and WPM-T412. WPM-R221 and WPM-R222 also wait for
questions 3 and 4.
