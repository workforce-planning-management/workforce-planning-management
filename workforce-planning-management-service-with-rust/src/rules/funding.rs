//! **Post funding** (WPM-R62, WPM-R83, WPM-R98): pure, database-free and clock-free rules about
//! where a post's money comes from and when it stops. Only a kind and an end date are held, never an
//! amount.

use chrono::NaiveDate;

/// Where the money comes from.
pub const KINDS: &[&str] = &["core", "time_limited", "external", "programme"];

/// Check a funding kind.
///
/// # Errors
///
/// A kind that is not on the list.
pub fn validate_kind(kind: &str) -> Result<(), String> {
    if KINDS.contains(&kind) {
        Ok(())
    } else {
        Err(format!("kind must be one of {}", KINDS.join(", ")))
    }
}

/// Check a funding record: a time-limited or external funding names its end; a funding that ends
/// is not already over before the engagement started.
///
/// # Errors
///
/// A refusal that says what to change.
pub fn validate(
    kind: &str,
    ends_on: Option<NaiveDate>,
    engagement_started_on: NaiveDate,
) -> Result<(), String> {
    validate_kind(kind)?;
    if matches!(kind, "time_limited" | "external") && ends_on.is_none() {
        return Err(format!("a {kind} funding needs the date it ends"));
    }
    if let Some(end) = ends_on
        && end < engagement_started_on
    {
        return Err("the funding cannot end before the engagement started".to_string());
    }
    Ok(())
}

/// How the contract end and the funding end compare.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Alignment {
    /// They end on the same day.
    Matched,
    /// The contract runs past the money: someone is paid with no funding.
    ContractOutlastsFunding,
    /// The money runs past the contract: funding with no one engaged, or a conversion to plan.
    FundingOutlastsContract,
    /// The funding has no end (core funding), so it cannot run out first.
    FundingOpenEnded,
    /// No funding is recorded, or the contract has no end date: nothing can be said.
    Unknown,
}

impl Alignment {
    /// The reported token.
    #[must_use]
    pub const fn as_str(self) -> &'static str {
        match self {
            Self::Matched => "matched",
            Self::ContractOutlastsFunding => "contract_outlasts_funding",
            Self::FundingOutlastsContract => "funding_outlasts_contract",
            Self::FundingOpenEnded => "funding_open_ended",
            Self::Unknown => "unknown",
        }
    }

    /// Whether a person should look at it.
    #[must_use]
    pub const fn needs_attention(self) -> bool {
        matches!(
            self,
            Self::ContractOutlastsFunding | Self::FundingOutlastsContract
        )
    }
}

/// Compare the two ends. `funding` is `None` when no record exists; `Some(None)` is a funding with
/// no end. An unknown is never a match.
#[must_use]
pub fn align(contract_ends_on: Option<NaiveDate>, funding: Option<Option<NaiveDate>>) -> Alignment {
    let Some(funding_end) = funding else {
        return Alignment::Unknown;
    };
    let Some(contract_end) = contract_ends_on else {
        return Alignment::Unknown;
    };
    match funding_end {
        None => Alignment::FundingOpenEnded,
        Some(f) if f == contract_end => Alignment::Matched,
        Some(f) if contract_end > f => Alignment::ContractOutlastsFunding,
        Some(_) => Alignment::FundingOutlastsContract,
    }
}

/// One department's contingent figures. A figure is `None` when the breakdown is withheld.
#[derive(Debug, Clone, PartialEq)]
pub struct DepartmentShare {
    /// The department.
    pub department: String,
    /// People employed in it; `None` when withheld.
    pub headcount: Option<usize>,
    /// Contractor share of FTE, in percent of one decimal place; `None` when withheld.
    pub contractor_percent: Option<f64>,
    /// Fixed-term share of FTE, in percent of one decimal place; `None` when withheld.
    pub fixed_term_percent: Option<f64>,
}

