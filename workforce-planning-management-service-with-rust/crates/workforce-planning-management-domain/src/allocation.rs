//! **Per-person allocation** (WPM-R275, WPM-D211): pure, database-free and clock-free.
//!
//! An allocation plans a share of one of a person's assignments (WPM-R161) for a programme, product or service over a run of
//! months. It records what a person is **planned** to do, never what they did and never how well (WPM-D211): there is no
//! utilization score here, and nothing ranks people.
//!
//! The rules:
//!
//! - **Tracking is per scope.** Where a pool or unit tracks allocation, a person with none that month is unallocated, a known
//!   zero. Where it does not, the standing is [`MonthStanding::Unknown`], never zero (WPM-D46), so a partial roll-out cannot
//!   report false availability.
//! - **What a person can give in a month** is their assignment FTE weighted by the days of the month the assignment is in force
//!   (and the engagement has not ended), and again less the days of approved leave, as pool supply is weighted
//!   ([`crate::capacity::available_days`]). The kind of leave is never read.
//! - **Signals** say what a person decides about; nothing is refused for being over: over-allocated, a clash with leave,
//!   available, an assignment over its own FTE, and an allocation running past the end of the assignment or engagement.
//!
//! FTE is in hundredths (`fte_centi`, `100` is one full-time equivalent); an assignment's `fte_percent` is the same unit. A
//! month is its first day. Dates are inclusive at both ends.

use crate::assignment::Assignment;
use crate::capacity::{
    MAX_HORIZON_MONTHS, WorkerFacts, available_days, days_in_month, month_start,
};
use jiff::civil::Date;
use uuid::Uuid;

/// Basis points in 100%.
pub const FULL_BP: i32 = 10_000;
/// The default share of available FTE below which a person reads as available: 80%.
pub const DEFAULT_AVAILABLE_BELOW_BP: i32 = 8_000;
/// The longest demand-source URN.
pub const SOURCE_MAX: usize = 200;

/// How a person-month standing explains itself. Shown with every answer.
pub const DERIVATION: &str = "Contracted = the assignment's FTE × the days of the month it is in force and the engagement has \
not ended ÷ the days in the month. Available = the same, less the days of approved leave. Allocated = the confirmed \
allocations covering the month; tentative ones are shown apart and not counted. Over-allocated: allocated exceeds \
contracted. Leave clash: allocated fits the contract but not the time left after approved leave. Available: allocated is \
below the threshold share of available. Where the scope does not track allocation, the standing is unknown, not zero.";

// ─── Status ─────────────────────────────────────────────────────────────────

/// Where an allocation stands.
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord)]
pub enum Status {
    /// Pencilled in: shown, not counted.
    Tentative,
    /// Agreed: counted.
    Confirmed,
    /// Finished or withdrawn: kept as history, not counted.
    Ended,
}

impl Status {
    /// Every status, in order.
    pub const ALL: [Self; 3] = [Self::Tentative, Self::Confirmed, Self::Ended];

    /// Parse the stored word.
    #[must_use]
    pub fn parse(text: &str) -> Option<Self> {
        match text {
            "tentative" => Some(Self::Tentative),
            "confirmed" => Some(Self::Confirmed),
            "ended" => Some(Self::Ended),
            _ => None,
        }
    }

    /// The stored word.
    #[must_use]
    pub fn as_str(self) -> &'static str {
        match self {
            Self::Tentative => "tentative",
            Self::Confirmed => "confirmed",
            Self::Ended => "ended",
        }
    }

    /// Whether a move from `self` to `to` is allowed. Tentative and confirmed move either way; either can end; an ended
    /// allocation is final (a new one is made instead). Staying put is not a move.
    #[must_use]
    pub fn can_move_to(self, to: Self) -> bool {
        matches!(
            (self, to),
            (Self::Tentative, Self::Confirmed | Self::Ended)
                | (Self::Confirmed, Self::Tentative | Self::Ended)
        )
    }
}

// ─── Records ────────────────────────────────────────────────────────────────

/// One allocation of a share of an assignment to a demand source.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Allocation {
    /// The programme, product or service, as an `EntityRef` URN (WPM-R258).
    pub source: String,
    /// The skill pool it draws on, if one is named.
    pub pool: Option<Uuid>,
    /// The first month (its first day).
    pub from_month: Date,
    /// The last month, inclusive (its first day).
    pub to_month: Date,
    /// The share of the assignment, in hundredths of an FTE.
    pub fte_centi: i64,
    /// Where it stands.
    pub status: Status,
}

impl Allocation {
    /// Whether the allocation covers `month` (any day of it).
    #[must_use]
    pub fn covers(&self, month: Date) -> bool {
        let month = month_start(month);
        self.from_month <= month && month <= self.to_month
    }

    /// Whether it counts in `month`: confirmed and covering it.
    #[must_use]
    pub fn counts_in(&self, month: Date) -> bool {
        self.status == Status::Confirmed && self.covers(month)
    }
}

/// One of a person's assignments with the allocations made against it.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Holding {
    /// The assignment (its FTE and dates).
    pub assignment: Assignment,
    /// The allocations against it.
    pub allocations: Vec<Allocation>,
}

/// What the rules need to know about a person: their holdings, when their engagement ends, and their approved leave. No name
/// and no reference to the person.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Person {
    /// The person's assignments and their allocations.
    pub holdings: Vec<Holding>,
    /// The last day of a fixed-term or contractor engagement (WPM-R79), if there is one.
    pub engagement_ends_on: Option<Date>,
    /// Approved leave as inclusive `(start, end)` ranges. The kind is never read.
    pub approved_leave: Vec<(Date, Date)>,
}

/// Whether the scope (a pool or unit) records allocation.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Tracking {
    /// Allocations are recorded here: none is a known zero.
    Tracked,
    /// Not recorded here: the standing is unknown.
    Untracked,
}

// ─── Validation ─────────────────────────────────────────────────────────────

/// Whether `source` is a plausible `EntityRef` URN: `type:rest`, both parts present, no whitespace, at most
/// [`SOURCE_MAX`] characters.
#[must_use]
pub fn valid_source(source: &str) -> bool {
    source.chars().count() <= SOURCE_MAX
        && !source.chars().any(char::is_whitespace)
        && source
            .split_once(':')
            .is_some_and(|(kind, rest)| !kind.is_empty() && !rest.is_empty())
}

