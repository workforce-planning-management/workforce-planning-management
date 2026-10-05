//! Hand-written `OpenAPI` 3 description of the WPM REST API.
//!
//! Summary-level by design: every path and verb is present with its
//! request/response essentials; the full field-by-field shapes live in
//! the spec (`../spec/domain-model.md`).

use serde_json::{Value, json};

/// The full `OpenAPI` document, served at `/api-docs/openapi.json`.
#[must_use]
#[allow(clippy::too_many_lines)] // one literal document
pub fn spec() -> Value {
    let ok = |desc: &str| json!({ "200": { "description": desc } });
    let created = json!({
        "200": { "description": "Created: {pid}" },
        "422": { "description": "Validation failure" }
    });
    let transition = json!({
        "200": { "description": "The updated record" },
        "422": { "description": "Illegal transition (names the current state)" }
    });
    json!({
        "openapi": "3.0.3",
        "info": {
            "title": "Workforce Planning Management Service API",
            "version": env!("CARGO_PKG_VERSION"),
            "description": "WPM across the worker lifecycle: requisitions/ATS, onboarding, worker records + org chart, time & attendance, leave, shifts, benefits, reviews, training, succession, payroll runs with derived payslips, benchmarking. Identities are EntityRef URNs (person:/worker:/organization:/course:). Money is minor units + ISO-4217. Validation failures return 422. API version is negotiated with the Accepts-version header (1.0)."
        },
        "paths": {
            "/api/workers": {
                "post": { "tags": ["hr-core"], "summary": "Create a worker (onboarding status; person/organization URNs; salary minor units)", "responses": created },
                "get": { "tags": ["hr-core"], "summary": "List workers (?department=&status=; per-row mask obligation)", "responses": ok("Workers") }
            },
            "/api/workers/{pid}": {
                "get": { "tags": ["hr-core"], "summary": "Fetch a worker (salary read audited; mask honoured)", "responses": ok("Worker") },
                "put": { "tags": ["hr-core"], "summary": "Update employment facts (manager change is cycle-checked)", "responses": ok("Updated worker") },
                "delete": { "tags": ["hr-core"], "summary": "Soft-delete a worker", "responses": ok("Deleted") }
            },
            "/api/workers/{pid}/status": { "post": { "tags": ["hr-core"], "summary": "Worker lifecycle transition (activation gated on mandatory onboarding items)", "responses": transition } },
            "/api/org-chart": { "get": { "tags": ["hr-core"], "summary": "The manager forest for one organization (?organization=)", "responses": ok("OrgNodes") } },
            "/api/benefit-plans": {
                "post": { "tags": ["benefits"], "summary": "Create a benefit plan (minor-unit costs)", "responses": created },
                "get": { "tags": ["benefits"], "summary": "List plans", "responses": ok("Plans") }
            },
            "/api/workers/{pid}/benefit-enrollments": {
                "post": { "tags": ["benefits"], "summary": "Enrol (double-enrolment refused)", "responses": created },
                "get": { "tags": ["benefits"], "summary": "List enrolments", "responses": ok("Enrollments") }
            },
            "/api/benefit-enrollments/{pid}": { "delete": { "tags": ["benefits"], "summary": "Unenrol (soft delete)", "responses": ok("Deleted") } },
            "/api/requisitions": {
                "post": { "tags": ["acquisition"], "summary": "Open a requisition (draft; headcount; salary band)", "responses": created },
                "get": { "tags": ["acquisition"], "summary": "List requisitions (?status=)", "responses": ok("Requisitions") }
            },
            "/api/requisitions/{pid}": { "get": { "tags": ["acquisition"], "summary": "Fetch a requisition", "responses": ok("Requisition") } },
            "/api/requisitions/{pid}/status": { "post": { "tags": ["acquisition"], "summary": "Requisition transition (filled requires hired ≥ headcount)", "responses": transition } },
            "/api/requisitions/{pid}/applications": {
                "post": { "tags": ["acquisition"], "summary": "Apply a candidate (consent-checked)", "responses": created },
                "get": { "tags": ["acquisition"], "summary": "List applications", "responses": ok("Applications") }
            },
            "/api/candidates": {
                "post": { "tags": ["acquisition"], "summary": "Add a candidate (consent_until bounds the pool)", "responses": created },
                "get": { "tags": ["acquisition"], "summary": "The pool (consent-expired excluded; ?expired=1 = purge queue)", "responses": ok("Candidates") }
            },
            "/api/applications/{pid}/stage": { "post": { "tags": ["acquisition"], "summary": "Stage transition; hired creates the worker in the same transaction", "responses": transition } },
            "/api/applications/{pid}/interviews": {
                "post": { "tags": ["acquisition"], "summary": "Schedule an interview (worker: interviewer URN)", "responses": created },
                "get": { "tags": ["acquisition"], "summary": "List interviews", "responses": ok("Interviews") }
            },
            "/api/interviews/{pid}": { "put": { "tags": ["acquisition"], "summary": "Record the outcome (pending|advance|reject)", "responses": ok("Interview") } },
            "/api/workers/{pid}/onboarding": {
                "post": { "tags": ["acquisition"], "summary": "Add checklist items", "responses": created },
                "get": { "tags": ["acquisition"], "summary": "The checklist", "responses": ok("Items") }
            },
            "/api/onboarding-items/{pid}/complete": { "post": { "tags": ["acquisition"], "summary": "Complete an item", "responses": ok("Item") } },
            "/api/onboarding-items/{pid}/waive": { "post": { "tags": ["acquisition"], "summary": "Waive an item (reason required, audited)", "responses": ok("Item") } },
            "/api/workers/{pid}/time-entries": {
                "post": { "tags": ["workforce"], "summary": "Record time (minutes; day capped at 24h)", "responses": created },
                "get": { "tags": ["workforce"], "summary": "Entries + derived per-day overtime (?from=&to=)", "responses": ok("Entries") }
            },
            "/api/time-entries/{pid}/approve": { "post": { "tags": ["workforce"], "summary": "Approve (only approved time feeds payroll)", "responses": ok("Entry") } },
            "/api/workers/{pid}/leave-entitlements": {
                "post": { "tags": ["workforce"], "summary": "Grant an entitlement (kind/year/days)", "responses": created },
                "get": { "tags": ["workforce"], "summary": "Balances", "responses": ok("Entitlements") }
            },
            "/api/workers/{pid}/leave-requests": {
                "post": { "tags": ["workforce"], "summary": "Request leave (annual over-balance 422; sick flags negative)", "responses": created },
                "get": { "tags": ["workforce"], "summary": "List requests", "responses": ok("Requests") }
            },
            "/api/leave-requests/{pid}/approve": { "post": { "tags": ["workforce"], "summary": "Approve (locks the row; decrements the balance in-tx)", "responses": transition } },
            "/api/leave-requests/{pid}/reject": { "post": { "tags": ["workforce"], "summary": "Reject", "responses": transition } },
            "/api/leave-requests/{pid}/cancel": { "post": { "tags": ["workforce"], "summary": "Cancel (approved-cancel restores the balance)", "responses": transition } },
            "/api/shifts": {
                "post": { "tags": ["workforce"], "summary": "Plan a shift", "responses": created },
                "get": { "tags": ["workforce"], "summary": "The rota (?department=&date=; assignments attached)", "responses": ok("Shifts") }
            },
            "/api/shifts/{pid}/assignments": { "post": { "tags": ["workforce"], "summary": "Assign (double-booking and leave-conflict refused)", "responses": created } },
            "/api/workforce/working-time": { "get": { "tags": ["workforce"], "summary": "Advisory working-time guardrails (?department=&as_of=): 17-week 48h average over recorded minutes + 11h rest-gap breaches across recent and planned assignments; flags only, nothing refused (WPM-D19)", "responses": ok("Signals") } },
            "/api/shift-assignments/{pid}": { "delete": { "tags": ["workforce"], "summary": "Unassign (soft delete)", "responses": ok("Deleted") } },
            "/api/review-cycles": {
                "post": { "tags": ["development"], "summary": "Open a review cycle", "responses": created },
                "get": { "tags": ["development"], "summary": "List cycles", "responses": ok("Cycles") }
            },
            "/api/review-cycles/{pid}/reviews": { "post": { "tags": ["development"], "summary": "Open a draft review", "responses": created } },
            "/api/workers/{pid}/reviews": { "get": { "tags": ["development"], "summary": "A worker's reviews (unshared content redacted; shared reads audited)", "responses": ok("Reviews") } },
            "/api/reviews/{pid}": {
                "get": { "tags": ["development"], "summary": "Review detail + goals + feedback (content read audited)", "responses": ok("ReviewDetail") },
                "put": { "tags": ["development"], "summary": "Author edits (drafts only; rating 1-5)", "responses": ok("Review") }
            },
            "/api/reviews/{pid}/status": { "post": { "tags": ["development"], "summary": "Review transition (draft→submitted→calibrated→shared)", "responses": transition } },
            "/api/reviews/{pid}/goals": { "post": { "tags": ["development"], "summary": "Add a weighted goal", "responses": created } },
            "/api/goals/{pid}": { "put": { "tags": ["development"], "summary": "Set goal status (open|met|missed)", "responses": ok("Goal") } },
            "/api/reviews/{pid}/feedback": { "post": { "tags": ["development"], "summary": "Add feedback", "responses": created } },
            "/api/workers/{pid}/training-enrollments": {
                "post": { "tags": ["development"], "summary": "Enrol against a course:/courseinstance: URN", "responses": created },
                "get": { "tags": ["development"], "summary": "List enrolments", "responses": ok("Enrollments") }
            },
            "/api/training-enrollments/{pid}": { "put": { "tags": ["development"], "summary": "Progress (completed stamps completion + certificate expiry)", "responses": ok("Enrollment") } },
            "/api/training/expiring": { "get": { "tags": ["development"], "summary": "Certificates expiring (?within_days=, default 90)", "responses": ok("Expiring") } },
            "/api/succession-plans": {
                "post": { "tags": ["development"], "summary": "Create a plan (criticality 1-5)", "responses": created },
                "get": { "tags": ["development"], "summary": "Plans + ranked candidates (read audited)", "responses": ok("Plans") }
            },
            "/api/succession-plans/gaps": { "get": { "tags": ["development"], "summary": "Criticality ≥ 4 roles with no ready_now candidate", "responses": ok("Gaps") } },
            "/api/succession-plans/{pid}": { "put": { "tags": ["talent"], "summary": "Restate criticality / risk_of_loss / expected vacancy / incumbent", "responses": ok("Plan") } },
            "/api/succession-candidates/{pid}": { "put": { "tags": ["talent"], "summary": "Move a successor's readiness or rank (readiness may go down)", "responses": ok("Candidate") } },
            "/api/succession-plans/{pid}/candidates": { "post": { "tags": ["development"], "summary": "Add a ranked candidate (ready_now|ready_1y|ready_2y)", "responses": created } },
            "/api/payroll-runs": {
                "post": { "tags": ["payroll"], "summary": "Open a draft run (organization URN + period)", "responses": created },
                "get": { "tags": ["payroll"], "summary": "List runs", "responses": ok("Runs") }
            },
            "/api/payroll-runs/{pid}": { "get": { "tags": ["payroll"], "summary": "Fetch a run", "responses": ok("Run") } },
            "/api/payroll-runs/{pid}/calculate": { "post": { "tags": ["payroll"], "summary": "Derive payslips (salary × FTE + approved overtime − deductions; stub tax; net invariant enforced)", "responses": transition } },
            "/api/payroll-runs/{pid}/approve": { "post": { "tags": ["payroll"], "summary": "Approve (immutable thereafter)", "responses": transition } },
            "/api/payroll-runs/{pid}/pay": { "post": { "tags": ["payroll"], "summary": "Mark paid", "responses": transition } },
            "/api/payroll-runs/{pid}/reopen": { "post": { "tags": ["payroll"], "summary": "Reopen calculated → draft", "responses": transition } },
            "/api/payroll-runs/{pid}/payslips": { "get": { "tags": ["payroll"], "summary": "The run's payslips (audited; mask honoured)", "responses": ok("Payslips") } },
            "/api/workers/{pid}/payslips": { "get": { "tags": ["payroll"], "summary": "One worker's payslips (self-service; audited; mask honoured)", "responses": ok("Payslips") } },
            "/api/benchmarks": {
                "post": { "tags": ["compensation"], "summary": "Record a market band (min ≤ median ≤ max)", "responses": created },
                "get": { "tags": ["compensation"], "summary": "List benchmarks", "responses": ok("Benchmarks") }
            },
            "/api/benchmarks/comparison": { "get": { "tags": ["compensation"], "summary": "Workers vs bands: below_min|within|above_max flags only (?organization=; audited)", "responses": ok("Comparison") } },
            "/api/assessment-instruments": {
                "post": { "tags": ["assessments"], "summary": "Add a test to the catalog (category + the scales it reports; a scale must suit the category)", "responses": created },
                "get": { "tags": ["assessments"], "summary": "The catalog (?category=aptitude|personality|psychometric|selection)", "responses": ok("Instruments") }
            },
            "/api/assessments": { "post": { "tags": ["assessments"], "summary": "Schedule a sitting for a candidate or worker (application-linked sittings must match the candidate)", "responses": created } },
            "/api/assessments/{pid}": {
                "get": { "tags": ["assessments"], "summary": "One sitting + its per-scale results (sensitive: mask honoured, unmasked reads audited)", "responses": ok("Assessment") },
                "delete": { "tags": ["assessments"], "summary": "Withdraw a sitting (soft delete)", "responses": ok("Deleted") }
            },
            "/api/assessments/{pid}/status": { "post": { "tags": ["assessments"], "summary": "Lifecycle move; completing requires ≥1 result and derives expires_on from the instrument validity", "responses": transition } },
            "/api/assessments/{pid}/results": { "post": { "tags": ["assessments"], "summary": "Record a scale result (upsert; percentile 0–100, band derived; scale must suit the category)", "responses": created } },
            "/api/assessments/analytics": { "get": { "tags": ["assessments"], "summary": "Aggregate sittings by category/status + band distribution (no individual scores)", "responses": ok("Analytics") } },
            "/api/applications/{pid}/assessments": { "get": { "tags": ["assessments"], "summary": "The hiring view: one application's sittings, outstanding count, results (mask honoured)", "responses": ok("Assessments") } },
            "/api/candidates/{pid}/assessment-profile": { "get": { "tags": ["assessments"], "summary": "A candidate's profile: current reading per scale, gaps, selection suitability", "responses": ok("Profile") } },
            "/api/workers/{pid}/assessment-profile": { "get": { "tags": ["assessments"], "summary": "A worker's profile (authorized + masked at the worker level)", "responses": ok("Profile") } },
            "/api/workers/{pid}/development-plans": {
                "post": { "tags": ["talent"], "summary": "Open an upskill or reskill plan with its skill steps (reskill must name a target role; upskill must not)", "responses": created },
                "get": { "tags": ["talent"], "summary": "The worker's plans with declared AND verified progress", "responses": ok("Plans") }
            },
            "/api/development-plans/{pid}/status": { "post": { "tags": ["talent"], "summary": "draft→active→completed (completing needs every item resolved)", "responses": transition } },
            "/api/development-plan-items/{pid}": { "put": { "tags": ["talent"], "summary": "Move one step's status (claiming achievement does not change declared proficiency)", "responses": ok("Item") } },
            "/api/talent-pipelines": {
                "post": { "tags": ["talent"], "summary": "Open a pipeline (succession|hiring|early_careers|internal_mobility)", "responses": created },
                "get": { "tags": ["talent"], "summary": "Pipelines with health (live pool excludes placed/exited)", "responses": ok("Pipelines") }
            },
            "/api/talent-pipelines/{pid}": { "get": { "tags": ["talent"], "summary": "One pipeline, its health, and its members", "responses": ok("Pipeline") } },
            "/api/talent-pipelines/{pid}/members": { "post": { "tags": ["talent"], "summary": "Add a candidate or worker at the identified stage (one row per subject)", "responses": created } },
            "/api/pipeline-members/{pid}/stage": { "post": { "tags": ["talent"], "summary": "Stage move; ready may regress to developing", "responses": transition } },
            "/api/early-career-programs": {
                "post": { "tags": ["early-careers"], "summary": "Add an apprenticeship / internship / graduate scheme (an apprenticeship must declare its off-the-job hours)", "responses": created },
                "get": { "tags": ["early-careers"], "summary": "The catalog with placement counts + conversion rate (?kind=)", "responses": ok("Programs") }
            },
            "/api/early-career-programs/{pid}/placements": { "post": { "tags": ["early-careers"], "summary": "Place someone on the programme (offered)", "responses": created } },
            "/api/program-placements/{pid}/hours": { "post": { "tags": ["early-careers"], "summary": "Log off-the-job training hours (active placements only; checked add)", "responses": ok("Placement") } },
            "/api/program-placements/{pid}/status": { "post": { "tags": ["early-careers"], "summary": "offered→active→completed; completing an apprenticeship requires its off-the-job hours", "responses": transition } },
            "/api/workers/{pid}/placements": { "get": { "tags": ["early-careers"], "summary": "One person's placements with hours against the requirement", "responses": ok("Placements") } },
            "/api/workforce-intelligence/overview": { "get": { "tags": ["intelligence"], "summary": "Headcount, FTE, tenure buckets, spans of control (?as_of=)", "responses": ok("Overview") } },
            "/api/workforce-intelligence/capability": { "get": { "tags": ["intelligence"], "summary": "Declared skill coverage + gaps, plans in flight, assessment coverage", "responses": ok("Capability") } },
            "/api/workforce-intelligence/capability-analysis": { "get": { "tags": ["intelligence"], "summary": "Strategic skill depth per skill and category (?min_proficiency=&min_depth=)", "responses": ok("CapabilityAnalysis") } },
            "/api/workers/{pid}/emergency-contacts": { "get": { "tags": ["hr-core"], "summary": "A worker's emergency contacts; readable only by the worker and HR", "responses": ok("EmergencyContacts") }, "post": { "tags": ["hr-core"], "summary": "Add an emergency contact (up to 5)", "responses": ok("Pid") } },
            "/api/emergency-contacts/{pid}": { "put": { "tags": ["hr-core"], "summary": "Change an emergency contact", "responses": ok("EmergencyContact") }, "delete": { "tags": ["hr-core"], "summary": "Remove an emergency contact", "responses": ok("Ok") } },
            "/api/workers/{pid}/backups": { "get": { "tags": ["hr-core"], "summary": "Who covers for a worker when they are out, in the order to ask", "responses": ok("Backups") }, "post": { "tags": ["hr-core"], "summary": "Name a backup (up to 3), optionally for a dated window", "responses": ok("Pid") } },
            "/api/backups/{pid}": { "put": { "tags": ["hr-core"], "summary": "Change a backup's rank, window or note", "responses": ok("Backup") }, "delete": { "tags": ["hr-core"], "summary": "Stop naming a backup", "responses": ok("Ok") } },
            "/api/workers/{pid}/cover": { "get": { "tags": ["hr-core"], "summary": "Who covers for a worker on a day (?on=): best-ranked backup in window, employed and not on approved leave", "responses": ok("Cover") } },
            "/api/rotas": { "get": { "tags": ["workforce"], "summary": "On-call rotas in the caller's organizations, with who is on call today", "responses": ok("Rotas") }, "post": { "tags": ["workforce"], "summary": "Create an on-call rota: members in order, a period in days, a start date", "responses": ok("Pid") } },
            "/api/rotas/{pid}": { "get": { "tags": ["workforce"], "summary": "A rota with its schedule as runs, swaps and days-on-call per member (?from=&to=, up to 92 days)", "responses": ok("Rota") }, "put": { "tags": ["workforce"], "summary": "Rename, re-time or re-order a rota", "responses": ok("Pid") }, "delete": { "tags": ["workforce"], "summary": "Retire a rota", "responses": ok("Ok") } },
            "/api/rotas/{pid}/on-call": { "get": { "tags": ["workforce"], "summary": "Who is on call on a day (?on=), skipping members on approved leave", "responses": ok("OnCall") } },
            "/api/rotas/{pid}/overrides": { "post": { "tags": ["workforce"], "summary": "A swap: a worker on call for a date window regardless of the rotation", "responses": ok("Pid") } },
            "/api/rota-overrides/{pid}": { "delete": { "tags": ["workforce"], "summary": "Undo a swap", "responses": ok("Ok") } },
            "/api/workers/{pid}/on-call": { "get": { "tags": ["workforce"], "summary": "A worker's on-call stretches across readable rotas (?from=&to=)", "responses": ok("OnCallRuns") } },
            "/api/rotas/{pid}/swap-requests": { "get": { "tags": ["workforce"], "summary": "A rota's swap requests, newest first", "responses": ok("SwapRequests") }, "post": { "tags": ["workforce"], "summary": "Ask a colleague to take the requester's on-call days in a window", "responses": ok("Pid") } },
            "/api/rota-swap-requests/{pid}/accept": { "post": { "tags": ["workforce"], "summary": "Accept: the requester's own on-call days in the window become the colleague's", "responses": ok("Ok") } },
            "/api/rota-swap-requests/{pid}/decline": { "post": { "tags": ["workforce"], "summary": "Decline a swap request", "responses": ok("Ok") } },
            "/api/rota-swap-requests/{pid}/cancel": { "post": { "tags": ["workforce"], "summary": "Withdraw an open swap request", "responses": ok("Ok") } },
            "/api/workers/{pid}/swap-requests": { "get": { "tags": ["workforce"], "summary": "A person's open swap requests: incoming (asked of them) and outgoing", "responses": ok("SwapRequests") } },
            "/api/announcements": { "get": { "tags": ["hr-core"], "summary": "The announcement feed for the caller's organizations: live posts, pinned first then newest (?organization=&include=all&limit=&offset=; include=all is for editors)", "responses": ok("Announcements") }, "post": { "tags": ["hr-core"], "summary": "Post an announcement (organization editors): title, plain-text body, optional pin, schedule and expiry", "responses": ok("Pid") } },
            "/api/announcements/{pid}": { "get": { "tags": ["hr-core"], "summary": "One announcement", "responses": ok("Announcement") }, "put": { "tags": ["hr-core"], "summary": "Edit an announcement", "responses": ok("Announcement") }, "delete": { "tags": ["hr-core"], "summary": "Retire an announcement", "responses": ok("Ok") } },
            "/api/directory": { "get": { "tags": ["hr-core"], "summary": "Employee directory: employed workers in the caller's organizations, searchable (?q=&department=&limit=&offset=); no salary or dates", "responses": ok("Directory entries") } },
            "/api/workforce-intelligence/metrics": { "get": { "tags": ["intelligence"], "summary": "Shared metric vocabulary: headcount, starters, leavers, turnover, span of control (?from=&to=)", "responses": ok("Metrics") } },
            "/api/workforce-intelligence/insights": { "get": { "tags": ["intelligence"], "summary": "Findings derived from the shared metrics, with suggested next steps and the thresholds used (?from=&to=)", "responses": ok("Insights") } },
            "/api/workforce-intelligence/headcount-history": { "get": { "tags": ["intelligence"], "summary": "Recorded headcount snapshots per organization x department x date (?organization=&from=&to=)", "responses": ok("HeadcountHistory") } },
            "/api/esco/occupations": { "get": { "tags": ["esco"], "summary": "Search the pinned ESCO occupations (?q=&limit=)", "responses": ok("EscoOccupations") } },
            "/api/esco/occupation": { "get": { "tags": ["esco"], "summary": "One ESCO occupation with essential and optional skills (?uri=)", "responses": ok("EscoOccupation") } },
            "/api/esco/skills": { "get": { "tags": ["esco"], "summary": "Search the pinned ESCO skills (?q=&limit=) with catalogue links", "responses": ok("EscoSkills") } },
            "/api/role-profiles/from-esco": { "post": { "tags": ["esco"], "summary": "Draft a role profile from an ESCO occupation at the planner's chosen level", "responses": ok("FromEsco") } },
            "/api/workers/{pid}/role-history": { "get": { "tags": ["career"], "summary": "Every role held in a framework, with start and stop (?framework=)", "responses": ok("RoleHistory") } },
            "/api/workers/{pid}/framework-roles/{framework}/past": { "post": { "tags": ["career"], "summary": "Record a past role with its dates (no overlap within a framework)", "responses": ok("Role") } },
            "/api/workers/{pid}/skill-history": { "get": { "tags": ["career"], "summary": "Timeline of skill levels (?skill_pid=)", "responses": ok("SkillHistory") } },
            "/api/workers/{pid}/skill-history/past": { "post": { "tags": ["career"], "summary": "Record a skill level held in the past, with its dates", "responses": ok("Pid") } },
            "/api/workers/{pid}/skills-as-of": { "get": { "tags": ["career"], "summary": "Skills, levels and roles held at the end of a date (?at=YYYY-MM-DD)", "responses": ok("AsOf") } },
            "/api/workers/{pid}/aspirations": { "get": { "tags": ["career"], "summary": "Aspirations, learning goals and growth ideas; private unless shared", "responses": ok("Aspirations") }, "post": { "tags": ["career"], "summary": "Record a future role or skill target", "responses": ok("Pid") } },
            "/api/workers/{pid}/transfer": { "post": { "tags": ["groups"], "summary": "Move a worker to another organization; group memberships that no longer fit end (kept as history)", "responses": ok("Transfer") } },
            "/api/groups": { "get": { "tags": ["groups"], "summary": "Groups (communities of practice / interest) with member counts", "responses": ok("Groups") }, "post": { "tags": ["groups"], "summary": "Start a group", "responses": ok("Pid") } },
            "/api/groups/{pid}": { "put": { "tags": ["groups"], "summary": "Rename or re-describe a group", "responses": ok("Pid") }, "delete": { "tags": ["groups"], "summary": "Retire a group; membership history stays", "responses": ok("Ok") } },
            "/api/groups/{pid}/skills": { "get": { "tags": ["groups"], "summary": "What the group knows, in aggregate (declared; skills under the floor withheld)", "responses": ok("GroupSkills") } },
            "/api/groups/{pid}/members": { "get": { "tags": ["groups"], "summary": "Members of a group (?include_past=true)", "responses": ok("GroupMembers") }, "post": { "tags": ["groups"], "summary": "A worker joins a group (many allowed) or changes role", "responses": ok("Member") } },
            "/api/groups/{pid}/members/{worker_pid}": { "delete": { "tags": ["groups"], "summary": "Leave a group; the membership is closed, not deleted", "responses": ok("Ok") } },
            "/api/workers/{pid}/groups": { "get": { "tags": ["groups"], "summary": "Every group a worker is in (?include_past=true)", "responses": ok("WorkerGroups") } },
            "/api/workers/{pid}/dotted-line": { "get": { "tags": ["reporting"], "summary": "Dotted-line managers and dotted-line reports (?include_past=true)", "responses": ok("DottedLine") } },
            "/api/workers/{pid}/dotted-line-managers": { "post": { "tags": ["reporting"], "summary": "Give a worker a dotted-line manager (anyone; several allowed)", "responses": ok("DottedLineManager") } },
            "/api/workers/{pid}/dotted-line-managers/{manager_pid}": { "delete": { "tags": ["reporting"], "summary": "End a dotted-line relationship; kept as history", "responses": ok("Ok") } },
            "/api/workers/{pid}/upline": { "get": { "tags": ["reporting"], "summary": "The management chain above a worker, nearest first (level 1 = direct manager)", "responses": ok("Upline") } },
            "/api/workers/{pid}/downline": { "get": { "tags": ["reporting"], "summary": "Everyone below a manager: direct and indirect reports with depth", "responses": ok("Downline") } },
            "/api/workers/{pid}/reports": { "get": { "tags": ["reporting"], "summary": "Direct or indirect reports (?kind=direct|indirect)", "responses": ok("Reports") } },
            "/api/workers/{pid}/downline-aspirations": { "get": { "tags": ["career"], "summary": "A manager's downline (direct and indirect reports) and the aspirations shared with managers or everyone", "responses": ok("DownlineAspirations") } },
            "/api/aspirations/{pid}": { "put": { "tags": ["career"], "summary": "Update status, horizon, target, note, or sharing", "responses": ok("Aspiration") }, "delete": { "tags": ["career"], "summary": "Drop an aspiration", "responses": ok("Ok") } },
            "/api/frameworks/selectable": { "get": { "tags": ["frameworks"], "summary": "Frameworks a person can choose a role in (UK GDAD PCF, ESCO)", "responses": ok("Selectable") } },
            "/api/workers/{pid}/framework-roles": { "get": { "tags": ["frameworks"], "summary": "The worker's current role in each framework", "responses": ok("FrameworkRoles") } },
            "/api/workers/{pid}/framework-roles/{framework}": { "put": { "tags": ["frameworks"], "summary": "Set my current role: a PCF role level (role_profile_pid) or an ESCO occupation (occupation_uri)", "responses": ok("FrameworkRole") }, "delete": { "tags": ["frameworks"], "summary": "Clear my role selection (declared skills stay)", "responses": ok("Ok") } },
            "/api/workers/{pid}/framework-roles/{framework}/skills": { "get": { "tags": ["frameworks"], "summary": "The selected role's skills with my declared level", "responses": ok("RoleSkills") }, "put": { "tags": ["frameworks"], "summary": "Select the skills I have, at my own 1-5 level (null deselects)", "responses": ok("Selected") } },
            "/api/skills/{pid}/usage": { "get": { "tags": ["skills"], "summary": "Where a skill is used, and whether it can be deleted", "responses": ok("SkillUsage") } },
            "/api/skills/{pid}/merge": { "post": { "tags": ["skills"], "summary": "Fold a skill into another, keeping the stronger statement where both appear", "responses": ok("Merged") } },
            "/api/skills/{pid}": { "delete": { "tags": ["skills"], "summary": "Delete a skill nothing uses (else merge it)", "responses": ok("Ok") }, "put": { "tags": ["skills"], "summary": "Rename and/or recategorise a skill (names are unique)", "responses": ok("Pid") } },
            "/api/skills/category-suggestions": { "get": { "tags": ["skills"], "summary": "Keyword-rule category suggestions for skills still 'other' (suggestions only)", "responses": ok("Suggestions") } },
            "/api/skills/category-suggestions/apply": { "post": { "tags": ["skills"], "summary": "Apply the current suggestion to chosen skills", "responses": ok("Applied") } },
            "/api/skills/{pid}/refs": { "post": { "tags": ["skills"], "summary": "Record a skill's reference in an external framework (e.g. an ESCO URI)", "responses": ok("Pid") } },
            "/api/skills/{pid}/refs/{framework_slug}": { "delete": { "tags": ["skills"], "summary": "Remove a skill's reference in a framework", "responses": ok("Ok") } },
            "/api/role-profiles": { "get": { "tags": ["roles"], "summary": "Role profiles with requirement counts", "responses": ok("RoleProfiles") }, "post": { "tags": ["roles"], "summary": "Create a role profile for a job title", "responses": ok("Pid") } },
            "/api/role-profiles/{pid}": { "get": { "tags": ["roles"], "summary": "A profile with its required skills, critical first", "responses": ok("RoleProfile") } },
            "/api/role-profiles/{pid}/requirements": { "put": { "tags": ["roles"], "summary": "Require a skill at a minimum proficiency 1-5 with an importance (upsert)", "responses": ok("Ok") } },
            "/api/role-profiles/{pid}/requirements/{skill_pid}": { "delete": { "tags": ["roles"], "summary": "Drop a requirement", "responses": ok("Ok") } },
            "/api/capability-frameworks": { "get": { "tags": ["roles"], "summary": "Frameworks role profiles were imported from, with attribution, licence and scale", "responses": ok("Frameworks") } },
            "/api/role-profiles/{pid}/progression": { "get": { "tags": ["roles"], "summary": "What changes going up a level in the same role of the same framework", "responses": ok("Progression") } },
            "/api/role-profiles/{pid}/gap": { "get": { "tags": ["roles"], "summary": "Can the workforce staff this role? Per-requirement met / below / undeclared counts over employed workers", "responses": ok("RoleGap") } },
            "/api/workers/{pid}/role-gap": { "get": { "tags": ["roles"], "summary": "One worker's declared proficiency vs a role profile (?role_profile_pid=)", "responses": ok("WorkerRoleGap") } },
            "/api/cpd-requirements": { "get": { "tags": ["cpd"], "summary": "CPD requirements (hours or points per period)", "responses": ok("CpdRequirements") }, "post": { "tags": ["cpd"], "summary": "Define a CPD requirement for a period (optionally one job title)", "responses": ok("Pid") } },
            "/api/workers/{pid}/cpd-entries": { "get": { "tags": ["cpd"], "summary": "A worker's CPD ledger, newest first", "responses": ok("CpdEntries") }, "post": { "tags": ["cpd"], "summary": "Record a CPD activity (amount in units, evidence optional)", "responses": ok("Pid") } },
            "/api/workers/{pid}/cpd-progress": { "get": { "tags": ["cpd"], "summary": "Recorded / verified vs each applicable requirement, plus registration expiry", "responses": ok("CpdProgress") } },
            "/api/workers/{pid}/registrations": { "get": { "tags": ["cpd"], "summary": "Professional registrations with expiry status", "responses": ok("Registrations") }, "post": { "tags": ["cpd"], "summary": "Record a professional registration and its expiry", "responses": ok("Pid") } },
            "/api/cpd-entries/{pid}/verify": { "post": { "tags": ["cpd"], "summary": "Verify an entry's evidence", "responses": ok("CpdEntry") } },
            "/api/cpd-entries/{pid}": { "delete": { "tags": ["cpd"], "summary": "Withdraw an entry (soft-delete)", "responses": ok("Ok") } },
            "/api/cpd/overview": { "get": { "tags": ["cpd"], "summary": "Aggregate: employed workers meeting each requirement; registrations expiring or expired", "responses": ok("CpdOverview") } },
            "/api/workers/{pid}/role-matches": { "get": { "tags": ["mobility"], "summary": "Roles ordered by this worker's own declared-skill fit (self-service; never ranks people)", "responses": ok("RoleMatches") } },
            "/api/workers/{pid}/opportunities": { "get": { "tags": ["mobility"], "summary": "Open requisitions in the worker's organization, with the worker's fit where a profile exists", "responses": ok("Opportunities") } },
            "/api/workers/{pid}/mobility-interests": { "get": { "tags": ["mobility"], "summary": "The worker's own expressed interests", "responses": ok("Interests") }, "post": { "tags": ["mobility"], "summary": "Express interest in a role profile or open requisition", "responses": ok("Pid") } },
            "/api/mobility-interests/{pid}": { "delete": { "tags": ["mobility"], "summary": "Withdraw an interest", "responses": ok("Ok") } },
            "/api/mobility/interest-summary": { "get": { "tags": ["mobility"], "summary": "Aggregate interest per target; never who", "responses": ok("InterestSummary") } },
            "/api/lms/completions": { "post": { "tags": ["lms"], "summary": "Apply a batch of LMS completions (idempotent per external_ref): updates enrollments and lands CPD credit", "responses": ok("LmsResults") } },
            "/api/change-initiatives": { "get": { "tags": ["change"], "summary": "AI / automation change initiatives with roles and skills touched", "responses": ok("Initiatives") }, "post": { "tags": ["change"], "summary": "Open a draft change initiative", "responses": ok("Pid") } },
            "/api/change-initiatives/{pid}": { "get": { "tags": ["change"], "summary": "An initiative with its role impacts and skill shifts", "responses": ok("Initiative") } },
            "/api/change-initiatives/{pid}/status": { "post": { "tags": ["change"], "summary": "draft -> active -> completed | cancelled", "responses": ok("Pid") } },
            "/api/change-initiatives/{pid}/role-impacts": { "put": { "tags": ["change"], "summary": "Record a role as displaced / reshaped / created (upsert)", "responses": ok("Ok") } },
            "/api/change-initiatives/{pid}/role-impacts/{role_profile_pid}": { "delete": { "tags": ["change"], "summary": "Remove a role impact", "responses": ok("Ok") } },
            "/api/change-initiatives/{pid}/skill-shifts": { "put": { "tags": ["change"], "summary": "Record a skill rising or declining (upsert)", "responses": ok("Ok") } },
            "/api/change-initiatives/{pid}/skill-shifts/{skill_pid}": { "delete": { "tags": ["change"], "summary": "Remove a skill shift", "responses": ok("Ok") } },
            "/api/change-initiatives/{pid}/readiness": { "get": { "tags": ["change"], "summary": "Aggregate readiness of the affected workforce (?min_proficiency=); never names individuals", "responses": ok("ChangeReadiness") } },
            "/api/workforce-plans": { "get": { "tags": ["planning"], "summary": "Workforce plans (scenarios) in the caller's organizations (?organization=)", "responses": ok("Plans") }, "post": { "tags": ["planning"], "summary": "Open a draft workforce plan: horizon, rationale, optional attrition assumption (basis points)", "responses": ok("Pid") } },
            "/api/workforce-plans/{pid}": { "get": { "tags": ["planning"], "summary": "A plan with demand lines and objectives", "responses": ok("Plan") } },
            "/api/workforce-plans/{pid}/status": { "post": { "tags": ["planning"], "summary": "draft -> active -> archived (one active plan per organization)", "responses": ok("Pid") } },
            "/api/workforce-plans/{pid}/demand-lines": { "put": { "tags": ["planning"], "summary": "Set planned headcount for a department (optionally a role) at a date (upsert)", "responses": ok("Pid") } },
            "/api/workforce-plans/{pid}/demand-lines/{line_pid}": { "delete": { "tags": ["planning"], "summary": "Remove a demand line", "responses": ok("Ok") } },
            "/api/workforce-plans/{pid}/demand-lines/{line_pid}/objectives": { "put": { "tags": ["planning"], "summary": "Replace the objectives a demand line serves", "responses": ok("Ok") } },
            "/api/workforce-plans/{pid}/objectives": { "post": { "tags": ["planning"], "summary": "Add a strategic objective", "responses": ok("Pid") } },
            "/api/workforce-plans/{pid}/forecast": { "get": { "tags": ["planning"], "summary": "Supply projection, headcount gap and competency gaps per department and date, with assumptions", "responses": ok("Forecast") } },
            "/api/workforce-plans/{pid}/cost": { "get": { "tags": ["planning"], "summary": "Annual cost of hiring to close the gaps vs the plan budget (?currency=); salary-derived, withheld without unmasked read", "responses": ok("PlanCost") } },
            "/api/workforce-plans/{pid}/alignment": { "get": { "tags": ["planning"], "summary": "Share of planned headcount tied to objectives; unresourced objectives; unaligned lines", "responses": ok("Alignment") } },
            "/api/workforce-intelligence/succession": { "get": { "tags": ["intelligence"], "summary": "Bench strength + single points of failure (criticality × risk of loss)", "responses": ok("Succession") } },
            "/api/workforce-intelligence/pipelines": { "get": { "tags": ["intelligence"], "summary": "Pipeline funnel + early-career conversion rates", "responses": ok("Pipelines") } },
            "/api/wellbeing-entitlements": {
                "post": { "tags": ["wellbeing"], "summary": "Add an entitlement rule (kind health|benefit; non-clinical predicates only: age band, departments, job titles; a benefit rule may link a benefit plan; WPM-D17/D18)", "responses": created },
                "get": { "tags": ["wellbeing"], "summary": "The configured entitlement rules (?kind=health|benefit)", "responses": ok("Entitlements") }
            },
            "/api/wellbeing-entitlements/{pid}": {
                "put": { "tags": ["wellbeing"], "summary": "Restate a rule (cohorts change year to year; acknowledgements untouched)", "responses": ok("Entitlement") },
                "delete": { "tags": ["wellbeing"], "summary": "Soft-close a rule (history kept)", "responses": ok("Deleted") }
            },
            "/api/workers/{pid}/wellbeing-prompts": { "get": { "tags": ["wellbeing"], "summary": "The worker's live prompts (self-service, worker-owned; one reminder max per multi-dose course; unknown age fails an age-banded rule; a plan-linked rule is quiet once enrolled)", "responses": ok("Prompts") } },
            "/api/workers/{pid}/wellbeing-acknowledgements": { "post": { "tags": ["wellbeing"], "summary": "Acknowledge a prompt (booked|done|declined|dismissed; a workflow fact, never a vaccination status; audited)", "responses": created } },
            "/api/wellbeing/uptake": { "get": { "tags": ["wellbeing"], "summary": "HR aggregate uptake: counts by response + rate with its terms; no individual appears", "responses": ok("Uptake") } },
            "/api/pulse-surveys": {
                "post": { "tags": ["wellbeing"], "summary": "Open an anonymous pulse survey (name, question, window)", "responses": created },
                "get": { "tags": ["wellbeing"], "summary": "The surveys with their open state", "responses": ok("Surveys") }
            },
            "/api/pulse-surveys/{pid}/responses": { "post": { "tags": ["wellbeing"], "summary": "Submit one anonymous 1-5 score (stored row has no author; actor-less audit; no handle returned; WPM-D20)", "responses": created } },
            "/api/pulse-surveys/{pid}/results": { "get": { "tags": ["wellbeing"], "summary": "K-floored aggregate (k=5): per-department + overall cells, suppressed below the floor (count withheld); counts are responses, not respondents", "responses": ok("Results") } },
            "/api/workers/{pid}/appraisals": {
                "post": { "tags": ["appraisals"], "summary": "Open a draft 360 for the subject (declared competencies; self nomination automatic)", "responses": created },
                "get": { "tags": ["appraisals"], "summary": "The subject's appraisals with nomination/response counts (never content)", "responses": ok("Appraisals") }
            },
            "/api/workers/{pid}/appraisal-requests": { "get": { "tags": ["appraisals"], "summary": "The rater's own pending 360 requests (collecting, nominated, not yet responded; $sub-owned)", "responses": ok("Requests") } },
            "/api/appraisals/{pid}": { "get": { "tags": ["appraisals"], "summary": "Detail: nominations with responded flags -- who responded, never what (WPM-D21)", "responses": ok("Appraisal") } },
            "/api/appraisals/{pid}/nominations": { "post": { "tags": ["appraisals"], "summary": "Invite a rater (draft only; group manager|peer|report; one per rater; max 12)", "responses": created } },
            "/api/appraisals/{pid}/status": { "post": { "tags": ["appraisals"], "summary": "draft -> collecting (needs >= 3 non-self raters) -> shared (stamps shared_on)", "responses": transition } },
            "/api/appraisals/{pid}/responses": { "post": { "tags": ["appraisals"], "summary": "One rater's response: collecting only, nominated only, once per rater, every declared competency scored 1-5 ($sub-owned)", "responses": created } },
            "/api/appraisals/{pid}/report": { "get": { "tags": ["appraisals"], "summary": "Group-floored report (shared only; reads audited): group x competency count+mean, pooled comments; peer/report cells under 3 responses withheld, count included", "responses": ok("Report") } },
            "/api/workers/{pid}/adjustment-requests": {
                "post": { "tags": ["adjustments"], "summary": "Ask for a reasonable adjustment: barrier + impact + change, all required; no diagnosis field exists (WPM-D25)", "responses": created },
                "get": { "tags": ["adjustments"], "summary": "The worker's requests ($sub-owned; masked reads withhold the words; unmasked reads audited)", "responses": ok("Requests") }
            },
            "/api/adjustment-requests/{pid}/status": { "post": { "tags": ["adjustments"], "summary": "Decide: requested -> agreed|declined|withdrawn; agreed -> in_place|withdrawn (practical note; audited; worker notified in-app)", "responses": transition } },
            "/api/workers/{pid}/ergonomic-assessments": {
                "post": { "tags": ["ergonomics"], "summary": "Open a DSE workstation assessment (default checklist when no items given; workstation, never the body -- WPM-D24)", "responses": created },
                "get": { "tags": ["ergonomics"], "summary": "The worker's assessments with items and open-issue counts", "responses": ok("Assessments") }
            },
            "/api/ergonomic-items/{pid}": { "put": { "tags": ["ergonomics"], "summary": "Answer one item (ok|issue + equipment note; open assessments only)", "responses": ok("Item") } },
            "/api/ergonomic-assessments/{pid}/complete": { "post": { "tags": ["ergonomics"], "summary": "Complete (every item must be answered; stamps the date; audited with the issue count)", "responses": transition } },
            "/api/ergonomics/issues": { "get": { "tags": ["ergonomics"], "summary": "Issue-flagged items by department (rota-tier visibility; equipment facts only)", "responses": ok("Issues") } },
            "/api/workers/{pid}/notifications": { "get": { "tags": ["notifications"], "summary": "The worker's in-app notifications, unread first ($sub-owned; reference-only bodies, WPM-D23)", "responses": ok("Notifications") } },
            "/api/notifications/{pid}/read": { "post": { "tags": ["notifications"], "summary": "Mark one notification read (owner-only)", "responses": ok("Notification") } },
            "/api/workers/{pid}/subject-access": { "get": { "tags": ["privacy"], "summary": "Subject-access export: everything WPM holds for this worker, exclusions named (audited; $sub/HR)", "responses": ok("Export") } },
            "/api/workers/{pid}/erase": { "post": { "tags": ["privacy"], "summary": "Erasure-as-anonymisation (WPM-D22): scrub identity + authored text, close appraisals; payroll rows remain; refused while employment is open (destructive)", "responses": transition } },
            "/api/retention": { "get": { "tags": ["privacy"], "summary": "Retention report: soft-deleted rows past the horizon per table + expired-consent candidates (WPM_RETENTION_DAYS, floor 30)", "responses": ok("Report") } },
            "/api/retention/sweep": { "post": { "tags": ["privacy"], "summary": "Hard-delete soft-deleted rows past the horizon; scrub expired-consent candidates (destructive; audited with counts)", "responses": ok("Counts") } },
            "/api/me/organizations": { "get": { "tags": ["organizations"], "summary": "The caller's own organization memberships, every org at once (no switcher; token-derived; signed out sees an empty list, never 401)", "responses": ok("Memberships") } },
            "/api/me/organizations/scope": { "get": { "tags": ["organizations"], "summary": "Every organization the caller can read, expanded through confederation (each membership's org plus every transitive descendant); a flat list of URNs, not membership rows", "responses": ok("Organization refs") } },
            "/api/organization-memberships": {
                "get": { "tags": ["organizations"], "summary": "Admin/HR listing by person (?person_ref=, required)", "responses": ok("Memberships") },
                "post": { "tags": ["organizations"], "summary": "Grant a role in an organization (worker_pid optional -- present for an employment-linked membership, absent for a staff/admin-only grant; duplicate (person, org, role) refused)", "responses": created }
            },
            "/api/organization-memberships/{pid}": { "delete": { "tags": ["organizations"], "summary": "Revoke (soft delete)", "responses": ok("Deleted") } },
            "/api/organization-confederations": {
                "get": { "tags": ["organizations"], "summary": "Direct confederation edges (?parent_organization_ref= or ?child_organization_ref=, at least one required)", "responses": ok("Confederations") },
                "post": { "tags": ["organizations"], "summary": "Declare that a parent organization transitively contains a child organization (self-loops, duplicates, and cycles refused; org_admin of the parent only)", "responses": created }
            },
            "/api/organization-confederations/{pid}": { "delete": { "tags": ["organizations"], "summary": "Revoke one confederation edge (soft delete)", "responses": ok("Deleted") } },
            "/api/audits/recent": { "get": { "tags": ["audit"], "summary": "Recent audit entries", "responses": ok("Audit entries") } },
            "/api/audits": { "get": { "tags": ["audit"], "summary": "Department-scoped trail (?department=&since=)", "responses": ok("Audit entries") } },
            "/api/audits/{entity_pid}": { "get": { "tags": ["audit"], "summary": "One record's audit trail", "responses": ok("Audit entries") } },
            "/api/events/recent": { "get": { "tags": ["events"], "summary": "Recent events (memory ring or outbox)", "responses": ok("Events") } },
            "/metrics.prom": { "get": { "tags": ["ops"], "summary": "Prometheus metrics (public)", "responses": ok("Exposition text") } }
        }
    })
}

