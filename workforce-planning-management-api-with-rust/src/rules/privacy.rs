//! Pure rules for subject rights & retention (WPM-R30 / WPM-D22):
//! when erasure is allowed, how the retention horizon is read, and
//! which tables the sweep covers. No I/O.

/// Worker statuses in which erasure is allowed: the employment
/// relationship is the lawful basis for the data, so an active (or
/// on-leave, or onboarding) worker cannot be erased.
#[must_use]
pub fn erasable(status: &str) -> bool {
    matches!(status, "terminated" | "retired")
}

/// Default retention horizon (days) when `WPM_RETENTION_DAYS` is unset.
pub const RETENTION_DEFAULT_DAYS: i64 = 365;

/// Horizon floor: a sweep that could run at 0 days would silently turn
/// every soft-delete into a hard-delete (WPM-D22).
pub const RETENTION_FLOOR_DAYS: i64 = 30;

/// Parse the retention horizon from the raw env value: unset / blank /
/// junk ⇒ the default; anything below the floor is clamped up to it.
#[must_use]
pub fn retention_days(raw: Option<&str>) -> i64 {
    raw.and_then(|value| value.trim().parse::<i64>().ok())
        .unwrap_or(RETENTION_DEFAULT_DAYS)
        .max(RETENTION_FLOOR_DAYS)
}

/// A kind of record, for retention (WPM-R91, WPM-D60). Every soft-deleting
/// table belongs to exactly one kind; a kind has one horizon.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum RecordKind {
    /// Candidates, applications, interviews, requisitions, onboarding.
    Recruitment,
    /// The worker record and the people around it.
    Employment,
    /// Payroll, expense claims and benefits.
    Pay,
    /// Time, leave, shifts and the on-call rota.
    TimeAndLeave,
    /// Reviews, appraisals, goals, succession and talent pipelines.
    Performance,
    /// Skills, training, assessments, mentoring and CPD.
    Learning,
    /// Adjustments, ergonomics, wellbeing and the pulse.
    Wellbeing,
    /// Plans, role profiles, announcements and reference data.
    Planning,
}

impl RecordKind {
    /// Every kind, in the order the schedule lists them.
    pub const ALL: [Self; 8] = [
        Self::Recruitment,
        Self::Employment,
        Self::Pay,
        Self::TimeAndLeave,
        Self::Performance,
        Self::Learning,
        Self::Wellbeing,
        Self::Planning,
    ];

    /// The stable key used in the API and in the environment name.
    #[must_use]
    pub const fn key(self) -> &'static str {
        match self {
            Self::Recruitment => "recruitment",
            Self::Employment => "employment",
            Self::Pay => "pay",
            Self::TimeAndLeave => "time_and_leave",
            Self::Performance => "performance",
            Self::Learning => "learning",
            Self::Wellbeing => "wellbeing",
            Self::Planning => "planning",
        }
    }

    /// The environment variable that overrides this kind's horizon.
    #[must_use]
    pub fn env_name(self) -> String {
        format!("WPM_RETENTION_{}_DAYS", self.key().to_ascii_uppercase())
    }

    /// The default horizon in calendar days after a record is deleted. These
    /// are cautious starting points a deployer replaces with their own legal
    /// basis (`spec/governance/retention-schedule.md`); never legal advice.
    #[must_use]
    pub const fn default_days(self) -> i64 {
        match self {
            Self::Recruitment => 180,
            Self::Employment | Self::Pay => 2190,
            Self::TimeAndLeave | Self::Performance => 730,
            Self::Learning => 1095,
            Self::Wellbeing | Self::Planning => 365,
        }
    }
}

