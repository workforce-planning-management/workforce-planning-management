//! Pure rules for **what learning costs and returns** (WPM-R227 to WPM-R230, WPM-D172 to WPM-D174): a plan's fees, the cost
//! of its hours, train or hire, and a plan's viability. DB-free and clock-free.
//!
//! Unknown is never zero. An absent fee, pay figure, parameter or benefit leaves the figure unresolved and names the input
//! that is missing, and a comparison with a missing side answers `unknown`, never the other side's win. Currencies are never
//! mixed. Everything here is an estimate that a person weighs; WPM decides nothing.
//!
//! The net present value and payback follow the project portfolio service's `viability` definitions (WPM-D174): the two
//! projects share a vocabulary, not code, so the arithmetic is repeated here and pinned by the same worked examples.

use std::collections::BTreeMap;

use serde::{Deserialize, Serialize};

/// Basis points in 100 percent.
const BP: i128 = 10_000;

/// Calendar days a daily cost is spread over: vacancy days are calendar days, so a working-day rate would overstate them.
pub const DAYS_PER_YEAR: i128 = 365;

/// The most periods a series may span, so a typo cannot ask for ten thousand years (WPM-D174).
pub const MAX_PERIODS: u32 = 600;

/// Two totals closer than this share of the larger answer `similar`: 1000 is 10 percent. Echoed in the answer.
pub const SIMILAR_MARGIN_BP: i64 = 1_000;

/// The longest source a stated benefit may carry.
pub const MAX_SOURCE_CHARS: usize = 500;

/// A course's fee: a fee is both an amount and a currency, or neither (WPM-R227).
///
/// # Errors
///
/// A reason when only one of the two is given, the amount is negative, or the currency is not a known code.
pub fn valid_fee(fee_minor: Option<i64>, currency: Option<&str>) -> Result<(), String> {
    match (fee_minor, currency) {
        (Some(_), None) | (None, Some(_)) => {
            Err("a fee needs both fee_minor and fee_currency, or neither".to_string())
        }
        (Some(minor), Some(_)) if minor < 0 => Err("fee_minor must be at least 0".to_string()),
        (Some(_), Some(code)) if !super::currency::is_known(code) => {
            Err("fee_currency must be an ISO-4217 code, e.g. GBP".to_string())
        }
        _ => Ok(()),
    }
}

/// What one step of a plan costs in fees.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum Fee {
    /// The step is not a course, so no fee applies (mentoring, work on the job).
    NotACourse,
    /// A course with no fee recorded: unknown, never free.
    Unknown,
    /// A recorded fee. Zero is a recorded free course.
    Known {
        /// Minor units.
        minor: i64,
        /// ISO-4217.
        currency: String,
    },
}

/// One step of a plan, as costing needs it.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Step {
    /// The fee.
    pub fee: Fee,
    /// Hours the step takes.
    pub hours: u32,
    /// Whether the hours are the per-level planning figure, not a course's stated hours.
    pub estimated_hours: bool,
}

/// A plan's direct cost.
#[derive(Debug, Clone, PartialEq, Eq, Serialize)]
pub struct PlanCost {
    /// Recorded fees per currency, never converted.
    pub fees: BTreeMap<String, i64>,
    /// Courses with a recorded fee.
    pub courses_with_fee: usize,
    /// Courses with none: the fees above leave these out.
    pub courses_without_fee: usize,
    /// Every step's hours.
    pub hours: u32,
    /// How many of those hours are the per-level planning figure.
    pub estimated_hours: u32,
}

/// The fees and hours of a plan's steps.
#[must_use]
pub fn plan_cost(steps: &[Step]) -> PlanCost {
    let mut out = PlanCost {
        fees: BTreeMap::new(),
        courses_with_fee: 0,
        courses_without_fee: 0,
        hours: 0,
        estimated_hours: 0,
    };
    for step in steps {
        match &step.fee {
            Fee::NotACourse => {}
            Fee::Unknown => out.courses_without_fee += 1,
            Fee::Known { minor, currency } => {
                out.courses_with_fee += 1;
                let total = out.fees.entry(currency.clone()).or_insert(0);
                *total = total.saturating_add(*minor);
            }
        }
        out.hours = out.hours.saturating_add(step.hours);
        if step.estimated_hours {
            out.estimated_hours = out.estimated_hours.saturating_add(step.hours);
        }
    }
    out
}

impl PlanCost {
    /// The one total fee in one currency, or `None` when a fee is unknown or the fees are in several currencies. A plan with
    /// no courses has a total of zero fees in any currency, so the caller's currency is used.
    #[must_use]
    pub fn single_fee(&self, currency: &str) -> Option<i64> {
        if self.courses_without_fee > 0 || self.fees.keys().any(|code| code != currency) {
            return None;
        }
        Some(self.fees.get(currency).copied().unwrap_or(0))
    }

    /// The single currency of the recorded fees, if there is exactly one.
    #[must_use]
    pub fn fee_currency(&self) -> Option<&str> {
        let mut codes = self.fees.keys();
        match (codes.next(), codes.next()) {
            (Some(code), None) => Some(code),
            _ => None,
        }
    }
}

/// `numerator / denominator`, half away from zero. The denominator must be positive.
fn divide(numerator: i128, denominator: i128) -> Option<i64> {
    let half = denominator / 2;
    let rounded = if numerator >= 0 {
        numerator.checked_add(half)? / denominator
    } else {
        numerator.checked_sub(half)? / denominator
    };
    i64::try_from(rounded).ok()
}

