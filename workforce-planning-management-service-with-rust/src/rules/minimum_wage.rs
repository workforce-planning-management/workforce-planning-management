//! Pure rules for the minimum-wage floor check (WPM-R218, WPM-D154, WPM-D156, WPM-D160): the pay a worker received for a
//! period against the legal floor for their age band, as an **indicative, advisory** result for HR. No I/O, no clock;
//! arithmetic is widened to `i128` and never panics.
//!
//! The floors are **jurisdiction parameters** (WPM-D144, WPM-D155), resolved from the worker's place of work by
//! `rules::jurisdiction`, never constants. Three things the check refuses to do:
//!
//! * **Read no rule as a pass.** With no floor configured the answer is `unknown` (`no_floor_configured`), never `ok`. A
//!   jurisdiction recorded as setting no statutory floor (Sweden and Denmark: pay floors sit in collective agreements) says
//!   `none_by_law`, which is a different answer from "not configured" (WPM-D160).
//! * **Keep no age.** The date of birth is used only to place the worker in a band at the end of the period (WPM-D114's
//!   "pay" purpose); a jurisdiction with one floor for everyone never reads it, and only the band and the outcome leave this
//!   module.
//! * **Claim no more than it knows.** The pay is whatever the caller counts (the payslip's gross), so accommodation offsets,
//!   tips, piece work, apprentices' special rules and which pay the law counts are **not modelled**; a result is indicative,
//!   and `ok` is not a statement that an employer complies.
//!
//! A floor has a **basis** (WPM-D160): hourly (the United Kingdom, Germany, France, Japan, South Korea, the United States) or
//! monthly (Spain, China). A monthly floor is brought to the period by calendar days, the number of payments a year (Spain
//! states 14) and the worker's FTE.

use chrono::{Datelike, NaiveDate};

/// The registry key of the age from which the adult floor applies, in whole years. Zero or unset means every worker is in the
/// adult band and no date of birth is read.
pub const ADULT_FROM_AGE_KEY: &str = "minimum_wage.adult_from_age";
/// The registry key of the age from which the young-adult floor applies, in whole years.
pub const YOUNG_ADULT_FROM_AGE_KEY: &str = "minimum_wage.young_adult_from_age";
/// The registry key of the currency the floors are stated in (a set holding one ISO 4217 code).
pub const CURRENCY_KEY: &str = "minimum_wage.currency";
/// The registry key that records a jurisdiction as setting no statutory floor (WPM-D160).
pub const NONE_BY_LAW_KEY: &str = "minimum_wage.none_by_law";
/// The registry key of the payments a year a monthly figure is paid in (Spain: 14). Unset means twelve.
pub const PAYMENTS_PER_YEAR_KEY: &str = "minimum_wage.payments_per_year";
/// The registry key of the margin, in basis points above the floor, within which a result is `near`. Unset means none.
pub const NEAR_MARGIN_BP_KEY: &str = "minimum_wage.near_margin_bp";

/// The payments a year taken when a profile gives none.
pub const DEFAULT_PAYMENTS_PER_YEAR: i64 = 12;

/// An age band. The bands are neutral labels for the cut-offs a profile sets; apprentices are not modelled.
#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash, PartialOrd, Ord)]
pub enum Band {
    /// From the adult age (or everyone, where a profile sets no cut-off).
    Adult,
    /// From the young-adult age up to the adult age.
    YoungAdult,
    /// Below the young-adult age.
    Youth,
}

impl Band {
    /// The three bands, adult first.
    pub const ALL: [Self; 3] = [Self::Adult, Self::YoungAdult, Self::Youth];

    /// The stored and shown name.
    #[must_use]
    pub const fn name(self) -> &'static str {
        match self {
            Self::Adult => "adult",
            Self::YoungAdult => "young_adult",
            Self::Youth => "youth",
        }
    }

    /// The registry key of this band's hourly floor, in minor units of the currency.
    #[must_use]
    pub const fn hourly_key(self) -> &'static str {
        match self {
            Self::Adult => "minimum_wage.hourly_minor.adult",
            Self::YoungAdult => "minimum_wage.hourly_minor.young_adult",
            Self::Youth => "minimum_wage.hourly_minor.youth",
        }
    }

    /// The registry key of this band's monthly floor, in minor units of the currency.
    #[must_use]
    pub const fn monthly_key(self) -> &'static str {
        match self {
            Self::Adult => "minimum_wage.monthly_minor.adult",
            Self::YoungAdult => "minimum_wage.monthly_minor.young_adult",
            Self::Youth => "minimum_wage.monthly_minor.youth",
        }
    }
}

