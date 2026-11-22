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

/// Default retention horizon (days) when `WORKFORCE_PLANNING_MANAGEMENT_RETENTION_DAYS` is unset.
pub const RETENTION_DEFAULT_DAYS: i64 = 365;

/// Horizon floor: a sweep that could run at 0 days would silently turn
/// every soft-delete into a hard-delete (WPM-D22).
pub const RETENTION_FLOOR_DAYS: i64 = 30;

/// Parse the retention horizon from the raw setting: absent gives the default (RCA-IN-3).
///
/// # Errors
///
/// A setting that is supplied but is not a whole number of days of at least [`RETENTION_FLOOR_DAYS`] (RCA-IN-2, RCA-IN-4): a sweep
/// that could run at 0 days would turn every soft-delete into a hard-delete (WPM-D22), so it is refused, never raised to the floor.
pub fn retention_days(raw: Option<&str>) -> Result<i64, String> {
    raw.map_or(Ok(RETENTION_DEFAULT_DAYS), |text| {
        retention_setting("WORKFORCE_PLANNING_MANAGEMENT_RETENTION_DAYS", text)
    })
}

/// One retention setting `text`, named `name`: whole days, at least the floor.
fn retention_setting(name: &str, text: &str) -> Result<i64, String> {
    match text.trim().parse::<i64>() {
        Ok(days) if days >= RETENTION_FLOOR_DAYS => Ok(days),
        _ => Err(format!(
            "{name} must be a whole number of days, at least {RETENTION_FLOOR_DAYS}, not {text:?}"
        )),
    }
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
        format!(
            "WORKFORCE_PLANNING_MANAGEMENT_RETENTION_{}_DAYS",
            self.key().to_ascii_uppercase()
        )
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
        | "pay_categories"
        | "pay_information_requests"
        | "payroll_runs"
        | "payslip_queries"
        | "payslips" => Pay,
        "allocations" | "leave_entitlements" | "leave_requests" | "rotas" | "shift_assignments"
        | "shifts" | "time_entries" => TimeAndLeave,
        "appraisals"
        | "development_plan_benefits"
        | "development_plans"
        | "early_career_programs"
        | "feedback_entries"
        | "goals"
        | "worker_goals"
        | "padr_versions"
        | "mobility_interests"
        | "appraisal_nominations"
        | "pipeline_members"
        | "program_placements"
        | "reviews"
        | "review_cycles"
        | "succession_candidates"
        | "succession_plans"
        | "talent_pipelines" => Performance,
        "assessment_instruments"
        | "assessments"
        | "course_requests"
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
    /// The legacy single `WORKFORCE_PLANNING_MANAGEMENT_RETENTION_DAYS`, applied to every kind with no
    /// override of its own.
    Legacy,
    /// The kind's own `WORKFORCE_PLANNING_MANAGEMENT_RETENTION_<KIND>_DAYS`.
    Override,
    /// A jurisdiction profile's `retention.<kind>.days` (WPM-T384): its floor, or its ceiling where it gives none.
    Profile,
}

