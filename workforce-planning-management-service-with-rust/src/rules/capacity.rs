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
///
/// Counted by interval arithmetic (the employed span within the month, less the merged leave spans),
/// not by walking the days: the view calls this for every worker and month (WPM-D90).
#[must_use]
pub fn available_days(worker: &WorkerFacts, month: NaiveDate) -> i64 {
    use chrono::Datelike;
    let first = i64::from(month_start(month).num_days_from_ce());
    let last = first + days_in_month(month_start(month)) - 1;
    // employed: hired_on <= day and (not terminated, or terminated_on > day)
    let from = first.max(i64::from(worker.hired_on.num_days_from_ce()));
    let to = worker
        .terminated_on
        .map_or(last, |t| last.min(i64::from(t.num_days_from_ce()) - 1));
    if from > to {
        return 0;
    }
    let mut leave: Vec<(i64, i64)> = worker
        .approved_leave
        .iter()
        .map(|(s, e)| {
            (
                i64::from(s.num_days_from_ce()).max(from),
                i64::from(e.num_days_from_ce()).min(to),
            )
        })
        .filter(|(s, e)| s <= e)
        .collect();
    leave.sort_unstable();
    let mut away = 0;
    let mut covered_to = from - 1;
    for (s, e) in leave {
        let s = s.max(covered_to + 1);
        if s <= e {
            away += e - s + 1;
            covered_to = e;
        }
    }
    to - from + 1 - away
}

/// The same count, one day at a time: the definition [`available_days`] must equal.
#[cfg(test)]
fn available_days_by_walking(worker: &WorkerFacts, month: NaiveDate) -> i64 {
    let first = month_start(month);
    let mut count = 0;
    for offset in 0..days_in_month(first) {
        let day = first
            .checked_add_days(chrono::Days::new(u64::try_from(offset).unwrap_or(0)))
            .unwrap();
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
    supply_from_weighted(
        pool_weighted_days(members, workers, month),
        reservation_bp,
        month,
    )
}

/// The part of a pool's supply that adds up across workers: FTE percent times available days, for
/// those who count toward the pool. Workers can be summed in batches and the batches added, so an
/// organization need not be held in memory whole (WPM-D90); [`supply_from_weighted`] finishes it.
#[must_use]
pub fn pool_weighted_days(members: &[Member], workers: &[WorkerFacts], month: NaiveDate) -> i64 {
    workers
        .iter()
        .filter(|w| counts_toward(members, w))
        .map(|w| i64::from(w.fte_percent.max(0)) * available_days(w, month))
        .sum()
}

/// A pool's supply in centi-FTE from the summed [`pool_weighted_days`]: less the operations
/// reservation, rounded half up. The rounding happens once, on the total.
#[must_use]
pub fn supply_from_weighted(weighted: i64, reservation_bp: i32, month: NaiveDate) -> i64 {
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
    /// Whether the pool's demand is known. A linked pool its peer has sent nothing to is not (WPM-D141), and stays unknown whatever a
    /// what-if adds to it.
    pub demand_known: bool,
}

/// A linked pool whose peer has sent no demand at all: its demand is **not known**, not zero (WPM-D141), so no month can be said to fit or
/// to be over. Every cell becomes `Unknown`; the figures stay as they are.
#[must_use]
pub fn with_demand_unknown(mut view: PoolView) -> PoolView {
    for cell in &mut view.cells {
        cell.status = CellStatus::Unknown;
        cell.remaining_shortfall_centi = 0;
    }
    view.over_total_centi = 0;
    view.over_months = 0;
    view.unknown_months = view.cells.len();
    view.demand_known = false;
    view
}

/// What a pool-month comes to, from its balance (`supply - active demand`), the firm partner capacity and whether a partner has not
/// answered. Also the shortfall left after firm partner capacity. The one place the rule is written, used by the view and the what-if.
fn classify(balance: i64, firm: i64, any_unknown: bool) -> (CellStatus, i64) {
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
    (status, remaining)
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
        let (status, remaining) = classify(balance, firm, any_unknown);
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
        demand_known: true,
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

// ─── What-if ────────────────────────────────────────────────────────────────

/// A proposed change to one programme's **active** claim on one pool in one month. Positive asks for more FTE, negative gives some back.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Change {
    /// The pool's key.
    pub pool: String,
    /// The month (its first day).
    pub month: NaiveDate,
    /// The change, in centi-FTE.
    pub fte_centi: i64,
}

/// A pool-month the proposal touches, before and after.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct CellChange {
    /// The pool's key.
    pub pool: String,
    /// The month.
    pub month: NaiveDate,
    /// Status now.
    pub before: CellStatus,
    /// Status with the proposal.
    pub after: CellStatus,
    /// `supply - demand` now, in centi-FTE.
    pub before_balance_centi: i64,
    /// `supply - demand` with the proposal.
    pub after_balance_centi: i64,
}

