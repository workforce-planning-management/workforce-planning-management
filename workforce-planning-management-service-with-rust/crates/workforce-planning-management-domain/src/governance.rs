//! **Governance load** (WPM-R64, WPM-D45): the recurring commitments (meeting, review, return,
//! self-assessment) that oversight asks of a **role**, and the senior hours a month they absorb.
//!
//! Pure and clock-free. A commitment names a role and a count of people holding it, never a person;
//! nothing here is a timesheet. The hours are an estimate that someone entered, and the total says
//! so.

use std::collections::BTreeMap;

/// What kind of commitment it is.
pub const KINDS: &[&str] = &["meeting", "review", "return", "self_assessment"];

/// How often it recurs.
pub const FREQUENCIES: &[&str] = &[
    "weekly",
    "fortnightly",
    "monthly",
    "quarterly",
    "half_yearly",
    "annually",
];

/// The longest a single occurrence may be, in minutes (a working week).
pub const MAX_MINUTES: i32 = 40 * 60;
/// The most people in one role who take part in one commitment.
pub const MAX_HOLDERS: i32 = 1000;

/// How many times a year a frequency occurs (`None` for an unknown frequency).
#[must_use]
pub fn occurrences_per_year(frequency: &str) -> Option<i64> {
    match frequency {
        "weekly" => Some(52),
        "fortnightly" => Some(26),
        "monthly" => Some(12),
        "quarterly" => Some(4),
        "half_yearly" => Some(2),
        "annually" => Some(1),
        _ => None,
    }
}

/// One commitment, as the rules see it.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Commitment {
    /// The role that carries it (a role, never a person).
    pub role: String,
    /// Whether the role is senior, so its hours count in the senior total.
    pub senior: bool,
    /// `meeting`, `review`, `return` or `self_assessment`.
    pub kind: String,
    /// How often it recurs.
    pub frequency: String,
    /// Minutes one occurrence takes one person.
    pub minutes: i32,
    /// People in the role who take part each time.
    pub holders: i32,
}

/// Check a commitment before it is stored.
///
/// # Errors
///
/// A sentence saying what is wrong.
pub fn validate(kind: &str, frequency: &str, minutes: i32, holders: i32) -> Result<(), String> {
    if !KINDS.contains(&kind) {
        return Err(format!("kind must be one of {KINDS:?}"));
    }
    if occurrences_per_year(frequency).is_none() {
        return Err(format!("frequency must be one of {FREQUENCIES:?}"));
    }
    if !(1..=MAX_MINUTES).contains(&minutes) {
        return Err(format!("minutes must be 1 to {MAX_MINUTES}"));
    }
    if !(1..=MAX_HOLDERS).contains(&holders) {
        return Err(format!("holders must be 1 to {MAX_HOLDERS}"));
    }
    Ok(())
}

/// Person-minutes a year one commitment absorbs: occurrences × minutes × people.
#[must_use]
pub fn minutes_per_year(c: &Commitment) -> i64 {
    occurrences_per_year(&c.frequency).unwrap_or(0) * i64::from(c.minutes) * i64::from(c.holders)
}

/// Hours a month for `minutes_a_year` person-minutes, in tenths of an hour, rounded to nearest.
#[must_use]
pub fn tenth_hours_per_month(minutes_a_year: i64) -> i64 {
    // minutes / 12 months / 60 → hours; ×10 for tenths; round half up on the whole division.
    (minutes_a_year * 10 + 360) / 720
}

/// The load of one role.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct RoleLoad {
    /// The role.
    pub role: String,
    /// Whether it is senior.
    pub senior: bool,
    /// Commitments that fall on it.
    pub commitments: usize,
    /// Person-hours a month, in tenths.
    pub tenth_hours_per_month: i64,
}

/// The register's total.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Load {
    /// Per role, heaviest first.
    pub roles: Vec<RoleLoad>,
    /// Senior person-hours a month, in tenths.
    pub senior_tenth_hours_per_month: i64,
    /// All person-hours a month, in tenths.
    pub all_tenth_hours_per_month: i64,
}