impl HorizonSource {
    /// The label the API states.
    #[must_use]
    pub const fn label(self) -> &'static str {
        match self {
            Self::Default => "default",
            Self::Legacy => "WORKFORCE_PLANNING_MANAGEMENT_RETENTION_DAYS",
            Self::Override => "override",
            Self::Profile => "profile",
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

/// The horizon for `kind`: its own override if set, else the legacy single value if set, else the kind's default (WPM-D22).
///
/// # Errors
///
/// A setting that is supplied but is not a whole number of days of at least [`RETENTION_FLOOR_DAYS`] (RCA-IN-2, RCA-IN-4). It is
/// refused whichever source it comes from, never ignored in favour of the next one and never raised to the floor.
pub fn horizon_for(
    kind: RecordKind,
    own: Option<&str>,
    legacy: Option<&str>,
) -> Result<Horizon, String> {
    let own = own
        .map(|text| retention_setting(&kind.env_name(), text))
        .transpose()?;
    let legacy = legacy
        .map(|text| retention_setting("WORKFORCE_PLANNING_MANAGEMENT_RETENTION_DAYS", text))
        .transpose()?;
    let (days, source) = match (own, legacy) {
        (Some(days), _) => (days, HorizonSource::Override),
        (None, Some(days)) => (days, HorizonSource::Legacy),
        (None, None) => (kind.default_days(), HorizonSource::Default),
    };
    Ok(Horizon { days, source })
}

/// The horizon a profile's `bounded` range gives (WPM-T384): its floor, so a record is kept as long as
/// the strictest floor requires and no longer; where only a ceiling is given, the ceiling. `None` when the range gives neither, or
/// gives fewer than [`RETENTION_FLOOR_DAYS`]: a stored value below the floor is corrupt data, so it is unknown, never raised to the
/// floor (RCA-IN-6).
#[must_use]
pub fn horizon_from_range(min: Option<i64>, max: Option<i64>) -> Option<Horizon> {
    min.or(max)
        .filter(|days| *days >= RETENTION_FLOOR_DAYS)
        .map(|days| Horizon {
            days,
            source: HorizonSource::Profile,
        })
}

/// Every table with a `deleted_at` column, for the retention sweep.
/// Kept in one place so a new soft-deleting table is added here (the
/// sweep test counts this list against the entity modules).
pub const SOFT_DELETED_TABLES: &[&str] = &[
    "adjustment_requests",
    "allocations",
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
    "development_plan_benefits",
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
    "pay_categories",
    "pay_information_requests",
    "payroll_runs",
    "payslip_queries",
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
    "worker_goals",
    "worker_skills",
    "workers",
    "workforce_plans",
];

#[cfg(test)]
mod tests {
    use super::*;

    /// A profile's range gives its floor, or its ceiling where there is no floor, never below the 30-day floor.
    #[test]
    fn a_profile_range_gives_its_floor_then_its_ceiling() {
        let days = |min, max| horizon_from_range(min, max).map(|h| (h.days, h.source.label()));
        assert_eq!(days(Some(400), Some(900)), Some((400, "profile")));
        assert_eq!(days(None, Some(900)), Some((900, "profile")));
        // below the floor is corrupt stored data: unknown, never raised to the floor (RCA-IN-6)
        assert_eq!(days(Some(10), None), None);
        assert_eq!(
            days(Some(RETENTION_FLOOR_DAYS), None),
            Some((RETENTION_FLOOR_DAYS, "profile"))
        );
        assert_eq!(days(Some(RETENTION_FLOOR_DAYS - 1), None), None);
        assert_eq!(days(None, None), None);
    }

    #[test]
    fn erasure_requires_a_closed_employment() {
        assert!(erasable("terminated") && erasable("retired"));
        for status in ["onboarding", "active", "on_leave", "offboarding"] {
            assert!(!erasable(status), "{status} still has a lawful basis");
        }
    }

    /// The horizon: the default when absent, a sane value passes through, and a supplied value that is
    /// junk or below the floor is refused, naming the setting (RCA-IN-2, RCA-IN-4).
    #[test]
    fn retention_horizon_defaults_when_absent_and_refuses_a_bad_value() {
        assert_eq!(retention_days(None), Ok(RETENTION_DEFAULT_DAYS));
        assert_eq!(retention_days(Some("730")), Ok(730));
        assert_eq!(retention_days(Some("30")), Ok(30));
        for bad in ["", "junk", "0", "-5", "29"] {
            let error = retention_days(Some(bad)).unwrap_err();
            assert!(error.contains("RETENTION_DAYS"), "{bad:?}: {error}");
            assert!(error.contains("at least 30"), "{bad:?}: {error}");
        }
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
            .try_fold(0_usize, usize::checked_add)
            .unwrap();
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
        let h = |own, legacy| horizon_for(kind, own, legacy).unwrap();
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
        // A supplied value that is junk or below the floor is refused whichever source it is, naming the
        // setting. It is not ignored for the next source and not raised, so no kind can be set to
        // hard-delete at once (RCA-IN-2).
        for (own, legacy, name) in [
            (Some("junk"), Some("500"), "RETENTION_WELLBEING_DAYS"),
            (Some(""), None, "RETENTION_WELLBEING_DAYS"),
            (Some("0"), None, "RETENTION_WELLBEING_DAYS"),
            (None, Some("-5"), "RETENTION_DAYS"),
            (None, Some("junk"), "RETENTION_DAYS"),
            (Some("90"), Some("junk"), "RETENTION_DAYS"),
        ] {
            let error = horizon_for(kind, own, legacy).unwrap_err();
            assert!(error.contains(name), "{own:?} {legacy:?}: {error}");
        }
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
            "WORKFORCE_PLANNING_MANAGEMENT_RETENTION_TIME_AND_LEAVE_DAYS"
        );
        assert_eq!(
            RecordKind::Pay.env_name(),
            "WORKFORCE_PLANNING_MANAGEMENT_RETENTION_PAY_DAYS"
        );
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
        assert_eq!(SOFT_DELETED_TABLES.len(), 62);
        for table in ["workers", "payslips", "candidates", "appraisals"] {
            assert!(SOFT_DELETED_TABLES.contains(&table));
        }
    }

    #[test]
    fn every_kind_and_source_has_its_stated_name() {
        let keys: Vec<&str> = RecordKind::ALL.iter().map(|k| k.key()).collect();
        assert_eq!(
            keys,
            [
                "recruitment",
                "employment",
                "pay",
                "time_and_leave",
                "performance",
                "learning",
                "wellbeing",
                "planning"
            ]
        );
        assert_eq!(
            RecordKind::TimeAndLeave.env_name(),
            "WORKFORCE_PLANNING_MANAGEMENT_RETENTION_TIME_AND_LEAVE_DAYS"
        );
        let labels: Vec<&str> = [
            HorizonSource::Default,
            HorizonSource::Legacy,
            HorizonSource::Override,
            HorizonSource::Profile,
        ]
        .iter()
        .map(|s| s.label())
        .collect();
        assert_eq!(
            labels,
            [
                "default",
                "WORKFORCE_PLANNING_MANAGEMENT_RETENTION_DAYS",
                "override",
                "profile"
            ]
        );
        assert_eq!(kind_of("allocations"), Some(RecordKind::TimeAndLeave));
        assert_eq!(kind_of("not_a_table"), None);
    }
}