/// The number of months from `from` to `to`, both included; zero when `to` is before `from`.
#[must_use]
pub fn months_spanned(from: Date, to: Date) -> i64 {
    let (from, to) = (month_start(from), month_start(to));
    if to < from {
        return 0;
    }
    #[expect(
        clippy::arithmetic_side_effects,
        reason = "RCA-EXC-2: years are i16 and months 1 to 12, so the result is under 400 000"
    )]
    let months = (i64::from(to.year()) - i64::from(from.year())) * 12 + i64::from(to.month())
        - i64::from(from.month())
        + 1;
    months
}

/// Problems with an allocation against its assignment. Running past the end of the assignment or the engagement is **not** a
/// problem: an end can be extended (WPM-R79), so it is a signal ([`runs_past_end`]). Starting outside the assignment is.
#[must_use]
pub fn allocation_problems(allocation: &Allocation, assignment: &Assignment) -> Vec<String> {
    let mut out = Vec::new();
    if !valid_source(&allocation.source) {
        out.push(format!(
            "source must be a programme, product or service reference such as `thing:<id>`, at most {SOURCE_MAX} characters"
        ));
    }
    if allocation.from_month.day() != 1 {
        out.push("from_month must be the first day of a month".to_string());
    }
    if allocation.to_month.day() != 1 {
        out.push("to_month must be the first day of a month".to_string());
    }
    let span = months_spanned(allocation.from_month, allocation.to_month);
    if span == 0 {
        out.push("to_month must not be before from_month".to_string());
    } else if span > i64::from(MAX_HORIZON_MONTHS) {
        out.push(format!(
            "an allocation covers at most {MAX_HORIZON_MONTHS} months, not {span}"
        ));
    }
    let most = i64::from(assignment.fte_percent.max(0_i32));
    if !(1..=most).contains(&allocation.fte_centi) {
        out.push(format!(
            "fte_centi must be 1 to the assignment's FTE ({most}), not {}",
            allocation.fte_centi
        ));
    }
    if month_start(allocation.from_month) < month_start(assignment.starts_on) {
        out.push("an allocation cannot start before its assignment's first month".to_string());
    }
    if let Some(end) = assignment.ends_on
        && month_start(allocation.from_month) > month_start(end)
    {
        out.push("an allocation cannot start after its assignment has ended".to_string());
    }
    out
}

/// A threshold share in basis points: 1 to 10 000.
///
/// # Errors
///
/// A value outside that range.
pub fn validate_threshold_bp(bp: i32) -> Result<(), String> {
    if (1..=FULL_BP).contains(&bp) {
        Ok(())
    } else {
        Err(format!(
            "the availability threshold must be 1 to {FULL_BP} basis points, not {bp}"
        ))
    }
}

// ─── Arithmetic ─────────────────────────────────────────────────────────────

/// The last day a holding can be worked: the earlier of the assignment's end and the engagement's end.
#[must_use]
pub fn last_day(assignment: &Assignment, engagement_ends_on: Option<Date>) -> Option<Date> {
    match (assignment.ends_on, engagement_ends_on) {
        (Some(a), Some(e)) => Some(a.min(e)),
        (a, e) => a.or(e),
    }
}

/// If the allocation runs into a month after the holding's last day, that day. An allocation ending in the month of the last
/// day is not past it: that month is weighted by its days.
#[must_use]
pub fn runs_past_end(
    allocation: &Allocation,
    assignment: &Assignment,
    engagement_ends_on: Option<Date>,
) -> Option<Date> {
    last_day(assignment, engagement_ends_on)
        .filter(|end| month_start(allocation.to_month) > month_start(*end))
}

fn facts(
    assignment: &Assignment,
    engagement_ends_on: Option<Date>,
    leave: &[(Date, Date)],
) -> WorkerFacts {
    // `WorkerFacts::terminated_on` is the first day not employed; our last day is inclusive.
    let terminated_on =
        last_day(assignment, engagement_ends_on).and_then(|last| last.tomorrow().ok());
    WorkerFacts {
        fte_percent: assignment.fte_percent,
        hired_on: assignment.starts_on,
        terminated_on,
        role_profiles: Vec::new(),
        skills: Vec::new(),
        approved_leave: leave.to_vec(),
    }
}

/// `fte × days ÷ days in the month`, rounded half up, in hundredths; `None` when the days are not within the month's own
/// (a count that cannot be worked is unknown, not clamped) or a figure does not fit.
fn weighted(fte_percent: i32, days: i64, month: Date) -> Option<i64> {
    let denominator = days_in_month(month);
    if !(0..=denominator).contains(&days) {
        return None;
    }
    let numerator = i64::from(fte_percent.max(0_i32)).checked_mul(days)?;
    numerator
        .checked_mul(2)?
        .checked_add(denominator)?
        .checked_div(denominator.checked_mul(2)?)
}

/// What a holding gives and takes in one month.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub struct HoldingMonth {
    /// The assignment's FTE weighted by the days it can be worked.
    pub contracted_centi: i64,
    /// The same, less approved leave.
    pub available_centi: i64,
    /// Confirmed allocations covering the month.
    pub allocated_centi: i64,
    /// Tentative allocations covering the month, shown apart.
    pub tentative_centi: i64,
}

/// One holding in one month; `None` when a figure does not fit or the days are not within the month.
#[must_use]
pub fn holding_month(holding: &Holding, person: &Person, month: Date) -> Option<HoldingMonth> {
    let month = month_start(month);
    let with_leave = facts(
        &holding.assignment,
        person.engagement_ends_on,
        &person.approved_leave,
    );
    let without_leave = WorkerFacts {
        approved_leave: Vec::new(),
        ..with_leave.clone()
    };
    let fte = holding.assignment.fte_percent;
    let sum = |status: Status| -> Option<i64> {
        holding
            .allocations
            .iter()
            .filter(|a| a.status == status && a.covers(month))
            .try_fold(0_i64, |total, a| total.checked_add(a.fte_centi))
    };
    Some(HoldingMonth {
        contracted_centi: weighted(fte, available_days(&without_leave, month), month)?,
        available_centi: weighted(fte, available_days(&with_leave, month), month)?,
        allocated_centi: sum(Status::Confirmed)?,
        tentative_centi: sum(Status::Tentative)?,
    })
}