/// The overall answer.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Answer {
    /// Nothing the proposal touches is over-committed or made worse.
    Fits,
    /// At least one pool-month becomes over-committed or more so.
    Breaks,
    /// A pool-month it touches cannot be said, because a partner has made no commitment.
    Unknown,
}

impl Answer {
    /// The token.
    #[must_use]
    pub const fn as_str(self) -> &'static str {
        match self {
            Self::Fits => "fits",
            Self::Breaks => "breaks",
            Self::Unknown => "unknown",
        }
    }
}

/// What a proposal does to the plan. Computed from the current view and **stored nowhere**.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct WhatIf {
    /// The overall answer.
    pub answer: Answer,
    /// Every pool-month the proposal touches, by pool then month.
    pub cells: Vec<CellChange>,
    /// Pool-months that were not over-committed and become so.
    pub becomes_over: usize,
    /// Pool-months that were over-committed and stop being so.
    pub stops_over: usize,
    /// Touched pool-months that cannot be said with the proposal.
    pub unknown: usize,
    /// The constraint pool now.
    pub constraint_before: Option<String>,
    /// The constraint pool with the proposal.
    pub constraint_after: Option<String>,
    /// The fewest months to put the whole proposal back for it to fit, `0` if it fits as asked; `None` if no start inside the horizon fits.
    pub earliest_shift_months: Option<u32>,
}

/// Problems with a proposal: some changes, each to a pool and a month in the view, none of more than [`MAX_FTE_CENTI`].
#[must_use]
pub fn change_problems(
    views: &BTreeMap<String, PoolView>,
    months: &[NaiveDate],
    changes: &[Change],
) -> Vec<String> {
    let mut out = Vec::new();
    if changes.is_empty() {
        out.push("propose at least one change".to_string());
    }
    for (i, c) in changes.iter().enumerate() {
        let n = i + 1;
        if !views.contains_key(&c.pool) {
            out.push(format!("change {n}: no such pool"));
        }
        if !months.contains(&c.month) {
            out.push(format!(
                "change {n}: the month is outside the view (start each month on its first day)"
            ));
        }
        if c.fte_centi == 0 || c.fte_centi.abs() > MAX_FTE_CENTI {
            out.push(format!(
                "change {n}: the change must be non-zero and within {MAX_FTE_CENTI} centi-FTE"
            ));
        }
    }
    out
}

/// The view with each month's active demand moved by `moves` (month, centi-FTE), never below zero.
fn moved(view: &PoolView, moves: &BTreeMap<NaiveDate, i64>) -> PoolView {
    let mut cells = Vec::with_capacity(view.cells.len());
    let (mut over_total, mut over_months, mut unknown_months) = (0, 0, 0);
    for cell in &view.cells {
        let active = (cell.demand_centi + moves.get(&cell.month).copied().unwrap_or(0)).max(0);
        let balance = cell.supply_centi - active;
        let any_unknown = cell.partners.iter().any(|p| p.commitment.is_none());
        let (status, remaining) = if view.demand_known {
            classify(balance, cell.firm_partner_centi, any_unknown)
        } else {
            (CellStatus::Unknown, 0)
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
            demand_centi: active,
            own_balance_centi: balance,
            remaining_shortfall_centi: remaining,
            status,
            ..cell.clone()
        });
    }
    PoolView {
        cells,
        over_total_centi: over_total,
        over_months,
        unknown_months,
        demand_known: view.demand_known,
    }
}

