//! Payslip arithmetic (WPM-R13, WPM-D4/D5), DB-free: minor units
//! only, every operation overflow-checked, and the reconciliation
//! invariant `net = gross − Σ deductions` enforced by construction
//! and re-checked before persist.
//!
//! The tax line is a deliberate **stub** (demo software): a flat rate
//! above a monthly allowance, held in a [`TaxStub`] the caller passes
//! (WPM-T383). The stub is written for **one currency**, because its
//! allowance is an amount of that currency's minor unit, and a payslip
//! in any other currency is refused rather than taxed with a figure that
//! means nothing there. Its example values ([`TaxStub::example`]) are the
//! shape of the UK's income tax for 2025-26 (a personal allowance of
//! £12,570 a year, divided by 12, and the 20% basic rate) and are written
//! for GBP; they are not a statement of any country's tax. Production
//! requires jurisdiction-correct tables (spec `regulatory.md`; gate WPM-G2).

/// One deduction line on a payslip.
#[derive(Debug, Clone, PartialEq, Eq, serde::Serialize, serde::Deserialize)]
pub struct Deduction {
    /// Human label (`tax`, `pension`, a benefit plan name, …).
    pub label: String,
    /// Amount in minor units (non-negative).
    pub amount_minor: i64,
}

/// A computed payslip: gross, the deduction lines, and the net.
#[derive(Debug, Clone, PartialEq, Eq, serde::Serialize, serde::Deserialize)]
pub struct Payslip {
    /// Gross pay in minor units.
    pub gross_minor: i64,
    /// The deduction lines.
    pub deductions: Vec<Deduction>,
    /// Net pay: always `gross - Σ deductions`.
    pub net_minor: i64,
}

/// Employment types paid **outside** payroll (WPM-D58): a contractor is paid against an
/// invoice through their supplier, so a payroll run produces no payslip for one even if a
/// rate is recorded in the salary field.
pub const OFF_PAYROLL_TYPES: &[&str] = &["contractor"];

/// Whether a worker of this employment type is paid through payroll.
#[must_use]
pub fn on_payroll(employment_type: &str) -> bool {
    !OFF_PAYROLL_TYPES.contains(&employment_type)
}

/// The example stub's flat tax rate (percent) above the allowance: the
/// UK's 20% basic rate, 2025-26 (WPM-T383).
pub const TAX_RATE_PERCENT: i64 = 20;
/// The example stub's monthly tax-free allowance in minor units of
/// [`EXAMPLE_CURRENCY`]: the UK's personal allowance for 2025-26,
/// £12,570 a year, divided by 12 (£1,047.50).
pub const TAX_FREE_MINOR: i64 = 104_750;
/// The currency the example stub's allowance is written in.
pub const EXAMPLE_CURRENCY: &str = "GBP";
/// The example's working minutes in a month, for the hourly rate of
/// overtime: a 7.5-hour day (`workforce::CONTRACTED_DAY_MINUTES`) times
/// about 21.67 working days. A starting point, no regime's.
pub const MONTH_MINUTES: i64 = 9750;
/// The most minutes a month can hold (31 days of 24 hours).
pub const MAX_MONTH_MINUTES: i64 = 31 * 24 * 60;

/// The stub's parameters: the currency it is written for, the monthly
/// allowance in that currency's minor unit, the flat rate above it, and
/// the working minutes in a month that turn a monthly base into an
/// hourly rate for overtime (WPM-T383).
#[derive(Debug, Clone, PartialEq, Eq, serde::Serialize)]
pub struct TaxStub {
    /// The ISO 4217 code the allowance is an amount of.
    pub currency: String,
    /// Monthly tax-free allowance, minor units of `currency`.
    pub allowance_minor: i64,
    /// Flat rate above the allowance, whole percent (0 to 100).
    pub rate_percent: i64,
    /// Working minutes in a month, for the overtime rate.
    pub month_minutes: i64,
}

impl TaxStub {
    /// The example stub: the UK-shaped values above, written for GBP.
    #[must_use]
    pub fn example() -> Self {
        Self {
            currency: EXAMPLE_CURRENCY.to_string(),
            allowance_minor: TAX_FREE_MINOR,
            rate_percent: TAX_RATE_PERCENT,
            month_minutes: MONTH_MINUTES,
        }
    }