/// The cost of a plan's hours, shown beside its fees and never added to them (WPM-R228).
#[derive(Debug, Clone, PartialEq, Eq, Serialize)]
#[serde(tag = "status", rename_all = "snake_case")]
pub enum HoursCost {
    /// Every input is known.
    Resolved {
        /// ISO-4217: the pay's currency.
        currency: String,
        /// The hours.
        hours: u32,
        /// The loaded cost of one hour.
        hourly_minor: i64,
        /// The hours' cost.
        minor: i64,
        /// The on-cost used, echoed.
        on_cost_bp: i64,
        /// The full-time year used, echoed.
        full_time_annual_hours: i64,
    },
    /// An input is missing; the names say which.
    Unresolved {
        /// The missing inputs.
        missing: Vec<&'static str>,
    },
}

impl HoursCost {
    /// The cost and its currency, when resolved.
    #[must_use]
    pub fn resolved(&self) -> Option<(i64, &str)> {
        match self {
            Self::Resolved {
                minor, currency, ..
            } => Some((*minor, currency)),
            Self::Unresolved { .. } => None,
        }
    }
}

/// The cost of `hours` at a person's pay. `salary` is the full-time-equivalent annual figure the record holds (payroll scales
/// it by FTE), so the full-time year is the divisor at any FTE: a part-timer's hour costs what a full-timer's does. The total
/// is worked from the salary, not from the rounded hourly figure, so rounding does not multiply.
#[must_use]
pub fn hours_cost(
    hours: u32,
    salary: Option<(i64, &str)>,
    on_cost_bp: Option<i64>,
    full_time_annual_hours: Option<i64>,
) -> HoursCost {
    let mut missing = Vec::new();
    if salary.is_none() {
        missing.push("pay");
    }
    if on_cost_bp.is_none() {
        missing.push("on_cost");
    }
    if full_time_annual_hours.is_none_or(|h| h <= 0) {
        missing.push("full_time_annual_hours");
    }
    let (Some((annual, currency)), Some(on_cost), Some(year)) =
        (salary, on_cost_bp, full_time_annual_hours)
    else {
        return HoursCost::Unresolved { missing };
    };
    if year <= 0 {
        return HoursCost::Unresolved { missing };
    }
    let factor = i128::from(annual).saturating_mul(BP + i128::from(on_cost.max(0)));
    let denominator = BP * i128::from(year);
    match (
        divide(factor, denominator),
        divide(factor.saturating_mul(i128::from(hours)), denominator),
    ) {
        (Some(hourly_minor), Some(minor)) => HoursCost::Resolved {
            currency: currency.to_string(),
            hours,
            hourly_minor,
            minor,
            on_cost_bp: on_cost,
            full_time_annual_hours: year,
        },
        _ => HoursCost::Unresolved {
            missing: vec!["pay"],
        },
    }
}

/// The train-or-hire answer. An estimate; WPM decides nothing.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize)]
#[serde(rename_all = "snake_case")]
pub enum Answer {
    /// Training costs less, by more than the margin.
    Train,
    /// Hiring costs less, by more than the margin.
    Hire,
    /// The two are within the margin.
    Similar,
    /// A side has a missing input.
    Unknown,
}

/// What the internal candidate's route rests on, in the comparison's one currency.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub struct TrainSide {
    /// Fees of the training.
    pub fees_minor: Option<i64>,
    /// The cost of the training's hours.
    pub hours_minor: Option<i64>,
    /// Ramp-up days the deployer sets.
    pub ramp_up_days: Option<i64>,
    /// The candidate's loaded annual cost at the seat's FTE.
    pub annual_minor: Option<i64>,
}

/// What an external hire rests on, in the comparison's one currency.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub struct HireSide {
    /// The recruitment cost the deployer sets.
    pub recruitment_minor: Option<i64>,
    /// Days the seat stays empty: the median of the organization's filled requisitions.
    pub vacancy_days: Option<i64>,
    /// Ramp-up days the deployer sets.
    pub ramp_up_days: Option<i64>,
    /// The seat's loaded annual cost at its FTE.
    pub annual_minor: Option<i64>,
}

/// The comparison, with each line so the arithmetic can be checked.
#[derive(Debug, Clone, PartialEq, Eq, Serialize)]
pub struct Comparison {
    /// The answer.
    pub answer: Answer,
    /// Training's total, when every input is known.
    pub train_total_minor: Option<i64>,
    /// Hiring's total, when every input is known.
    pub hire_total_minor: Option<i64>,
    /// Ramp-up at the candidate's daily cost.
    pub train_ramp_up_minor: Option<i64>,
    /// Vacancy days at the seat's daily cost.
    pub vacancy_minor: Option<i64>,
    /// Ramp-up at the seat's daily cost.
    pub hire_ramp_up_minor: Option<i64>,
    /// One year of the seat's cost less the candidate's: positive when the hire would cost more.
    pub pay_difference_minor: Option<i64>,
    /// The margin `similar` uses, echoed.
    pub similar_margin_bp: i64,
    /// Inputs that are missing, by name.
    pub missing: Vec<&'static str>,
}

/// `days` of an annual cost.
fn days_of(annual: i64, days: i64) -> Option<i64> {
    divide(
        i128::from(annual).checked_mul(i128::from(days))?,
        DAYS_PER_YEAR,
    )
}

/// Whether `a` and `b` lie within [`SIMILAR_MARGIN_BP`] of the larger.
fn within_margin(a: i64, b: i64) -> bool {
    let gap = (i128::from(a) - i128::from(b)).abs();
    let larger = i128::from(a).abs().max(i128::from(b).abs());
    gap * BP <= larger * i128::from(SIMILAR_MARGIN_BP)
}

