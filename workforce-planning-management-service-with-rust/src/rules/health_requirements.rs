//! **Workplace health requirements** (WPM-R128, WPM-D75): pure rules for recording whether a
//! worker meets a requirement their area sets, such as a required immunization.
//!
//! This is the narrow exception to "WPM holds no health data", and it is bounded:
//!
//! - **The status, never the reason.** A record answers "is this person cleared for this
//!   requirement?" and nothing about their health beyond it: no diagnosis, no product or batch, no
//!   reason for an exemption (an exemption is *recorded as exempt* by occupational health; the reason
//!   stays in their own system).
//! - **Off until a deployer records the lawful basis they rely on.**
//! - **Written only by an occupational-health role**, not by HR, a manager or the worker.
//! - A manager or HR sees only **cleared** or **not cleared**, never the status; the worker sees
//!   their own; occupational health sees the detail.
//! - **Never an input to any score or decision about a person** apart from that clearance, which
//!   the deployer's own rule for the area applies.

use chrono::{Days, NaiveDate};
use std::collections::BTreeMap;

/// The default calendar days before a due date at which a reminder shows.
pub const DEFAULT_REMINDER_CALENDAR_DAYS: u32 = 60;
/// The longest a re-check period or a reminder window may be, in calendar days.
pub const MAX_CALENDAR_DAYS: u32 = 3_660;

/// What occupational health records. A status, never a reason.
pub const STATUSES: &[&str] = &["up_to_date", "exempt_recorded", "declined"];

/// Where a worker stands against one requirement on a day.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Standing {
    /// Nothing recorded.
    NotRecorded,
    /// Recorded and valid.
    UpToDate,
    /// Recorded and valid, but due again within the reminder window.
    DueSoon,
    /// Recorded, and past its due date.
    Overdue,
    /// Occupational health recorded an exemption (the reason is not held here).
    ExemptRecorded,
    /// Recorded as declined.
    Declined,
}

impl Standing {
    /// The token.
    #[must_use]
    pub const fn as_str(self) -> &'static str {
        match self {
            Self::NotRecorded => "not_recorded",
            Self::UpToDate => "up_to_date",
            Self::DueSoon => "due_soon",
            Self::Overdue => "overdue",
            Self::ExemptRecorded => "exempt_recorded",
            Self::Declined => "declined",
        }
    }
}

/// Where a worker stands. `status` is the recorded token, `next_due` the date it falls due again.
#[must_use]
pub fn standing(
    status: Option<&str>,
    next_due: Option<NaiveDate>,
    today: NaiveDate,
    reminder_calendar_days: u32,
) -> Standing {
    let Some(status) = status else {
        return Standing::NotRecorded;
    };
    if status == "declined" {
        return Standing::Declined;
    }
    let exempt = status == "exempt_recorded";
    match next_due {
        Some(due) if today > due => Standing::Overdue,
        Some(due)
            if !exempt
                && due
                    .checked_sub_days(Days::new(u64::from(reminder_calendar_days)))
                    .is_some_and(|start| today >= start) =>
        {
            Standing::DueSoon
        }
        _ if exempt => Standing::ExemptRecorded,
        _ => Standing::UpToDate,
    }
}

/// Whether a manager or HR is told "cleared": the requirement is met or occupational health has
/// recorded an exemption. A declined, overdue or unrecorded one is **not cleared**.
#[must_use]
pub const fn is_cleared(standing: Standing) -> bool {
    matches!(
        standing,
        Standing::UpToDate | Standing::DueSoon | Standing::ExemptRecorded
    )
}

/// Whether a requirement applies to someone in this department. An empty list is everyone.
#[must_use]
pub fn applies(departments: &[String], department: &str) -> bool {
    departments.is_empty()
        || departments
            .iter()
            .any(|d| d.eq_ignore_ascii_case(department))
}

/// The date a record falls due again: the day recorded plus the requirement's re-check period in
/// calendar days; `None` for a one-off requirement.
#[must_use]
pub fn next_due_after(
    recorded_on: NaiveDate,
    recheck_calendar_days: Option<u32>,
) -> Option<NaiveDate> {
    recheck_calendar_days.and_then(|d| recorded_on.checked_add_days(Days::new(u64::from(d))))
}

