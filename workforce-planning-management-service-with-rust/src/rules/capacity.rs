//! **Delivery capacity** (WPM-R56, WPM-R59, WPM-R60, WPM-D45, WPM-D46, WPM-D50): pure,
//! database-free and clock-free arithmetic.
//!
//! Capacity is planned by **skill pool**, not by person (WPM-D45). A pool's monthly
//! supply is the FTE of the workers who count toward it, weighted by the days of the
//! month they are employed and not on approved leave, less the pool's operations
//! reservation. Demand is what programmes claim from the pool. Partner commitments are
//! kept beside the pool's own supply, and a month with no commitment from a partner is
//! **unknown**, never zero (WPM-D46).
//!
//! All FTE figures are whole hundredths of an FTE ("centi-FTE": `100` is one full-time
//! equivalent). A month is identified by its first day.
//!
//! The start check is a **suggestion with its derivation**; a person decides (WPM-D50).
//! It answers a capacity question only. Scheduling the work belongs to
//! project-portfolio-management (WPM-D55).

use chrono::{Datelike, Months, NaiveDate};
use std::collections::{BTreeMap, BTreeSet};
use uuid::Uuid;

/// The longest horizon the view or a start check covers, in months.
pub const MAX_HORIZON_MONTHS: u32 = 36;
/// Basis points in 100% (the operations reservation scale).
pub const FULL_BP: i32 = 10_000;
/// The most FTE a single claim or commitment may state (10 000 FTE), as a sanity bound.
pub const MAX_FTE_CENTI: i64 = 1_000_000;

/// How the view explains itself. Shown with every capacity answer.
pub const DERIVATION: &str = "Supply = for each worker who counts toward the pool, FTE × the days of \
the month they are employed and not on approved leave ÷ the days in the month, summed, less the \
pool's operations reservation. Demand = the FTE that active programme claims take from the pool. \
Confirmed and committed partner capacity covers a shortfall; requested capacity does not. A month \
in which a partner has made no commitment is unknown, not zero.";

// ─── Months ─────────────────────────────────────────────────────────────────

/// The first day of the month containing `date`.
#[must_use]
pub fn month_start(date: NaiveDate) -> NaiveDate {
    date.with_day(1).unwrap_or(date)
}

/// `count` consecutive months starting at the month of `start`.
///
/// # Errors
///
/// A `count` of zero or more than [`MAX_HORIZON_MONTHS`].
pub fn months(start: NaiveDate, count: u32) -> Result<Vec<NaiveDate>, String> {
    if count == 0 || count > MAX_HORIZON_MONTHS {
        return Err(format!(
            "the horizon is 1 to {MAX_HORIZON_MONTHS} months, not {count}"
        ));
    }
    let first = month_start(start);
    (0..count)
        .map(|i| {
            first
                .checked_add_months(Months::new(i))
                .ok_or_else(|| "the horizon runs past the supported calendar".to_string())
        })
        .collect()
}

/// `month` moved by `by` months (still the first of a month).
#[must_use]
pub fn shift_month(month: NaiveDate, by: u32) -> Option<NaiveDate> {
    month_start(month).checked_add_months(Months::new(by))
}

/// The number of calendar days in the month starting `month`.
#[must_use]
pub fn days_in_month(month: NaiveDate) -> i64 {
    let first = month_start(month);
    first
        .checked_add_months(Months::new(1))
        .map_or(31, |next| (next - first).num_days())
}

// ─── Validation ─────────────────────────────────────────────────────────────

/// An operations reservation in basis points: `0..=10000`.
///
/// # Errors
///
/// A value outside that range.
pub fn validate_reservation_bp(bp: i32) -> Result<(), String> {
    if (0..=FULL_BP).contains(&bp) {
        Ok(())
    } else {
        Err(format!(
            "operations_reservation_bp must be between 0 and {FULL_BP}, not {bp}"
        ))
    }
}

/// An FTE claim or commitment in hundredths: positive and bounded.
///
/// # Errors
///
/// Zero, negative, or above [`MAX_FTE_CENTI`].
pub fn validate_fte_centi(fte_centi: i64) -> Result<(), String> {
    if (1..=MAX_FTE_CENTI).contains(&fte_centi) {
        Ok(())
    } else {
        Err(format!(
            "fte_centi must be between 1 and {MAX_FTE_CENTI} (100 is one FTE), not {fte_centi}"
        ))
    }
}

/// A month given as a date must be its first day.
///
/// # Errors
///
/// Any other day of the month.
pub fn validate_month(month: NaiveDate) -> Result<(), String> {
    if month.day() == 1 {
        Ok(())
    } else {
        Err(format!(
            "month must be the first day of a month, not {month}"
        ))
    }
}

/// A pool member names a role profile **or** a skill (with a minimum proficiency),
/// never both and never neither.
///
/// # Errors
///
/// Both or neither given, or a proficiency outside `1..=5`.
pub fn validate_member(
    role_profile: Option<Uuid>,
    skill: Option<Uuid>,
    min_proficiency: Option<i32>,
) -> Result<(), String> {
    match (role_profile, skill) {
        (Some(_), None) => {
            if min_proficiency.is_some() {
                return Err("a role profile member has no minimum proficiency".to_string());
            }
            Ok(())
        }
        (None, Some(_)) => match min_proficiency {
            Some(p) if (1..=5).contains(&p) => Ok(()),
            _ => Err("a skill member needs a minimum proficiency from 1 to 5".to_string()),
        },
        _ => Err("give either a role profile or a skill, not both and not neither".to_string()),
    }
}