/// Sum the parts, or `None` if any is unknown or the sum overflows.
fn total(parts: &[Option<i64>]) -> Option<i64> {
    parts
        .iter()
        .try_fold(0_i64, |sum, part| sum.checked_add((*part)?))
}

/// Compare training the candidate with hiring (WPM-R229). In this comparison the fees and the hours are summed, because each
/// side needs one figure; every part is returned beside the total so neither is hidden in it.
#[must_use]
pub fn train_or_hire(train: TrainSide, hire: HireSide) -> Comparison {
    let mut missing = Vec::new();
    let mut need = |known: bool, name: &'static str| {
        if !known && !missing.contains(&name) {
            missing.push(name);
        }
    };
    need(train.fees_minor.is_some(), "fee");
    need(train.hours_minor.is_some(), "hours_cost");
    need(train.ramp_up_days.is_some(), "learning_ramp_up_days");
    need(train.annual_minor.is_some(), "candidate_pay");
    need(hire.recruitment_minor.is_some(), "recruitment_cost");
    need(hire.vacancy_days.is_some(), "vacancy_days");
    need(hire.ramp_up_days.is_some(), "recruitment_ramp_up_days");
    need(hire.annual_minor.is_some(), "seat_cost");

    let of = |annual: Option<i64>, days: Option<i64>| days_of(annual?, days?);
    let train_ramp_up_minor = of(train.annual_minor, train.ramp_up_days);
    let vacancy_minor = of(hire.annual_minor, hire.vacancy_days);
    let hire_ramp_up_minor = of(hire.annual_minor, hire.ramp_up_days);
    let pay_difference_minor = match (hire.annual_minor, train.annual_minor) {
        (Some(seat), Some(candidate)) => seat.checked_sub(candidate),
        _ => None,
    };
    let train_total_minor = total(&[train.fees_minor, train.hours_minor, train_ramp_up_minor]);
    let hire_total_minor = total(&[
        hire.recruitment_minor,
        vacancy_minor,
        hire_ramp_up_minor,
        pay_difference_minor,
    ]);
    let answer = match (train_total_minor, hire_total_minor) {
        (Some(t), Some(h)) if within_margin(t, h) => Answer::Similar,
        (Some(t), Some(h)) if t < h => Answer::Train,
        (Some(_), Some(_)) => Answer::Hire,
        _ => Answer::Unknown,
    };
    Comparison {
        answer,
        train_total_minor,
        hire_total_minor,
        train_ramp_up_minor,
        vacancy_minor,
        hire_ramp_up_minor,
        pay_difference_minor,
        similar_margin_bp: SIMILAR_MARGIN_BP,
        missing,
    }
}

/// One cash flow.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub struct Flow {
    /// The period it falls in; 0 is now.
    pub period: u32,
    /// Minor units; negative is out, positive is in.
    pub amount_minor: i64,
}

/// A point in time between whole periods.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub struct Payback {
    /// Whole periods after the last outflow's period.
    pub periods: u32,
    /// Basis points of the next period, 0 to 9999.
    pub fraction_bp: u32,
}

/// What the figures say.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum Verdict {
    /// NPV above zero.
    Viable,
    /// NPV exactly zero.
    BreakEven,
    /// NPV below zero.
    NotViable,
    /// No investment or no benefit is recorded: nothing to judge.
    Unmeasured,
}

/// Why a figure is absent.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum Absent {
    /// No flow is an outflow.
    NoInvestment,
    /// No flow is an inflow.
    NoBenefit,
    /// The series is longer than [`MAX_PERIODS`] or the arithmetic overflowed.
    OutOfRange,
}

/// The result, with its inputs echoed so the arithmetic is checkable.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct Viability {
    /// The discount rate used, basis points per period.
    pub rate_bp: u32,
    /// Net present value.
    pub npv_minor: Option<i64>,
    /// Total outflows, as a positive number.
    pub investment_minor: i64,
    /// Total inflows.
    pub benefit_minor: i64,
    /// When the undiscounted running total first reaches zero.
    pub payback: Option<Payback>,
    /// When the discounted running total first reaches zero.
    pub discounted_payback: Option<Payback>,
    /// The verdict.
    pub verdict: Verdict,
    /// Why the figures are absent, if they are.
    pub absent: Option<Absent>,
}

/// Present value of `amount` after `period` periods at `rate_bp`, half away from zero at each step.
fn present_value(amount: i64, period: u32, rate_bp: u32) -> Option<i128> {
    let mut value = i128::from(amount);
    let divisor = BP + i128::from(rate_bp);
    for _ in 0..period {
        let scaled = value.checked_mul(BP)?;
        let half = divisor / 2;
        value = if scaled >= 0 {
            (scaled + half) / divisor
        } else {
            (scaled - half) / divisor
        };
    }
    Some(value)
}

/// The first point the running total reaches zero after money has gone out, linear inside the period. `None` when it never does.
fn payback_of(flows: &[Flow], rate_bp: u32, last_out: u32) -> Option<Payback> {
    let mut sorted = flows.to_vec();
    sorted.sort_by_key(|f| f.period);
    let mut running: i128 = 0;
    for f in sorted {
        let value = present_value(f.amount_minor, f.period, rate_bp)?;
        let before = running;
        running += value;
        if before < 0 && running >= 0 && value > 0 && f.period > last_out {
            let fraction = u32::try_from(-before * BP / value).unwrap_or(0).min(9_999);
            return Some(Payback {
                periods: f.period.saturating_sub(last_out).saturating_sub(1),
                fraction_bp: fraction,
            });
        }
    }
    None
}