/// What a person-month says. Each signal carries the amount it is about.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum Signal {
    /// Confirmed allocations exceed what the person is contracted for this month, by this much.
    OverAllocated {
        /// Hundredths of an FTE over.
        by_centi: i64,
    },
    /// The allocations fit the contract but not the time left after approved leave, short by this much.
    LeaveClash {
        /// Hundredths of an FTE short.
        short_centi: i64,
    },
    /// Allocated below the threshold share of what is available; this much is free.
    Available {
        /// Hundredths of an FTE free.
        free_centi: i64,
    },
    /// One assignment carries more than its own FTE this month (the person's total may still fit).
    AssignmentOverAllocated {
        /// The index of the holding in [`Person::holdings`].
        holding: usize,
        /// Hundredths of an FTE over.
        by_centi: i64,
    },
}

/// A person's month, added over their holdings.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct PersonMonth {
    /// The month (its first day).
    pub month: Date,
    /// The totals over the holdings.
    pub totals: HoldingMonth,
    /// What the month says, in a fixed order: the person's standing first, then each assignment over its own FTE.
    pub signals: Vec<Signal>,
}

/// A person-month, or unknown where the scope does not track allocation.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum MonthStanding {
    /// The scope does not record allocation, or a total does not fit a 64-bit count: nothing can be said, and nothing is
    /// assumed.
    Unknown,
    /// The month, worked out.
    Known(PersonMonth),
}

/// A person's standing in `month`. `available_below_bp` is the threshold share (validate it with [`validate_threshold_bp`]).
///
/// Over-allocation and a leave clash partition the cases where the allocations do not fit: over the contract is
/// over-allocated whatever the leave; within the contract but over what leave leaves is a clash. A person with nothing to
/// give that month (not yet started, or ended) is never "available".
#[must_use]
pub fn person_month(
    person: &Person,
    month: Date,
    tracking: Tracking,
    available_below_bp: i32,
) -> MonthStanding {
    if tracking == Tracking::Untracked {
        return MonthStanding::Unknown;
    }
    standing(person, month, available_below_bp).map_or(MonthStanding::Unknown, MonthStanding::Known)
}

/// The month worked out, or `None` when a total or a difference does not fit.
fn standing(person: &Person, month: Date, available_below_bp: i32) -> Option<PersonMonth> {
    let month = month_start(month);
    let mut totals = HoldingMonth::default();
    let mut assignment_signals = Vec::new();
    for (index, holding) in person.holdings.iter().enumerate() {
        let one = holding_month(holding, person, month)?;
        totals.contracted_centi = totals.contracted_centi.checked_add(one.contracted_centi)?;
        totals.available_centi = totals.available_centi.checked_add(one.available_centi)?;
        totals.allocated_centi = totals.allocated_centi.checked_add(one.allocated_centi)?;
        totals.tentative_centi = totals.tentative_centi.checked_add(one.tentative_centi)?;
        if one.allocated_centi > one.contracted_centi {
            assignment_signals.push(Signal::AssignmentOverAllocated {
                holding: index,
                by_centi: one.allocated_centi.checked_sub(one.contracted_centi)?,
            });
        }
    }
    let mut signals = Vec::new();
    // The threshold is validated by the caller (`validate_threshold_bp`); one outside 0 to 10 000 is no standing, not a clamped one.
    if !(0_i32..=FULL_BP).contains(&available_below_bp) {
        return None;
    }
    let threshold = i64::from(available_below_bp);
    if totals.allocated_centi > totals.contracted_centi {
        signals.push(Signal::OverAllocated {
            by_centi: totals
                .allocated_centi
                .checked_sub(totals.contracted_centi)?,
        });
    } else if totals.allocated_centi > totals.available_centi {
        signals.push(Signal::LeaveClash {
            short_centi: totals.allocated_centi.checked_sub(totals.available_centi)?,
        });
    } else if totals.available_centi > 0
        && totals.allocated_centi.checked_mul(i64::from(FULL_BP))?
            < totals.available_centi.checked_mul(threshold)?
    {
        signals.push(Signal::Available {
            free_centi: totals.available_centi.checked_sub(totals.allocated_centi)?,
        });
    }
    signals.extend(assignment_signals);
    Some(PersonMonth {
        month,
        totals,
        signals,
    })
}

// ─── Reconciliation with pool demand ────────────────────────────────────────

/// The confirmed allocations to `source` from `pool` in `month`, added; `None` when the total does not fit a 64-bit count.
/// Pass every allocation in the scope.
#[must_use]
pub fn sum_confirmed<'a>(
    allocations: impl IntoIterator<Item = &'a Allocation>,
    pool: Uuid,
    source: &str,
    month: Date,
) -> Option<i64> {
    allocations
        .into_iter()
        .filter(|a| a.pool == Some(pool) && a.source == source && a.counts_in(month))
        .try_fold(0_i64, |total, a| total.checked_add(a.fte_centi))
}

/// How the named people allocated to a source compare with what the source claims from the pool (WPM-R57).
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Staffing {
    /// The pool does not track allocation: nothing can be compared.
    Unknown,
    /// No active claim for this source, pool and month; this much is allocated anyway.
    NotClaimed {
        /// Hundredths of an FTE allocated.
        allocated_centi: i64,
    },
    /// The allocations equal the claim.
    Staffed,
    /// The claim is not yet covered by named people, by this much.
    Understaffed {
        /// Hundredths of an FTE short.
        by_centi: i64,
    },
    /// More is allocated than claimed, by this much.
    Overstaffed {
        /// Hundredths of an FTE over.
        by_centi: i64,
    },
}