/// Check a requirement definition.
///
/// # Errors
///
/// A blank or over-long name or description, too many departments, or a period out of range.
pub fn validate_requirement(
    name: &str,
    description: Option<&str>,
    departments: &[String],
    recheck_calendar_days: Option<u32>,
    reminder_calendar_days: u32,
) -> Result<(), String> {
    if name.trim().is_empty() || name.chars().count() > 120 {
        return Err("name is required, up to 120 characters".to_string());
    }
    if description.is_some_and(|d| d.chars().count() > 300) {
        return Err("description is up to 300 characters; say what is required, not why anyone is or is not".to_string());
    }
    if departments.len() > 50
        || departments
            .iter()
            .any(|d| d.trim().is_empty() || d.chars().count() > 120)
    {
        return Err(
            "departments is a list of up to 50 non-blank names (empty means everyone)".to_string(),
        );
    }
    if recheck_calendar_days.is_some_and(|d| d == 0 || d > MAX_CALENDAR_DAYS) {
        return Err(format!(
            "the re-check period is 1 to {MAX_CALENDAR_DAYS} calendar days"
        ));
    }
    if reminder_calendar_days > MAX_CALENDAR_DAYS {
        return Err(format!(
            "the reminder window is at most {MAX_CALENDAR_DAYS} calendar days"
        ));
    }
    Ok(())
}

/// Check what occupational health records.
///
/// # Errors
///
/// A status that is not on the list, a date in the future, or a due date not after the day recorded.
pub fn validate_record(
    status: &str,
    recorded_on: NaiveDate,
    next_due: Option<NaiveDate>,
    today: NaiveDate,
) -> Result<(), String> {
    if !STATUSES.contains(&status) {
        return Err(format!("status must be one of {}", STATUSES.join(", ")));
    }
    if recorded_on > today {
        return Err("recorded_on cannot be in the future".to_string());
    }
    if next_due.is_some_and(|d| d <= recorded_on) {
        return Err("next_due must be after the day it was recorded".to_string());
    }
    Ok(())
}

/// One requirement in one department, as an aggregate.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Compliance {
    /// The requirement.
    pub requirement: String,
    /// The department.
    pub department: String,
    /// People the requirement applies to; `None` below the floor.
    pub applicable: Option<usize>,
    /// How many of them are cleared; `None` below the floor.
    pub cleared: Option<usize>,
}

/// Aggregate by requirement and department. `rows` is `(requirement, department, cleared)` for each
/// person it applies to. A group below the floor shows nothing.
#[must_use]
pub fn compliance(rows: &[(String, String, bool)], floor: usize) -> Vec<Compliance> {
    let mut by: BTreeMap<(&str, &str), (usize, usize)> = BTreeMap::new();
    for (requirement, department, cleared) in rows {
        let cell = by.entry((requirement, department)).or_insert((0, 0));
        cell.0 += 1;
        cell.1 += usize::from(*cleared);
    }
    by.into_iter()
        .map(|((requirement, department), (applicable, cleared))| {
            let shown = applicable >= floor;
            Compliance {
                requirement: requirement.to_string(),
                department: department.to_string(),
                applicable: shown.then_some(applicable),
                cleared: shown.then_some(cleared),
            }
        })
        .collect()
}

/// The gate: monitoring is on only when a lawful basis is recorded.
#[must_use]
pub fn gate(basis: Option<&str>) -> Option<String> {
    basis
        .map(str::trim)
        .filter(|b| !b.is_empty())
        .map(ToString::to_string)
}

#[cfg(test)]
mod tests {
    use super::*;