/// Assess a series of flows in one currency at `rate_bp` per period (WPM-D174). No investment or no benefit is `Unmeasured`,
/// never `NotViable`: a plan that has not estimated its benefit has not failed.
#[must_use]
pub fn assess(flows: &[Flow], rate_bp: u32) -> Viability {
    let sum = |wanted: fn(i64) -> bool| -> i128 {
        flows
            .iter()
            .filter(|f| wanted(f.amount_minor))
            .map(|f| i128::from(f.amount_minor).abs())
            .sum()
    };
    let investment = sum(|a| a < 0);
    let benefit = sum(|a| a > 0);
    let to_i64 = |n: i128| i64::try_from(n).unwrap_or(i64::MAX);
    let unmeasured = |absent: Absent| Viability {
        rate_bp,
        npv_minor: None,
        investment_minor: to_i64(investment),
        benefit_minor: to_i64(benefit),
        payback: None,
        discounted_payback: None,
        verdict: Verdict::Unmeasured,
        absent: Some(absent),
    };
    if flows.iter().any(|f| f.period > MAX_PERIODS) {
        return unmeasured(Absent::OutOfRange);
    }
    if investment == 0 {
        return unmeasured(Absent::NoInvestment);
    }
    if benefit == 0 {
        return unmeasured(Absent::NoBenefit);
    }
    let mut npv: i128 = 0;
    for f in flows {
        match present_value(f.amount_minor, f.period, rate_bp) {
            Some(value) => npv += value,
            None => return unmeasured(Absent::OutOfRange),
        }
    }
    let Ok(npv_minor) = i64::try_from(npv) else {
        return unmeasured(Absent::OutOfRange);
    };
    let last_out = flows
        .iter()
        .filter(|f| f.amount_minor < 0)
        .map(|f| f.period)
        .max()
        .unwrap_or(0);
    Viability {
        rate_bp,
        npv_minor: Some(npv_minor),
        investment_minor: to_i64(investment),
        benefit_minor: to_i64(benefit),
        payback: payback_of(flows, 0, last_out),
        discounted_payback: payback_of(flows, rate_bp, last_out),
        verdict: match npv_minor {
            n if n < 0 => Verdict::NotViable,
            0 => Verdict::BreakEven,
            _ => Verdict::Viable,
        },
        absent: None,
    }
}

/// A stated yearly benefit (WPM-D173): an estimate with a named source, never derived from a skill level or a score.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Benefit {
    /// The year it falls in; 1 is the first year after now.
    pub year: u32,
    /// Minor units.
    pub amount_minor: i64,
    /// ISO-4217.
    pub currency: String,
}

/// Check a stated benefit.
///
/// # Errors
///
/// A reason when the year is outside 1 to [`MAX_PERIODS`], the amount is not above zero, the currency is unknown, or the
/// source is blank or too long.
pub fn valid_benefit(
    year: i32,
    amount_minor: i64,
    currency: &str,
    source: &str,
) -> Result<(), String> {
    if !u32::try_from(year).is_ok_and(|y| (1..=MAX_PERIODS).contains(&y)) {
        return Err(format!("year must be between 1 and {MAX_PERIODS}"));
    }
    if amount_minor <= 0 {
        return Err("amount_minor must be above 0".to_string());
    }
    if !super::currency::is_known(currency) {
        return Err("currency must be an ISO-4217 code, e.g. GBP".to_string());
    }
    let source = source.trim();
    if source.is_empty() || source.chars().count() > MAX_SOURCE_CHARS {
        return Err(format!(
            "source is required (up to {MAX_SOURCE_CHARS} characters)"
        ));
    }
    Ok(())
}

/// A plan's viability two ways, because the fees are money out and the hours are time (WPM-R228, WPM-R230).
#[derive(Debug, Clone, PartialEq, Eq, Serialize)]
pub struct PlanViability {
    /// The one currency, when there is one.
    pub currency: Option<String>,
    /// The discount rate, echoed.
    pub rate_bp: Option<u32>,
    /// Viability counting the fees alone. `None` with `missing` when it cannot be worked.
    pub on_fees: Option<Viability>,
    /// Viability counting the fees and the hours.
    pub with_hours: Option<Viability>,
    /// Inputs that are missing, by name.
    pub missing: Vec<&'static str>,
}

/// The one currency of the benefits, or `None` if they use several.
fn benefit_currency(benefits: &[Benefit]) -> Option<&str> {
    let first = benefits.first()?.currency.as_str();
    benefits
        .iter()
        .all(|b| b.currency == first)
        .then_some(first)
}