/// Compare an active claim with the confirmed allocations to it. A difference that does not fit a 64-bit count is
/// [`Staffing::Unknown`].
#[must_use]
pub fn reconcile(tracking: Tracking, claim_centi: Option<i64>, allocated_centi: i64) -> Staffing {
    match (tracking, claim_centi) {
        (Tracking::Untracked, _) => Staffing::Unknown,
        (Tracking::Tracked, None) => Staffing::NotClaimed { allocated_centi },
        (Tracking::Tracked, Some(claim)) => match allocated_centi.cmp(&claim) {
            std::cmp::Ordering::Equal => Staffing::Staffed,
            std::cmp::Ordering::Less => claim
                .checked_sub(allocated_centi)
                .map_or(Staffing::Unknown, |by_centi| Staffing::Understaffed {
                    by_centi,
                }),
            std::cmp::Ordering::Greater => allocated_centi
                .checked_sub(claim)
                .map_or(Staffing::Unknown, |by_centi| Staffing::Overstaffed {
                    by_centi,
                }),
        },
    }
}

// ─── Who may read and write ─────────────────────────────────────────────────

/// How a caller stands to the person whose allocations are asked for, as the service establishes it from the verified token
/// and memberships. Each flag is a fact, never a request parameter.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
#[expect(
    clippy::struct_excessive_bools,
    reason = "one flag per declared check; they are independent"
)]
pub struct Reader {
    /// The caller is the person.
    pub is_self: bool,
    /// The caller is one of the person's line managers.
    pub line_manager: bool,
    /// The caller holds HR rights in the person's organization.
    pub hr: bool,
    /// The caller holds the resource-manager role (WPM-R276) in the person's organization.
    pub resource_manager: bool,
    /// The caller owns the demand source the allocations are for.
    pub source_owner: bool,
}

/// What a caller may see of someone's allocations.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Read {
    /// The allocations themselves, with the person.
    Named,
    /// Counts and FTE with the floor; no person.
    CountsOnly,
}

/// What `reader` may see. A source's owner sees names only where the deployer's policy grants it (`source_owners_named`).
#[must_use]
pub fn read_access(reader: Reader, source_owners_named: bool) -> Read {
    if reader.is_self
        || reader.line_manager
        || reader.hr
        || reader.resource_manager
        || (reader.source_owner && source_owners_named)
    {
        Read::Named
    } else {
        Read::CountsOnly
    }
}

/// Whether a named read is audited: every read of **another** person's allocations is; a person reading their own is not.
#[must_use]
pub fn named_read_is_audited(reader: Reader, source_owners_named: bool) -> bool {
    read_access(reader, source_owners_named) == Read::Named && !reader.is_self
}

/// Whether the caller may create, change or end allocations in the person's organization: HR and resource managers. A line
/// manager or a source's owner may ask, but not write; nobody writes their own (a resource manager's own allocations are
/// written by another resource manager or HR), so a person cannot plan away their own over-allocation unseen.
#[must_use]
pub fn may_write(reader: Reader) -> bool {
    reader.hr || (reader.resource_manager && !reader.is_self)
}

#[cfg(test)]
mod tests {
    use super::*;
    use jiff::civil::date as d;

    fn assignment(fte: i32, starts: Date, ends: Option<Date>) -> Assignment {
        Assignment {
            fte_percent: fte,
            starts_on: starts,
            ends_on: ends,
            primary: true,
        }
    }

    /// `holding_month`, which holds for every holding these tests build.
    fn holding_in(holding: &Holding, person: &Person, month: Date) -> HoldingMonth {
        holding_month(holding, person, month).expect("a month that fits")
    }

    fn alloc(source: &str, from: Date, to: Date, fte: i64, status: Status) -> Allocation {
        Allocation {
            source: source.to_string(),
            pool: None,
            from_month: from,
            to_month: to,
            fte_centi: fte,
            status,
        }
    }

    fn confirmed(fte: i64, from: Date, to: Date) -> Allocation {
        alloc("thing:programme-a", from, to, fte, Status::Confirmed)
    }

    fn person(holdings: Vec<Holding>) -> Person {
        Person {
            holdings,
            engagement_ends_on: None,
            approved_leave: Vec::new(),
        }
    }

    fn holding(a: Assignment, allocations: Vec<Allocation>) -> Holding {
        Holding {
            assignment: a,
            allocations,
        }
    }

    fn known(standing: MonthStanding) -> PersonMonth {
        match standing {
            MonthStanding::Known(m) => m,
            MonthStanding::Unknown => panic!("expected a known month"),
        }
    }

    const JAN: (i16, i8) = (2026, 1);

    fn m(year: i16, month: i8) -> Date {
        d(year, month, 1)
    }

    // ── status ──

    #[test]
    fn status_words_round_trip() {
        for s in Status::ALL {
            assert_eq!(Status::parse(s.as_str()), Some(s));
        }
        assert_eq!(
            Status::parse("Confirmed"),
            None,
            "the stored word is lower case"
        );
        assert_eq!(Status::parse(""), None);
    }

    #[test]
    fn every_status_move_is_decided() {
        use Status::{Confirmed, Ended, Tentative};
        let allowed = [
            (Tentative, Confirmed),
            (Tentative, Ended),
            (Confirmed, Tentative),
            (Confirmed, Ended),
        ];
        for from in Status::ALL {
            for to in Status::ALL {
                assert_eq!(
                    from.can_move_to(to),
                    allowed.contains(&(from, to)),
                    "{from:?} -> {to:?}"
                );
            }
        }
    }

    // ── validation ──

    #[test]
    fn a_source_is_a_reference() {
        assert!(valid_source("thing:programme-a"));
        assert!(valid_source("service:urn:x:1"), "the rest may hold colons");
        for bad in ["", "programme", ":x", "thing:", "thing: x", "thing:x\ty"] {
            assert!(!valid_source(bad), "{bad:?}");
        }
        assert!(valid_source(&format!("t:{}", "x".repeat(SOURCE_MAX - 2))));
        assert!(!valid_source(&format!("t:{}", "x".repeat(SOURCE_MAX - 1))));
    }

    #[test]
    fn months_are_counted_inclusively() {
        assert_eq!(months_spanned(m(2026, 1), m(2026, 1)), 1);
        assert_eq!(months_spanned(m(2026, 1), m(2026, 12)), 12);
        assert_eq!(months_spanned(m(2026, 11), m(2027, 2)), 4);
        assert_eq!(months_spanned(m(2026, 2), m(2026, 1)), 0);
        assert_eq!(
            months_spanned(d(2026, 1, 31), d(2026, 2, 1)),
            2,
            "any day stands for its month"
        );
    }