    /// Nothing else may read a worker's health records: no score, ranking or decision takes them as
    /// an input. Only the controller, the entity, the export and the wiring may name the table.
    #[test]
    fn no_other_code_reads_the_health_records() {
        let root = std::path::Path::new(env!("CARGO_MANIFEST_DIR")).join("src");
        let allowed = [
            "controllers/health_requirements.rs",
            "controllers/privacy.rs",
            "models/_entities/worker_health_records.rs",
            "models/_entities/mod.rs",
            "models/_entities/prelude.rs",
            "app.rs",
            "rules/health_requirements.rs",
        ];
        let mut stack = vec![root.clone()];
        let mut offenders = Vec::new();
        while let Some(dir) = stack.pop() {
            for entry in std::fs::read_dir(&dir).unwrap() {
                let path = entry.unwrap().path();
                if path.is_dir() {
                    stack.push(path);
                } else if path.extension().and_then(|e| e.to_str()) == Some("rs") {
                    let rel = path
                        .strip_prefix(&root)
                        .unwrap()
                        .to_string_lossy()
                        .replace('\\', "/");
                    let text = std::fs::read_to_string(&path).unwrap();
                    if (text.contains("worker_health_records")
                        || text.contains("WorkerHealthRecords"))
                        && !allowed.contains(&rel.as_str())
                    {
                        offenders.push(rel);
                    }
                }
            }
        }
        assert!(
            offenders.is_empty(),
            "code that reads health records: {offenders:?}"
        );
    }

    /// The attribute-scoped writes in `rules::self_service` are exactly the occupational-health
    /// routes this controller registers.
    #[test]
    fn the_attribute_scoped_writes_match_the_routes() {
        let text = std::fs::read_to_string(
            std::path::Path::new(env!("CARGO_MANIFEST_DIR"))
                .join("src/controllers/health_requirements.rs"),
        )
        .unwrap();
        let flat = text.split_whitespace().collect::<Vec<_>>().join(" ");
        for (attribute, method, _) in crate::rules::self_service::ATTRIBUTE_WRITES {
            assert_eq!(*attribute, "occupational_health");
            let token = format!(
                "\"/workers/{{pid}}/health-requirements/{{req_pid}}\", {}(",
                method.to_ascii_lowercase()
            );
            assert!(
                flat.contains(&token),
                "no {method} route for the occupational-health list"
            );
        }
        assert_eq!(
            crate::rules::self_service::attribute_for_write(
                "PUT",
                "/api/workers/a/health-requirements/b"
            ),
            Some("occupational_health")
        );
        assert_eq!(
            crate::rules::self_service::attribute_for_write(
                "GET",
                "/api/workers/a/health-requirements/b"
            ),
            None
        );
        assert_eq!(
            crate::rules::self_service::attribute_for_write(
                "PUT",
                "/api/workers/a/health-requirements"
            ),
            None
        );
        assert_eq!(
            crate::rules::self_service::attribute_for_write("PUT", "/api/workers/a/leave"),
            None
        );
    }

    fn d(y: i32, m: u32, day: u32) -> NaiveDate {
        NaiveDate::from_ymd_opt(y, m, day).unwrap()
    }

    #[test]
    fn standing_at_each_boundary() {
        let today = d(2026, 6, 15);
        let st = |status, due| standing(status, due, today, 60);
        assert_eq!(st(None, None), Standing::NotRecorded);
        assert_eq!(
            st(Some("declined"), Some(d(2030, 1, 1))),
            Standing::Declined
        );
        assert_eq!(
            st(Some("up_to_date"), None),
            Standing::UpToDate,
            "a one-off never expires"
        );
        assert_eq!(
            st(Some("up_to_date"), Some(d(2027, 6, 15))),
            Standing::UpToDate
        );
        // 60 calendar days before the due date is the first day of the reminder window.
        assert_eq!(
            st(Some("up_to_date"), Some(d(2026, 8, 14))),
            Standing::DueSoon,
            "due in 60 calendar days"
        );
        assert_eq!(
            st(Some("up_to_date"), Some(d(2026, 8, 15))),
            Standing::UpToDate,
            "61 calendar days away"
        );
        assert_eq!(
            st(Some("up_to_date"), Some(today)),
            Standing::DueSoon,
            "due today is still valid"
        );
        assert_eq!(
            st(Some("up_to_date"), Some(d(2026, 6, 14))),
            Standing::Overdue,
            "yesterday"
        );
        // An exemption is recorded as such, and its review can lapse like a due date.
        assert_eq!(st(Some("exempt_recorded"), None), Standing::ExemptRecorded);
        assert_eq!(
            st(Some("exempt_recorded"), Some(d(2026, 7, 1))),
            Standing::ExemptRecorded,
            "no reminder for an exemption"
        );
        assert_eq!(
            st(Some("exempt_recorded"), Some(d(2026, 6, 1))),
            Standing::Overdue
        );
    }