fn pools_of(views: &BTreeMap<String, PoolView>) -> Vec<(&str, &PoolView)> {
    views.iter().map(|(k, v)| (k.as_str(), v)).collect()
}

/// Apply a proposal to the current views and say what changes. `programme_active` is the programme's own active claim by
/// `(pool, month)`: a change cannot take back more than the programme holds. `shift` moves every change later by that many months
/// (used to find the earliest start that fits); a change moved past the last month of the view is dropped and the answer is then `None`
/// for it in [`what_if`].
/// A proposal applied: the views with it, and the `(pool, month)` cells it touches.
type Applied = (BTreeMap<String, PoolView>, Vec<(String, NaiveDate)>);

fn apply(
    views: &BTreeMap<String, PoolView>,
    programme_active: &BTreeMap<(String, NaiveDate), i64>,
    changes: &[Change],
    shift: u32,
) -> Option<Applied> {
    let mut by_pool: BTreeMap<&str, BTreeMap<NaiveDate, i64>> = BTreeMap::new();
    let mut touched = Vec::new();
    for c in changes {
        let month = shift_month(c.month, shift)?;
        let have = programme_active
            .get(&(c.pool.clone(), c.month))
            .copied()
            .unwrap_or(0);
        let delta = c.fte_centi.max(-have);
        *by_pool
            .entry(&c.pool)
            .or_default()
            .entry(month)
            .or_insert(0) += delta;
        touched.push((c.pool.clone(), month));
    }
    touched.sort();
    touched.dedup();
    let after = views
        .iter()
        .map(|(k, v)| {
            let view = by_pool
                .get(k.as_str())
                .map_or_else(|| v.clone(), |m| moved(v, m));
            (k.clone(), view)
        })
        .collect();
    Some((after, touched))
}

fn cell_at<'a>(
    views: &'a BTreeMap<String, PoolView>,
    pool: &str,
    month: NaiveDate,
) -> Option<&'a Cell> {
    views.get(pool)?.cells.iter().find(|c| c.month == month)
}

/// How a shifted or unshifted proposal comes out: breaks if a touched cell becomes over-committed or more so; otherwise unknown if a
/// touched cell cannot be said; otherwise fits.
fn judge(
    before: &BTreeMap<String, PoolView>,
    after: &BTreeMap<String, PoolView>,
    touched: &[(String, NaiveDate)],
) -> Answer {
    let mut unknown = false;
    for (pool, month) in touched {
        let (Some(b), Some(a)) = (cell_at(before, pool, *month), cell_at(after, pool, *month))
        else {
            continue;
        };
        let worse = a.status == CellStatus::Over
            && (b.status != CellStatus::Over
                || a.remaining_shortfall_centi > b.remaining_shortfall_centi);
        if worse {
            return Answer::Breaks;
        }
        if a.status == CellStatus::Unknown && a.demand_centi > b.demand_centi {
            unknown = true;
        }
    }
    if unknown {
        Answer::Unknown
    } else {
        Answer::Fits
    }
}

/// What a proposal does: which pool-months become over-committed or stop being so, the constraint pool before and after, and the
/// earliest start that fits. Reads the current views; changes nothing.
#[must_use]
pub fn what_if(
    views: &BTreeMap<String, PoolView>,
    programme_active: &BTreeMap<(String, NaiveDate), i64>,
    changes: &[Change],
) -> WhatIf {
    let (after, touched) =
        apply(views, programme_active, changes, 0).unwrap_or_else(|| (views.clone(), Vec::new()));
    let mut cells = Vec::new();
    let (mut becomes_over, mut stops_over, mut unknown) = (0, 0, 0);
    for (pool, month) in &touched {
        let (Some(b), Some(a)) = (cell_at(views, pool, *month), cell_at(&after, pool, *month))
        else {
            continue;
        };
        if a.status == CellStatus::Over && b.status != CellStatus::Over {
            becomes_over += 1;
        }
        if b.status == CellStatus::Over && a.status != CellStatus::Over {
            stops_over += 1;
        }
        if a.status == CellStatus::Unknown {
            unknown += 1;
        }
        cells.push(CellChange {
            pool: pool.clone(),
            month: *month,
            before: b.status,
            after: a.status,
            before_balance_centi: b.own_balance_centi,
            after_balance_centi: a.own_balance_centi,
        });
    }
    let answer = judge(views, &after, &touched);
    let max_shift = views
        .values()
        .next()
        .map_or(0, |v| u32::try_from(v.cells.len()).unwrap_or(0));
    let earliest_shift_months = (0..max_shift).find(|k| {
        apply(views, programme_active, changes, *k)
            .is_some_and(|(shifted, t)| judge(views, &shifted, &t) == Answer::Fits)
    });
    WhatIf {
        answer,
        cells,
        becomes_over,
        stops_over,
        unknown,
        constraint_before: constraint_pool(&pools_of(views)).map(ToString::to_string),
        constraint_after: constraint_pool(&pools_of(&after)).map(ToString::to_string),
        earliest_shift_months,
    }
}

