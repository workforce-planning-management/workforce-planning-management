//! **Need-to-know on reads** (WPM-R114, WPM-D71): pure, database-free rules that say who
//! may read each `GET` route.
//!
//! The policy engine decides by *action*, so a signed-in caller who holds no attributes can
//! read, and the controllers that never loaded the record never asked whether they should. This
//! module is the table that closes that gap: every `GET` route under `/api` is classified, and a
//! test fails when a route is added without a classification, so a new route cannot be open by
//! omission.
//!
//! The classification is only the *table*. The middleware in [`crate::need_to_know`] loads the
//! worker the route is about and turns it into [`Facts`]; [`permits`] decides.

/// Who a worker-scoped read is for.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum WorkerLevel {
    /// Colleague-level facts (who reports to whom, who is on call): the worker, their line
    /// managers, privileged callers, and anyone in an organization the caller belongs to.
    InScope,
    /// A personal record a line manager legitimately needs: the worker, their line managers up
    /// the chain, and privileged callers.
    WithManagers,
    /// A record that is the worker's own: the worker and privileged callers only. Not their
    /// manager.
    SelfOnly,
}

/// Who may read a route.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Audience {
    /// Reference data and aggregates: any signed-in caller.
    Open,
    /// The controller loads the record and applies its own rule (and the tests for that rule
    /// say what it is); the guard adds nothing.
    ControllerDecides,
    /// HR, payroll, service and administrator callers only.
    Privileged,
    /// A route about one worker, `/api/workers/{pid}...`.
    Worker(WorkerLevel),
    /// `/api/reviews/{pid}`: the review's worker decides, as for [`WorkerLevel::WithManagers`].
    ReviewRecord,
}

use Audience::{ControllerDecides, Open, Privileged, ReviewRecord, Worker};
use WorkerLevel::{InScope, SelfOnly, WithManagers};

