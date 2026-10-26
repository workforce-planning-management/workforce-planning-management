//! Pure rules for **collective consultation duties** (WPM-R104): from a proposal to dismiss people at one
//! establishment, say whether a collective duty **may** apply, the latest day consultation and notice to the
//! authority must begin, and the checklist.
//!
//! DB-free and clock-free (`today` is an input). The thresholds and periods are **configuration with a named source**
//! ([`Profile`]), not code: [`great_britain`] and [`eu_directive_98_59`] are examples, marked as the deployer's to
//! confirm, and a deployer's own profile is built the same way. The answer is a **suggestion with its evidence**
//! (WPM-D50), never a verdict. Periods are counted in calendar days. Nothing here names a person: a proposal is
//! counts and dates.

use chrono::{Duration, NaiveDate};

/// How many dismissals meet a test, for an establishment of a given size.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Threshold {
    /// At least this many.
    Count(u32),
    /// At least this percentage of the establishment's normal headcount, rounded up.
    Percent(u32),
}

/// One test of a profile: for establishments of `size_from..=size_to`, dismissals proposed within `window_days` of
/// each other that reach `threshold` may trigger the duty.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct Test {
    /// The smallest establishment (normal headcount) the test applies to.
    pub size_from: u32,
    /// The largest, or `None` for no upper bound.
    pub size_to: Option<u32>,
    /// The dismissals that meet it.
    pub threshold: Threshold,
    /// Dismissals count together when they fall within this many calendar days.
    pub window_days: i64,
}

/// A minimum period before the first dismissal takes effect, from a number of dismissals up.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct Period {
    /// The period applies from this many dismissals counted together.
    pub from_dismissals: u32,
    /// Calendar days before the first dismissal takes effect.
    pub days: i64,
}

/// A jurisdiction's thresholds and periods, with where they come from.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Profile {
    /// A short token the API reports.
    pub id: &'static str,
    /// Where the numbers were taken from.
    pub source: &'static str,
    /// The tests; any one met means a duty may apply.
    pub tests: Vec<Test>,
    /// Minimum consultation before the first dismissal; none where the rule is "in good time".
    pub consultation: Vec<Period>,
    /// Minimum notice to the public authority before the first dismissal.
    pub authority_notice: Vec<Period>,
    /// What to give and do, as tokens a screen turns into words.
    pub checklist: Vec<&'static str>,
}

/// Great Britain: 20 or more dismissals at one establishment within 90 days; consultation at least 30 days before the
/// first takes effect for 20–99 and 45 for 100 or more; the same notice to the Secretary of State.
///
/// An example for a deployer to confirm; not legal advice (WPM-R104).
#[must_use]
pub fn great_britain() -> Profile {
    Profile {
        id: "great_britain",
        source: "Trade Union and Labour Relations (Consolidation) Act 1992, ss. 188 and 193; https://www.gov.uk/redundant-your-rights/consultation",
        tests: vec![Test {
            size_from: 0,
            size_to: None,
            threshold: Threshold::Count(20),
            window_days: 90,
        }],
        consultation: vec![
            Period {
                from_dismissals: 20,
                days: 30,
            },
            Period {
                from_dismissals: 100,
                days: 45,
            },
        ],
        authority_notice: vec![
            Period {
                from_dismissals: 20,
                days: 30,
            },
            Period {
                from_dismissals: 100,
                days: 45,
            },
        ],
        checklist: vec![
            "inform_representatives_in_writing",
            "consult_with_a_view_to_agreement",
            "notify_the_authority",
            "keep_a_record_of_the_consultation",
        ],
    }
}

/// The EU collective redundancies directive: over 30 days, 10 dismissals where the establishment normally employs
/// more than 20 and fewer than 100, 10% where 100 to 299, 30 where 300 or more; or over 90 days, 20 whatever the size.
/// Consultation "in good time" (no fixed minimum here); the authority is notified at least 30 days before the first
/// dismissal takes effect.
///
/// An example for a deployer to confirm; member states may go further (WPM-R104).
#[must_use]
pub fn eu_directive_98_59() -> Profile {
    Profile {
        id: "eu_directive_98_59",
        source: "Council Directive 98/59/EC, Articles 1 to 4; https://www.legislation.gov.uk/eudr/1998/59",
        tests: vec![
            Test {
                size_from: 21,
                size_to: Some(99),
                threshold: Threshold::Count(10),
                window_days: 30,
            },
            Test {
                size_from: 100,
                size_to: Some(299),
                threshold: Threshold::Percent(10),
                window_days: 30,
            },
            Test {
                size_from: 300,
                size_to: None,
                threshold: Threshold::Count(30),
                window_days: 30,
            },
            Test {
                size_from: 0,
                size_to: None,
                threshold: Threshold::Count(20),
                window_days: 90,
            },
        ],
        consultation: vec![],
        authority_notice: vec![Period {
            from_dismissals: 0,
            days: 30,
        }],
        checklist: vec![
            "consult_in_good_time_with_a_view_to_agreement",
            "give_representatives_the_relevant_information",
            "notify_the_authority",
        ],
    }
}