    #[test]
    fn a_good_allocation_has_no_problems() {
        let a = assignment(60, d(2026, 1, 15), Some(d(2026, 12, 31)));
        assert_eq!(
            allocation_problems(&confirmed(60, m(2026, 1), m(2026, 12)), &a),
            Vec::<String>::new()
        );
        assert_eq!(
            allocation_problems(&confirmed(1, m(2026, 12), m(2026, 12)), &a),
            Vec::<String>::new()
        );
    }

    #[test]
    fn allocation_problems_are_each_named() {
        let a = assignment(50, d(2026, 3, 1), Some(d(2026, 8, 31)));
        let cases: [(Allocation, &str); 9] = [
            (
                alloc("nope", m(2026, 3), m(2026, 4), 10, Status::Confirmed),
                "source",
            ),
            (confirmed(10, d(2026, 3, 2), m(2026, 4)), "from_month"),
            (confirmed(10, m(2026, 3), d(2026, 4, 30)), "to_month"),
            (confirmed(10, m(2026, 5), m(2026, 4)), "before from_month"),
            (confirmed(0, m(2026, 3), m(2026, 4)), "fte_centi"),
            (confirmed(51, m(2026, 3), m(2026, 4)), "fte_centi"),
            (
                confirmed(10, m(2026, 2), m(2026, 4)),
                "before its assignment",
            ),
            (
                confirmed(10, m(2026, 9), m(2026, 9)),
                "after its assignment has ended",
            ),
            (confirmed(-5, m(2026, 3), m(2026, 3)), "fte_centi"),
        ];
        for (allocation, expected) in cases {
            let problems = allocation_problems(&allocation, &a);
            assert!(
                problems.iter().any(|p| p.contains(expected)),
                "{allocation:?}: {problems:?}"
            );
        }
    }

    #[test]
    fn an_allocation_covers_at_most_the_horizon() {
        let a = assignment(100, d(2026, 1, 1), None);
        assert!(
            allocation_problems(&confirmed(10, m(2026, 1), m(2028, 12)), &a).is_empty(),
            "36 months"
        );
        let too_long = allocation_problems(&confirmed(10, m(2026, 1), m(2029, 1)), &a);
        assert!(
            too_long.iter().any(|p| p.contains("at most 36")),
            "{too_long:?}"
        );
    }

    #[test]
    fn running_past_an_end_is_a_signal_not_a_problem() {
        let a = assignment(100, d(2026, 1, 1), Some(d(2026, 6, 30)));
        let long = confirmed(50, m(2026, 1), m(2026, 9));
        assert_eq!(allocation_problems(&long, &a), Vec::<String>::new());
        assert_eq!(runs_past_end(&long, &a, None), Some(d(2026, 6, 30)));
        assert_eq!(
            runs_past_end(&confirmed(50, m(2026, 1), m(2026, 6)), &a, None),
            None,
            "ending in the last month is fine"
        );
        let open = assignment(100, d(2026, 1, 1), None);
        assert_eq!(runs_past_end(&long, &open, None), None);
        assert_eq!(
            runs_past_end(&long, &open, Some(d(2026, 3, 15))),
            Some(d(2026, 3, 15)),
            "the engagement ends first"
        );
        assert_eq!(
            runs_past_end(&long, &a, Some(d(2026, 12, 31))),
            Some(d(2026, 6, 30)),
            "the earlier end wins"
        );
    }

    #[test]
    fn thresholds_are_bounded() {
        assert!(validate_threshold_bp(1).is_ok());
        assert!(validate_threshold_bp(FULL_BP).is_ok());
        assert!(validate_threshold_bp(DEFAULT_AVAILABLE_BELOW_BP).is_ok());
        assert!(validate_threshold_bp(0).is_err());
        assert!(validate_threshold_bp(FULL_BP + 1).is_err());
    }

    // ── arithmetic ──

    #[test]
    fn a_full_month_gives_the_whole_fte() {
        let p = person(vec![holding(assignment(60, d(2025, 1, 1), None), vec![])]);
        let one = holding_in(&p.holdings[0], &p, m(JAN.0, JAN.1));
        assert_eq!((one.contracted_centi, one.available_centi), (60, 60));
    }

    #[test]
    fn a_mid_month_start_is_weighted_by_its_days() {
        // Starts 16 January: 16 of 31 days; 100 × 16 / 31 = 51.6 → 52.
        let p = person(vec![holding(assignment(100, d(2026, 1, 16), None), vec![])]);
        let one = holding_in(&p.holdings[0], &p, m(2026, 1));
        assert_eq!(one.contracted_centi, 52);
        assert_eq!(
            holding_in(&p.holdings[0], &p, m(2025, 12)).contracted_centi,
            0,
            "before the start"
        );
    }

    #[test]
    fn the_last_day_is_inclusive() {
        // Ends 14 February 2026 (a 28-day month): 14 days, 100 × 14 / 28 = 50.
        let p = person(vec![holding(
            assignment(100, d(2025, 1, 1), Some(d(2026, 2, 14))),
            vec![],
        )]);
        assert_eq!(
            holding_in(&p.holdings[0], &p, m(2026, 2)).contracted_centi,
            50
        );
        assert_eq!(
            holding_in(&p.holdings[0], &p, m(2026, 3)).contracted_centi,
            0
        );
    }

    #[test]
    fn an_engagement_end_stops_the_holding() {
        let mut p = person(vec![holding(assignment(100, d(2025, 1, 1), None), vec![])]);
        p.engagement_ends_on = Some(d(2026, 2, 14));
        assert_eq!(
            holding_in(&p.holdings[0], &p, m(2026, 2)).contracted_centi,
            50
        );
        assert_eq!(
            holding_in(&p.holdings[0], &p, m(2026, 3)).contracted_centi,
            0
        );
    }

    #[test]
    fn leave_reduces_available_not_contracted() {
        let mut p = person(vec![holding(assignment(100, d(2025, 1, 1), None), vec![])]);
        // 10 days in April (30 days): 100 × 20 / 30 = 66.7 → 67.
        p.approved_leave = vec![(d(2026, 4, 6), d(2026, 4, 15))];
        let one = holding_in(&p.holdings[0], &p, m(2026, 4));
        assert_eq!((one.contracted_centi, one.available_centi), (100, 67));
    }