/// Where a demand claim came from (WPM-D140): entered here, or received from the peer.
pub const DEMAND_SOURCES: [&str; 2] = ["local", "ppm"];

/// Which source of demand a pool counts. **A linked pool counts the peer's claims and an unlinked pool its own; never both** (WPM-D140),
/// so removing the link returns the pool to its local figures at once and two numbers for one pool are never added.
#[must_use]
pub const fn counted_source(linked: bool) -> &'static str {
    if linked { "ppm" } else { "local" }
}

/// Whether text is a usable id for the peer's pool: a UUID in its hyphenated lower-case form, as the column requires.
#[must_use]
pub fn peer_pool_ref_ok(text: &str) -> bool {
    text.len() == 36 && Uuid::parse_str(text).is_ok_and(|u| u.hyphenated().to_string() == text)
}

/// The supply a linked pool gives its peer, month by month: the figure, or `None` where fewer than `floor` workers make it up (small-cell
/// suppression, [`crate::rules::suppression`]), because the FTE of one or two people says who they are. A month with no worker is a real
/// zero and is shown. No total is given.
#[must_use]
pub fn peer_supply(supply_centi: &[i64], workers: &[usize], floor: usize) -> Vec<Option<i64>> {
    supply_centi
        .iter()
        .zip(workers)
        .map(|(centi, n)| crate::rules::suppression::visible(*n, floor).then_some(*centi))
        .collect()
}

/// Whether a contingent engagement that ends on `ends_on` would end before a programme's proposed new end. An engagement already over
/// (`ends_on` before `today`) is not an effect of the proposal; one ending on the new end date itself covers it.
#[must_use]
pub fn ends_before_programme(ends_on: NaiveDate, today: NaiveDate, new_end: NaiveDate) -> bool {
    ends_on >= today && ends_on < new_end
}