    #[test]
    fn cleared_is_a_yes_or_no_and_hides_the_status() {
        for (s, want) in [
            (Standing::UpToDate, true),
            (Standing::DueSoon, true),
            (Standing::ExemptRecorded, true),
            (Standing::NotRecorded, false),
            (Standing::Overdue, false),
            (Standing::Declined, false),
        ] {
            assert_eq!(is_cleared(s), want, "{s:?}");
        }
    }

    #[test]
    fn a_requirement_applies_to_its_departments_or_to_everyone() {
        assert!(applies(&[], "engineering"));
        assert!(
            applies(&["Clinic".to_string(), "Lab".to_string()], "lab"),
            "case does not matter"
        );
        assert!(!applies(&["Clinic".to_string()], "engineering"));
    }

    #[test]
    fn the_next_due_date_is_the_recheck_period_in_calendar_days() {
        assert_eq!(
            next_due_after(d(2026, 6, 15), Some(365)),
            Some(d(2027, 6, 15))
        );
        assert_eq!(
            next_due_after(d(2026, 6, 15), Some(1)),
            Some(d(2026, 6, 16))
        );
        assert_eq!(next_due_after(d(2026, 6, 15), None), None);
    }

    #[test]
    fn a_requirement_says_what_is_required_and_not_why() {
        assert!(
            validate_requirement("Hepatitis B", Some("Three doses"), &[], Some(365), 60).is_ok()
        );
        assert!(validate_requirement("  ", None, &[], None, 60).is_err());
        assert!(validate_requirement("x", Some(&"y".repeat(301)), &[], None, 60).is_err());
        assert!(validate_requirement("x", None, &[String::new()], None, 60).is_err());
        assert!(validate_requirement("x", None, &[], Some(0), 60).is_err());
        assert!(validate_requirement("x", None, &[], Some(MAX_CALENDAR_DAYS + 1), 60).is_err());
        assert!(validate_requirement("x", None, &[], None, MAX_CALENDAR_DAYS + 1).is_err());
    }

    #[test]
    fn a_record_is_a_status_a_date_and_a_due_date() {
        let today = d(2026, 6, 15);
        assert!(validate_record("up_to_date", today, Some(d(2027, 6, 15)), today).is_ok());
        assert!(validate_record("exempt_recorded", today, None, today).is_ok());
        assert!(validate_record("declined", today, None, today).is_ok());
        assert!(
            validate_record("vaccinated", today, None, today).is_err(),
            "not on the list"
        );
        assert!(
            validate_record("up_to_date", d(2026, 6, 16), None, today).is_err(),
            "future"
        );
        assert!(
            validate_record("up_to_date", today, Some(today), today).is_err(),
            "due on the day recorded"
        );
    }

    #[test]
    fn the_aggregate_shows_a_group_only_at_the_floor() {
        let row = |r: &str, dept: &str, c: bool| (r.to_string(), dept.to_string(), c);
        let mut rows = Vec::new();
        for i in 0..10 {
            rows.push(row("hep_b", "clinic", i < 7));
        }
        for _ in 0..4 {
            rows.push(row("hep_b", "lab", true));
        }
        let out = compliance(&rows, 10);
        let clinic = out.iter().find(|c| c.department == "clinic").unwrap();
        assert_eq!((clinic.applicable, clinic.cleared), (Some(10), Some(7)));
        let lab = out.iter().find(|c| c.department == "lab").unwrap();
        assert_eq!(
            (lab.applicable, lab.cleared),
            (None, None),
            "four people: nothing shown"
        );
        assert_eq!(compliance(&[], 10).len(), 0);
    }

    #[test]
    fn it_is_off_without_a_recorded_basis() {
        assert_eq!(gate(None), None);
        assert_eq!(gate(Some("   ")), None);
        assert_eq!(
            gate(Some(" Our occupational health policy ")),
            Some("Our occupational health policy".to_string())
        );
    }
}
