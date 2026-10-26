//! Pure rules for **financial-planning-led workforce planning** (WPM-R37):
//! what closing a plan's headcount gap by hiring would cost per year, and
//! whether it fits a budget.
//!
//! DB-free. Money is whole minor units in a single currency — a total never
//! mixes currencies — and every figure is an **annual run-rate** of the
//! salaries only (plus a stated on-cost), not recruitment or onboarding.
//! A department average is published only for a cohort large enough not to
//! expose an individual's salary ([`MIN_COHORT`]).

/// The fewest salaried workers a department average may rest on.
pub const MIN_COHORT: usize = 5;

/// Mean of a cohort's annual salaries (minor units); `None` when the cohort
/// is smaller than [`MIN_COHORT`] — an average of one or two people is that
/// person's salary.
#[must_use]
pub fn cohort_average_minor(salaries: &[i64]) -> Option<i64> {
    if salaries.len() < MIN_COHORT {
        return None;
    }
    let total: i128 = salaries.iter().map(|s| i128::from(*s)).sum();
    let n = i128::try_from(salaries.len()).ok()?;
    i64::try_from((total + n / 2) / n).ok()
}

/// An annual cost with a stated employer on-cost (basis points of salary:
/// 2500 = +25%), rounded to the nearest minor unit.
#[must_use]
pub fn with_on_cost(annual_minor: i64, on_cost_bp: i32) -> i64 {
    let factor = i128::from(10_000 + on_cost_bp.max(0));
    i64::try_from((i128::from(annual_minor) * factor + 5_000) / 10_000).unwrap_or(i64::MAX)
}

/// How many people must be hired to close a headcount gap (zero for a
/// surplus).
#[must_use]
pub fn hires_to_close(gap: i64) -> usize {
    usize::try_from(gap.max(0)).unwrap_or(0)
}

/// Where a unit cost came from.
#[derive(Debug, PartialEq, Eq, Clone, Copy)]
pub enum CostSource {
    /// The median of a salary benchmark for the role.
    Benchmark,
    /// The average salary of the department's current workers.
    DepartmentAverage,
}

impl CostSource {
    /// The token the API reports.
    #[must_use]
    pub fn as_str(self) -> &'static str {
        match self {
            Self::Benchmark => "benchmark",
            Self::DepartmentAverage => "department_average",
        }
    }
}

/// Choose a unit cost: a role benchmark when there is one, otherwise a
/// publishable department average, otherwise none (never a guess).
#[must_use]
pub fn pick_unit_cost(
    benchmark_minor: Option<i64>,
    department_average_minor: Option<i64>,
) -> Option<(i64, CostSource)> {
    benchmark_minor
        .map(|b| (b, CostSource::Benchmark))
        .or_else(|| department_average_minor.map(|a| (a, CostSource::DepartmentAverage)))
}

/// Weighted mean of `(unit_cost, weight)` pairs, rounded; `None` when the
/// weights sum to zero.
#[must_use]
pub fn weighted_unit_cost(parts: &[(i64, usize)]) -> Option<i64> {
    let weight: i128 = parts
        .iter()
        .map(|(_, w)| i128::try_from(*w).unwrap_or(0))
        .sum();
    if weight == 0 {
        return None;
    }
    let total: i128 = parts
        .iter()
        .map(|(cost, w)| i128::from(*cost) * i128::try_from(*w).unwrap_or(0))
        .sum();
    i64::try_from((total + weight / 2) / weight).ok()
}

/// Annual cost of `hires` people at `unit_annual_minor`, with on-cost.
#[must_use]
pub fn annual_cost(hires: usize, unit_annual_minor: i64, on_cost_bp: i32) -> i64 {
    let hires = i128::try_from(hires).unwrap_or(0);
    let each = i128::from(with_on_cost(unit_annual_minor, on_cost_bp));
    i64::try_from(hires * each).unwrap_or(i64::MAX)
}

/// The annual cost of one demand group, or why there is none: the reason is
/// `salary_not_visible` first (money is withheld whatever else is known), then
/// `insufficient_history` (no gap to cost), then `no_unit_cost`.
#[must_use]
pub fn group_annual(
    hires: Option<usize>,
    unit: Option<(i64, CostSource)>,
    salary_visible: bool,
    on_cost_bp: i32,
) -> (Option<i64>, Option<&'static str>) {
    match (hires, unit, salary_visible) {
        (_, _, false) => (None, Some("salary_not_visible")),
        (None, _, _) => (None, Some("insufficient_history")),
        (Some(_), None, _) => (None, Some("no_unit_cost")),
        (Some(h), Some((unit_cost, _)), true) => {
            (Some(annual_cost(h, unit_cost, on_cost_bp)), None)
        }
    }
}

/// The plan's budget when it is stated in the currency being costed; budgets
/// are never converted.
#[must_use]
pub fn budget_in(
    budget_minor: Option<i64>,
    budget_currency: Option<&str>,
    currency: &str,
) -> Option<i64> {
    match (budget_minor, budget_currency) {
        (Some(minor), Some(code)) if code == currency => Some(minor),
        _ => None,
    }
}