    /// Check a stub: a known currency, a non-negative allowance, a rate
    /// of 0 to 100 and 1 to [`MAX_MONTH_MINUTES`] minutes in a month.
    ///
    /// # Errors
    ///
    /// The first problem found.
    pub fn validate(&self) -> Result<(), String> {
        if !super::currency::is_known(&self.currency) {
            return Err(format!(
                "the stub's currency {} is not an ISO 4217 code with a minor unit",
                self.currency
            ));
        }
        if self.allowance_minor < 0 {
            return Err("the stub's allowance must not be negative".to_string());
        }
        if !(0..=100).contains(&self.rate_percent) {
            return Err("the stub's rate must be 0 to 100 percent".to_string());
        }
        if !(1..=MAX_MONTH_MINUTES).contains(&self.month_minutes) {
            return Err(format!(
                "the minutes in a month must be 1 to {MAX_MONTH_MINUTES}"
            ));
        }
        Ok(())
    }

    /// Whether this stub may tax a payslip in `currency`: only its own.
    ///
    /// # Errors
    ///
    /// A refusal naming both currencies.
    pub fn accepts(&self, currency: &str) -> Result<(), String> {
        if currency == self.currency {
            Ok(())
        } else {
            Err(format!(
                "the payroll stub is written for {}, so a payslip in {currency} is refused",
                self.currency
            ))
        }
    }
}

/// Monthly base pay: `annual × fte% / 100 / 12`, overflow-refused.
///
/// # Errors
///
/// `"overflow"`-flavoured message when the arithmetic overflows, or a
/// refusal for a non-positive salary / out-of-range FTE.
pub fn monthly_base_minor(annual_salary_minor: i64, fte_percent: i32) -> Result<i64, String> {
    if annual_salary_minor < 0 {
        return Err("salary must be non-negative".to_string());
    }
    if !(1..=100).contains(&fte_percent) {
        return Err(format!("fte_percent {fte_percent} out of range 1-100"));
    }
    annual_salary_minor
        .checked_mul(i64::from(fte_percent))
        .and_then(|v| v.checked_div(100))
        .and_then(|v| v.checked_div(12))
        .ok_or_else(|| "salary arithmetic overflows".to_string())
}

/// Overtime pay for the period: `minutes × hourly`, where hourly
/// derives from the monthly base over `month_minutes`.
///
/// # Errors
///
/// On arithmetic overflow, or `month_minutes` not positive.
pub fn overtime_pay_minor(
    monthly_base_minor: i64,
    overtime_minutes: i64,
    month_minutes: i64,
) -> Result<i64, String> {
    if overtime_minutes <= 0 {
        return Ok(0);
    }
    if month_minutes <= 0 {
        return Err("the minutes in a month must be positive".to_string());
    }
    monthly_base_minor
        .checked_mul(overtime_minutes)
        .and_then(|v| v.checked_div(month_minutes))
        .ok_or_else(|| "overtime arithmetic overflows".to_string())
}

/// The stub tax line: the stub's rate of gross above its allowance,
/// floor zero.
///
/// # Errors
///
/// On arithmetic overflow.
pub fn stub_tax_minor(stub: &TaxStub, gross_minor: i64) -> Result<i64, String> {
    let taxable = gross_minor.saturating_sub(stub.allowance_minor).max(0);
    taxable
        .checked_mul(stub.rate_percent)
        .map(|v| v / 100)
        .ok_or_else(|| "tax arithmetic overflows".to_string())
}