/// Assess a plan. With no stated benefit nothing is judged: the answer is `unmeasured`, never `not viable`, and no benefit is
/// ever inferred. A fee that is unknown means the investment is unknown, so no figure is worked from the part that is known.
#[must_use]
pub fn plan_viability(
    cost: &PlanCost,
    hours: &HoursCost,
    benefits: &[Benefit],
    rate_bp: Option<u32>,
) -> PlanViability {
    let mut missing = Vec::new();
    if benefits.is_empty() {
        missing.push("benefit");
    }
    if rate_bp.is_none() {
        missing.push("discount_rate");
    }
    if cost.courses_without_fee > 0 {
        missing.push("fee");
    }
    let currency = benefit_currency(benefits);
    let mixed = (!benefits.is_empty() && currency.is_none())
        || cost.fees.len() > 1
        || currency
            .zip(cost.fee_currency())
            .is_some_and(|(a, b)| a != b);
    if mixed {
        missing.push("one_currency");
    }
    let currency = currency.map(ToString::to_string);
    let unmeasured = |currency: Option<String>, missing: Vec<&'static str>| PlanViability {
        currency,
        rate_bp,
        on_fees: None,
        with_hours: None,
        missing,
    };
    let (Some(rate), Some(code), false) = (rate_bp, currency.clone(), mixed) else {
        return unmeasured(currency, missing);
    };
    let Some(fees) = cost.single_fee(&code) else {
        return unmeasured(currency, missing);
    };
    let inflows = benefits.iter().map(|b| Flow {
        period: b.year,
        amount_minor: b.amount_minor,
    });
    let series = |out: i64| -> Vec<Flow> {
        std::iter::once(Flow {
            period: 0,
            amount_minor: -out,
        })
        .filter(|f| f.amount_minor != 0)
        .chain(inflows.clone())
        .collect()
    };
    let with_hours = match hours {
        HoursCost::Resolved {
            minor,
            currency: pay,
            ..
        } if *pay == code => fees
            .checked_add(*minor)
            .map(|out| assess(&series(out), rate)),
        HoursCost::Resolved { .. } => {
            missing.push("one_currency");
            None
        }
        HoursCost::Unresolved { missing: inputs } => {
            missing.extend(inputs.iter().copied());
            None
        }
    };
    PlanViability {
        currency,
        rate_bp,
        on_fees: Some(assess(&series(fees), rate)),
        with_hours,
        missing,
    }
}

/// One currency's line of a learning budget.
#[derive(Debug, Clone, PartialEq, Eq, Serialize)]
pub struct BudgetLine {
    /// ISO-4217.
    pub currency: String,
    /// The budget, if one is set in this currency.
    pub budget_minor: Option<i64>,
    /// Recorded fees.
    pub spent_minor: i64,
    /// Budget less spend, when there is a budget.
    pub remaining_minor: Option<i64>,
}

/// Recorded fees against a budget, per currency (WPM-R227): spend in a currency with no budget has no remainder, and a budget
/// is never converted to cover it.
#[must_use]
pub fn budget_lines(
    budgets: &BTreeMap<String, i64>,
    spent: &BTreeMap<String, i64>,
) -> Vec<BudgetLine> {
    let mut codes: Vec<&String> = budgets.keys().chain(spent.keys()).collect();
    codes.sort();
    codes.dedup();
    codes
        .into_iter()
        .map(|code| {
            let budget_minor = budgets.get(code).copied();
            let spent_minor = spent.get(code).copied().unwrap_or(0);
            BudgetLine {
                currency: code.clone(),
                budget_minor,
                spent_minor,
                remaining_minor: budget_minor.and_then(|b| b.checked_sub(spent_minor)),
            }
        })
        .collect()
}

#[cfg(test)]
mod tests {
    use super::*;

    fn fee(minor: i64, currency: &str, hours: u32) -> Step {
        Step {
            fee: Fee::Known {
                minor,
                currency: currency.to_string(),
            },
            hours,
            estimated_hours: false,
        }
    }

    fn unknown(hours: u32) -> Step {
        Step {
            fee: Fee::Unknown,
            hours,
            estimated_hours: false,
        }
    }

    fn mentoring(hours: u32) -> Step {
        Step {
            fee: Fee::NotACourse,
            hours,
            estimated_hours: true,
        }
    }

    fn flow(period: u32, amount_minor: i64) -> Flow {
        Flow {
            period,
            amount_minor,
        }
    }

    fn benefit(year: u32, amount_minor: i64, currency: &str) -> Benefit {
        Benefit {
            year,
            amount_minor,
            currency: currency.to_string(),
        }
    }

    #[allow(clippy::unnecessary_wraps)] // what `hours_cost` takes
    fn pay(minor: i64) -> Option<(i64, &'static str)> {
        Some((minor, "GBP"))
    }

    fn a_train_side() -> TrainSide {
        TrainSide {
            fees_minor: Some(100_000),
            hours_minor: Some(200_000),
            ramp_up_days: Some(30),
            annual_minor: Some(3_650_000),
        }
    }

    fn a_hire_side() -> HireSide {
        HireSide {
            recruitment_minor: Some(500_000),
            vacancy_days: Some(60),
            ramp_up_days: Some(90),
            annual_minor: Some(3_650_000),
        }
    }

    #[test]
    fn a_fee_is_both_parts_or_neither() {
        assert!(valid_fee(None, None).is_ok());
        assert!(valid_fee(Some(0), Some("GBP")).is_ok());
        assert!(valid_fee(Some(1), None).is_err());
        assert!(valid_fee(None, Some("GBP")).is_err());
        assert!(valid_fee(Some(-1), Some("GBP")).is_err());
        assert!(valid_fee(Some(1), Some("ZZZ")).is_err());
        assert!(valid_fee(Some(1), Some("gbp")).is_err());
    }

    #[test]
    fn plan_cost_sums_per_currency_and_counts_the_unknown() {
        let cost = plan_cost(&[
            fee(1_000, "GBP", 10),
            fee(500, "GBP", 5),
            fee(700, "EUR", 7),
            unknown(3),
            mentoring(20),
        ]);
        assert_eq!(cost.fees.get("GBP"), Some(&1_500));
        assert_eq!(cost.fees.get("EUR"), Some(&700));
        assert_eq!(cost.courses_with_fee, 3);
        assert_eq!(cost.courses_without_fee, 1);
        assert_eq!(cost.hours, 45);
        assert_eq!(cost.estimated_hours, 20);
    }