/// How many fixed-term and contractor engagements in each pool end before the programme's proposed new end. A count under the floor is
/// `None` (small-cell suppression, [`crate::rules::suppression`]); zero is shown. There is **no total**: a worker can hold more than one
/// pool's skills, so a total would count them twice, and a sum of shown cells would be wrong rather than safe.
#[must_use]
pub fn ending_counts(
    per_pool: &[(String, Vec<NaiveDate>)],
    today: NaiveDate,
    new_end: NaiveDate,
    floor: usize,
) -> Vec<(String, Option<usize>)> {
    per_pool
        .iter()
        .map(|(pool, ends)| {
            let n = ends
                .iter()
                .filter(|e| ends_before_programme(**e, today, new_end))
                .count();
            (
                pool.clone(),
                crate::rules::suppression::visible(n, floor).then_some(n),
            )
        })
        .collect()
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

    #[test]
    fn supply_summed_in_batches_equals_supply_of_the_whole() {
        let jan = NaiveDate::from_ymd_opt(2026, 1, 1).unwrap();
        let role = Uuid::from_u128(1);
        let mut staff = Vec::new();
        for i in 0..7 {
            staff.push(WorkerFacts {
                fte_percent: 40 + 10 * i,
                hired_on: NaiveDate::from_ymd_opt(2025, 1, 1).unwrap(),
                terminated_on: None,
                role_profiles: vec![role],
                skills: Vec::new(),
                approved_leave: Vec::new(),
            });
        }
        let members = [Member::RoleProfile(role)];
        for reservation in [0, 1_234, 10_000] {
            let whole = pool_supply_centi(&members, reservation, &staff, jan);
            let weighted: i64 = staff
                .chunks(3)
                .map(|batch| pool_weighted_days(&members, batch, jan))
                .sum();
            assert_eq!(supply_from_weighted(weighted, reservation, jan), whole);
        }
    }

    #[test]
    fn counting_by_intervals_equals_walking_the_days() {
        // a small deterministic generator: every combination of hire, end and overlapping leave
        let base = NaiveDate::from_ymd_opt(2026, 1, 20).unwrap();
        let at = |n: i64| base + chrono::Days::new(u64::try_from(n.rem_euclid(120)).unwrap());
        let mut state = 12_345_u64;
        let mut next = |m: i64| {
            state = state
                .wrapping_mul(6_364_136_223_846_793_005)
                .wrapping_add(1);
            i64::try_from(state >> 33).unwrap() % m
        };
        for _ in 0..4_000 {
            let hired_on = at(next(90) - 20);
            let terminated_on = (next(3) > 0).then(|| at(next(110)));
            let approved_leave = (0..next(4))
                .map(|_| {
                    let s = at(next(100));
                    (s, s + chrono::Days::new(u64::try_from(next(20)).unwrap()))
                })
                .collect();
            let worker = WorkerFacts {
                fte_percent: 100,
                hired_on,
                terminated_on,
                role_profiles: Vec::new(),
                skills: Vec::new(),
                approved_leave,
            };
            for month in [(2026, 1), (2026, 2), (2026, 3), (2026, 4)] {
                let month = NaiveDate::from_ymd_opt(month.0, month.1, 1).unwrap();
                assert_eq!(
                    available_days(&worker, month),
                    available_days_by_walking(&worker, month),
                    "{worker:?} in {month}"
                );
            }
        }
    }

    // ─── What-if (WPM-T158) ─────────────────────────────────────────────────

    fn wi_m(n: u32) -> NaiveDate {
        NaiveDate::from_ymd_opt(2027, n, 1).unwrap()
    }

    fn wi_six() -> Vec<NaiveDate> {
        (1..=6).map(wi_m).collect()
    }

    fn wi_demand(programme: &str, month: u32, fte: i64) -> Demand {
        Demand {
            programme: programme.to_string(),
            month: wi_m(month),
            fte_centi: fte,
            status: DemandStatus::Active,
        }
    }

    fn wi_pool(
        supply: i64,
        demands: &[Demand],
        commitments: &[Commitment],
    ) -> BTreeMap<String, PoolView> {
        let months = wi_six();
        let mut views = BTreeMap::new();
        views.insert(
            "nurses".to_string(),
            build_pool(&months, &[supply; 6], demands, commitments),
        );
        views
    }

    fn wi_change(month: u32, fte: i64) -> Change {
        Change {
            pool: "nurses".to_string(),
            month: wi_m(month),
            fte_centi: fte,
        }
    }

    fn wi_none() -> BTreeMap<(String, NaiveDate), i64> {
        BTreeMap::new()
    }

    #[test]
    fn a_change_that_fits_changes_no_status() {
        let views = wi_pool(500, &[wi_demand("programme:a", 1, 300)], &[]);
        let w = what_if(&views, &wi_none(), &[wi_change(1, 100)]);
        assert_eq!(w.answer, Answer::Fits);
        assert_eq!((w.becomes_over, w.stops_over, w.unknown), (0, 0, 0));
        assert_eq!(w.cells.len(), 1);
        assert_eq!(
            (
                w.cells[0].before_balance_centi,
                w.cells[0].after_balance_centi
            ),
            (200, 100)
        );
        assert_eq!(w.earliest_shift_months, Some(0));
        assert_eq!((w.constraint_before, w.constraint_after), (None, None));
    }

    #[test]
    fn a_change_that_breaks_a_pool_month_says_so_and_names_the_constraint_pool() {
        let views = wi_pool(500, &[wi_demand("programme:a", 1, 300)], &[]);
        let w = what_if(&views, &wi_none(), &[wi_change(1, 300)]);
        assert_eq!(w.answer, Answer::Breaks);
        assert_eq!(w.becomes_over, 1);
        assert_eq!(
            (w.cells[0].before, w.cells[0].after),
            (CellStatus::Fits, CellStatus::Over)
        );
        assert_eq!(
            (w.constraint_before, w.constraint_after),
            (None, Some("nurses".to_string()))
        );
    }

    #[test]
    fn the_earliest_start_that_fits_is_found_by_moving_the_whole_proposal_later() {
        // January and February are busy; March onward is free.
        let busy = [
            wi_demand("programme:a", 1, 450),
            wi_demand("programme:a", 2, 450),
        ];
        let views = wi_pool(500, &busy, &[]);
        let w = what_if(&views, &wi_none(), &[wi_change(1, 200)]);
        assert_eq!(w.answer, Answer::Breaks);
        assert_eq!(
            w.earliest_shift_months,
            Some(2),
            "March is the first month it fits"
        );
        let never = [
            wi_demand("programme:a", 1, 450),
            wi_demand("programme:a", 2, 450),
            wi_demand("programme:a", 3, 450),
            wi_demand("programme:a", 4, 450),
            wi_demand("programme:a", 5, 450),
            wi_demand("programme:a", 6, 450),
        ];
        let w = what_if(&wi_pool(500, &never, &[]), &wi_none(), &[wi_change(1, 200)]);
        assert_eq!(
            w.earliest_shift_months, None,
            "no month in the view has room"
        );
    }

    #[test]
    fn a_change_in_a_month_with_an_unanswered_partner_cannot_be_said() {
        let partner = Commitment {
            partner: "organization:p".to_string(),
            month: wi_m(2),
            fte_centi: 0,
            status: CommitmentStatus::Requested,
        };
        // January has the partner's answer (none, but the partner is known from February), so January is unknown when short.
        let views = wi_pool(100, &[], &[partner]);
        let w = what_if(&views, &wi_none(), &[wi_change(1, 300)]);
        assert_eq!(
            w.answer,
            Answer::Unknown,
            "short, and a partner has not answered"
        );
        assert_eq!(w.unknown, 1);
        assert_eq!(w.cells[0].after, CellStatus::Unknown);
        assert_eq!(w.becomes_over, 0, "unknown is not over");
    }

    #[test]
    fn giving_capacity_back_can_stop_a_month_being_over() {
        let views = wi_pool(
            500,
            &[
                wi_demand("programme:a", 1, 400),
                wi_demand("programme:b", 1, 300),
            ],
            &[],
        );
        assert_eq!(views["nurses"].cells[0].status, CellStatus::Over);
        let mut held = BTreeMap::new();
        held.insert(("nurses".to_string(), wi_m(1)), 300);
        let w = what_if(&views, &held, &[wi_change(1, -300)]);
        assert_eq!(w.stops_over, 1);
        assert_eq!(w.answer, Answer::Fits);
        assert_eq!(
            (w.constraint_before, w.constraint_after),
            (Some("nurses".to_string()), None)
        );
    }

    #[test]
    fn a_programme_cannot_give_back_more_than_it_holds() {
        let views = wi_pool(500, &[wi_demand("programme:a", 1, 200)], &[]);
        let mut held = BTreeMap::new();
        held.insert(("nurses".to_string(), wi_m(1)), 200);
        let w = what_if(&views, &held, &[wi_change(1, -900)]);
        assert_eq!(
            w.cells[0].after_balance_centi, 500,
            "the programme's 200 is taken back, not 900"
        );
    }

    #[test]
    fn a_change_that_does_not_make_an_existing_shortfall_worse_is_not_a_break() {
        let views = wi_pool(500, &[wi_demand("programme:a", 1, 600)], &[]);
        let mut held = BTreeMap::new();
        held.insert(("nurses".to_string(), wi_m(1)), 600);
        let w = what_if(&views, &held, &[wi_change(1, -50)]);
        assert_eq!(w.answer, Answer::Fits, "100 short became 50 short");
        assert_eq!(w.becomes_over, 0);
        assert_eq!(
            what_if(&views, &wi_none(), &[wi_change(1, 50)]).answer,
            Answer::Breaks,
            "and 50 more is worse"
        );
    }

    #[test]
    fn a_proposal_is_checked_before_it_is_run() {
        let views = wi_pool(500, &[], &[]);
        let months = wi_six();
        assert_eq!(
            change_problems(&views, &months, &[wi_change(1, 100)]),
            Vec::<String>::new()
        );
        assert_eq!(change_problems(&views, &months, &[]).len(), 1);
        let mut bad = wi_change(1, 100);
        bad.pool = "other".to_string();
        assert_eq!(change_problems(&views, &months, &[bad]).len(), 1);
        assert_eq!(
            change_problems(
                &views,
                &months,
                &[Change {
                    pool: "nurses".into(),
                    month: NaiveDate::from_ymd_opt(2027, 1, 15).unwrap(),
                    fte_centi: 10
                }]
            )
            .len(),
            1,
            "not the first of a month"
        );
        assert_eq!(
            change_problems(&views, &months, &[wi_change(1, 0)]).len(),
            1
        );
        assert_eq!(
            change_problems(&views, &months, &[wi_change(1, MAX_FTE_CENTI + 1)]).len(),
            1
        );
    }

    #[test]
    fn the_current_view_is_not_changed_by_asking() {
        let views = wi_pool(500, &[wi_demand("programme:a", 1, 300)], &[]);
        let copy = views.clone();
        let _ = what_if(&views, &wi_none(), &[wi_change(1, 900)]);
        assert_eq!(views, copy);
    }

    #[test]
    fn the_view_and_the_what_if_classify_a_cell_the_same_way() {
        for (supply, active, firm, unknown) in [
            (500, 300, 0, false),
            (100, 300, 0, false),
            (100, 300, 150, false),
            (100, 300, 50, true),
            (100, 300, 50, false),
        ] {
            let (status, remaining) = classify(supply - active, firm, unknown);
            let partners = if unknown {
                vec![PartnerCell {
                    partner: "p".into(),
                    commitment: None,
                }]
            } else {
                vec![]
            };
            let cell = Cell {
                month: wi_m(1),
                supply_centi: supply,
                demand_centi: active,
                proposed_centi: 0,
                own_balance_centi: supply - active,
                firm_partner_centi: firm,
                remaining_shortfall_centi: remaining,
                partners,
                status,
            };
            let view = PoolView {
                cells: vec![cell.clone()],
                over_total_centi: 0,
                over_months: 0,
                unknown_months: 0,
                demand_known: true,
            };
            let again = moved(&view, &BTreeMap::new());
            assert_eq!(again.cells[0].status, status);
            assert_eq!(again.cells[0].remaining_shortfall_centi, remaining);
        }
    }

    #[test]
    fn the_programme_floor_is_its_own_claim_and_not_the_cell_total() {
        // Another programme holds 300 of the same month; this one holds 200 and offers to give back 900.
        let views = wi_pool(
            500,
            &[
                wi_demand("programme:a", 1, 200),
                wi_demand("programme:b", 1, 300),
            ],
            &[],
        );
        let mut held = BTreeMap::new();
        held.insert(("nurses".to_string(), wi_m(1)), 200);
        let w = what_if(&views, &held, &[wi_change(1, -900)]);
        assert_eq!(
            (
                w.cells[0].before_balance_centi,
                w.cells[0].after_balance_centi
            ),
            (0, 200),
            "only the programme's own 200 comes back; the other programme's 300 stays"
        );
    }

    #[test]
    fn an_unknown_month_is_not_a_fit_so_the_search_moves_on_to_a_month_that_is() {
        // Short supply in January to March with a partner who has not answered; plenty from April.
        let partner = Commitment {
            partner: "organization:p".to_string(),
            month: wi_m(6),
            fte_centi: 0,
            status: CommitmentStatus::Requested,
        };
        let months = wi_six();
        let mut views = BTreeMap::new();
        views.insert(
            "nurses".to_string(),
            build_pool(&months, &[100, 100, 100, 900, 900, 900], &[], &[partner]),
        );
        let w = what_if(&views, &wi_none(), &[wi_change(1, 300)]);
        assert_eq!(w.answer, Answer::Unknown);
        assert_eq!(
            w.earliest_shift_months,
            Some(3),
            "April is the first month that can be said to fit"
        );
    }

    #[test]
    fn an_engagement_ends_before_the_new_end_only_if_it_is_running_now_and_ends_earlier() {
        let today = d(2026, 10, 1);
        let end = d(2027, 3, 31);
        assert!(ends_before_programme(d(2027, 1, 31), today, end));
        assert!(
            ends_before_programme(today, today, end),
            "ending today still counts"
        );
        assert!(
            !ends_before_programme(d(2026, 9, 30), today, end),
            "already over"
        );
        assert!(
            !ends_before_programme(end, today, end),
            "ending on the last day covers it"
        );
        assert!(!ends_before_programme(d(2027, 6, 30), today, end));
    }

    #[test]
    fn ending_counts_hide_a_small_count_show_zero_and_give_no_total() {
        let today = d(2026, 10, 1);
        let end = d(2027, 3, 31);
        let early = d(2027, 1, 1);
        let late = d(2027, 12, 31);
        let pools = vec![
            ("a".to_string(), vec![early; 5]),
            ("b".to_string(), vec![early; 4]),
            ("c".to_string(), vec![late; 9]),
            ("d".to_string(), vec![early, early, late]),
        ];
        assert_eq!(
            ending_counts(&pools, today, end, 5),
            vec![
                ("a".to_string(), Some(5)),
                ("b".to_string(), None),
                ("c".to_string(), Some(0)),
                ("d".to_string(), None),
            ]
        );
    }

    #[test]
    fn a_linked_pool_counts_the_peers_claims_and_an_unlinked_one_its_own() {
        assert_eq!(counted_source(true), "ppm");
        assert_eq!(counted_source(false), "local");
        assert_ne!(counted_source(true), counted_source(false));
        assert!(
            DEMAND_SOURCES.contains(&counted_source(true))
                && DEMAND_SOURCES.contains(&counted_source(false))
        );
    }

    #[test]
    fn a_peer_pool_ref_is_a_lower_case_hyphenated_uuid_and_nothing_else() {
        assert!(peer_pool_ref_ok("123e4567-e89b-42d3-a456-426614174000"));
        for bad in [
            "",
            "pool one",
            "123E4567-E89B-42D3-A456-426614174000",
            "123e4567e89b42d3a456426614174000",
            "thing:123e4567-e89b-42d3-a456-426614174000",
            "123e4567-e89b-42d3-a456-42661417400",
        ] {
            assert!(!peer_pool_ref_ok(bad), "{bad}");
        }
    }

    #[test]
    fn supply_for_a_peer_is_hidden_where_too_few_workers_make_it_up() {
        assert_eq!(
            peer_supply(&[500, 400, 0, 100], &[5, 4, 0, 1], 5),
            vec![Some(500), None, Some(0), None]
        );
        assert_eq!(
            peer_supply(&[500], &[5], 0),
            vec![Some(500)],
            "a floor of 0 reads as 1"
        );
        assert_eq!(
            peer_supply(&[500, 400], &[7], 5),
            vec![Some(500)],
            "a month with no worker count is not invented"
        );
    }

    #[test]
    fn a_pool_whose_peer_has_sent_no_demand_is_unknown_in_every_month_not_fitting() {
        let months = [d(2027, 1, 1), d(2027, 2, 1)];
        let view = build_pool(&months, &[100, 100], &[], &[]);
        assert!(view.cells.iter().all(|c| c.status == CellStatus::Fits));
        let unknown = with_demand_unknown(view);
        assert!(
            unknown
                .cells
                .iter()
                .all(|c| c.status == CellStatus::Unknown)
        );
        assert_eq!(
            (
                unknown.unknown_months,
                unknown.over_months,
                unknown.over_total_centi
            ),
            (2, 0, 0)
        );
        assert_eq!(
            unknown.cells[0].supply_centi, 100,
            "the supply is still known"
        );
        assert!(!unknown.demand_known);
        let after = moved(&unknown, &BTreeMap::from([(months[0], 50)]));
        assert_eq!(
            after.cells[0].status,
            CellStatus::Unknown,
            "a what-if cannot make unknown demand fit"
        );
        let over = with_demand_unknown(build_pool(
            &months,
            &[0, 0],
            &[demand("thing:x", months[0], 50, DemandStatus::Active)],
            &[],
        ));
        assert_eq!(
            over.over_months, 0,
            "an overcommitted month is not claimed when demand is unknown"
        );
    }
}