/// The contingent share of the workforce (WPM-R62).
#[derive(Debug, Clone, PartialEq)]
pub struct ContingentShare {
    /// People employed.
    pub headcount: usize,
    /// FTE employed.
    pub fte: f64,
    /// Contractor share of FTE; `None` when there is no FTE to take a share of.
    pub contractor_percent: Option<f64>,
    /// Fixed-term share of FTE; `None` when there is no FTE to take a share of.
    pub fixed_term_percent: Option<f64>,
    /// Whether the department breakdown is shown.
    pub departments_shown: bool,
    /// By department, in name order. Every figure is `None` unless every department reaches the floor.
    pub departments: Vec<DepartmentShare>,
}

fn percent(part: i64, whole: i64) -> Option<f64> {
    if whole <= 0 {
        return None;
    }
    #[allow(clippy::cast_precision_loss)]
    Some(((part as f64) * 1000.0 / (whole as f64)).round() / 10.0)
}

/// The share of FTE that is contractor, and that is fixed-term, each shown separately. `rows` is
/// `(department, employment_type, fte_percent)` for each person employed. The organization's figure
/// is always shown; the breakdown by department is shown only when every department reaches the
/// floor, so a withheld one cannot be worked out from the others.
#[must_use]
pub fn contingent_share(rows: &[(String, String, i32)], floor: usize) -> ContingentShare {
    use std::collections::BTreeMap;
    #[derive(Default)]
    struct Sum {
        people: usize,
        fte: i64,
        contractor: i64,
        fixed_term: i64,
    }
    let mut by: BTreeMap<&str, Sum> = BTreeMap::new();
    let mut all = Sum::default();
    for (department, kind, fte) in rows {
        let fte = i64::from(*fte);
        for sum in [by.entry(department).or_default(), &mut all] {
            sum.people += 1;
            sum.fte += fte;
            match kind.as_str() {
                "contractor" => sum.contractor += fte,
                "fixed_term" => sum.fixed_term += fte,
                _ => {}
            }
        }
    }
    let shown = by.values().all(|s| s.people >= floor);
    #[allow(clippy::cast_precision_loss)]
    let fte = all.fte as f64 / 100.0;
    ContingentShare {
        headcount: all.people,
        fte,
        contractor_percent: percent(all.contractor, all.fte),
        fixed_term_percent: percent(all.fixed_term, all.fte),
        departments_shown: shown,
        departments: by
            .into_iter()
            .map(|(department, s)| DepartmentShare {
                department: department.to_string(),
                headcount: shown.then_some(s.people),
                contractor_percent: if shown {
                    percent(s.contractor, s.fte)
                } else {
                    None
                },
                fixed_term_percent: if shown {
                    percent(s.fixed_term, s.fte)
                } else {
                    None
                },
            })
            .collect(),
    }
}

/// People whose funding ends in one month (or has already ended).
#[derive(Debug, Clone, PartialEq)]
pub struct FundingMonth {
    /// `YYYY-MM`, or `already_ended` for funding that has run out under someone still employed.
    pub month: String,
    /// People; `None` when the count is above zero but below the floor.
    pub people: Option<usize>,
    /// FTE; `None` when withheld.
    pub fte: Option<f64>,
}

/// Funding ends, by month, for the next `months` months (WPM-R62). `ends` is `(funding end,
/// fte_percent)` for each person employed whose funding has an end. Nobody counts twice. A month
/// with no one shows zero; a month with some, but fewer than the floor, is withheld; **no total is
/// shown**, so a withheld month cannot be worked out. Funding that has already ended is its own
/// entry, because someone is still engaged on it.
#[must_use]
pub fn funding_horizon(
    ends: &[(NaiveDate, i32)],
    today: NaiveDate,
    months: u32,
    floor: usize,
) -> Vec<FundingMonth> {
    use chrono::Datelike;
    let index = |d: NaiveDate| i64::from(d.year()) * 12 + i64::from(d.month0());
    let start = index(today);
    let mut buckets: Vec<(usize, i64)> = vec![(0, 0); months as usize + 1];
    for (end, fte) in ends {
        let slot = if *end < today {
            0
        } else {
            let offset = index(*end) - start;
            if offset >= i64::from(months) {
                continue;
            }
            usize::try_from(offset).unwrap_or(0) + 1
        };
        buckets[slot].0 += 1;
        buckets[slot].1 += i64::from(*fte);
    }
    buckets
        .into_iter()
        .enumerate()
        .map(|(slot, (people, fte))| {
            let month = if slot == 0 {
                "already_ended".to_string()
            } else {
                let i = start + i64::try_from(slot).unwrap_or(1) - 1;
                format!("{:04}-{:02}", i.div_euclid(12), i.rem_euclid(12) + 1)
            };
            let shown = people == 0 || people >= floor;
            #[allow(clippy::cast_precision_loss)]
            FundingMonth {
                month,
                people: shown.then_some(people),
                fte: shown.then_some(fte as f64 / 100.0),
            }
        })
        .collect()
}