    #[test]
    fn leave_across_a_month_boundary_counts_in_each_month() {
        let mut p = person(vec![holding(assignment(100, d(2025, 1, 1), None), vec![])]);
        // 27 to 31 March (5 of 31) and 1 to 5 April (5 of 30).
        p.approved_leave = vec![(d(2026, 3, 27), d(2026, 4, 5))];
        assert_eq!(
            holding_in(&p.holdings[0], &p, m(2026, 3)).available_centi,
            84,
            "100 × 26 / 31 = 83.9"
        );
        assert_eq!(
            holding_in(&p.holdings[0], &p, m(2026, 4)).available_centi,
            83,
            "100 × 25 / 30 = 83.3"
        );
    }

    #[test]
    fn only_confirmed_allocations_count_and_tentative_are_shown_apart() {
        let allocations = vec![
            confirmed(30, m(2026, 1), m(2026, 3)),
            alloc("thing:b", m(2026, 1), m(2026, 1), 20, Status::Tentative),
            alloc("thing:c", m(2026, 1), m(2026, 1), 40, Status::Ended),
            confirmed(10, m(2026, 2), m(2026, 2)),
        ];
        let p = person(vec![holding(
            assignment(100, d(2025, 1, 1), None),
            allocations,
        )]);
        let jan = holding_in(&p.holdings[0], &p, m(2026, 1));
        assert_eq!((jan.allocated_centi, jan.tentative_centi), (30, 20));
        assert_eq!(
            holding_in(&p.holdings[0], &p, m(2026, 2)).allocated_centi,
            40
        );
        assert_eq!(
            holding_in(&p.holdings[0], &p, d(2026, 3, 20)).allocated_centi,
            30,
            "any day names its month"
        );
        assert_eq!(
            holding_in(&p.holdings[0], &p, m(2026, 4)).allocated_centi,
            0
        );
    }

    // ── signals ──

    #[test]
    fn untracked_is_unknown_never_zero() {
        let p = person(vec![holding(assignment(100, d(2025, 1, 1), None), vec![])]);
        assert_eq!(
            person_month(
                &p,
                m(2026, 1),
                Tracking::Untracked,
                DEFAULT_AVAILABLE_BELOW_BP
            ),
            MonthStanding::Unknown
        );
        let tracked = known(person_month(
            &p,
            m(2026, 1),
            Tracking::Tracked,
            DEFAULT_AVAILABLE_BELOW_BP,
        ));
        assert_eq!(
            tracked.signals,
            vec![Signal::Available { free_centi: 100 }],
            "tracked and none: a known zero"
        );
    }

    #[test]
    fn over_the_contract_is_over_allocated() {
        let p = person(vec![holding(
            assignment(60, d(2025, 1, 1), None),
            vec![
                confirmed(40, m(2026, 1), m(2026, 1)),
                confirmed(40, m(2026, 1), m(2026, 1)),
            ],
        )]);
        let month = known(person_month(
            &p,
            m(2026, 1),
            Tracking::Tracked,
            DEFAULT_AVAILABLE_BELOW_BP,
        ));
        assert_eq!(
            month.signals,
            vec![
                Signal::OverAllocated { by_centi: 20 },
                Signal::AssignmentOverAllocated {
                    holding: 0,
                    by_centi: 20
                }
            ]
        );
    }

    #[test]
    fn within_the_contract_but_not_the_leave_is_a_clash() {
        let mut p = person(vec![holding(
            assignment(100, d(2025, 1, 1), None),
            vec![confirmed(100, m(2026, 4), m(2026, 4))],
        )]);
        p.approved_leave = vec![(d(2026, 4, 6), d(2026, 4, 15))];
        let month = known(person_month(
            &p,
            m(2026, 4),
            Tracking::Tracked,
            DEFAULT_AVAILABLE_BELOW_BP,
        ));
        assert_eq!(month.signals, vec![Signal::LeaveClash { short_centi: 33 }]);
    }

    #[test]
    fn over_the_contract_with_leave_is_over_allocated_not_a_clash() {
        let mut p = person(vec![holding(
            assignment(50, d(2025, 1, 1), None),
            vec![
                confirmed(50, m(2026, 4), m(2026, 4)),
                confirmed(10, m(2026, 4), m(2026, 4)),
            ],
        )]);
        p.approved_leave = vec![(d(2026, 4, 6), d(2026, 4, 15))];
        let month = known(person_month(
            &p,
            m(2026, 4),
            Tracking::Tracked,
            DEFAULT_AVAILABLE_BELOW_BP,
        ));
        assert_eq!(month.signals[0], Signal::OverAllocated { by_centi: 10 });
        assert!(
            !month
                .signals
                .iter()
                .any(|s| matches!(s, Signal::LeaveClash { .. }))
        );
    }

    #[test]
    fn availability_follows_the_threshold() {
        let at = |allocated: i64, threshold: i32| {
            let p = person(vec![holding(
                assignment(100, d(2025, 1, 1), None),
                vec![confirmed(allocated, m(2026, 1), m(2026, 1))],
            )]);
            known(person_month(&p, m(2026, 1), Tracking::Tracked, threshold)).signals
        };
        assert_eq!(at(79, 8_000), vec![Signal::Available { free_centi: 21 }]);
        assert_eq!(at(80, 8_000), vec![], "at the threshold is not below it");
        assert_eq!(at(100, 8_000), vec![], "fully allocated says nothing");
        assert_eq!(at(99, FULL_BP), vec![Signal::Available { free_centi: 1 }]);
        assert_eq!(
            at(1, 1),
            vec![],
            "a threshold of 0.01% flags only nothing allocated"
        );
    }

    #[test]
    fn nobody_is_available_in_a_month_they_cannot_work() {
        let p = person(vec![holding(assignment(100, d(2026, 6, 1), None), vec![])]);
        let before = known(person_month(
            &p,
            m(2026, 1),
            Tracking::Tracked,
            DEFAULT_AVAILABLE_BELOW_BP,
        ));
        assert_eq!(before.signals, Vec::<Signal>::new());
        let mut away = person(vec![holding(assignment(100, d(2025, 1, 1), None), vec![])]);
        away.approved_leave = vec![(d(2026, 2, 1), d(2026, 2, 28))];
        let month = known(person_month(
            &away,
            m(2026, 2),
            Tracking::Tracked,
            DEFAULT_AVAILABLE_BELOW_BP,
        ));
        assert!(
            month.signals.is_empty(),
            "a month on leave is not availability: {:?}",
            month.signals
        );
    }