/// The kind a soft-deleting table belongs to, or `None` for a table the
/// schedule does not know (a test pins that this is never so for
/// [`SOFT_DELETED_TABLES`]).
#[must_use]
pub fn kind_of(table: &str) -> Option<RecordKind> {
    use RecordKind::{
        Employment, Learning, Pay, Performance, Planning, Recruitment, TimeAndLeave, Wellbeing,
    };
    Some(match table {
        "applications" | "candidates" | "interviews" | "onboarding_items" | "requisitions" => {
            Recruitment
        }
        "emergency_contacts" | "worker_aspirations" | "worker_backups" | "workers" => Employment,
        "benefit_enrollments"
        | "benefit_plans"
        | "expense_claims"
        | "payroll_runs"
        | "payslips" => Pay,
        "leave_entitlements" | "leave_requests" | "rotas" | "shift_assignments" | "shifts"
        | "time_entries" => TimeAndLeave,
        "appraisals"
        | "development_plans"
        | "early_career_programs"
        | "feedback_entries"
        | "goals"
        | "mobility_interests"
        | "pipeline_members"
        | "program_placements"
        | "reviews"
        | "review_cycles"
        | "succession_candidates"
        | "succession_plans"
        | "talent_pipelines" => Performance,
        "assessment_instruments"
        | "assessments"
        | "cpd_entries"
        | "cpd_requirements"
        | "learning_paths"
        | "mentorships"
        | "path_enrollments"
        | "professional_registrations"
        | "skill_courses"
        | "skills"
        | "training_enrollments"
        | "worker_skills" => Learning,
        "adjustment_requests"
        | "ergonomic_assessments"
        | "ergonomic_items"
        | "pulse_surveys"
        | "wellbeing_entitlements" => Wellbeing,
        "announcements" | "benchmarks" | "change_initiatives" | "groups" | "role_profiles"
        | "workforce_plans" => Planning,
        _ => return None,
    })
}

/// Where a horizon came from.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum HorizonSource {
    /// The built-in default for the kind.
    Default,
    /// The legacy single `WPM_RETENTION_DAYS`, applied to every kind with no
    /// override of its own.
    Legacy,
    /// The kind's own `WPM_RETENTION_<KIND>_DAYS`.
    Override,
}

impl HorizonSource {
    /// The label the API states.
    #[must_use]
    pub const fn label(self) -> &'static str {
        match self {
            Self::Default => "default",
            Self::Legacy => "WPM_RETENTION_DAYS",
            Self::Override => "override",
        }
    }
}

/// A kind's horizon in calendar days, and where it came from.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct Horizon {
    /// Calendar days after deletion; never below [`RETENTION_FLOOR_DAYS`].
    pub days: i64,
    /// Where the value came from.
    pub source: HorizonSource,
}

/// The horizon for `kind`: its own override if set and numeric, else the
/// legacy single value if set and numeric, else the kind's default; always
/// floored (WPM-D22). A junk value is ignored, not trusted.
#[must_use]
pub fn horizon_for(kind: RecordKind, own: Option<&str>, legacy: Option<&str>) -> Horizon {
    let parse = |raw: Option<&str>| raw.and_then(|v| v.trim().parse::<i64>().ok());
    let (days, source) = if let Some(days) = parse(own) {
        (days, HorizonSource::Override)
    } else if let Some(days) = parse(legacy) {
        (days, HorizonSource::Legacy)
    } else {
        (kind.default_days(), HorizonSource::Default)
    };
    Horizon {
        days: days.max(RETENTION_FLOOR_DAYS),
        source,
    }
}