    #[test]
    fn a_free_course_is_a_recorded_zero_and_counts_as_known() {
        let cost = plan_cost(&[fee(0, "GBP", 2)]);
        assert_eq!(cost.courses_with_fee, 1);
        assert_eq!(cost.courses_without_fee, 0);
        assert_eq!(cost.single_fee("GBP"), Some(0));
    }

    #[test]
    fn a_step_that_is_not_a_course_needs_no_fee() {
        let cost = plan_cost(&[mentoring(20)]);
        assert_eq!(cost.courses_without_fee, 0);
        assert_eq!(cost.single_fee("GBP"), Some(0));
        assert_eq!(cost.fee_currency(), None);
    }

    #[test]
    fn one_total_needs_every_fee_known_in_that_currency() {
        assert_eq!(plan_cost(&[unknown(1)]).single_fee("GBP"), None);
        assert_eq!(plan_cost(&[fee(1, "EUR", 1)]).single_fee("GBP"), None);
        let both = plan_cost(&[fee(1, "EUR", 1), fee(1, "GBP", 1)]);
        assert_eq!(both.single_fee("GBP"), None);
        assert_eq!(both.fee_currency(), None);
        assert_eq!(plan_cost(&[fee(9, "GBP", 1)]).single_fee("GBP"), Some(9));
    }

    #[test]
    fn the_hours_are_costed_at_the_loaded_rate() {
        // 36 000.00 a year plus 25 percent, over 1800 hours, is 25.00 an hour; ten hours are 250.00.
        let cost = hours_cost(10, pay(3_600_000), Some(2_500), Some(1_800));
        assert_eq!(
            cost,
            HoursCost::Resolved {
                currency: "GBP".to_string(),
                hours: 10,
                hourly_minor: 2_500,
                minor: 25_000,
                on_cost_bp: 2_500,
                full_time_annual_hours: 1_800,
            }
        );
    }

    #[test]
    fn the_total_is_not_the_rounded_rate_multiplied() {
        // 1000 over 3 hours a year is 333.33 an hour, which rounds to 333; 3000 hours cost 1 000 000, not 999 000.
        let HoursCost::Resolved {
            hourly_minor,
            minor,
            ..
        } = hours_cost(3_000, pay(1_000), Some(0), Some(3))
        else {
            panic!("resolved");
        };
        assert_eq!(hourly_minor, 333);
        assert_eq!(minor, 1_000_000);
    }

    #[test]
    fn half_a_minor_unit_rounds_up() {
        let HoursCost::Resolved { minor, .. } = hours_cost(1, pay(3), Some(0), Some(2)) else {
            panic!("resolved");
        };
        assert_eq!(minor, 2);
    }

    #[test]
    fn a_missing_input_leaves_the_hours_unresolved_and_says_which() {
        assert_eq!(
            hours_cost(10, None, Some(0), Some(1_800)),
            HoursCost::Unresolved {
                missing: vec!["pay"]
            }
        );
        assert_eq!(
            hours_cost(10, None, None, None),
            HoursCost::Unresolved {
                missing: vec!["pay", "on_cost", "full_time_annual_hours"]
            }
        );
        assert_eq!(
            hours_cost(10, pay(1), Some(0), Some(0)),
            HoursCost::Unresolved {
                missing: vec!["full_time_annual_hours"]
            }
        );
    }

    #[test]
    fn a_zero_on_cost_is_a_stated_value() {
        assert!(
            hours_cost(1, pay(1_800), Some(0), Some(1_800))
                .resolved()
                .is_some()
        );
    }

    #[test]
    fn train_or_hire_totals_every_part() {
        let c = train_or_hire(a_train_side(), a_hire_side());
        // 3 650 000 a year is 10 000 a day.
        assert_eq!(c.train_ramp_up_minor, Some(300_000));
        assert_eq!(c.train_total_minor, Some(600_000));
        assert_eq!(c.vacancy_minor, Some(600_000));
        assert_eq!(c.hire_ramp_up_minor, Some(900_000));
        assert_eq!(c.pay_difference_minor, Some(0));
        assert_eq!(c.hire_total_minor, Some(2_000_000));
        assert_eq!(c.answer, Answer::Train);
        assert_eq!(c.missing, Vec::<&str>::new());
    }

    #[test]
    fn hiring_can_win() {
        let c = train_or_hire(
            TrainSide {
                fees_minor: Some(9_000_000),
                ..a_train_side()
            },
            a_hire_side(),
        );
        assert_eq!(c.answer, Answer::Hire);
    }

    #[test]
    fn a_higher_seat_cost_adds_to_the_hire_and_a_lower_one_subtracts() {
        let dearer = train_or_hire(
            a_train_side(),
            HireSide {
                annual_minor: Some(3_650_000 + 365_000),
                ..a_hire_side()
            },
        );
        assert_eq!(dearer.pay_difference_minor, Some(365_000));
        let cheaper = train_or_hire(
            a_train_side(),
            HireSide {
                annual_minor: Some(3_650_000 - 365_000),
                ..a_hire_side()
            },
        );
        assert_eq!(cheaper.pay_difference_minor, Some(-365_000));
    }