/// One band's floors, each in minor units of the profile's currency.
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
pub struct Floor {
    /// The floor for one hour worked.
    pub hourly_minor: Option<i64>,
    /// The floor for one monthly payment as the law states it (Spain: €1,221, paid 14 times a year).
    pub monthly_minor: Option<i64>,
}

impl Floor {
    /// Whether the band has a floor on either basis.
    #[must_use]
    pub const fn is_set(&self) -> bool {
        self.hourly_minor.is_some() || self.monthly_minor.is_some()
    }
}

/// The floors of all three bands.
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
pub struct Floors {
    /// The adult band's.
    pub adult: Floor,
    /// The young-adult band's.
    pub young_adult: Floor,
    /// The youth band's.
    pub youth: Floor,
}

impl Floors {
    /// The floor of `band`.
    #[must_use]
    pub const fn of(&self, band: Band) -> Floor {
        match band {
            Band::Adult => self.adult,
            Band::YoungAdult => self.young_adult,
            Band::Youth => self.youth,
        }
    }

    /// Whether any band has a floor.
    #[must_use]
    pub const fn any(&self) -> bool {
        self.adult.is_set() || self.young_adult.is_set() || self.youth.is_set()
    }
}

/// What the check is given about one worker's period.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct Inputs {
    /// The pay counted for the period, in minor units (the payslip's gross, in v1).
    pub pay_minor: i64,
    /// The minutes worked in the period.
    pub worked_minutes: i64,
    /// The calendar days in the period, both ends included.
    pub period_days: i64,
    /// The worker's FTE as a percentage, 1 to 100.
    pub fte_percent: i64,
    /// The payments a year a monthly floor is stated for.
    pub payments_per_year: i64,
    /// The margin above a floor, in basis points, within which the result is `near`.
    pub near_margin_bp: i64,
}

/// The result for one worker.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Outcome {
    /// Pay is below the floor.
    Below,
    /// Pay meets the floor but is within the configured margin of it.
    Near,
    /// Pay meets the floor by more than the margin. Not a statement of compliance.
    Ok,
    /// The jurisdiction sets no statutory floor (pay floors may sit in collective agreements): neither a pass nor a gap.
    NoneByLaw,
    /// The check could not be made, and why.
    Unknown(&'static str),
}

impl Outcome {
    /// The stored and shown name.
    #[must_use]
    pub const fn name(self) -> &'static str {
        match self {
            Self::Below => "below",
            Self::Near => "near",
            Self::Ok => "ok",
            Self::NoneByLaw => "none_by_law",
            Self::Unknown(_) => "unknown",
        }
    }

    /// Why the result is `unknown`.
    #[must_use]
    pub const fn reason(self) -> Option<&'static str> {
        match self {
            Self::Unknown(reason) => Some(reason),
            _ => None,
        }
    }

    /// How serious: higher is worse. `unknown` and `none_by_law` rank below `ok` as results to act on, but are never `ok`.
    const fn severity(self) -> u8 {
        match self {
            Self::Below => 3,
            Self::Near => 2,
            Self::Ok => 1,
            Self::NoneByLaw | Self::Unknown(_) => 0,
        }
    }
}

/// Whole years completed on `on` by someone born on `born`, never below zero. A birthday on 29 February is reached on 1 March
/// in a year with no 29 February.
#[must_use]
pub fn completed_years(born: NaiveDate, on: NaiveDate) -> i64 {
    let mut years = i64::from(on.year()) - i64::from(born.year());
    if (on.month(), on.day()) < (born.month(), born.day()) {
        years -= 1;
    }
    years.max(0)
}

/// The band for a worker, from the age on the last day of the period.
///
/// `adult_from` and `young_adult_from` are the profile's cut-offs. With none (or an adult cut-off of zero) every worker is in
/// the adult band and `age` is not needed.
///
/// # Errors
///
/// `no_date_of_birth` when the profile has cut-offs and the worker's age is not known.
pub const fn band_for(
    age: Option<i64>,
    adult_from: Option<i64>,
    young_adult_from: Option<i64>,
) -> Result<Band, &'static str> {
    let Some(adult_from) = adult_from else {
        return Ok(Band::Adult);
    };
    if adult_from <= 0 {
        return Ok(Band::Adult);
    }
    let Some(age) = age else {
        return Err("no_date_of_birth");
    };
    if age >= adult_from {
        return Ok(Band::Adult);
    }
    match young_adult_from {
        Some(young) if age >= young => Ok(Band::YoungAdult),
        _ => Ok(Band::Youth),
    }
}