// ─── Supply ─────────────────────────────────────────────────────────────────

/// One way a worker counts toward a pool.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Member {
    /// The worker holds this role profile.
    RoleProfile(Uuid),
    /// The worker has this skill at at least this proficiency.
    Skill {
        /// The skill.
        skill: Uuid,
        /// The lowest proficiency (1 to 5) that counts.
        min_proficiency: i32,
    },
}

/// What the capacity arithmetic needs to know about a worker. No name, no reference to
/// the person: a worker is an FTE with dates, a role and some skills.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct WorkerFacts {
    /// Contracted FTE as a percentage (`100` is full time).
    pub fte_percent: i32,
    /// First day of employment.
    pub hired_on: NaiveDate,
    /// Day employment ended, if it has.
    pub terminated_on: Option<NaiveDate>,
    /// The role profiles the worker holds.
    pub role_profiles: Vec<Uuid>,
    /// Declared skills and their proficiency.
    pub skills: Vec<(Uuid, i32)>,
    /// Approved leave as inclusive `(start, end)` date ranges.
    pub approved_leave: Vec<(NaiveDate, NaiveDate)>,
}

/// Whether a worker counts toward a pool with these members. A worker who qualifies in
/// several ways counts once. A pool with no members counts nobody.
#[must_use]
pub fn counts_toward(members: &[Member], worker: &WorkerFacts) -> bool {
    members.iter().any(|member| match member {
        Member::RoleProfile(role) => worker.role_profiles.contains(role),
        Member::Skill {
            skill,
            min_proficiency,
        } => worker
            .skills
            .iter()
            .any(|(s, level)| s == skill && level >= min_proficiency),
    })
}

/// The days of `month` on which the worker is employed and not on approved leave.
#[must_use]
pub fn available_days(worker: &WorkerFacts, month: NaiveDate) -> i64 {
    let first = month_start(month);
    let mut count = 0;
    for offset in 0..days_in_month(first) {
        let Some(day) =
            first.checked_add_days(chrono::Days::new(u64::try_from(offset).unwrap_or(0)))
        else {
            continue;
        };
        let employed =
            worker.hired_on <= day && worker.terminated_on.is_none_or(|ended| ended > day);
        let on_leave = worker
            .approved_leave
            .iter()
            .any(|(start, end)| *start <= day && day <= *end);
        if employed && !on_leave {
            count += 1;
        }
    }
    count
}

/// A pool's supply in `month`, in centi-FTE: the workers who count toward it, weighted by
/// available days, less the operations reservation, rounded half up. A pool with no
/// members, or none employed that month, has a supply of `0` (a known zero: the pool
/// exists and has nobody).
#[must_use]
pub fn pool_supply_centi(
    members: &[Member],
    reservation_bp: i32,
    workers: &[WorkerFacts],
    month: NaiveDate,
) -> i64 {
    let weighted: i64 = workers
        .iter()
        .filter(|w| counts_toward(members, w))
        .map(|w| i64::from(w.fte_percent.max(0)) * available_days(w, month))
        .sum();
    let kept = i64::from(FULL_BP - reservation_bp.clamp(0, FULL_BP));
    let denominator = days_in_month(month) * i64::from(FULL_BP);
    (2 * weighted * kept + denominator) / (2 * denominator)
}

// ─── Demand, commitments and the grid ───────────────────────────────────────

/// Where a programme's claim on a pool stands.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum DemandStatus {
    /// Asked for, not yet agreed to start: shown, not counted against supply.
    Proposed,
    /// Agreed to start: counted against supply.
    Active,
    /// Finished or withdrawn: ignored.
    Closed,
}

impl DemandStatus {
    /// Every status token.
    pub const ALL: [&'static str; 3] = ["proposed", "active", "closed"];

    /// Parse a token.
    #[must_use]
    pub fn parse(text: &str) -> Option<Self> {
        match text {
            "proposed" => Some(Self::Proposed),
            "active" => Some(Self::Active),
            "closed" => Some(Self::Closed),
            _ => None,
        }
    }
}

/// A programme's claim on a pool for one month.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Demand {
    /// The programme's `EntityRef` URN (owned by project-portfolio-management).
    pub programme: String,
    /// The month (its first day).
    pub month: NaiveDate,
    /// FTE wanted, in hundredths.
    pub fte_centi: i64,
    /// Where the claim stands.
    pub status: DemandStatus,
}

/// How firm a partner's commitment is.
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord)]
pub enum CommitmentStatus {
    /// Asked for; does not cover a shortfall.
    Requested,
    /// Agreed in principle; covers a shortfall.
    Committed,
    /// Confirmed; covers a shortfall.
    Confirmed,
}