/// Budget remaining after `cost`, and whether the cost fits.
#[must_use]
pub fn against_budget(cost_minor: i64, budget_minor: i64) -> (i64, bool) {
    (budget_minor - cost_minor, cost_minor <= budget_minor)
}

/// Whether `code` looks like an ISO-4217 currency code (three uppercase
/// ASCII letters).
#[must_use]
pub fn valid_currency(code: &str) -> bool {
    code.len() == 3 && code.bytes().all(|b| b.is_ascii_uppercase())
}

#[cfg(test)]
mod tests {
    use super::*;

    /// A department average needs a large enough cohort.
    #[test]
    fn small_cohorts_are_not_averaged() {
        assert_eq!(cohort_average_minor(&[]), None);
        assert_eq!(
            cohort_average_minor(&[3_000_000; 4]),
            None,
            "four is too few"
        );
        assert_eq!(cohort_average_minor(&[3_000_000; 5]), Some(3_000_000));
        assert_eq!(
            cohort_average_minor(&[1_000, 2_000, 3_000, 4_000, 5_000]),
            Some(3_000)
        );
        assert_eq!(
            cohort_average_minor(&[1, 2, 2, 2, 2]),
            Some(2),
            "rounds to nearest"
        );
    }

    /// On-cost is a stated percentage, rounded; negative input is ignored.
    #[test]
    fn on_cost() {
        assert_eq!(with_on_cost(4_000_000, 0), 4_000_000);
        assert_eq!(with_on_cost(4_000_000, 2500), 5_000_000);
        assert_eq!(with_on_cost(100, 3333), 133);
        assert_eq!(with_on_cost(4_000_000, -500), 4_000_000);
    }

    /// Hires close a shortfall; a surplus needs none.
    #[test]
    fn hires() {
        assert_eq!(hires_to_close(3), 3);
        assert_eq!(hires_to_close(0), 0);
        assert_eq!(hires_to_close(-4), 0);
    }

    /// Benchmark beats department average; neither ⇒ nothing, not a guess.
    #[test]
    fn unit_cost_selection() {
        assert_eq!(
            pick_unit_cost(Some(5), Some(9)),
            Some((5, CostSource::Benchmark))
        );
        assert_eq!(
            pick_unit_cost(None, Some(9)),
            Some((9, CostSource::DepartmentAverage))
        );
        assert_eq!(pick_unit_cost(None, None), None);
        assert_eq!(CostSource::Benchmark.as_str(), "benchmark");
    }

    /// Weighted means and annual cost arithmetic.
    #[test]
    fn weighted_and_annual() {
        assert_eq!(weighted_unit_cost(&[(100, 1), (200, 3)]), Some(175));
        assert_eq!(weighted_unit_cost(&[(100, 0)]), None);
        assert_eq!(weighted_unit_cost(&[]), None);
        assert_eq!(annual_cost(3, 4_000_000, 2500), 15_000_000);
        assert_eq!(annual_cost(0, 4_000_000, 2500), 0);
    }

    /// Budget comparison and currency codes.
    #[test]
    fn budget_and_currency() {
        assert_eq!(against_budget(80, 100), (20, true));
        assert_eq!(against_budget(100, 100), (0, true));
        assert_eq!(against_budget(120, 100), (-20, false));
        assert!(valid_currency("GBP"));
        assert!(!valid_currency("gbp") && !valid_currency("GB") && !valid_currency("GBPP"));
    }

    /// Money is withheld before anything else is considered; a missing gap or unit cost says why.
    #[test]
    fn a_group_cost_names_the_reason_it_has_none() {
        let unit = Some((5_000_000, CostSource::Benchmark));
        assert_eq!(
            group_annual(Some(2), unit, false, 0),
            (None, Some("salary_not_visible"))
        );
        assert_eq!(
            group_annual(None, unit, false, 0),
            (None, Some("salary_not_visible"))
        );
        assert_eq!(
            group_annual(None, unit, true, 0),
            (None, Some("insufficient_history"))
        );
        assert_eq!(
            group_annual(Some(2), None, true, 0),
            (None, Some("no_unit_cost"))
        );
        assert_eq!(
            group_annual(Some(2), unit, true, 2_000),
            (Some(12_000_000), None)
        );
        assert_eq!(
            group_annual(Some(0), unit, true, 2_000),
            (Some(0), None),
            "no hires costs nothing, it is not unknown"
        );
    }

    /// A budget in another currency is not compared, never converted.
    #[test]
    fn a_budget_applies_only_in_its_own_currency() {
        assert_eq!(budget_in(Some(9), Some("GBP"), "GBP"), Some(9));
        assert_eq!(budget_in(Some(9), Some("GBP"), "EUR"), None);
        assert_eq!(budget_in(Some(9), None, "GBP"), None);
        assert_eq!(budget_in(None, Some("GBP"), "GBP"), None);
    }
}