#[cfg(test)]
mod tests {
    /// The document parses, declares `OpenAPI` 3, and covers the mounted
    /// API surface (spot-checked against the route table).
    #[test]
    fn spec_shape() {
        let doc = super::spec();
        assert_eq!(doc["openapi"], "3.0.3");
        let paths = doc["paths"].as_object().unwrap();
        for p in [
            "/api/workers",
            "/api/workers/{pid}/status",
            "/api/applications/{pid}/stage",
            "/api/leave-requests/{pid}/approve",
            "/api/payroll-runs/{pid}/calculate",
            "/api/benchmarks/comparison",
            "/api/succession-plans/gaps",
            "/api/assessments/{pid}/results",
            "/api/workers/{pid}/assessment-profile",
            "/api/workers/{pid}/development-plans",
            "/api/talent-pipelines/{pid}/members",
            "/api/early-career-programs/{pid}/placements",
            "/api/program-placements/{pid}/status",
            "/api/workforce-intelligence/succession",
            "/api/wellbeing-entitlements",
            "/api/workers/{pid}/wellbeing-prompts",
            "/api/workers/{pid}/wellbeing-acknowledgements",
            "/api/wellbeing/uptake",
            "/api/workforce/working-time",
            "/api/pulse-surveys",
            "/api/pulse-surveys/{pid}/results",
            "/api/workers/{pid}/appraisals",
            "/api/appraisals/{pid}/responses",
            "/api/appraisals/{pid}/report",
            "/api/workers/{pid}/subject-access",
            "/api/workers/{pid}/erase",
            "/api/retention/sweep",
            "/api/workers/{pid}/notifications",
            "/api/notifications/{pid}/read",
            "/api/workers/{pid}/ergonomic-assessments",
            "/api/ergonomics/issues",
            "/api/workers/{pid}/adjustment-requests",
            "/api/adjustment-requests/{pid}/status",
            "/api/me/organizations",
            "/api/me/organizations/scope",
            "/api/organization-memberships",
            "/api/organization-memberships/{pid}",
            "/api/organization-confederations",
            "/api/organization-confederations/{pid}",
        ] {
            assert!(paths.contains_key(p), "missing {p}");
        }
    }
}