/// Add up the register. A role is senior if any of its commitments says so, so one stray entry
/// cannot drop a role out of the senior total. Totals are summed before rounding, so rounding
/// does not drift with the number of commitments.
#[must_use]
pub fn load(commitments: &[Commitment]) -> Load {
    let mut by_role: BTreeMap<&str, (bool, usize, i64)> = BTreeMap::new();
    for c in commitments {
        let e = by_role.entry(c.role.as_str()).or_insert((false, 0, 0));
        e.0 |= c.senior;
        e.1 += 1;
        e.2 += minutes_per_year(c);
    }
    let senior_total: i64 = by_role.values().filter(|v| v.0).map(|v| v.2).sum();
    let all_total: i64 = by_role.values().map(|v| v.2).sum();
    let mut roles: Vec<RoleLoad> = by_role
        .into_iter()
        .map(|(role, (senior, n, mins))| RoleLoad {
            role: role.to_string(),
            senior,
            commitments: n,
            tenth_hours_per_month: tenth_hours_per_month(mins),
        })
        .collect();
    roles.sort_by(|a, b| {
        b.tenth_hours_per_month
            .cmp(&a.tenth_hours_per_month)
            .then_with(|| a.role.cmp(&b.role))
    });
    Load {
        roles,
        senior_tenth_hours_per_month: tenth_hours_per_month(senior_total),
        all_tenth_hours_per_month: tenth_hours_per_month(all_total),
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn c(role: &str, senior: bool, frequency: &str, minutes: i32, holders: i32) -> Commitment {
        Commitment {
            role: role.into(),
            senior,
            kind: "meeting".into(),
            frequency: frequency.into(),
            minutes,
            holders,
        }
    }

    #[test]
    fn every_frequency_has_a_count_and_an_unknown_has_none() {
        for f in FREQUENCIES {
            assert!(occurrences_per_year(f).is_some(), "{f}");
        }
        assert_eq!(occurrences_per_year("daily"), None);
        assert_eq!(occurrences_per_year("weekly"), Some(52));
        assert_eq!(occurrences_per_year("annually"), Some(1));
    }

    #[test]
    fn a_commitment_is_checked_before_it_is_stored() {
        assert!(validate("meeting", "monthly", 90, 3).is_ok());
        assert!(validate("party", "monthly", 90, 3).is_err());
        assert!(validate("meeting", "daily", 90, 3).is_err());
        assert!(validate("meeting", "monthly", 0, 3).is_err());
        assert!(validate("meeting", "monthly", MAX_MINUTES + 1, 3).is_err());
        assert!(validate("meeting", "monthly", 90, 0).is_err());
        assert!(validate("meeting", "monthly", 90, MAX_HOLDERS + 1).is_err());
        assert!(validate("meeting", "monthly", MAX_MINUTES, MAX_HOLDERS).is_ok());
    }

    #[test]
    fn minutes_multiply_occurrences_and_people() {
        // Monthly, 90 minutes, 3 people: 12 × 90 × 3 = 3240 person-minutes a year = 4.5 hours a month.
        let x = c("director", true, "monthly", 90, 3);
        assert_eq!(minutes_per_year(&x), 3240);
        assert_eq!(tenth_hours_per_month(3240), 45);
    }

    #[test]
    fn tenths_round_to_nearest() {
        assert_eq!(tenth_hours_per_month(0), 0);
        // 720 minutes a year = 1 hour a month = 10 tenths.
        assert_eq!(tenth_hours_per_month(720), 10);
        // 36 minutes a year is half a tenth of an hour a month, which rounds up; 35 does not.
        assert_eq!(tenth_hours_per_month(71), 1);
        assert_eq!(tenth_hours_per_month(35), 0);
        assert_eq!(tenth_hours_per_month(36), 1);
    }

    #[test]
    fn senior_hours_count_only_senior_roles() {
        let l = load(&[
            c("director", true, "monthly", 60, 1),
            c("analyst", false, "weekly", 60, 2),
        ]);
        assert_eq!(l.senior_tenth_hours_per_month, 10);
        // analyst: 52 × 60 × 2 = 6240 minutes a year = 8.67 hours a month → 8.7
        assert_eq!(
            l.all_tenth_hours_per_month,
            tenth_hours_per_month(720 + 6240)
        );
        assert_eq!(l.roles[0].role, "analyst", "heaviest first");
        assert!(!l.roles[0].senior && l.roles[1].senior);
    }

    #[test]
    fn a_role_is_senior_if_any_commitment_says_so() {
        let l = load(&[
            c("director", false, "monthly", 60, 1),
            c("director", true, "monthly", 60, 1),
        ]);
        assert_eq!(l.roles.len(), 1);
        assert!(l.roles[0].senior);
        assert_eq!(l.roles[0].commitments, 2);
        assert_eq!(l.senior_tenth_hours_per_month, 20);
        // Whichever order they were entered in.
        let reversed = load(&[
            c("director", true, "monthly", 60, 1),
            c("director", false, "monthly", 60, 1),
        ]);
        assert!(reversed.roles[0].senior);
        assert_eq!(reversed.senior_tenth_hours_per_month, 20);
    }

    #[test]
    fn totals_are_summed_before_rounding() {
        // Ten roles each 0.04 of an hour a month would each round to 0.0, but together are 0.4.
        let many: Vec<_> = (0..10)
            .map(|i| c(&format!("r{i}"), true, "annually", 29, 1))
            .collect();
        assert!(
            many.iter()
                .all(|x| tenth_hours_per_month(minutes_per_year(x)) == 0)
        );
        assert_eq!(
            load(&many).senior_tenth_hours_per_month,
            tenth_hours_per_month(290)
        );
    }

    #[test]
    fn an_empty_register_is_zero() {
        let l = load(&[]);
        assert_eq!(l.roles.len(), 0);
        assert_eq!(
            (l.senior_tenth_hours_per_month, l.all_tenth_hours_per_month),
            (0, 0)
        );
    }
}