/// Compute one payslip from the period inputs: monthly base (salary ×
/// FTE), overtime pay, then deductions (stub tax + the benefit
/// worker-costs). The net is derived, never supplied.
///
/// # Errors
///
/// A payslip in a currency the stub is not written for; overflow
/// anywhere; a negative benefit cost.
pub fn compute_payslip(
    stub: &TaxStub,
    currency: &str,
    annual_salary_minor: i64,
    fte_percent: i32,
    overtime_minutes: i64,
    benefit_costs: &[(String, i64)],
) -> Result<Payslip, String> {
    stub.accepts(currency)?;
    let base = monthly_base_minor(annual_salary_minor, fte_percent)?;
    let overtime = overtime_pay_minor(base, overtime_minutes, stub.month_minutes)?;
    let gross = base
        .checked_add(overtime)
        .ok_or_else(|| "gross overflows".to_string())?;
    let mut deductions = vec![Deduction {
        label: "tax".to_string(),
        amount_minor: stub_tax_minor(stub, gross)?,
    }];
    for (label, cost) in benefit_costs {
        if *cost < 0 {
            return Err(format!("benefit cost for {label} is negative"));
        }
        deductions.push(Deduction {
            label: label.clone(),
            amount_minor: *cost,
        });
    }
    let total: i64 = deductions
        .iter()
        .try_fold(0_i64, |acc, d| acc.checked_add(d.amount_minor))
        .ok_or_else(|| "deductions overflow".to_string())?;
    let net = gross
        .checked_sub(total)
        .ok_or_else(|| "net overflows".to_string())?;
    let slip = Payslip {
        gross_minor: gross,
        deductions,
        net_minor: net,
    };
    reconcile(&slip)?;
    Ok(slip)
}

/// The persist-gate reconciliation: `net = gross − Σ deductions`
/// (WPM-R13). Called by [`compute_payslip`] and again by the
/// controller before insert, so a hand-constructed slip cannot lie.
///
/// # Errors
///
/// When the invariant does not hold or the sum overflows.
pub fn reconcile(slip: &Payslip) -> Result<(), String> {
    let total: i64 = slip
        .deductions
        .iter()
        .try_fold(0_i64, |acc, d| acc.checked_add(d.amount_minor))
        .ok_or_else(|| "deductions overflow".to_string())?;
    let expected = slip
        .gross_minor
        .checked_sub(total)
        .ok_or_else(|| "net overflows".to_string())?;
    if expected == slip.net_minor {
        Ok(())
    } else {
        Err(format!(
            "payslip does not reconcile: gross {} - deductions {} != net {}",
            slip.gross_minor, total, slip.net_minor
        ))
    }
}

#[cfg(test)]
mod tests {
    #[test]
    fn a_contractor_is_not_on_payroll_and_everyone_else_is() {
        use super::on_payroll;
        assert!(!on_payroll("contractor"));
        for kind in crate::rules::tokens::EMPLOYMENT_TYPES {
            assert_eq!(on_payroll(kind), *kind != "contractor", "{kind}");
        }
        // An unknown type is not silently dropped from pay.
        assert!(on_payroll("something_new"));
    }

    use super::*;

    /// Base pay pro-rates by FTE and refuses junk inputs.
    #[test]
    fn monthly_base_pro_rates() {
        assert_eq!(monthly_base_minor(3_600_000, 100).unwrap(), 300_000);
        assert_eq!(monthly_base_minor(3_600_000, 50).unwrap(), 150_000);
        assert!(monthly_base_minor(-1, 100).is_err());
        assert!(monthly_base_minor(100, 0).is_err());
        assert!(monthly_base_minor(100, 101).is_err());
        assert!(
            monthly_base_minor(i64::MAX, 99)
                .unwrap_err()
                .contains("overflow")
        );
    }

    /// Overtime pay derives from the monthly rate; non-positive
    /// minutes cost nothing.
    #[test]
    fn overtime_pay() {
        // £3,000.00 monthly => hourly-ish rate over 9750 min.
        assert_eq!(
            overtime_pay_minor(300_000, MONTH_MINUTES, MONTH_MINUTES).unwrap(),
            300_000
        );
        assert_eq!(overtime_pay_minor(300_000, 0, MONTH_MINUTES).unwrap(), 0);
        assert_eq!(overtime_pay_minor(300_000, -60, MONTH_MINUTES).unwrap(), 0);
        assert!(overtime_pay_minor(i64::MAX, 2, MONTH_MINUTES).is_err());
        // Another month: 8 hours a day for 20 days is 9,600 minutes.
        assert_eq!(overtime_pay_minor(9_600, 60, 9_600).unwrap(), 60);
        assert!(overtime_pay_minor(9_600, 60, 0).is_err());
    }

    /// The stub tax kicks in only above the allowance.
    #[test]
    fn stub_tax() {
        let stub = TaxStub::example();
        assert_eq!(stub_tax_minor(&stub, 100_000).unwrap(), 0);
        assert_eq!(stub_tax_minor(&stub, TAX_FREE_MINOR).unwrap(), 0);
        assert_eq!(
            stub_tax_minor(&stub, TAX_FREE_MINOR + 100_000).unwrap(),
            20_000
        );
        assert_eq!(stub_tax_minor(&stub, i64::MIN).unwrap(), 0);
    }