    #[test]
    fn close_totals_are_similar_and_the_margin_is_echoed() {
        let c = train_or_hire(
            TrainSide {
                fees_minor: Some(1_500_000),
                ..a_train_side()
            },
            a_hire_side(),
        );
        // 2 000 000 against 2 000 000.
        assert_eq!(c.answer, Answer::Similar);
        assert_eq!(c.similar_margin_bp, SIMILAR_MARGIN_BP);
        assert!(within_margin(90, 100));
        assert!(!within_margin(89, 100));
        assert!(within_margin(0, 0));
    }

    #[test]
    fn a_missing_input_on_either_side_is_unknown_not_a_win() {
        let no_fee = train_or_hire(
            TrainSide {
                fees_minor: None,
                ..a_train_side()
            },
            a_hire_side(),
        );
        assert_eq!(no_fee.answer, Answer::Unknown);
        assert_eq!(no_fee.train_total_minor, None);
        assert_eq!(no_fee.hire_total_minor, Some(2_000_000));
        assert_eq!(no_fee.missing, vec!["fee"]);
        let no_recruitment = train_or_hire(
            a_train_side(),
            HireSide {
                recruitment_minor: None,
                ..a_hire_side()
            },
        );
        assert_eq!(no_recruitment.answer, Answer::Unknown);
        assert_eq!(no_recruitment.train_total_minor, Some(600_000));
        assert_eq!(no_recruitment.missing, vec!["recruitment_cost"]);
    }

    #[test]
    fn nothing_known_lists_every_input() {
        let c = train_or_hire(TrainSide::default(), HireSide::default());
        assert_eq!(c.answer, Answer::Unknown);
        assert_eq!(c.missing.len(), 8);
        assert_eq!(c.pay_difference_minor, None);
    }

    #[test]
    fn a_missing_pay_figure_blanks_only_what_rests_on_it() {
        let c = train_or_hire(
            TrainSide {
                annual_minor: None,
                ..a_train_side()
            },
            a_hire_side(),
        );
        assert_eq!(c.train_ramp_up_minor, None);
        assert_eq!(c.pay_difference_minor, None);
        assert_eq!(c.vacancy_minor, Some(600_000));
        assert_eq!(c.answer, Answer::Unknown);
    }

    // The next three pin the project portfolio service's worked examples (WPM-D174).
    #[test]
    fn npv_discounts_each_period_and_the_rate_is_echoed() {
        // Each period divides by 1.1 and rounds: 600 becomes 545, then that becomes 495. 545 + 495 - 1000 = 40.
        let v = assess(&[flow(0, -1000), flow(1, 600), flow(2, 600)], 1_000);
        assert_eq!(v.rate_bp, 1_000);
        assert_eq!(v.npv_minor, Some(40));
        assert_eq!(v.investment_minor, 1000);
        assert_eq!(v.benefit_minor, 1200);
        assert_eq!(v.verdict, Verdict::Viable);
    }

    #[test]
    fn payback_is_linear_inside_the_period_and_discounting_makes_it_later() {
        let v = assess(&[flow(0, -1000), flow(1, 600), flow(2, 600)], 1_000);
        let plain = v.payback.expect("paid back");
        assert_eq!((plain.periods, plain.fraction_bp), (1, 6_666));
        let discounted = v.discounted_payback.expect("paid back");
        assert_eq!(discounted.periods, 1);
        assert!(discounted.fraction_bp > plain.fraction_bp);
    }

    #[test]
    fn a_negative_flow_rounds_away_from_zero_at_each_step() {
        assert_eq!(present_value(-15, 1, 10_000), Some(-8));
        assert_eq!(present_value(15, 1, 10_000), Some(8));
    }

    #[test]
    fn verdicts_follow_the_sign_of_the_npv() {
        assert_eq!(
            assess(&[flow(0, -100), flow(1, 50)], 0).verdict,
            Verdict::NotViable
        );
        assert_eq!(
            assess(&[flow(0, -100), flow(1, 100)], 0).verdict,
            Verdict::BreakEven
        );
        assert_eq!(
            assess(&[flow(0, -100), flow(1, 101)], 0).verdict,
            Verdict::Viable
        );
    }

    #[test]
    fn no_benefit_or_no_investment_is_unmeasured_never_not_viable() {
        let none = assess(&[flow(0, -100)], 500);
        assert_eq!(none.verdict, Verdict::Unmeasured);
        assert_eq!(none.absent, Some(Absent::NoBenefit));
        assert_eq!(none.npv_minor, None);
        let free = assess(&[flow(1, 100)], 500);
        assert_eq!(free.verdict, Verdict::Unmeasured);
        assert_eq!(free.absent, Some(Absent::NoInvestment));
    }

    #[test]
    fn a_series_past_the_cap_is_out_of_range() {
        let v = assess(&[flow(0, -100), flow(MAX_PERIODS + 1, 100)], 0);
        assert_eq!(v.absent, Some(Absent::OutOfRange));
        let at_the_cap = assess(&[flow(0, -100), flow(MAX_PERIODS, 100)], 0);
        assert_eq!(at_the_cap.absent, None);
    }

    #[test]
    fn a_benefit_is_stated_with_a_source() {
        assert!(valid_benefit(1, 100, "GBP", "Avoids one external hire").is_ok());
        assert!(valid_benefit(0, 100, "GBP", "x").is_err());
        assert!(valid_benefit(601, 100, "GBP", "x").is_err());
        assert!(valid_benefit(1, 0, "GBP", "x").is_err());
        assert!(valid_benefit(1, 100, "ZZZ", "x").is_err());
        assert!(valid_benefit(1, 100, "GBP", "   ").is_err());
        assert!(valid_benefit(1, 100, "GBP", &"x".repeat(501)).is_err());
    }