/// Every `GET` route under `/api` (without the prefix), and who may read it. `{}` matches one
/// path segment. Keep it sorted by route so a review is a diff.
pub const READS: &[(&str, Audience)] = &[
    ("announcements", ControllerDecides),
    ("announcements/{}", ControllerDecides),
    ("applications/{}/interviews", Privileged),
    ("applications/{}/assessments", Privileged),
    ("appraisals/{}", ControllerDecides),
    ("appraisals/{}/report", ControllerDecides),
    ("assessment-instruments", Open),
    ("assessments/analytics", Open),
    ("assessments/{}", ControllerDecides),
    ("audits", Privileged),
    ("audits/recent", Privileged),
    ("audits/verify", Privileged),
    ("audits/{}", Privileged),
    ("benchmarks", Privileged),
    ("benchmarks/comparison", Privileged),
    ("benefit-plans", Open),
    ("candidates", Privileged),
    ("candidates/{}/assessment-profile", Privileged),
    ("workforce-intelligence/capability", Open),
    ("workforce-intelligence/capability-analysis", Open),
    ("capability-frameworks", Open),
    ("capacity", Open),
    ("capacity/settings", Open),
    ("capacity/start-decisions", Open),
    ("change-initiatives", Open),
    ("change-initiatives/{}", Open),
    ("change-initiatives/{}/readiness", Open),
    ("cpd-requirements", Open),
    ("cpd/overview", Privileged),
    ("directory", Open),
    ("early-career-programs", Open),
    ("engagements/missing-end-date", Privileged),
    ("ergonomics/issues", Privileged),
    ("equality-monitoring/summary", Privileged),
    ("esco/occupation", Open),
    ("esco/occupations", Open),
    ("esco/skills", Open),
    ("events/recent", Privileged),
    ("expense-claims", ControllerDecides),
    ("expense-claims/{}", ControllerDecides),
    ("flexible-working", ControllerDecides),
    ("frameworks/selectable", Open),
    ("groups", Open),
    ("groups/{}/members", ControllerDecides),
    ("groups/{}/skills", ControllerDecides),
    ("health-requirements", ControllerDecides),
    ("health-requirements/clearance", ControllerDecides),
    ("health-requirements/compliance", ControllerDecides),
    ("health-requirements/records", ControllerDecides),
    ("workforce-intelligence/headcount-history", Open),
    ("workforce-intelligence/insights", Open),
    ("job-levels", Open),
    ("job-levels/{}", Open),
    ("job-levels/{}/levels/{}", Open),
    ("learning-paths", Open),
    ("learning-paths/{}/progress", Privileged),
    ("learning/mentorship-overview", Open),
    ("learning/skills-matrix", Open),
    ("learning/training-analytics", Open),
    ("me/contact-details", ControllerDecides),
    ("me/equality-monitoring", ControllerDecides),
    ("me/emergency-contacts", ControllerDecides),
    ("me/flexible-working", ControllerDecides),
    ("me/health-requirements", ControllerDecides),
    ("me/organizations", Open),
    ("me/resignation", ControllerDecides),
    ("me/organizations/scope", Open),
    ("me/time-off", ControllerDecides),
    ("mentorships/{}", Privileged),
    ("workforce-intelligence/metrics", Open),
    ("mobility/interest-summary", Open),
    ("movements", ControllerDecides),
    ("movements/{}", ControllerDecides),
    ("movements/{}/handover", ControllerDecides),
    ("movements/{}/handover/actions", ControllerDecides),
    ("org-chart", Open),
    ("organization-confederations", ControllerDecides),
    ("organization-memberships", ControllerDecides),
    ("workforce-intelligence/overview", Open),
    ("partner-commitments", Open),
    ("pay-scales", Open),
    ("pay-scales/{}", Open),
    ("pay-scales/{}/position", Open),
    ("payroll-runs", Privileged),
    ("payroll-runs/{}", Privileged),
    ("payroll-runs/{}/payslips", Privileged),
    ("workforce-intelligence/pipelines", Open),
    ("workforce-intelligence/training-demand", Open),
    ("programme-demands", Open),
    ("pulse-surveys", Open),
    ("pulse-surveys/{}/results", ControllerDecides),
    ("requisitions", Privileged),
    ("requisitions/{}", Privileged),
    ("requisitions/{}/applications", Privileged),
    ("resignations", Privileged),
    ("resignations/summary", Privileged),
    ("retention", Privileged),
    ("retention/schedule", Privileged),
    ("review-cycles", Open),
    ("reviews/{}", ReviewRecord),
    ("role-profiles", Open),
    ("role-profiles/{}", Open),
    ("role-profiles/{}/gap", Open),
    ("role-profiles/{}/grade", Open),
    ("role-profiles/{}/progression", Open),
    ("rotas", Open),
    ("rotas/{}", Open),
    ("rotas/{}/on-call", Open),
    ("rotas/{}/swap-requests", Open),
    ("shifts", Open),
    ("skill-pools", Open),
    ("skill-pools/{}", Open),
    ("skills", Open),
    ("skills/category-suggestions", Open),
    ("skills/{}/courses", Open),
    ("skills/{}/usage", Open),
    ("workforce-intelligence/succession", Privileged),
    ("succession-plans", Privileged),
    ("succession-plans/gaps", Privileged),
    ("talent-pipelines", Privileged),
    ("talent-pipelines/{}", Privileged),
    ("training/expiring", Privileged),
    ("wellbeing-entitlements", Open),
    ("wellbeing/uptake", ControllerDecides),
    ("workers", ControllerDecides),
    ("workers/{}", Worker(InScope)),
    ("workers/{}/adjustment-requests", Worker(SelfOnly)),
    ("workers/{}/announcement-reads", Worker(SelfOnly)),
    ("workers/{}/appraisal-requests", ControllerDecides),
    ("workers/{}/appraisals", ControllerDecides),
    ("workers/{}/aspirations", ControllerDecides),
    ("workers/{}/assessment-profile", Worker(SelfOnly)),
    ("workers/{}/backups", Worker(InScope)),
    ("workers/{}/benefit-enrollments", Worker(SelfOnly)),
    ("workers/{}/contractor-details", Worker(SelfOnly)),
    ("workers/{}/cover", Worker(InScope)),
    ("workers/{}/contact-details", Worker(SelfOnly)),
    ("workers/{}/cpd-entries", Worker(WithManagers)),
    ("workers/{}/cpd-progress", Worker(WithManagers)),
    ("workers/{}/development-plans", Worker(WithManagers)),
    ("workers/{}/dotted-line", Worker(InScope)),
    ("workers/{}/downline", Worker(InScope)),
    ("workers/{}/downline-aspirations", ControllerDecides),
    ("workers/{}/emergency-contacts", ControllerDecides),
    ("workers/{}/ergonomic-assessments", Worker(SelfOnly)),
    ("workers/{}/engagement", ControllerDecides),
    (
        "workers/{}/engagement/status-assessments",
        ControllerDecides,
    ),
    ("workers/{}/expense-claims", ControllerDecides),
    ("workers/{}/flexible-working", Worker(WithManagers)),
    ("workers/{}/framework-roles", Worker(WithManagers)),
    ("workers/{}/framework-roles/{}/skills", Worker(WithManagers)),
    ("workers/{}/groups", Worker(InScope)),
    ("workers/{}/job-level", ControllerDecides),
    ("workers/{}/leave-entitlements", Worker(WithManagers)),
    ("workers/{}/leave-requests", Worker(WithManagers)),
    ("workers/{}/mobility-interests", Worker(SelfOnly)),
    ("workers/{}/movements", ControllerDecides),
    ("workers/{}/notifications", Worker(SelfOnly)),
    ("workers/{}/on-call", Worker(InScope)),
    ("workers/{}/onboarding", Worker(WithManagers)),
    ("workers/{}/opportunities", Worker(SelfOnly)),
    ("workers/{}/pay-position", ControllerDecides),
    ("workers/{}/payslips", Worker(SelfOnly)),
    ("workers/{}/placements", Worker(WithManagers)),
    ("workers/{}/registrations", Worker(WithManagers)),
    ("workers/{}/reports", Worker(InScope)),
    ("workers/{}/resignation", Worker(WithManagers)),
    ("workers/{}/reviews", Worker(WithManagers)),
    ("workers/{}/role-gap", Worker(WithManagers)),
    ("workers/{}/role-history", Worker(WithManagers)),
    ("workers/{}/role-matches", Worker(SelfOnly)),
    ("workers/{}/skill-gaps", Worker(WithManagers)),
    ("workers/{}/skill-history", Worker(WithManagers)),
    ("workers/{}/skills", Worker(WithManagers)),
    ("workers/{}/skills-as-of", Worker(WithManagers)),
    ("workers/{}/subject-access", ControllerDecides),
    ("workers/{}/swap-requests", Worker(InScope)),
    ("workers/{}/time-entries", Worker(WithManagers)),
    ("workers/{}/training-enrollments", Worker(WithManagers)),
    ("workers/{}/training-plan", Worker(WithManagers)),
    ("workers/{}/upline", Worker(InScope)),
    ("workers/{}/wellbeing-prompts", Worker(SelfOnly)),
    ("workforce-intelligence/skill-gaps", Open),
    ("workforce-plans", Open),
    ("workforce-plans/{}", Open),
    ("workforce-plans/{}/alignment", Open),
    ("workforce-plans/{}/cost", Open),
    ("workforce-plans/{}/forecast", Open),
    ("workforce/working-time", Privileged),
];