impl CommitmentStatus {
    /// Every status token.
    pub const ALL: [&'static str; 3] = ["requested", "committed", "confirmed"];

    /// Parse a token.
    #[must_use]
    pub fn parse(text: &str) -> Option<Self> {
        match text {
            "requested" => Some(Self::Requested),
            "committed" => Some(Self::Committed),
            "confirmed" => Some(Self::Confirmed),
            _ => None,
        }
    }

    /// The token.
    #[must_use]
    pub const fn as_str(self) -> &'static str {
        match self {
            Self::Requested => "requested",
            Self::Committed => "committed",
            Self::Confirmed => "confirmed",
        }
    }

    /// Whether this commitment can cover a shortfall.
    #[must_use]
    pub const fn is_firm(self) -> bool {
        !matches!(self, Self::Requested)
    }
}

/// A partner organization's capacity for a pool in a month, as aggregate FTE. It never
/// names partner staff.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Commitment {
    /// The partner's `EntityRef` URN.
    pub partner: String,
    /// The month (its first day).
    pub month: NaiveDate,
    /// FTE offered, in hundredths.
    pub fte_centi: i64,
    /// How firm it is.
    pub status: CommitmentStatus,
}

/// One partner's row in one month. `None` means the partner has said nothing: unknown.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct PartnerCell {
    /// The partner's URN.
    pub partner: String,
    /// The commitment, or `None` for unknown.
    pub commitment: Option<(CommitmentStatus, i64)>,
}

/// What a pool-month comes to.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum CellStatus {
    /// Own supply covers the active demand.
    Fits,
    /// Own supply falls short and firm partner capacity covers the rest.
    FitsWithPartners,
    /// Short even with firm partner capacity, and every partner has answered.
    Over,
    /// Short, and at least one partner has made no commitment, so it cannot be said.
    Unknown,
}

impl CellStatus {
    /// The token.
    #[must_use]
    pub const fn as_str(self) -> &'static str {
        match self {
            Self::Fits => "fits",
            Self::FitsWithPartners => "fits_with_partners",
            Self::Over => "over",
            Self::Unknown => "unknown",
        }
    }
}

/// One month of one pool.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Cell {
    /// The month (its first day).
    pub month: NaiveDate,
    /// Own supply, in centi-FTE.
    pub supply_centi: i64,
    /// Demand from active programme claims.
    pub demand_centi: i64,
    /// Demand from claims not yet agreed (shown, not counted).
    pub proposed_centi: i64,
    /// `supply - demand`; negative is a shortfall before partners.
    pub own_balance_centi: i64,
    /// Confirmed plus committed partner FTE.
    pub firm_partner_centi: i64,
    /// The shortfall left after firm partner capacity (`0` when nothing is left).
    pub remaining_shortfall_centi: i64,
    /// Each known partner this month, answered or not.
    pub partners: Vec<PartnerCell>,
    /// The result.
    pub status: CellStatus,
}

/// A pool across the horizon.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct PoolView {
    /// One cell per month.
    pub cells: Vec<Cell>,
    /// The shortfall summed over the months that are `Over`, in centi-FTE-months.
    pub over_total_centi: i64,
    /// How many months are `Over`.
    pub over_months: usize,
    /// How many months cannot be said.
    pub unknown_months: usize,
}

/// Build a pool's view over `months`, given its supply in each (same order), the demand
/// claims and the partner commitments. Where a partner has two commitments for one month
/// the later in the list wins.
#[must_use]
pub fn build_pool(
    months: &[NaiveDate],
    supply_centi: &[i64],
    demand: &[Demand],
    commitments: &[Commitment],
) -> PoolView {
    let partners: BTreeSet<&str> = commitments.iter().map(|c| c.partner.as_str()).collect();
    let mut cells = Vec::with_capacity(months.len());
    let mut over_total = 0;
    let mut over_months = 0;
    let mut unknown_months = 0;
    for (index, month) in months.iter().enumerate() {
        let supply = supply_centi.get(index).copied().unwrap_or(0);
        let sum = |status: DemandStatus| -> i64 {
            demand
                .iter()
                .filter(|d| d.month == *month && d.status == status)
                .map(|d| d.fte_centi)
                .sum()
        };
        let active = sum(DemandStatus::Active);
        let proposed = sum(DemandStatus::Proposed);
        let mut by_partner: BTreeMap<&str, (CommitmentStatus, i64)> = BTreeMap::new();
        for c in commitments.iter().filter(|c| c.month == *month) {
            by_partner.insert(c.partner.as_str(), (c.status, c.fte_centi));
        }
        let partner_cells: Vec<PartnerCell> = partners
            .iter()
            .map(|p| PartnerCell {
                partner: (*p).to_string(),
                commitment: by_partner.get(p).copied(),
            })
            .collect();
        let firm: i64 = partner_cells
            .iter()
            .filter_map(|p| p.commitment)
            .filter(|(status, _)| status.is_firm())
            .map(|(_, fte)| fte)
            .sum();
        let any_unknown = partner_cells.iter().any(|p| p.commitment.is_none());
        let balance = supply - active;
        let remaining = (-(balance + firm)).max(0);
        let status = if balance >= 0 {
            CellStatus::Fits
        } else if remaining == 0 {
            CellStatus::FitsWithPartners
        } else if any_unknown {
            CellStatus::Unknown
        } else {
            CellStatus::Over
        };
        match status {
            CellStatus::Over => {
                over_total += remaining;
                over_months += 1;
            }
            CellStatus::Unknown => unknown_months += 1,
            _ => {}
        }
        cells.push(Cell {
            month: *month,
            supply_centi: supply,
            demand_centi: active,
            proposed_centi: proposed,
            own_balance_centi: balance,
            firm_partner_centi: firm,
            remaining_shortfall_centi: remaining,
            partners: partner_cells,
            status,
        });
    }
    PoolView {
        cells,
        over_total_centi: over_total,
        over_months,
        unknown_months,
    }
}