    #[test]
    fn two_assignments_add_up_and_each_keeps_its_own_limit() {
        // 50 + 50: the person's 80 fits 100, but 70 on the first assignment is over its 50.
        let p = person(vec![
            holding(
                assignment(50, d(2025, 1, 1), None),
                vec![confirmed(70, m(2026, 1), m(2026, 1))],
            ),
            holding(
                assignment(50, d(2025, 1, 1), None),
                vec![confirmed(10, m(2026, 1), m(2026, 1))],
            ),
        ]);
        let month = known(person_month(
            &p,
            m(2026, 1),
            Tracking::Tracked,
            DEFAULT_AVAILABLE_BELOW_BP,
        ));
        assert_eq!(month.totals.contracted_centi, 100);
        assert_eq!(month.totals.allocated_centi, 80);
        assert_eq!(
            month.signals,
            vec![Signal::AssignmentOverAllocated {
                holding: 0,
                by_centi: 20
            }]
        );
    }

    #[test]
    fn part_time_is_measured_against_its_own_fte() {
        let p = person(vec![holding(
            assignment(40, d(2025, 1, 1), None),
            vec![confirmed(40, m(2026, 1), m(2026, 1))],
        )]);
        let month = known(person_month(
            &p,
            m(2026, 1),
            Tracking::Tracked,
            DEFAULT_AVAILABLE_BELOW_BP,
        ));
        assert!(
            month.signals.is_empty(),
            "40 of 40 is fully allocated: {:?}",
            month.signals
        );
    }

    #[test]
    fn a_leaver_with_allocations_after_leaving_is_over_allocated() {
        let p = person(vec![holding(
            assignment(100, d(2025, 1, 1), Some(d(2026, 2, 14))),
            vec![confirmed(100, m(2026, 1), m(2026, 3))],
        )]);
        let feb = known(person_month(
            &p,
            m(2026, 2),
            Tracking::Tracked,
            DEFAULT_AVAILABLE_BELOW_BP,
        ));
        assert_eq!(feb.signals[0], Signal::OverAllocated { by_centi: 50 });
        let mar = known(person_month(
            &p,
            m(2026, 3),
            Tracking::Tracked,
            DEFAULT_AVAILABLE_BELOW_BP,
        ));
        assert_eq!(mar.signals[0], Signal::OverAllocated { by_centi: 100 });
    }

    /// Every combination over a small grid: the signals always partition as the rules say.
    #[test]
    fn signals_partition_over_a_grid() {
        for fte in [10, 40, 50, 100] {
            for leave_days in [0_i8, 1, 10, 30] {
                for allocated in [0_i64, 1, 39, 40, 79, 80, 100, 120] {
                    let mut p = person(vec![holding(
                        assignment(fte, d(2025, 1, 1), None),
                        if allocated > 0 {
                            vec![confirmed(allocated, m(2026, 4), m(2026, 4))]
                        } else {
                            vec![]
                        },
                    )]);
                    if leave_days > 0 {
                        p.approved_leave = vec![(d(2026, 4, 1), d(2026, 4, leave_days))];
                    }
                    let month = known(person_month(
                        &p,
                        m(2026, 4),
                        Tracking::Tracked,
                        DEFAULT_AVAILABLE_BELOW_BP,
                    ));
                    let t = month.totals;
                    let standing: Vec<_> = month
                        .signals
                        .iter()
                        .filter(|s| !matches!(s, Signal::AssignmentOverAllocated { .. }))
                        .collect();
                    assert!(
                        standing.len() <= 1,
                        "at most one standing signal: {standing:?}"
                    );
                    let expected_over = t.allocated_centi > t.contracted_centi;
                    let expected_clash = !expected_over && t.allocated_centi > t.available_centi;
                    assert_eq!(
                        matches!(standing.first(), Some(Signal::OverAllocated { .. })),
                        expected_over
                    );
                    assert_eq!(
                        matches!(standing.first(), Some(Signal::LeaveClash { .. })),
                        expected_clash
                    );
                    if let Some(Signal::Available { free_centi }) = standing.first() {
                        assert!(*free_centi > 0 && t.available_centi > 0);
                    }
                    assert!(t.available_centi <= t.contracted_centi);
                }
            }
        }
    }

    // ── access ──

    #[test]
    fn named_reads_follow_the_relation() {
        let none = Reader::default();
        assert_eq!(
            read_access(none, true),
            Read::CountsOnly,
            "a stranger sees counts"
        );
        for reader in [
            Reader {
                is_self: true,
                ..none
            },
            Reader {
                line_manager: true,
                ..none
            },
            Reader { hr: true, ..none },
            Reader {
                resource_manager: true,
                ..none
            },
        ] {
            assert_eq!(read_access(reader, false), Read::Named, "{reader:?}");
        }
        let owner = Reader {
            source_owner: true,
            ..none
        };
        assert_eq!(
            read_access(owner, false),
            Read::CountsOnly,
            "an owner sees names only where the policy grants it"
        );
        assert_eq!(read_access(owner, true), Read::Named);
    }

    #[test]
    fn only_reads_of_another_person_are_audited() {
        let none = Reader::default();
        assert!(
            !named_read_is_audited(
                Reader {
                    is_self: true,
                    ..none
                },
                true
            ),
            "your own is not audited"
        );
        assert!(
            !named_read_is_audited(
                Reader {
                    is_self: true,
                    hr: true,
                    ..none
                },
                true
            ),
            "nor is your own when you also hold HR access: it is still your own"
        );
        assert!(named_read_is_audited(
            Reader {
                resource_manager: true,
                ..none
            },
            false
        ));
        assert!(named_read_is_audited(
            Reader {
                line_manager: true,
                ..none
            },
            false
        ));
        assert!(named_read_is_audited(Reader { hr: true, ..none }, false));
        assert!(named_read_is_audited(
            Reader {
                source_owner: true,
                ..none
            },
            true
        ));
        assert!(
            !named_read_is_audited(
                Reader {
                    source_owner: true,
                    ..none
                },
                false
            ),
            "counts are not a named read"
        );
        assert!(
            !named_read_is_audited(none, true),
            "counts are not a named read"
        );
    }