#[cfg(test)]
mod tests {
    use super::*;

    fn d(y: i32, m: u32, day: u32) -> NaiveDate {
        NaiveDate::from_ymd_opt(y, m, day).unwrap()
    }

    #[test]
    fn the_kinds_are_the_four() {
        for k in KINDS {
            assert!(validate_kind(k).is_ok());
        }
        assert!(validate_kind("lottery").is_err());
        assert!(validate_kind("").is_err());
    }

    #[test]
    fn time_limited_and_external_funding_need_an_end() {
        let start = d(2026, 1, 5);
        assert!(validate("time_limited", None, start).is_err());
        assert!(validate("external", None, start).is_err());
        assert!(validate("core", None, start).is_ok());
        assert!(validate("programme", None, start).is_ok());
        assert!(validate("time_limited", Some(d(2027, 1, 1)), start).is_ok());
    }

    #[test]
    fn funding_cannot_end_before_the_engagement_started() {
        let start = d(2026, 1, 5);
        assert!(validate("core", Some(d(2026, 1, 4)), start).is_err());
        assert!(validate("core", Some(d(2026, 1, 5)), start).is_ok());
    }

    #[test]
    fn the_ends_are_compared() {
        let c = Some(d(2026, 12, 31));
        assert_eq!(align(c, Some(Some(d(2026, 12, 31)))), Alignment::Matched);
        assert_eq!(
            align(c, Some(Some(d(2026, 9, 30)))),
            Alignment::ContractOutlastsFunding
        );
        assert_eq!(
            align(c, Some(Some(d(2027, 3, 31)))),
            Alignment::FundingOutlastsContract
        );
        assert_eq!(align(c, Some(None)), Alignment::FundingOpenEnded);
    }

    #[test]
    fn an_unknown_is_never_a_match() {
        assert_eq!(align(Some(d(2026, 12, 31)), None), Alignment::Unknown);
        assert_eq!(align(None, Some(Some(d(2026, 12, 31)))), Alignment::Unknown);
        assert_eq!(align(None, None), Alignment::Unknown);
        assert_eq!(align(None, Some(None)), Alignment::Unknown);
    }

    #[test]
    fn only_a_mismatch_needs_attention() {
        assert!(Alignment::ContractOutlastsFunding.needs_attention());
        assert!(Alignment::FundingOutlastsContract.needs_attention());
        for a in [
            Alignment::Matched,
            Alignment::FundingOpenEnded,
            Alignment::Unknown,
        ] {
            assert!(!a.needs_attention(), "{a:?}");
        }
    }

    fn row(dept: &str, kind: &str, fte: i32) -> (String, String, i32) {
        (dept.to_string(), kind.to_string(), fte)
    }

    fn people(dept: &str, kind: &str, n: usize) -> Vec<(String, String, i32)> {
        (0..n).map(|_| row(dept, kind, 100)).collect()
    }

    #[test]
    fn contractors_and_fixed_term_are_shown_apart() {
        let mut rows = people("A", "permanent", 6);
        rows.extend(people("A", "contractor", 2));
        rows.extend(people("A", "fixed_term", 2));
        let s = contingent_share(&rows, 5);
        assert_eq!(s.contractor_percent, Some(20.0));
        assert_eq!(s.fixed_term_percent, Some(20.0));
        assert_eq!(s.headcount, 10);
    }