/// Whether `pattern` (segments, `{}` = any one segment) matches `path` (segments).
pub(crate) fn matches(pattern: &str, path: &str) -> bool {
    let mut p = pattern.split('/');
    let mut s = path.split('/');
    loop {
        match (p.next(), s.next()) {
            (None, None) => return true,
            (Some("{}"), Some(seg)) if !seg.is_empty() => {}
            (Some(a), Some(b)) if a == b => {}
            _ => return false,
        }
    }
}

/// The audience of a `GET` path (with or without the `/api` prefix and a trailing slash), or
/// `None` when the route is not in [`READS`]. A path outside `/api` is not this module's.
#[must_use]
pub fn classify_read(path: &str) -> Option<Audience> {
    let path = path.trim_end_matches('/');
    let rest = path
        .strip_prefix("/api/")
        .or_else(|| path.strip_prefix('/'))?;
    READS
        .iter()
        .find(|(pattern, _)| matches(pattern, rest))
        .map(|(_, audience)| *audience)
}

/// The worker pid segment and the remainder of a `/api/workers/{pid}...` path.
#[must_use]
pub fn worker_pid_of(path: &str) -> Option<&str> {
    let path = path.trim_end_matches('/');
    let rest = path.strip_prefix("/api/workers/")?;
    let pid = rest.split('/').next()?;
    (!pid.is_empty()).then_some(pid)
}

/// What the caller is to the worker a route is about. Ordered: each relation includes what the
/// ones before it may read.
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Default)]
pub enum Relation {
    /// Nothing to do with this worker.
    #[default]
    Stranger,
    /// Belongs to an organization the worker belongs to.
    InScope,
    /// A line manager of the worker, directly or higher up.
    Manager,
    /// The worker.
    Own,
}