/// Compare pay with one comparison's floor: `pay × scale` against `required_scaled`, both already widened.
///
/// `pay_scaled` is the pay multiplied by the comparison's denominator and `required_scaled` the floor multiplied by its numerator
/// parts, so no division happens and no rounding can move a result.
fn compare(pay_scaled: i128, required_scaled: i128, near_margin_bp: i64) -> Outcome {
    if pay_scaled < required_scaled {
        return Outcome::Below;
    }
    let margin = i128::from(near_margin_bp.max(0));
    // pay < floor × (10 000 + margin) / 10 000, kept in integers.
    if margin > 0
        && pay_scaled.saturating_mul(10_000) < required_scaled.saturating_mul(10_000 + margin)
    {
        return Outcome::Near;
    }
    Outcome::Ok
}

/// Check one worker's period.
///
/// * `band`: from [`band_for`]; an error is passed through as `unknown`, but only when some floor is configured.
/// * `floors`: all three bands' floors as resolved for the worker's place of work and the period's last day.
/// * `none_by_law`: whether the jurisdiction is recorded as setting no statutory floor.
/// * `floor_currency` and `pay_currency`: the currency the floors are stated in (none if the profile gives none) and the
///   currency of the pay; they must match.
///
/// Hourly and monthly floors that both apply are both checked and the worse result is returned.
#[must_use]
pub fn check(
    band: Result<Band, &'static str>,
    floors: &Floors,
    none_by_law: bool,
    floor_currency: Option<&str>,
    pay_currency: &str,
    inputs: &Inputs,
) -> Outcome {
    if !floors.any() {
        return if none_by_law {
            Outcome::NoneByLaw
        } else {
            Outcome::Unknown("no_floor_configured")
        };
    }
    let Some(floor_currency) = floor_currency else {
        return Outcome::Unknown("no_floor_currency");
    };
    if !floor_currency.eq_ignore_ascii_case(pay_currency) {
        return Outcome::Unknown("currency_mismatch");
    }
    let band = match band {
        Ok(band) => band,
        Err(reason) => return Outcome::Unknown(reason),
    };
    let floor = floors.of(band);
    if !floor.is_set() {
        return Outcome::Unknown("no_floor_for_band");
    }
    let pay = i128::from(inputs.pay_minor);
    let mut results: Vec<Outcome> = Vec::with_capacity(2);
    if let Some(hourly) = floor.hourly_minor
        && inputs.worked_minutes > 0
    {
        // pay / (minutes / 60) >= hourly  <=>  pay × 60 >= hourly × minutes
        results.push(compare(
            pay.saturating_mul(60),
            i128::from(hourly).saturating_mul(i128::from(inputs.worked_minutes)),
            inputs.near_margin_bp,
        ));
    }
    if let Some(monthly) = floor.monthly_minor {
        if inputs.period_days <= 0 {
            return Outcome::Unknown("no_period");
        }
        if !(1..=100).contains(&inputs.fte_percent) {
            return Outcome::Unknown("no_fte");
        }
        let payments = if inputs.payments_per_year > 0 {
            inputs.payments_per_year
        } else {
            DEFAULT_PAYMENTS_PER_YEAR
        };
        // floor for the period = monthly × payments × days / 365 × fte / 100; pay × 36 500 against monthly × payments × days × fte.
        let required_scaled = i128::from(monthly)
            .saturating_mul(i128::from(payments))
            .saturating_mul(i128::from(inputs.period_days))
            .saturating_mul(i128::from(inputs.fte_percent));
        results.push(compare(
            pay.saturating_mul(36_500),
            required_scaled,
            inputs.near_margin_bp,
        ));
    }
    results
        .into_iter()
        .max_by_key(|outcome| outcome.severity())
        .unwrap_or(Outcome::Unknown("no_hours"))
}

#[cfg(test)]
mod tests {
    use super::*;