/// A proposal to dismiss people at one establishment: counts and dates, no names.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Proposal {
    /// The establishment's normal headcount.
    pub establishment_size: u32,
    /// Dismissals this proposal makes.
    pub dismissals: u32,
    /// The day the first of them takes effect.
    pub earliest_termination: NaiveDate,
    /// Dismissals already proposed at the same establishment: `(takes effect on, how many)`.
    pub others: Vec<(NaiveDate, u32)>,
    /// A longer consultation period an agreement requires, in calendar days.
    pub agreed_period_days: Option<i64>,
}

/// What was counted for the test that was met, or came closest.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Evidence {
    /// Dismissals counted together in the window that includes this proposal's date.
    pub counted: u32,
    /// What the test needed, or `None` when no test applies to this establishment size.
    pub needed: Option<u32>,
    /// The window of that test, in calendar days.
    pub window_days: Option<i64>,
}

/// The suggestion: whether a duty may apply, by when to start, and what to do.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Assessment {
    /// The profile used, and its source: a reader can see which numbers these are.
    pub profile: &'static str,
    /// Where the profile's numbers came from.
    pub source: &'static str,
    /// A duty **may** apply. Not a verdict, and not "no duty" when `false`: only that no test here is met.
    pub may_apply: bool,
    /// What was counted against what was needed.
    pub evidence: Evidence,
    /// The consultation period in force (the profile's, or the agreed one if longer), when there is one.
    pub consultation_days: Option<i64>,
    /// The latest day consultation should begin, for the earliest termination date.
    pub latest_consultation_start: Option<NaiveDate>,
    /// The latest day the authority should be notified.
    pub latest_authority_notice: Option<NaiveDate>,
    /// Days from `today` to the later of those two dates' earlier one; negative when already passed.
    pub days_remaining: Option<i64>,
    /// The date has passed.
    pub already_late: bool,
    /// What to give and do.
    pub checklist: Vec<&'static str>,
    /// Always true: the thresholds are the deployer's to confirm.
    pub deployer_to_confirm: bool,
}

/// The dismissals that fall together in the best `window_days` window that includes `date`.
fn counted_in_window(
    date: NaiveDate,
    own: u32,
    others: &[(NaiveDate, u32)],
    window_days: i64,
) -> u32 {
    let mut best = own;
    let window = window_days.max(1);
    let mut starts: Vec<NaiveDate> = others.iter().map(|(d, _)| *d).collect();
    starts.push(date);
    for start in starts {
        let end = start + Duration::days(window - 1);
        if start > date || end < date {
            continue;
        }
        let inside: u32 = others
            .iter()
            .filter(|(d, _)| *d >= start && *d <= end)
            .map(|(_, n)| *n)
            .fold(0, u32::saturating_add);
        let own_in = if date >= start && date <= end { own } else { 0 };
        best = best.max(inside.saturating_add(own_in));
    }
    best
}

fn needed(threshold: Threshold, size: u32) -> u32 {
    match threshold {
        Threshold::Count(n) => n,
        Threshold::Percent(p) => (u64::from(size) * u64::from(p))
            .div_ceil(100)
            .try_into()
            .unwrap_or(u32::MAX),
    }
}

fn period_for(periods: &[Period], counted: u32) -> Option<i64> {
    periods
        .iter()
        .filter(|p| counted >= p.from_dismissals)
        .map(|p| p.days)
        .max()
}