/// What the middleware learned about the caller and the record. All derived from the verified
/// token and the database; nothing here is supplied by the request.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub struct Facts {
    /// The caller holds HR, payroll, service or administrator rights — globally for a route
    /// that is not about one worker, or in the worker's organization for one that is.
    pub privileged: bool,
    /// The caller's closest relation to the worker.
    pub relation: Relation,
}

/// Whether a caller with these facts may read a worker-scoped route of this level.
#[must_use]
pub fn permits(level: WorkerLevel, facts: &Facts) -> bool {
    if facts.privileged {
        return true;
    }
    let needed = match level {
        WorkerLevel::SelfOnly => Relation::Own,
        WorkerLevel::WithManagers => Relation::Manager,
        WorkerLevel::InScope => Relation::InScope,
    };
    facts.relation >= needed
}

/// Whether a privileged-only route may be read.
#[must_use]
pub const fn permits_privileged(facts: &Facts) -> bool {
    facts.privileged
}

/// How many levels up a management chain is followed. A chain longer than this is a data error
/// (or a cycle); the walk stops and the caller is not treated as a manager.
pub const MAX_MANAGER_DEPTH: usize = 20;

#[cfg(test)]
mod tests {
    use super::*;

    fn facts(privileged: bool, relation: Relation) -> Facts {
        Facts {
            privileged,
            relation,
        }
    }

    #[test]
    fn a_stranger_reads_nothing_personal() {
        let stranger = Facts::default();
        for level in [
            WorkerLevel::SelfOnly,
            WorkerLevel::WithManagers,
            WorkerLevel::InScope,
        ] {
            assert!(!permits(level, &stranger), "{level:?}");
        }
        assert!(!permits_privileged(&stranger));
    }

    #[test]
    fn each_level_admits_exactly_who_it_names() {
        let manager = facts(false, Relation::Manager);
        let colleague = facts(false, Relation::InScope);
        let own = facts(false, Relation::Own);
        let hr = facts(true, Relation::Stranger);
        // Self-only: the worker and privileged callers, not the manager, not a colleague.
        assert!(permits(WorkerLevel::SelfOnly, &own));
        assert!(permits(WorkerLevel::SelfOnly, &hr));
        assert!(!permits(WorkerLevel::SelfOnly, &manager));
        assert!(!permits(WorkerLevel::SelfOnly, &colleague));
        // With managers: also the line manager, still not a colleague.
        assert!(permits(WorkerLevel::WithManagers, &manager));
        assert!(permits(WorkerLevel::WithManagers, &own));
        assert!(!permits(WorkerLevel::WithManagers, &colleague));
        // In scope: also anyone in the organization.
        assert!(permits(WorkerLevel::InScope, &colleague));
        assert!(permits(WorkerLevel::InScope, &manager));
        // A privileged caller reads everything worker-scoped.
        for level in [
            WorkerLevel::SelfOnly,
            WorkerLevel::WithManagers,
            WorkerLevel::InScope,
        ] {
            assert!(permits(level, &hr), "{level:?}");
        }
        assert!(permits_privileged(&hr));
        assert!(
            !permits_privileged(&own),
            "being the worker is not being privileged"
        );
    }

    #[test]
    fn paths_classify_with_or_without_prefix_and_trailing_slash() {
        assert_eq!(classify_read("/api/candidates"), Some(Privileged));
        assert_eq!(classify_read("/api/candidates/"), Some(Privileged));
        assert_eq!(classify_read("candidates"), None, "needs the leading slash");
        assert_eq!(classify_read("/candidates"), Some(Privileged));
        assert_eq!(
            classify_read("/api/workers/0c4f1e2a-0000-4000-8000-000000000001/leave-requests"),
            Some(Worker(WithManagers))
        );
        assert_eq!(
            classify_read("/api/workers/0c4f1e2a-0000-4000-8000-000000000001"),
            Some(Worker(InScope))
        );
        assert_eq!(classify_read("/api/job-levels/L5/levels/l5"), Some(Open));
        // Specific beats wildcard: `skills/category-suggestions` is not `skills/{}/…`.
        assert_eq!(
            classify_read("/api/skills/category-suggestions"),
            Some(Open)
        );
        // Unknown routes are unclassified, never open.
        assert_eq!(classify_read("/api/something-new"), None);
        assert_eq!(classify_read("/api/workers/x/something-new"), None);
        assert_eq!(
            classify_read("/api/audits//recent"),
            None,
            "empty segments do not match"
        );
        assert_eq!(classify_read("/health"), None, "not an API route");
    }