    fn date(y: i32, m: u32, d: u32) -> NaiveDate {
        NaiveDate::from_ymd_opt(y, m, d).unwrap()
    }

    /// The `GB` shape: three bands, hourly, in pence.
    fn three_bands() -> Floors {
        Floors {
            adult: Floor {
                hourly_minor: Some(1271),
                monthly_minor: None,
            },
            young_adult: Floor {
                hourly_minor: Some(1085),
                monthly_minor: None,
            },
            youth: Floor {
                hourly_minor: Some(800),
                monthly_minor: None,
            },
        }
    }

    /// The Spanish shape: one monthly floor of €1,221 in 14 payments, in cents.
    fn monthly_only() -> Floors {
        Floors {
            adult: Floor {
                hourly_minor: None,
                monthly_minor: Some(122_100),
            },
            ..Floors::default()
        }
    }

    fn inputs(pay_minor: i64, worked_minutes: i64) -> Inputs {
        Inputs {
            pay_minor,
            worked_minutes,
            period_days: 30,
            fte_percent: 100,
            payments_per_year: 12,
            near_margin_bp: 0,
        }
    }

    fn run(floors: &Floors, band: Band, i: &Inputs) -> Outcome {
        check(Ok(band), floors, false, Some("GBP"), "GBP", i)
    }

    #[test]
    fn hourly_pay_exactly_at_the_floor_is_ok_and_a_penny_under_is_below() {
        // 10 hours at £12.71 is £127.10.
        assert_eq!(
            run(&three_bands(), Band::Adult, &inputs(12_710, 600)),
            Outcome::Ok
        );
        assert_eq!(
            run(&three_bands(), Band::Adult, &inputs(12_709, 600)),
            Outcome::Below
        );
        assert_eq!(
            run(&three_bands(), Band::Adult, &inputs(0, 600)),
            Outcome::Below
        );
    }

    #[test]
    fn the_band_picks_the_floor() {
        // The same £110.00 for 10 hours: above the youth floor, below the young-adult and adult floors.
        let i = inputs(11_000, 600);
        assert_eq!(run(&three_bands(), Band::Youth, &i), Outcome::Ok);
        assert_eq!(run(&three_bands(), Band::YoungAdult, &i), Outcome::Ok);
        assert_eq!(run(&three_bands(), Band::Adult, &i), Outcome::Below);
    }

    #[test]
    fn near_is_within_the_margin_above_the_floor_and_never_without_one() {
        let mut i = inputs(12_800, 600); // £128.00 against £127.10: 0.71 percent above
        assert_eq!(
            run(&three_bands(), Band::Adult, &i),
            Outcome::Ok,
            "no margin configured"
        );
        i.near_margin_bp = 100; // one percent
        assert_eq!(run(&three_bands(), Band::Adult, &i), Outcome::Near);
        i.near_margin_bp = 50; // half a percent
        assert_eq!(run(&three_bands(), Band::Adult, &i), Outcome::Ok);
        // Below stays below however large the margin.
        i.pay_minor = 12_000;
        i.near_margin_bp = 5000;
        assert_eq!(run(&three_bands(), Band::Adult, &i), Outcome::Below);
    }

    #[test]
    fn no_hours_is_unknown_not_a_pass() {
        assert_eq!(
            run(&three_bands(), Band::Adult, &inputs(100_000, 0)),
            Outcome::Unknown("no_hours")
        );
        assert_eq!(
            run(&three_bands(), Band::Adult, &inputs(100_000, -5)),
            Outcome::Unknown("no_hours")
        );
    }

    #[test]
    fn no_floor_is_unknown_and_none_by_law_is_its_own_answer() {
        let none = Floors::default();
        let i = inputs(100_000, 600);
        assert_eq!(
            check(Ok(Band::Adult), &none, false, None, "SEK", &i),
            Outcome::Unknown("no_floor_configured"),
            "not configured is never ok"
        );
        assert_eq!(
            check(Ok(Band::Adult), &none, true, None, "SEK", &i),
            Outcome::NoneByLaw
        );
        // A band error does not hide the answer when there is no floor at all.
        assert_eq!(
            check(Err("no_date_of_birth"), &none, true, None, "SEK", &i),
            Outcome::NoneByLaw
        );
        assert_eq!(Outcome::NoneByLaw.name(), "none_by_law");
        assert_eq!(Outcome::Unknown("x").reason(), Some("x"));
        assert_eq!(Outcome::Ok.reason(), None);
    }