    #[test]
    fn hr_and_resource_managers_write_and_nobody_else() {
        let none = Reader::default();
        assert!(may_write(Reader { hr: true, ..none }));
        assert!(may_write(Reader {
            resource_manager: true,
            ..none
        }));
        assert!(
            may_write(Reader {
                hr: true,
                is_self: true,
                ..none
            }),
            "HR may correct their own, audited as any write"
        );
        assert!(
            !may_write(Reader {
                resource_manager: true,
                is_self: true,
                ..none
            }),
            "a resource manager does not write their own"
        );
        for reader in [
            none,
            Reader {
                is_self: true,
                ..none
            },
            Reader {
                line_manager: true,
                ..none
            },
            Reader {
                source_owner: true,
                ..none
            },
        ] {
            assert!(!may_write(reader), "{reader:?}");
        }
    }

    // ── reconciliation ──

    #[test]
    fn confirmed_allocations_are_summed_by_pool_source_and_month() {
        let pool = Uuid::from_u128(1);
        let other = Uuid::from_u128(2);
        let mut a = confirmed(30, m(2026, 1), m(2026, 3));
        a.pool = Some(pool);
        let mut b = confirmed(20, m(2026, 2), m(2026, 2));
        b.pool = Some(pool);
        let mut c = confirmed(50, m(2026, 2), m(2026, 2));
        c.pool = Some(other);
        let mut t = alloc(
            "thing:programme-a",
            m(2026, 2),
            m(2026, 2),
            15,
            Status::Tentative,
        );
        t.pool = Some(pool);
        let mut wrong_source = alloc(
            "thing:programme-b",
            m(2026, 2),
            m(2026, 2),
            15,
            Status::Confirmed,
        );
        wrong_source.pool = Some(pool);
        let unpooled = confirmed(5, m(2026, 2), m(2026, 2));
        let all = [a, b, c, t, wrong_source, unpooled];
        assert_eq!(
            sum_confirmed(&all, pool, "thing:programme-a", m(2026, 2)),
            Some(50)
        );
        assert_eq!(
            sum_confirmed(&all, pool, "thing:programme-a", m(2026, 1)),
            Some(30)
        );
        assert_eq!(
            sum_confirmed(&all, other, "thing:programme-a", m(2026, 2)),
            Some(50)
        );
        assert_eq!(
            sum_confirmed(&all, pool, "thing:programme-a", m(2026, 4)),
            Some(0)
        );
    }

    #[test]
    fn reconciliation_says_how_far_apart() {
        assert_eq!(
            reconcile(Tracking::Untracked, Some(100), 100),
            Staffing::Unknown
        );
        assert_eq!(reconcile(Tracking::Untracked, None, 0), Staffing::Unknown);
        assert_eq!(
            reconcile(Tracking::Tracked, None, 30),
            Staffing::NotClaimed {
                allocated_centi: 30
            }
        );
        assert_eq!(
            reconcile(Tracking::Tracked, Some(100), 100),
            Staffing::Staffed
        );
        assert_eq!(
            reconcile(Tracking::Tracked, Some(100), 60),
            Staffing::Understaffed { by_centi: 40 }
        );
        assert_eq!(
            reconcile(Tracking::Tracked, Some(100), 0),
            Staffing::Understaffed { by_centi: 100 }
        );
        assert_eq!(
            reconcile(Tracking::Tracked, Some(100), 130),
            Staffing::Overstaffed { by_centi: 30 }
        );
    }

    #[test]
    fn totals_that_do_not_fit_are_unknown_not_wrapped() {
        let big = |fte: i64| {
            holding(
                assignment(100, d(2025, 1, 1), None),
                vec![confirmed(fte, m(2026, 4), m(2026, 4))],
            )
        };
        let stand =
            |p: &Person| person_month(p, m(2026, 4), Tracking::Tracked, DEFAULT_AVAILABLE_BELOW_BP);
        assert_eq!(
            stand(&person(vec![big(i64::MAX), big(1)])),
            MonthStanding::Unknown,
            "allocated totals overflow"
        );
        assert_eq!(
            stand(&person(vec![big(i64::MIN)])),
            MonthStanding::Unknown,
            "the share of what is available overflows"
        );
        assert!(matches!(
            stand(&person(vec![big(1)])),
            MonthStanding::Known(_)
        ));
        assert_eq!(
            reconcile(Tracking::Tracked, Some(i64::MIN), i64::MAX),
            Staffing::Unknown
        );
        assert_eq!(
            reconcile(Tracking::Tracked, Some(i64::MAX), i64::MIN),
            Staffing::Unknown
        );
    }

    #[test]
    fn days_outside_the_month_and_totals_past_a_count_are_unknown_not_clamped() {
        let feb = d(2026, 2, 1);
        assert_eq!(weighted(100, 28, feb), Some(100));
        assert_eq!(weighted(100, 29, feb), None);
        assert_eq!(weighted(100, -1, feb), None);
        assert_eq!(weighted(i32::MAX, 28, feb), Some(i64::from(i32::MAX)));
        let pool = Uuid::from_u128(1);
        let mut a = alloc("thing:programme-a", feb, feb, i64::MAX, Status::Confirmed);
        a.pool = Some(pool);
        let mut b = a.clone();
        b.fte_centi = 1;
        assert_eq!(
            sum_confirmed([&a, &b], pool, "thing:programme-a", feb),
            None
        );
        assert_eq!(
            sum_confirmed([&a], pool, "thing:programme-a", feb),
            Some(i64::MAX)
        );
    }

    #[test]
    fn a_threshold_outside_zero_to_ten_thousand_is_no_standing() {
        let p = person(vec![holding(assignment(100, d(2026, 1, 1), None), vec![])]);
        for bp in [-1, FULL_BP + 1] {
            assert_eq!(
                person_month(&p, d(2026, 1, 1), Tracking::Tracked, bp),
                MonthStanding::Unknown
            );
        }
        assert!(matches!(
            person_month(&p, d(2026, 1, 1), Tracking::Tracked, FULL_BP),
            MonthStanding::Known(_)
        ));
    }
}