    #[test]
    fn the_share_is_of_fte_not_of_heads() {
        let rows = vec![row("A", "permanent", 100), row("A", "contractor", 50)];
        assert_eq!(contingent_share(&rows, 1).contractor_percent, Some(33.3));
    }

    #[test]
    fn no_fte_gives_no_share_not_zero() {
        let s = contingent_share(&[], 5);
        assert_eq!(s.contractor_percent, None);
        assert_eq!(s.fixed_term_percent, None);
    }

    #[test]
    fn departments_show_only_when_every_one_reaches_the_floor() {
        let mut rows = people("Big", "permanent", 8);
        rows.extend(people("Big", "contractor", 2));
        rows.extend(people("Small", "contractor", 4));
        let s = contingent_share(&rows, 5);
        assert!(!s.departments_shown);
        assert!(s.departments.iter().all(|d| d.headcount.is_none()
            && d.contractor_percent.is_none()
            && d.fixed_term_percent.is_none()));
        assert!(s.contractor_percent.is_some());
        rows.extend(people("Small", "permanent", 1));
        let s = contingent_share(&rows, 5);
        assert!(s.departments_shown);
        assert_eq!(s.departments[0].contractor_percent, Some(20.0));
    }

    #[test]
    fn the_floor_is_inclusive_for_departments() {
        assert!(contingent_share(&people("A", "permanent", 5), 5).departments_shown);
        assert!(!contingent_share(&people("A", "permanent", 4), 5).departments_shown);
    }

    fn ends(spec: &[(NaiveDate, i32)]) -> Vec<(NaiveDate, i32)> {
        spec.to_vec()
    }

    #[test]
    fn the_horizon_buckets_by_month_and_crosses_the_year() {
        let today = d(2026, 11, 20);
        let spec: Vec<_> = (0..6)
            .map(|_| (d(2026, 11, 30), 100))
            .chain((0..5).map(|_| (d(2027, 1, 5), 50)))
            .collect();
        let h = funding_horizon(&ends(&spec), today, 3, 5);
        assert_eq!(h.len(), 4);
        assert_eq!(h[0].month, "already_ended");
        assert_eq!(h[1].month, "2026-11");
        assert_eq!(h[1].people, Some(6));
        assert_eq!(h[1].fte, Some(6.0));
        assert_eq!(h[2].month, "2026-12");
        assert_eq!(h[2].people, Some(0));
        assert_eq!(h[3].month, "2027-01");
        assert_eq!(h[3].fte, Some(2.5));
    }

    #[test]
    fn a_month_with_a_few_is_withheld_and_a_month_with_none_is_zero() {
        let today = d(2026, 11, 20);
        let spec = vec![(d(2026, 12, 1), 100), (d(2026, 12, 2), 100)];
        let h = funding_horizon(&spec, today, 2, 5);
        assert_eq!(h[2].people, None);
        assert_eq!(h[2].fte, None);
        assert_eq!(h[1].people, Some(0));
    }

    #[test]
    fn funding_already_over_is_its_own_entry_and_the_far_future_is_left_out() {
        let today = d(2026, 11, 20);
        let mut spec: Vec<_> = (0..5).map(|_| (d(2026, 11, 19), 100)).collect();
        spec.push((d(2030, 1, 1), 100));
        let h = funding_horizon(&spec, today, 12, 5);
        assert_eq!(h[0].people, Some(5));
        assert!(h[1..].iter().all(|m| m.people == Some(0)));
    }

    #[test]
    fn the_end_of_the_window_is_exclusive_and_today_is_not_over() {
        let today = d(2026, 11, 20);
        let h = funding_horizon(&[(today, 100)], today, 1, 1);
        assert_eq!(h[0].people, Some(0));
        assert_eq!(h[1].people, Some(1));
        assert_eq!(
            funding_horizon(&[(d(2026, 12, 1), 100)], today, 1, 1)[1].people,
            Some(0)
        );
    }
}