    #[test]
    fn a_floor_in_another_currency_or_none_is_not_compared() {
        let i = inputs(1_000_000, 600);
        assert_eq!(
            check(
                Ok(Band::Adult),
                &three_bands(),
                false,
                Some("EUR"),
                "GBP",
                &i
            ),
            Outcome::Unknown("currency_mismatch")
        );
        assert_eq!(
            check(Ok(Band::Adult), &three_bands(), false, None, "GBP", &i),
            Outcome::Unknown("no_floor_currency")
        );
        assert_eq!(
            check(
                Ok(Band::Adult),
                &three_bands(),
                false,
                Some("gbp"),
                "GBP",
                &i
            ),
            Outcome::Ok,
            "codes compare without case"
        );
    }

    #[test]
    fn a_band_with_no_floor_is_unknown_not_the_adult_floor() {
        let only_adult = Floors {
            adult: three_bands().adult,
            ..Floors::default()
        };
        assert_eq!(
            run(&only_adult, Band::Youth, &inputs(1_000_000, 600)),
            Outcome::Unknown("no_floor_for_band")
        );
    }

    #[test]
    fn a_monthly_floor_is_brought_to_the_period_by_days_payments_and_fte() {
        // Spain: €1,221 a month in 14 payments is €17,094 a year. A whole 365-day period at full time.
        let year = |pay| Inputs {
            period_days: 365,
            payments_per_year: 14,
            ..inputs(pay, 0)
        };
        assert_eq!(
            run(&monthly_only(), Band::Adult, &year(1_709_400)),
            Outcome::Ok
        );
        assert_eq!(
            run(&monthly_only(), Band::Adult, &year(1_709_399)),
            Outcome::Below
        );
        // Half time: half the floor.
        let half = Inputs {
            fte_percent: 50,
            ..year(854_700)
        };
        assert_eq!(run(&monthly_only(), Band::Adult, &half), Outcome::Ok);
        let under_half = Inputs {
            pay_minor: 854_699,
            ..half
        };
        assert_eq!(
            run(&monthly_only(), Band::Adult, &under_half),
            Outcome::Below
        );
        // Twelve payments instead of fourteen is a lower yearly floor for the same monthly figure? No: fewer payments, less a year.
        let twelve = Inputs {
            payments_per_year: 12,
            ..year(1_465_200)
        };
        assert_eq!(run(&monthly_only(), Band::Adult, &twelve), Outcome::Ok);
        // Hours are not needed for a monthly floor.
        assert_eq!(
            run(&monthly_only(), Band::Adult, &year(1_709_400)),
            Outcome::Ok
        );
    }

    #[test]
    fn a_monthly_floor_needs_a_period_and_a_fte() {
        let bad_period = Inputs {
            period_days: 0,
            ..inputs(1_000_000, 0)
        };
        assert_eq!(
            run(&monthly_only(), Band::Adult, &bad_period),
            Outcome::Unknown("no_period")
        );
        for fte in [0, 101, -1] {
            let bad = Inputs {
                fte_percent: fte,
                ..inputs(1_000_000, 0)
            };
            assert_eq!(
                run(&monthly_only(), Band::Adult, &bad),
                Outcome::Unknown("no_fte")
            );
        }
    }

    #[test]
    fn when_both_bases_apply_the_worse_result_wins() {
        let both = Floors {
            adult: Floor {
                hourly_minor: Some(1000),
                monthly_minor: Some(200_000),
            },
            ..Floors::default()
        };
        let i = Inputs {
            period_days: 365,
            payments_per_year: 12,
            // 100 hours at 10.00 is 1,000.00 for the hourly floor; 12 × 2,000.00 = 24,000.00 for the monthly floor.
            ..inputs(150_000, 6_000)
        };
        // Hourly: 1,500.00 >= 1,000.00 ok. Monthly: 1,500.00 < 24,000.00 below.
        assert_eq!(run(&both, Band::Adult, &i), Outcome::Below);
        // Without hours the monthly comparison alone still answers.
        let no_hours = Inputs {
            worked_minutes: 0,
            ..i
        };
        assert_eq!(run(&both, Band::Adult, &no_hours), Outcome::Below);
    }