    /// The example is the GBP one, and checks; a stub in another
    /// currency of another exponent carries its own allowance and rate.
    #[test]
    fn a_stub_is_written_for_one_currency() {
        let example = TaxStub::example();
        assert_eq!(example.currency, "GBP");
        assert_eq!(example.validate(), Ok(()));
        // A yen stub (exponent 0): an allowance of 100,000 yen, 10%.
        let yen = TaxStub {
            currency: "JPY".to_string(),
            allowance_minor: 100_000,
            rate_percent: 10,
            month_minutes: 9_600,
        };
        assert_eq!(yen.validate(), Ok(()));
        assert_eq!(stub_tax_minor(&yen, 300_000).unwrap(), 20_000);
        let slip = compute_payslip(&yen, "JPY", 3_600_000, 100, 0, &[]).unwrap();
        assert_eq!(slip.gross_minor, 300_000);
        assert_eq!(slip.net_minor, 280_000);
        // A payslip in a currency the stub was not written for is refused.
        let refused = compute_payslip(&example, "JPY", 3_600_000, 100, 0, &[]).unwrap_err();
        assert!(
            refused.contains("GBP") && refused.contains("JPY"),
            "{refused}"
        );
        assert!(compute_payslip(&yen, "GBP", 3_600_000, 100, 0, &[]).is_err());
        // Bad stubs.
        for (bad, why) in [
            (
                TaxStub {
                    currency: "XXX".into(),
                    ..TaxStub::example()
                },
                "currency",
            ),
            (
                TaxStub {
                    allowance_minor: -1,
                    ..TaxStub::example()
                },
                "allowance",
            ),
            (
                TaxStub {
                    rate_percent: 101,
                    ..TaxStub::example()
                },
                "rate",
            ),
            (
                TaxStub {
                    rate_percent: -1,
                    ..TaxStub::example()
                },
                "rate",
            ),
            (
                TaxStub {
                    month_minutes: 0,
                    ..TaxStub::example()
                },
                "minutes",
            ),
            (
                TaxStub {
                    month_minutes: MAX_MONTH_MINUTES + 1,
                    ..TaxStub::example()
                },
                "minutes",
            ),
        ] {
            assert!(bad.validate().unwrap_err().contains(why), "{why}");
        }
    }

    /// A full slip reconciles by construction: gross = base + overtime,
    /// net = gross − tax − benefits.
    #[test]
    fn compute_reconciles() {
        let stub = TaxStub::example();
        let slip = compute_payslip(
            &stub,
            "GBP",
            3_600_000, // £36,000.00
            100,
            0,
            &[("pension".to_string(), 15_000)],
        )
        .unwrap();
        assert_eq!(slip.gross_minor, 300_000);
        let tax = stub_tax_minor(&stub, 300_000).unwrap();
        assert_eq!(slip.net_minor, 300_000 - tax - 15_000);
        assert!(reconcile(&slip).is_ok());
    }

    /// Tampering with the net fails the persist gate; a negative
    /// benefit cost is refused outright.
    #[test]
    fn reconcile_gate_catches_lies() {
        let stub = TaxStub::example();
        let mut slip = compute_payslip(&stub, "GBP", 3_600_000, 100, 0, &[]).unwrap();
        slip.net_minor += 1;
        assert!(reconcile(&slip).unwrap_err().contains("reconcile"));
        assert!(
            compute_payslip(&stub, "GBP", 3_600_000, 100, 0, &[("x".to_string(), -1)]).is_err()
        );
    }

    /// Overflow anywhere is an error, never a wrap or a panic.
    #[test]
    fn overflow_is_refused() {
        assert!(compute_payslip(&TaxStub::example(), "GBP", i64::MAX, 100, i64::MAX, &[]).is_err());
        let slip = Payslip {
            gross_minor: 0,
            deductions: vec![
                Deduction {
                    label: "a".into(),
                    amount_minor: i64::MAX,
                },
                Deduction {
                    label: "b".into(),
                    amount_minor: 1,
                },
            ],
            net_minor: 0,
        };
        assert!(reconcile(&slip).unwrap_err().contains("overflow"));
    }
}