/// Whether a collective duty may apply to `proposal` under `profile`, and what follows.
#[must_use]
pub fn assess(profile: &Profile, proposal: &Proposal, today: NaiveDate) -> Assessment {
    let applicable = profile.tests.iter().filter(|t| {
        proposal.establishment_size >= t.size_from
            && t.size_to.is_none_or(|to| proposal.establishment_size <= to)
    });
    // For each applicable test: what was counted and what it needed. A test met wins; else the closest (the least still short).
    let mut met: Option<(u32, u32, i64)> = None;
    let mut closest: Option<(u32, u32, i64)> = None;
    for test in applicable {
        let counted = counted_in_window(
            proposal.earliest_termination,
            proposal.dismissals,
            &proposal.others,
            test.window_days,
        );
        let need = needed(test.threshold, proposal.establishment_size).max(1);
        let entry = (counted, need, test.window_days);
        if counted >= need {
            if met.is_none_or(|(c, _, _)| counted > c) {
                met = Some(entry);
            }
        } else if closest.is_none_or(|(c, n, _)| need - counted < n.saturating_sub(c)) {
            closest = Some(entry);
        }
    }
    let chosen = met.or(closest);
    let may_apply = met.is_some();
    let evidence = Evidence {
        counted: chosen.map_or(proposal.dismissals, |c| c.0),
        needed: chosen.map(|c| c.1),
        window_days: chosen.map(|c| c.2),
    };
    let counted = evidence.counted;
    let (consultation_days, consult_start, notice_by) = if may_apply {
        let profile_days = period_for(&profile.consultation, counted);
        let days = match (profile_days, proposal.agreed_period_days) {
            (Some(p), Some(a)) => Some(p.max(a)),
            (p, a) => p.or(a),
        };
        (
            days,
            days.map(|d| proposal.earliest_termination - Duration::days(d)),
            period_for(&profile.authority_notice, counted)
                .map(|d| proposal.earliest_termination - Duration::days(d)),
        )
    } else {
        (None, None, None)
    };
    let nearest = [consult_start, notice_by].into_iter().flatten().min();
    let days_remaining = nearest.map(|d| (d - today).num_days());
    Assessment {
        profile: profile.id,
        source: profile.source,
        may_apply,
        evidence,
        consultation_days,
        latest_consultation_start: consult_start,
        latest_authority_notice: notice_by,
        days_remaining,
        already_late: days_remaining.is_some_and(|d| d < 0),
        checklist: if may_apply {
            profile.checklist.clone()
        } else {
            Vec::new()
        },
        deployer_to_confirm: true,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn day(m: u32, d: u32) -> NaiveDate {
        NaiveDate::from_ymd_opt(2027, m, d).unwrap()
    }

    fn proposal(size: u32, dismissals: u32) -> Proposal {
        Proposal {
            establishment_size: size,
            dismissals,
            earliest_termination: day(6, 30),
            others: vec![],
            agreed_period_days: None,
        }
    }

    /// Great Britain: 19 is under, 20 is the threshold; 99 gets 30 days and 100 gets 45.
    #[test]
    fn gb_thresholds_and_periods_at_each_edge() {
        let gb = great_britain();
        let today = day(1, 1);
        assert!(!assess(&gb, &proposal(500, 19), today).may_apply);
        let twenty = assess(&gb, &proposal(500, 20), today);
        assert!(twenty.may_apply);
        assert_eq!(twenty.consultation_days, Some(30));
        assert_eq!(twenty.latest_consultation_start, Some(day(5, 31)));
        let ninety_nine = assess(&gb, &proposal(500, 99), today);
        assert_eq!(ninety_nine.consultation_days, Some(30));
        let hundred = assess(&gb, &proposal(500, 100), today);
        assert_eq!(hundred.consultation_days, Some(45));
        assert_eq!(
            hundred.latest_consultation_start,
            Some(day(6, 30) - Duration::days(45))
        );
        assert_eq!(
            hundred.latest_authority_notice,
            hundred.latest_consultation_start
        );
    }

    /// The 90-day window: dismissals 89 days apart count together, 90 days apart do not (a window is 90 days inclusive).
    #[test]
    fn a_proposal_that_crosses_the_ninety_day_window() {
        let gb = great_britain();
        let mut p = proposal(500, 10);
        p.others = vec![(day(6, 30) - Duration::days(89), 10)];
        assert!(
            assess(&gb, &p, day(1, 1)).may_apply,
            "90 days inclusive: 89 days apart is inside"
        );
        p.others = vec![(day(6, 30) - Duration::days(90), 10)];
        let apart = assess(&gb, &p, day(1, 1));
        assert!(!apart.may_apply, "90 days apart is outside");
        assert_eq!(
            apart.evidence,
            Evidence {
                counted: 10,
                needed: Some(20),
                window_days: Some(90)
            }
        );
        p.others = vec![(day(6, 30) + Duration::days(89), 10)];
        assert!(
            assess(&gb, &p, day(1, 1)).may_apply,
            "later dismissals count too"
        );
    }

    /// Dismissals spread so that no single window holds enough are not added up.
    #[test]
    fn two_far_apart_groups_are_not_summed() {
        let mut p = proposal(500, 10);
        p.others = vec![
            (day(6, 30) - Duration::days(80), 6),
            (day(6, 30) + Duration::days(80), 6),
        ];
        let a = assess(&great_britain(), &p, day(1, 1));
        assert!(!a.may_apply);
        assert_eq!(a.evidence.counted, 16, "10 with either group, never all 22");
    }

    /// EU: size bands, the percentage (rounded up) and the 90-day alternative that ignores size. Above 20 dismissals the
    /// 90-day test is always met, so the band tests are exercised below it.
    #[test]
    fn eu_thresholds_by_establishment_size() {
        let eu = eu_directive_98_59();
        let t = day(1, 1);
        assert!(
            !assess(&eu, &proposal(20, 10), t).may_apply,
            "20 or fewer employed: only the 90-day test applies"
        );
        assert!(assess(&eu, &proposal(21, 10), t).may_apply);
        assert!(!assess(&eu, &proposal(99, 9), t).may_apply);
        assert!(assess(&eu, &proposal(100, 10), t).may_apply, "10% of 100");
        assert!(!assess(&eu, &proposal(100, 9), t).may_apply);
        assert!(
            !assess(&eu, &proposal(150, 14), t).may_apply,
            "10% of 150 is 15"
        );
        assert!(assess(&eu, &proposal(150, 15), t).may_apply);
        assert!(
            !assess(&eu, &proposal(155, 15), t).may_apply,
            "10% of 155 rounds up to 16"
        );
        assert!(assess(&eu, &proposal(155, 16), t).may_apply);
        assert!(
            !assess(&eu, &proposal(300, 19), t).may_apply,
            "30 of 300, and 19 is under the 90-day 20"
        );
        assert!(
            assess(&eu, &proposal(300, 20), t).may_apply,
            "20 in 90 days whatever the size"
        );
        assert!(assess(&eu, &proposal(5, 20), t).may_apply);
        assert!(!assess(&eu, &proposal(5, 19), t).may_apply);
    }

    /// The 30-day window of the size tests is narrower than the 90-day one.
    #[test]
    fn the_thirty_day_band_test_needs_dismissals_within_thirty_days() {
        let eu = eu_directive_98_59();
        let mut p = proposal(50, 5);
        p.others = vec![(day(6, 30) - Duration::days(29), 5)];
        assert!(
            assess(&eu, &p, day(1, 1)).may_apply,
            "10 within 30 days at a 50-person establishment"
        );
        p.others = vec![(day(6, 30) - Duration::days(30), 5)];
        assert!(
            !assess(&eu, &p, day(1, 1)).may_apply,
            "30 days apart is a 31-day span"
        );
    }

    /// The EU profile has no fixed consultation minimum, only notice to the authority 30 days ahead.
    #[test]
    fn the_eu_profile_gives_a_notice_date_and_no_invented_consultation_period() {
        let a = assess(&eu_directive_98_59(), &proposal(300, 30), day(1, 1));
        assert_eq!(a.consultation_days, None);
        assert_eq!(a.latest_consultation_start, None);
        assert_eq!(a.latest_authority_notice, Some(day(5, 31)));
    }

    /// A longer agreed procedure wins; a shorter one never shortens the legal minimum.
    #[test]
    fn an_agreed_procedure_only_lengthens() {
        let gb = great_britain();
        let mut p = proposal(500, 20);
        p.agreed_period_days = Some(60);
        assert_eq!(assess(&gb, &p, day(1, 1)).consultation_days, Some(60));
        p.agreed_period_days = Some(10);
        assert_eq!(assess(&gb, &p, day(1, 1)).consultation_days, Some(30));
        let eu = assess(
            &eu_directive_98_59(),
            &{
                let mut q = proposal(300, 30);
                q.agreed_period_days = Some(45);
                q
            },
            day(1, 1),
        );
        assert_eq!(
            eu.latest_consultation_start,
            Some(day(6, 30) - Duration::days(45))
        );
    }

    /// Time left is read from the day given, and negative means late.
    #[test]
    fn days_remaining_and_lateness_come_from_the_given_day() {
        let gb = great_britain();
        let p = proposal(500, 20);
        let early = assess(&gb, &p, day(5, 30));
        assert_eq!((early.days_remaining, early.already_late), (Some(1), false));
        let on = assess(&gb, &p, day(5, 31));
        assert_eq!(
            (on.days_remaining, on.already_late),
            (Some(0), false),
            "the latest day itself is not late"
        );
        let late = assess(&gb, &p, day(6, 5));
        assert_eq!((late.days_remaining, late.already_late), (Some(-5), true));
    }

    /// Below every threshold there is no date, no checklist, and the answer says it is for the deployer to confirm.
    #[test]
    fn a_proposal_under_the_thresholds_is_not_called_clear() {
        let a = assess(&great_britain(), &proposal(500, 3), day(1, 1));
        assert!(!a.may_apply);
        assert!(
            a.checklist.is_empty()
                && a.latest_consultation_start.is_none()
                && a.days_remaining.is_none()
        );
        assert!(a.deployer_to_confirm);
        assert!(a.source.contains("1992"));
    }

    /// Nothing goes wrong at the ends of the number line.
    #[test]
    fn extreme_counts_do_not_panic() {
        let mut p = proposal(u32::MAX, u32::MAX);
        p.others = vec![(day(6, 30), u32::MAX)];
        let a = assess(&eu_directive_98_59(), &p, day(1, 1));
        assert!(a.may_apply);
        assert!(!assess(&great_britain(), &proposal(0, 0), day(1, 1)).may_apply);
    }
}