/// Every table with a `deleted_at` column, for the retention sweep.
/// Kept in one place so a new soft-deleting table is added here (the
/// sweep test counts this list against the entity modules).
pub const SOFT_DELETED_TABLES: &[&str] = &[
    "adjustment_requests",
    "announcements",
    "applications",
    "appraisals",
    "assessment_instruments",
    "assessments",
    "benchmarks",
    "benefit_enrollments",
    "benefit_plans",
    "candidates",
    "change_initiatives",
    "cpd_entries",
    "cpd_requirements",
    "development_plans",
    "early_career_programs",
    "emergency_contacts",
    "ergonomic_assessments",
    "ergonomic_items",
    "expense_claims",
    "feedback_entries",
    "goals",
    "groups",
    "interviews",
    "learning_paths",
    "leave_entitlements",
    "leave_requests",
    "mentorships",
    "mobility_interests",
    "onboarding_items",
    "path_enrollments",
    "payroll_runs",
    "payslips",
    "pipeline_members",
    "professional_registrations",
    "program_placements",
    "pulse_surveys",
    "requisitions",
    "review_cycles",
    "reviews",
    "role_profiles",
    "rotas",
    "shift_assignments",
    "shifts",
    "skill_courses",
    "skills",
    "succession_candidates",
    "succession_plans",
    "talent_pipelines",
    "time_entries",
    "training_enrollments",
    "wellbeing_entitlements",
    "worker_aspirations",
    "worker_backups",
    "worker_skills",
    "workers",
    "workforce_plans",
];

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn erasure_requires_a_closed_employment() {
        assert!(erasable("terminated") && erasable("retired"));
        for status in ["onboarding", "active", "on_leave", "offboarding"] {
            assert!(!erasable(status), "{status} still has a lawful basis");
        }
    }

    /// The horizon: default on unset/junk, floor-clamped, and a sane
    /// value passes through.
    #[test]
    fn retention_horizon_defaults_and_floors() {
        assert_eq!(retention_days(None), RETENTION_DEFAULT_DAYS);
        assert_eq!(retention_days(Some("")), RETENTION_DEFAULT_DAYS);
        assert_eq!(retention_days(Some("junk")), RETENTION_DEFAULT_DAYS);
        assert_eq!(retention_days(Some("730")), 730);
        assert_eq!(
            retention_days(Some("0")),
            RETENTION_FLOOR_DAYS,
            "0 would hard-delete"
        );
        assert_eq!(retention_days(Some("-5")), RETENTION_FLOOR_DAYS);
        assert_eq!(retention_days(Some("30")), 30);
    }

    #[test]
    fn every_swept_table_has_exactly_one_kind() {
        for table in SOFT_DELETED_TABLES {
            assert!(kind_of(table).is_some(), "{table} has no record kind");
        }
        assert_eq!(kind_of("not_a_table"), None);
        let total: usize = RecordKind::ALL
            .iter()
            .map(|kind| {
                SOFT_DELETED_TABLES
                    .iter()
                    .filter(|t| kind_of(t) == Some(*kind))
                    .count()
            })
            .sum();
        assert_eq!(total, SOFT_DELETED_TABLES.len(), "no table in two kinds");
        for kind in RecordKind::ALL {
            assert!(
                SOFT_DELETED_TABLES.iter().any(|t| kind_of(t) == Some(kind)),
                "{} is empty",
                kind.key()
            );
        }
    }

    #[test]
    fn a_horizon_is_the_override_then_the_legacy_value_then_the_default() {
        use HorizonSource::{Default, Legacy, Override};
        let kind = RecordKind::Wellbeing;
        let h = |own, legacy| horizon_for(kind, own, legacy);
        assert_eq!(
            h(None, None),
            Horizon {
                days: 365,
                source: Default
            }
        );
        assert_eq!(
            h(None, Some("500")),
            Horizon {
                days: 500,
                source: Legacy
            }
        );
        assert_eq!(
            h(Some("90"), Some("500")),
            Horizon {
                days: 90,
                source: Override
            }
        );
        // Junk is ignored, and the next source is used.
        assert_eq!(
            h(Some("junk"), Some("500")),
            Horizon {
                days: 500,
                source: Legacy
            }
        );
        assert_eq!(
            h(Some(""), Some("junk")),
            Horizon {
                days: 365,
                source: Default
            }
        );
        // The floor holds for every source, so no kind can be set to hard-delete at once.
        assert_eq!(h(Some("0"), None).days, RETENTION_FLOOR_DAYS);
        assert_eq!(h(None, Some("-5")).days, RETENTION_FLOOR_DAYS);
    }

    #[test]
    fn defaults_are_at_or_above_the_floor_and_names_are_stable() {
        for kind in RecordKind::ALL {
            assert!(
                kind.default_days() >= RETENTION_FLOOR_DAYS,
                "{}",
                kind.key()
            );
        }
        assert_eq!(
            RecordKind::TimeAndLeave.env_name(),
            "WPM_RETENTION_TIME_AND_LEAVE_DAYS"
        );
        assert_eq!(RecordKind::Pay.env_name(), "WPM_RETENTION_PAY_DAYS");
        assert_eq!(RecordKind::Recruitment.default_days(), 180);
    }

    /// The sweep list is sorted and duplicate-free (each table swept
    /// exactly once), and covers the known soft-deleting tables.
    #[test]
    fn sweep_table_list_is_sound() {
        let mut sorted = SOFT_DELETED_TABLES.to_vec();
        sorted.sort_unstable();
        sorted.dedup();
        assert_eq!(sorted, SOFT_DELETED_TABLES, "sorted and unique");
        assert_eq!(SOFT_DELETED_TABLES.len(), 56);
        for table in ["workers", "payslips", "candidates", "appraisals"] {
            assert!(SOFT_DELETED_TABLES.contains(&table));
        }
    }
}