    #[test]
    fn no_stated_benefit_is_unmeasured() {
        let cost = plan_cost(&[fee(1_000, "GBP", 10)]);
        let hours = hours_cost(10, pay(1_800_000), Some(0), Some(1_800));
        let v = plan_viability(&cost, &hours, &[], Some(500));
        assert_eq!(v.on_fees, None);
        assert_eq!(v.with_hours, None);
        assert_eq!(v.missing, vec!["benefit"]);
    }

    #[test]
    fn a_plan_is_assessed_on_fees_and_with_hours() {
        let cost = plan_cost(&[fee(1_000, "GBP", 10)]);
        let hours = hours_cost(10, pay(180_000), Some(0), Some(1_800));
        let v = plan_viability(&cost, &hours, &[benefit(1, 1_500, "GBP")], Some(0));
        assert_eq!(v.currency.as_deref(), Some("GBP"));
        let on_fees = v.on_fees.expect("on fees");
        assert_eq!(on_fees.npv_minor, Some(500));
        assert_eq!(on_fees.verdict, Verdict::Viable);
        let with_hours = v.with_hours.expect("with hours");
        assert_eq!(with_hours.investment_minor, 2_000);
        assert_eq!(with_hours.npv_minor, Some(-500));
        assert_eq!(with_hours.verdict, Verdict::NotViable);
        assert_eq!(v.missing, Vec::<&str>::new());
    }

    #[test]
    fn unresolved_hours_still_leave_the_fees_view() {
        let cost = plan_cost(&[fee(1_000, "GBP", 10)]);
        let hours = hours_cost(10, None, Some(0), Some(1_800));
        let v = plan_viability(&cost, &hours, &[benefit(1, 1_500, "GBP")], Some(0));
        assert!(v.on_fees.is_some());
        assert_eq!(v.with_hours, None);
        assert_eq!(v.missing, vec!["pay"]);
    }

    #[test]
    fn an_unknown_fee_means_no_figure_at_all() {
        let cost = plan_cost(&[fee(1_000, "GBP", 10), unknown(5)]);
        let hours = hours_cost(15, pay(180_000), Some(0), Some(1_800));
        let v = plan_viability(&cost, &hours, &[benefit(1, 9_000, "GBP")], Some(0));
        assert_eq!(v.on_fees, None);
        assert_eq!(v.with_hours, None);
        assert_eq!(v.missing, vec!["fee"]);
    }

    #[test]
    fn currencies_are_never_mixed() {
        let hours = hours_cost(1, pay(180_000), Some(0), Some(1_800));
        let gbp = plan_cost(&[fee(1_000, "GBP", 1)]);
        let two = plan_viability(
            &gbp,
            &hours,
            &[benefit(1, 1, "GBP"), benefit(2, 1, "EUR")],
            Some(0),
        );
        assert_eq!(two.on_fees, None);
        assert_eq!(two.missing, vec!["one_currency"]);
        let other = plan_viability(&gbp, &hours, &[benefit(1, 1, "EUR")], Some(0));
        assert_eq!(other.on_fees, None);
        assert_eq!(other.missing, vec!["one_currency"]);
        let euro_pay = hours_cost(1, Some((180_000, "EUR")), Some(0), Some(1_800));
        let pay_differs = plan_viability(&gbp, &euro_pay, &[benefit(1, 2_000, "GBP")], Some(0));
        assert!(pay_differs.on_fees.is_some());
        assert_eq!(pay_differs.with_hours, None);
        assert_eq!(pay_differs.missing, vec!["one_currency"]);
    }

    #[test]
    fn no_discount_rate_is_no_figure() {
        let cost = plan_cost(&[fee(1_000, "GBP", 1)]);
        let hours = hours_cost(1, pay(180_000), Some(0), Some(1_800));
        let v = plan_viability(&cost, &hours, &[benefit(1, 2_000, "GBP")], None);
        assert_eq!(v.on_fees, None);
        assert_eq!(v.missing, vec!["discount_rate"]);
    }

    #[test]
    fn a_plan_of_only_mentoring_has_no_fee_investment() {
        let cost = plan_cost(&[mentoring(20)]);
        let hours = hours_cost(20, pay(180_000), Some(0), Some(1_800));
        let v = plan_viability(&cost, &hours, &[benefit(1, 3_000, "GBP")], Some(0));
        assert_eq!(
            v.on_fees.expect("assessed").absent,
            Some(Absent::NoInvestment)
        );
        assert_eq!(v.with_hours.expect("assessed").npv_minor, Some(1_000));
    }

    #[test]
    fn budget_lines_compare_each_currency_alone() {
        let budgets = BTreeMap::from([("GBP".to_string(), 1_000_i64)]);
        let spent = BTreeMap::from([("GBP".to_string(), 400_i64), ("EUR".to_string(), 50_i64)]);
        let lines = budget_lines(&budgets, &spent);
        assert_eq!(lines.len(), 2);
        assert_eq!(lines[0].currency, "EUR");
        assert_eq!(lines[0].budget_minor, None);
        assert_eq!(lines[0].remaining_minor, None);
        assert_eq!(lines[1].remaining_minor, Some(600));
    }

    #[test]
    fn a_budget_with_no_spend_is_all_remaining() {
        let budgets = BTreeMap::from([("GBP".to_string(), 1_000_i64)]);
        let lines = budget_lines(&budgets, &BTreeMap::new());
        assert_eq!(lines[0].spent_minor, 0);
        assert_eq!(lines[0].remaining_minor, Some(1_000));
    }
}