    #[test]
    fn the_worker_pid_is_the_segment_after_workers() {
        assert_eq!(worker_pid_of("/api/workers/abc"), Some("abc"));
        assert_eq!(
            worker_pid_of("/api/workers/abc/leave-requests"),
            Some("abc")
        );
        assert_eq!(worker_pid_of("/api/workers/"), None);
        assert_eq!(worker_pid_of("/api/workers"), None);
        assert_eq!(worker_pid_of("/api/candidates/abc"), None);
    }

    #[test]
    fn the_table_has_no_duplicate_and_no_shadowed_pattern() {
        for (i, (a, _)) in READS.iter().enumerate() {
            for (b, _) in &READS[..i] {
                assert_ne!(a, b, "duplicate route {a}");
            }
        }
        // A wildcard route must not hide a literal one listed after it.
        for (i, (earlier, _)) in READS.iter().enumerate() {
            for (later, _) in &READS[i + 1..] {
                if earlier.contains("{}") && !later.contains("{}") {
                    assert!(
                        !matches(earlier, later),
                        "`{earlier}` shadows `{later}`; list the literal first"
                    );
                }
            }
        }
    }

    /// Every `GET` route the controllers register under `/api` is in the table, and every entry
    /// in the table is a real route. This is the test that makes a new route unclassified-by-default
    /// into a failing build (WPM-D71).
    #[test]
    fn every_get_route_is_classified_and_every_entry_is_real() {
        let dir = std::path::Path::new(env!("CARGO_MANIFEST_DIR")).join("src/controllers");
        let mut routes: Vec<String> = Vec::new();
        for entry in std::fs::read_dir(dir).expect("controllers directory") {
            let path = entry.expect("entry").path();
            if path.extension().and_then(|e| e.to_str()) != Some("rs") {
                continue;
            }
            let text = std::fs::read_to_string(&path).expect("controller source");
            // Collapse whitespace so a call formatted over several lines reads as one.
            let flat = text.split_whitespace().collect::<Vec<_>>().join(" ");
            let Some((_, after_prefix)) = flat.split_once(".prefix(\"") else {
                continue;
            };
            let Some((prefix, body)) = after_prefix.split_once('"') else {
                continue;
            };
            if !prefix.starts_with("/api") {
                continue;
            }
            let prefix = prefix.trim_start_matches("/api");
            for call in body.split(".add(").skip(1) {
                let Some(call) = call.trim_start().strip_prefix('"') else {
                    continue;
                };
                let Some((route, rest)) = call.split_once('"') else {
                    continue;
                };
                // The call ends at its closing `))`, so a later `get(` in the file is not this route's.
                let call_text = rest.split("))").next().unwrap_or(rest);
                if call_text.contains("get(") {
                    routes.push(format!("{prefix}{route}"));
                }
            }
        }
        assert!(
            routes.len() > 100,
            "found only {} routes: the scan is broken",
            routes.len()
        );
        let mut unclassified = Vec::new();
        for route in &routes {
            let concrete = route
                .split('/')
                .map(|seg| {
                    if seg.starts_with('{') {
                        "x".to_string()
                    } else {
                        seg.to_string()
                    }
                })
                .collect::<Vec<_>>()
                .join("/");
            if classify_read(&format!("/api{concrete}")).is_none() {
                unclassified.push(route.clone());
            }
        }
        assert!(
            unclassified.is_empty(),
            "GET routes with no entry in rules::access::READS: {unclassified:?}"
        );
        // Every entry is matched by a real route (catches typos and stale entries).
        let concrete: Vec<String> = routes
            .iter()
            .map(|route| {
                route
                    .trim_start_matches('/')
                    .split('/')
                    .map(|seg| if seg.starts_with('{') { "x" } else { seg })
                    .collect::<Vec<_>>()
                    .join("/")
            })
            .collect();
        let stale: Vec<&str> = READS
            .iter()
            .filter(|(pattern, _)| !concrete.iter().any(|c| matches(pattern, c)))
            .map(|(pattern, _)| *pattern)
            .collect();
        assert!(
            stale.is_empty(),
            "entries in READS that match no route: {stale:?}"
        );
    }
}