/// The **constraint pool**: the one most over-committed over the horizon, or `None` when
/// no pool is over. Months that cannot be said do not count toward it. A tie goes to the
/// lowest key, so the answer is stable.
#[must_use]
pub fn constraint_pool<'a>(pools: &[(&'a str, &PoolView)]) -> Option<&'a str> {
    pools
        .iter()
        .filter(|(_, view)| view.over_total_centi > 0)
        .max_by(|(ka, a), (kb, b)| {
            a.over_total_centi
                .cmp(&b.over_total_centi)
                .then_with(|| kb.cmp(ka))
        })
        .map(|(key, _)| *key)
}

/// The programmes with active demand in a pool.
#[must_use]
pub fn programmes_drawing(demand: &[Demand]) -> BTreeSet<String> {
    demand
        .iter()
        .filter(|d| d.status == DemandStatus::Active && d.fte_centi > 0)
        .map(|d| d.programme.clone())
        .collect()
}

// ─── Start check ────────────────────────────────────────────────────────────

/// A programme's wish: FTE from pools in months.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Candidate {
    /// The programme's URN.
    pub programme: String,
    /// `(pool key, month, centi-FTE)` lines.
    pub lines: Vec<(String, NaiveDate, i64)>,
}

/// How a start check came out overall.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Fit {
    /// Every line fits.
    Yes,
    /// At least one line does not fit.
    No,
    /// Nothing fails, but something cannot be said (an unanswered partner, or a month
    /// outside the horizon).
    Unknown,
}

impl Fit {
    /// The token.
    #[must_use]
    pub const fn as_str(self) -> &'static str {
        match self {
            Self::Yes => "yes",
            Self::No => "no",
            Self::Unknown => "unknown",
        }
    }
}

/// Why one line does not fit, or cannot be said.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct LineResult {
    /// The pool.
    pub pool: String,
    /// The month the line falls in after any shift.
    pub month: NaiveDate,
    /// FTE the programme wants there (lines on one pool-month added together).
    pub needed_centi: i64,
    /// FTE available there: supply less active demand, plus firm partner capacity.
    pub available_centi: Option<i64>,
    /// How far short (`0` when it fits or cannot be said).
    pub short_by_centi: i64,
    /// `fits`, `short`, `unknown_partner` or `outside_horizon`.
    pub outcome: &'static str,
}

/// The work-in-progress check on the constraint pool.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Wip {
    /// The constraint pool.
    pub pool: String,
    /// The limit on active programmes drawing on it.
    pub limit: i32,
    /// Programmes already active on it.
    pub active: usize,
    /// How many there would be if this one started.
    pub would_be: usize,
    /// Whether that is above the limit.
    pub exceeds: bool,
}

/// The answer to "can this programme start?": a suggestion with its evidence.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct StartCheck {
    /// Whether the programme fits starting as asked.
    pub fit: Fit,
    /// Each line as asked (no shift).
    pub lines: Vec<LineResult>,
    /// The months of delay that give the first fit known to hold, or `None` if there is
    /// none within the horizon.
    pub earliest_shift_months: Option<u32>,
    /// The work-in-progress check, when a limit is set and a pool is the constraint.
    pub wip: Option<Wip>,
    /// A sentence saying how the answer was reached.
    pub derivation: String,
}

fn check_lines(
    candidate: &Candidate,
    pools: &BTreeMap<String, PoolView>,
    shift: u32,
) -> Vec<LineResult> {
    let mut needed: BTreeMap<(String, NaiveDate), i64> = BTreeMap::new();
    for (pool, month, fte) in &candidate.lines {
        let moved = shift_month(*month, shift).unwrap_or(*month);
        *needed.entry((pool.clone(), moved)).or_insert(0) += fte;
    }
    needed
        .into_iter()
        .map(|((pool, month), needed_centi)| {
            let cell = pools
                .get(&pool)
                .and_then(|view| view.cells.iter().find(|c| c.month == month));
            match cell {
                None => LineResult {
                    pool,
                    month,
                    needed_centi,
                    available_centi: None,
                    short_by_centi: 0,
                    outcome: "outside_horizon",
                },
                Some(cell) => {
                    let available = cell.own_balance_centi + cell.firm_partner_centi;
                    let short = (needed_centi - available).max(0);
                    let any_unknown = cell.partners.iter().any(|p| p.commitment.is_none());
                    let outcome = if short == 0 {
                        "fits"
                    } else if any_unknown {
                        "unknown_partner"
                    } else {
                        "short"
                    };
                    LineResult {
                        pool,
                        month,
                        needed_centi,
                        available_centi: Some(available),
                        short_by_centi: if outcome == "short" { short } else { 0 },
                        outcome,
                    }
                }
            }
        })
        .collect()
}