    #[test]
    fn completed_years_turn_over_on_the_birthday_and_handle_29_february() {
        let born = date(2005, 6, 15);
        assert_eq!(completed_years(born, date(2026, 6, 14)), 20);
        assert_eq!(completed_years(born, date(2026, 6, 15)), 21);
        assert_eq!(completed_years(born, date(2026, 6, 16)), 21);
        let leap = date(2004, 2, 29);
        assert_eq!(completed_years(leap, date(2025, 2, 28)), 20);
        assert_eq!(completed_years(leap, date(2025, 3, 1)), 21);
        assert_eq!(completed_years(leap, date(2028, 2, 29)), 24);
        // A date before birth is zero, not negative.
        assert_eq!(completed_years(date(2030, 1, 1), date(2026, 1, 1)), 0);
    }

    #[test]
    fn bands_follow_the_cut_offs_and_read_no_age_when_there_are_none() {
        // Cut-offs 21 and 18, as in the GB example.
        assert_eq!(band_for(Some(17), Some(21), Some(18)), Ok(Band::Youth));
        assert_eq!(band_for(Some(18), Some(21), Some(18)), Ok(Band::YoungAdult));
        assert_eq!(band_for(Some(20), Some(21), Some(18)), Ok(Band::YoungAdult));
        assert_eq!(band_for(Some(21), Some(21), Some(18)), Ok(Band::Adult));
        assert_eq!(band_for(Some(80), Some(21), Some(18)), Ok(Band::Adult));
        // No young-adult cut-off: below the adult age is youth.
        assert_eq!(band_for(Some(19), Some(21), None), Ok(Band::Youth));
        // The age is needed when there are cut-offs, and only then.
        assert_eq!(band_for(None, Some(21), Some(18)), Err("no_date_of_birth"));
        assert_eq!(band_for(None, None, None), Ok(Band::Adult));
        assert_eq!(band_for(None, Some(0), Some(18)), Ok(Band::Adult));
        assert_eq!(
            band_for(Some(3), None, None),
            Ok(Band::Adult),
            "no cut-offs: everyone is adult"
        );
    }

    #[test]
    fn a_missing_age_does_not_stop_a_single_floor_from_being_checked() {
        let band = band_for(None, None, None);
        assert_eq!(
            check(
                band,
                &three_bands(),
                false,
                Some("GBP"),
                "GBP",
                &inputs(12_710, 600)
            ),
            Outcome::Ok
        );
        let needs_age = band_for(None, Some(21), Some(18));
        assert_eq!(
            check(
                needs_age,
                &three_bands(),
                false,
                Some("GBP"),
                "GBP",
                &inputs(12_710, 600)
            ),
            Outcome::Unknown("no_date_of_birth")
        );
    }

    #[test]
    fn huge_values_neither_panic_nor_overflow_into_a_pass() {
        let big = Floors {
            adult: Floor {
                hourly_minor: Some(i64::MAX),
                monthly_minor: Some(i64::MAX),
            },
            ..Floors::default()
        };
        let i = Inputs {
            pay_minor: i64::MAX,
            worked_minutes: i64::MAX,
            period_days: i64::MAX,
            fte_percent: 100,
            payments_per_year: i64::MAX,
            near_margin_bp: i64::MAX,
        };
        // It must answer; with a floor this large nothing short of the same size passes, and there is no panic.
        let _ = run(&big, Band::Adult, &i);
        let small = Inputs { pay_minor: 1, ..i };
        assert_eq!(run(&big, Band::Adult, &small), Outcome::Below);
        // A negative pay is below any floor.
        assert_eq!(
            run(&three_bands(), Band::Adult, &inputs(-1, 600)),
            Outcome::Below
        );
    }

    #[test]
    fn the_keys_are_the_registrys() {
        for band in Band::ALL {
            assert!(band.hourly_key().starts_with("minimum_wage.hourly_minor."));
            assert!(
                band.monthly_key()
                    .starts_with("minimum_wage.monthly_minor.")
            );
            assert!(band.hourly_key().ends_with(band.name()));
            assert!(band.monthly_key().ends_with(band.name()));
        }
        for key in [
            ADULT_FROM_AGE_KEY,
            YOUNG_ADULT_FROM_AGE_KEY,
            CURRENCY_KEY,
            NONE_BY_LAW_KEY,
            PAYMENTS_PER_YEAR_KEY,
            NEAR_MARGIN_BP_KEY,
        ] {
            assert!(key.starts_with("minimum_wage."), "{key}");
        }
    }
}