fn fit_of(lines: &[LineResult]) -> Fit {
    if lines.iter().any(|l| l.outcome == "short") {
        Fit::No
    } else if lines.iter().any(|l| l.outcome != "fits") {
        Fit::Unknown
    } else {
        Fit::Yes
    }
}

/// Check whether a programme can start, given each pool's view (which counts only
/// **active** demand, so a candidate whose own claims are still proposed is not counted
/// twice). `active_on_constraint` is the set of programmes with active demand on the
/// constraint pool, and `wip_limit` the organization's limit on them.
#[must_use]
pub fn start_check(
    candidate: &Candidate,
    pools: &BTreeMap<String, PoolView>,
    wip_limit: Option<i32>,
    constraint: Option<&str>,
    active_on_constraint: &BTreeSet<String>,
) -> StartCheck {
    let lines = check_lines(candidate, pools, 0);
    let fit = fit_of(&lines);
    let latest = pools
        .values()
        .filter_map(|v| v.cells.len().checked_sub(1))
        .max()
        .unwrap_or(0);
    let earliest_shift_months = (0..=u32::try_from(latest).unwrap_or(0)).find(|shift| {
        let shifted = check_lines(candidate, pools, *shift);
        fit_of(&shifted) == Fit::Yes
    });
    let wip = match (wip_limit, constraint) {
        (Some(limit), Some(pool)) => {
            let draws = candidate.lines.iter().any(|(p, _, _)| p == pool);
            let active = active_on_constraint.len();
            let adds = usize::from(draws && !active_on_constraint.contains(&candidate.programme));
            let would_be = active + adds;
            Some(Wip {
                pool: pool.to_string(),
                limit,
                active,
                would_be,
                exceeds: i64::try_from(would_be).unwrap_or(i64::MAX) > i64::from(limit),
            })
        }
        _ => None,
    };
    let derivation = match (fit, earliest_shift_months) {
        (Fit::Yes, _) => "Every month the programme asks for fits within supply and firm partner capacity.".to_string(),
        (_, Some(0)) => "Some months cannot be said (an unanswered partner or a month outside the horizon); the check found no month that is short.".to_string(),
        (_, Some(shift)) => format!(
            "Starting {shift} month(s) later is the first delay at which every month fits within supply and firm partner capacity."
        ),
        (_, None) => "No delay within the horizon makes every month fit.".to_string(),
    };
    StartCheck {
        fit,
        lines,
        earliest_shift_months,
        wip,
        derivation,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn d(y: i32, m: u32, day: u32) -> NaiveDate {
        NaiveDate::from_ymd_opt(y, m, day).unwrap()
    }

    fn worker(fte: i32, role: Option<Uuid>) -> WorkerFacts {
        WorkerFacts {
            fte_percent: fte,
            hired_on: d(2020, 1, 1),
            terminated_on: None,
            role_profiles: role.into_iter().collect(),
            skills: vec![],
            approved_leave: vec![],
        }
    }

    fn demand(programme: &str, month: NaiveDate, fte: i64, status: DemandStatus) -> Demand {
        Demand {
            programme: programme.to_string(),
            month,
            fte_centi: fte,
            status,
        }
    }

    fn commit(partner: &str, month: NaiveDate, fte: i64, status: CommitmentStatus) -> Commitment {
        Commitment {
            partner: partner.to_string(),
            month,
            fte_centi: fte,
            status,
        }
    }

    #[test]
    fn months_are_first_of_month_and_bounded() {
        let m = months(d(2026, 1, 20), 3).unwrap();
        assert_eq!(m, vec![d(2026, 1, 1), d(2026, 2, 1), d(2026, 3, 1)]);
        assert!(months(d(2026, 1, 1), 0).is_err());
        assert!(months(d(2026, 1, 1), MAX_HORIZON_MONTHS + 1).is_err());
        assert_eq!(days_in_month(d(2026, 2, 1)), 28);
        assert_eq!(days_in_month(d(2028, 2, 1)), 29);
        assert!(validate_month(d(2026, 2, 1)).is_ok());
        assert!(validate_month(d(2026, 2, 2)).is_err());
    }

    #[test]
    fn validators_hold_their_boundaries() {
        assert!(validate_reservation_bp(0).is_ok());
        assert!(validate_reservation_bp(10_000).is_ok());
        assert!(validate_reservation_bp(-1).is_err());
        assert!(validate_reservation_bp(10_001).is_err());
        assert!(validate_fte_centi(1).is_ok());
        assert!(validate_fte_centi(0).is_err());
        assert!(validate_fte_centi(MAX_FTE_CENTI + 1).is_err());
        let role = Some(Uuid::nil());
        let skill = Some(Uuid::from_u128(1));
        assert!(validate_member(role, None, None).is_ok());
        assert!(validate_member(role, None, Some(3)).is_err());
        assert!(validate_member(None, skill, Some(1)).is_ok());
        assert!(validate_member(None, skill, Some(5)).is_ok());
        assert!(validate_member(None, skill, Some(0)).is_err());
        assert!(validate_member(None, skill, Some(6)).is_err());
        assert!(validate_member(None, skill, None).is_err());
        assert!(validate_member(role, skill, Some(3)).is_err());
        assert!(validate_member(None, None, None).is_err());
    }

    #[test]
    fn membership_by_role_or_by_skill_at_a_minimum() {
        let role = Uuid::from_u128(7);
        let skill = Uuid::from_u128(8);
        let members = [
            Member::RoleProfile(role),
            Member::Skill {
                skill,
                min_proficiency: 3,
            },
        ];
        assert!(counts_toward(&members, &worker(100, Some(role))));
        let mut by_skill = worker(100, None);
        by_skill.skills = vec![(skill, 3)];
        assert!(
            counts_toward(&members, &by_skill),
            "exactly the minimum counts"
        );
        by_skill.skills = vec![(skill, 2)];
        assert!(
            !counts_toward(&members, &by_skill),
            "below the minimum does not"
        );
        assert!(
            !counts_toward(&[], &worker(100, Some(role))),
            "no members, nobody"
        );
    }

    #[test]
    fn supply_weights_by_available_days_and_takes_the_reservation_off() {
        let role = Uuid::from_u128(1);
        let members = [Member::RoleProfile(role)];
        let jan = d(2026, 1, 1);
        // Two full-time and one half-time worker: 2.50 FTE.
        let staff = vec![
            worker(100, Some(role)),
            worker(100, Some(role)),
            worker(50, Some(role)),
            worker(100, None),
        ];
        assert_eq!(pool_supply_centi(&members, 0, &staff, jan), 250);
        // A 20% reservation keeps 80%: 2.00 FTE.
        assert_eq!(pool_supply_centi(&members, 2_000, &staff, jan), 200);
        // A 100% reservation leaves nothing, and is a known zero.
        assert_eq!(pool_supply_centi(&members, 10_000, &staff, jan), 0);
        // No members: a known zero.
        assert_eq!(pool_supply_centi(&[], 0, &staff, jan), 0);
    }

    #[test]
    fn supply_follows_hire_termination_and_approved_leave() {
        let role = Uuid::from_u128(1);
        let members = [Member::RoleProfile(role)];
        let jan = d(2026, 1, 1); // 31 days
        let mut starter = worker(100, Some(role));
        starter.hired_on = d(2026, 1, 17); // employed 17th..31st = 15 days
        assert_eq!(available_days(&starter, jan), 15);
        assert_eq!(pool_supply_centi(&members, 0, &[starter.clone()], jan), 48); // 15/31
        let mut leaver = worker(100, Some(role));
        leaver.terminated_on = Some(d(2026, 1, 11)); // last day is the 10th
        assert_eq!(available_days(&leaver, jan), 10);
        let mut away = worker(100, Some(role));
        away.approved_leave = vec![
            (d(2025, 12, 28), d(2026, 1, 9)),
            (d(2026, 1, 8), d(2026, 1, 12)),
        ];
        // Leave covers 1st..12th, even where the two ranges overlap: 19 days left.
        assert_eq!(available_days(&away, jan), 19);
        // Not yet hired: nothing.
        let mut future = worker(100, Some(role));
        future.hired_on = d(2026, 2, 1);
        assert_eq!(available_days(&future, jan), 0);
    }

    #[test]
    fn a_pool_with_no_partners_is_fits_or_over_never_unknown() {
        let months = months(d(2026, 1, 1), 3).unwrap();
        let demand = [
            demand("urn:p1", months[0], 200, DemandStatus::Active),
            demand("urn:p1", months[1], 300, DemandStatus::Active),
            demand("urn:p2", months[1], 999, DemandStatus::Proposed),
            demand("urn:p3", months[2], 999, DemandStatus::Closed),
        ];
        let view = build_pool(&months, &[250, 250, 250], &demand, &[]);
        assert_eq!(view.cells[0].status, CellStatus::Fits);
        assert_eq!(view.cells[1].status, CellStatus::Over);
        assert_eq!(view.cells[1].remaining_shortfall_centi, 50);
        assert_eq!(
            view.cells[1].proposed_centi, 999,
            "proposed is shown, not counted"
        );
        assert_eq!(view.cells[2].demand_centi, 0, "closed is ignored");
        assert_eq!(view.over_total_centi, 50);
        assert_eq!(view.over_months, 1);
        assert_eq!(view.unknown_months, 0);
    }

    #[test]
    fn an_exactly_met_demand_fits_and_one_centi_over_does_not() {
        let months = months(d(2026, 1, 1), 2).unwrap();
        let demand = [
            demand("urn:p", months[0], 250, DemandStatus::Active),
            demand("urn:p", months[1], 251, DemandStatus::Active),
        ];
        let view = build_pool(&months, &[250, 250], &demand, &[]);
        assert_eq!(view.cells[0].status, CellStatus::Fits);
        assert_eq!(view.cells[1].status, CellStatus::Over);
    }

    #[test]
    fn a_missing_partner_month_is_unknown_not_zero() {
        let months = months(d(2026, 1, 1), 3).unwrap();
        let demand = [
            demand("urn:p", months[0], 400, DemandStatus::Active),
            demand("urn:p", months[1], 400, DemandStatus::Active),
            demand("urn:p", months[2], 400, DemandStatus::Active),
        ];
        let commitments = [
            commit("urn:a", months[0], 150, CommitmentStatus::Confirmed),
            commit("urn:a", months[1], 100, CommitmentStatus::Requested),
            // No commitment from urn:a in the third month.
            commit("urn:b", months[0], 0, CommitmentStatus::Confirmed),
            commit("urn:b", months[1], 0, CommitmentStatus::Confirmed),
            commit("urn:b", months[2], 0, CommitmentStatus::Confirmed),
        ];
        let view = build_pool(&months, &[250, 250, 250], &demand, &commitments);
        // Month 1: short by 150, confirmed 150 covers it.
        assert_eq!(view.cells[0].status, CellStatus::FitsWithPartners);
        // Month 2: short by 150; a requested 100 does not cover it; everyone has answered.
        assert_eq!(view.cells[1].status, CellStatus::Over);
        assert_eq!(view.cells[1].remaining_shortfall_centi, 150);
        // Month 3: short by 150 and urn:a is silent: cannot be said, and not counted as over.
        assert_eq!(view.cells[2].status, CellStatus::Unknown);
        assert!(
            view.cells[2]
                .partners
                .iter()
                .any(|p| p.commitment.is_none())
        );
        assert_eq!(view.unknown_months, 1);
        assert_eq!(view.over_total_centi, 150, "unknown does not add to over");
    }

    #[test]
    fn a_silent_partner_does_not_matter_when_own_supply_fits() {
        let months = months(d(2026, 1, 1), 1).unwrap();
        let demand = [demand("urn:p", months[0], 100, DemandStatus::Active)];
        let commitments = [commit(
            "urn:a",
            d(2026, 6, 1),
            50,
            CommitmentStatus::Confirmed,
        )];
        let view = build_pool(&months, &[250], &demand, &commitments);
        assert_eq!(view.cells[0].status, CellStatus::Fits);
    }

    #[test]
    fn the_constraint_pool_is_the_most_over_and_ties_are_stable() {
        let months = months(d(2026, 1, 1), 1).unwrap();
        let over = |short: i64| {
            build_pool(
                &months,
                &[0],
                &[demand("urn:p", months[0], short, DemandStatus::Active)],
                &[],
            )
        };
        let fine = build_pool(&months, &[100], &[], &[]);
        let (a, b, c) = (over(50), over(80), over(80));
        assert_eq!(
            constraint_pool(&[("a", &a), ("b", &b), ("f", &fine)]),
            Some("b")
        );
        assert_eq!(
            constraint_pool(&[("z", &b), ("c", &c)]),
            Some("c"),
            "tie goes to the lowest key"
        );
        assert_eq!(constraint_pool(&[("f", &fine)]), None);
        assert_eq!(constraint_pool(&[]), None);
    }

    fn one_pool(supply: &[i64], demand_by_month: &[i64]) -> BTreeMap<String, PoolView> {
        let months = months(d(2026, 1, 1), u32::try_from(supply.len()).unwrap()).unwrap();
        let demand: Vec<Demand> = demand_by_month
            .iter()
            .enumerate()
            .map(|(i, f)| demand("urn:existing", months[i], *f, DemandStatus::Active))
            .collect();
        BTreeMap::from([(
            "pool".to_string(),
            build_pool(&months, supply, &demand, &[]),
        )])
    }

    fn wish(lines: &[(u32, i64)]) -> Candidate {
        Candidate {
            programme: "urn:new".to_string(),
            lines: lines
                .iter()
                .map(|(m, f)| ("pool".to_string(), d(2026, *m, 1), *f))
                .collect(),
        }
    }

    #[test]
    fn a_programme_that_fits_says_so() {
        let pools = one_pool(&[300, 300], &[100, 100]);
        let check = start_check(
            &wish(&[(1, 200), (2, 200)]),
            &pools,
            None,
            None,
            &BTreeSet::new(),
        );
        assert_eq!(check.fit, Fit::Yes);
        assert_eq!(check.earliest_shift_months, Some(0));
        assert!(check.lines.iter().all(|l| l.outcome == "fits"));
    }

    #[test]
    fn the_failing_months_and_the_earliest_fitting_delay() {
        // Free capacity: 100, 50, 200, 200 (supply 300 minus 200, 250, 100, 100).
        let pools = one_pool(&[300, 300, 300, 300], &[200, 250, 100, 100]);
        // The programme wants 150 in each of two consecutive months.
        let check = start_check(
            &wish(&[(1, 150), (2, 150)]),
            &pools,
            None,
            None,
            &BTreeSet::new(),
        );
        assert_eq!(check.fit, Fit::No);
        let short: Vec<(u32, i64)> = check
            .lines
            .iter()
            .filter(|l| l.outcome == "short")
            .map(|l| (l.month.month(), l.short_by_centi))
            .collect();
        assert_eq!(short, vec![(1, 50), (2, 100)]);
        // Two months later it lands on months 3 and 4, both with 200 free.
        assert_eq!(check.earliest_shift_months, Some(2));
    }

    #[test]
    fn a_programme_that_can_never_fit_has_no_start() {
        let pools = one_pool(&[100, 100, 100], &[0, 0, 0]);
        // It wants more than any month ever has.
        let check = start_check(&wish(&[(1, 500)]), &pools, None, None, &BTreeSet::new());
        assert_eq!(check.fit, Fit::No);
        assert_eq!(check.earliest_shift_months, None);
        assert!(check.derivation.contains("No delay"));
    }

    #[test]
    fn a_shift_past_the_horizon_is_not_a_fit() {
        // Only the first month is short; but the programme spans months 2 and 3 of a 3-month
        // horizon, and shifting by 1 would put month 3 outside it.
        let pools = one_pool(&[100, 100, 100], &[0, 90, 0]);
        let check = start_check(
            &wish(&[(2, 50), (3, 50)]),
            &pools,
            None,
            None,
            &BTreeSet::new(),
        );
        assert_eq!(check.fit, Fit::No);
        assert_eq!(
            check.earliest_shift_months, None,
            "later months are outside the horizon"
        );
    }

    #[test]
    fn an_unknown_partner_month_makes_the_answer_unknown_not_no() {
        let months = months(d(2026, 1, 1), 1).unwrap();
        let commitments = [commit(
            "urn:a",
            d(2026, 3, 1),
            10,
            CommitmentStatus::Confirmed,
        )];
        let view = build_pool(&months, &[100], &[], &commitments);
        let pools = BTreeMap::from([("pool".to_string(), view)]);
        let check = start_check(&wish(&[(1, 150)]), &pools, None, None, &BTreeSet::new());
        assert_eq!(check.fit, Fit::Unknown);
        assert_eq!(check.lines[0].outcome, "unknown_partner");
        assert_eq!(check.lines[0].short_by_centi, 0);
    }

    #[test]
    fn a_pool_the_view_does_not_know_is_outside_the_horizon() {
        let pools = one_pool(&[100], &[0]);
        let check = start_check(
            &Candidate {
                programme: "urn:new".to_string(),
                lines: vec![("other".to_string(), d(2026, 1, 1), 10)],
            },
            &pools,
            None,
            None,
            &BTreeSet::new(),
        );
        assert_eq!(check.fit, Fit::Unknown);
        assert_eq!(check.lines[0].outcome, "outside_horizon");
    }

    #[test]
    fn the_work_in_progress_limit_counts_the_candidate_once() {
        let pools = one_pool(&[1_000], &[0]);
        let active: BTreeSet<String> = ["urn:a", "urn:b"]
            .iter()
            .map(|s| (*s).to_string())
            .collect();
        let at_limit = start_check(&wish(&[(1, 10)]), &pools, Some(2), Some("pool"), &active);
        let wip = at_limit.wip.unwrap();
        assert_eq!((wip.active, wip.would_be, wip.exceeds), (2, 3, true));
        let room = start_check(&wish(&[(1, 10)]), &pools, Some(3), Some("pool"), &active);
        assert!(!room.wip.unwrap().exceeds);
        // A programme already active is not counted again.
        let mut already = active.clone();
        already.insert("urn:new".to_string());
        let again = start_check(&wish(&[(1, 10)]), &pools, Some(3), Some("pool"), &already);
        assert_eq!(again.wip.unwrap().would_be, 3);
        // A candidate that does not draw on the constraint pool adds nothing.
        let elsewhere = Candidate {
            programme: "urn:new".to_string(),
            lines: vec![("other".to_string(), d(2026, 1, 1), 10)],
        };
        let off = start_check(&elsewhere, &pools, Some(2), Some("pool"), &active);
        assert!(!off.wip.unwrap().exceeds);
        // No limit, or no constraint pool: no check.
        assert!(
            start_check(&wish(&[(1, 10)]), &pools, None, Some("pool"), &active)
                .wip
                .is_none()
        );
        assert!(
            start_check(&wish(&[(1, 10)]), &pools, Some(2), None, &active)
                .wip
                .is_none()
        );
    }

    #[test]
    fn programmes_drawing_counts_active_claims_only() {
        let m = d(2026, 1, 1);
        let set = programmes_drawing(&[
            demand("urn:a", m, 10, DemandStatus::Active),
            demand("urn:b", m, 10, DemandStatus::Proposed),
            demand("urn:c", m, 10, DemandStatus::Closed),
        ]);
        assert_eq!(set.len(), 1);
        assert!(set.contains("urn:a"));
    }

    #[test]
    fn status_tokens_round_trip() {
        for token in DemandStatus::ALL {
            assert!(DemandStatus::parse(token).is_some(), "{token}");
        }
        for token in CommitmentStatus::ALL {
            assert_eq!(CommitmentStatus::parse(token).unwrap().as_str(), token);
        }
        assert!(DemandStatus::parse("done").is_none());
        assert!(CommitmentStatus::parse("maybe").is_none());
        assert!(CommitmentStatus::Confirmed.is_firm());
        assert!(!CommitmentStatus::Requested.is_firm());
    }
}
