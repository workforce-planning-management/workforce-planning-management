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
    let mut document = json!({
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
            "/api/workers/possible-duplicates": { "post": { "tags": ["hr-core"], "summary": "Which existing workers might this name (and worker number) be? Read-only; names and worker numbers only; a name alone is review, never match", "responses": ok("Possible duplicates, best first") } },
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
                "post": { "tags": ["acquisition"], "summary": "Open a requisition (draft; headcount; salary band; optional programme thing: URN; basis permanent, fixed_term, contractor or intern with expected_duration_months in calendar months, required for the first two ending bases). A hire carries the basis, and the end date it implies, onto the worker unless told otherwise", "responses": created },
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
                "post": { "tags": ["workforce"], "summary": "Grant an entitlement (kind/year/calendar days)", "responses": created },
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
            "/api/rotas/{pid}": { "get": { "tags": ["workforce"], "summary": "A rota with its schedule as runs, swaps and days-on-call per member (?from=&to=, up to 92 calendar days)", "responses": ok("Rota") }, "put": { "tags": ["workforce"], "summary": "Rename, re-time or re-order a rota", "responses": ok("Pid") }, "delete": { "tags": ["workforce"], "summary": "Retire a rota", "responses": ok("Ok") } },
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
            "/api/announcements/{pid}/read": { "post": { "tags": ["hr-core"], "summary": "Mark an announcement read for a worker (themself or HR); idempotent", "responses": ok("Ok") } },
            "/api/workers/{pid}/announcement-reads": { "get": { "tags": ["hr-core"], "summary": "The announcements this person has read (ids); theirs alone", "responses": ok("Pids") } },
            "/api/workers/{pid}/skill-gaps": { "get": { "tags": ["development"], "summary": "One person's skill gaps, ranked: their current role's requirements, their own targets and (for them alone) aspirations, merged per skill; undeclared = unknown, no shortfall", "responses": ok("SkillGaps") } },
            "/api/workforce-intelligence/skill-gaps": { "get": { "tags": ["intelligence"], "summary": "The workforce's skill gaps ranked by importance × levels short (?department=&limit=); counts only, nobody named, no aspirations", "responses": ok("WorkforceSkillGaps") } },
            "/api/skills/{pid}/courses": { "get": { "tags": ["development"], "summary": "The catalogue courses that build a skill (hours, levels each adds) and the skill's hours-per-level figure", "responses": ok("SkillCourses") }, "post": { "tags": ["development"], "summary": "Add a catalogue course for a skill", "responses": ok("SkillCourse") } },
            "/api/skill-courses/{pid}": { "delete": { "tags": ["development"], "summary": "Take a course off a skill", "responses": ok("Ok") } },
            "/api/skills/{pid}/training-hours": { "put": { "tags": ["development"], "summary": "Set or clear a skill's planning figure: hours to gain one proficiency level", "responses": ok("TrainingHours") } },
            "/api/workers/{pid}/training-plan": { "get": { "tags": ["development"], "summary": "One person's training plan from their skill gaps: courses and hours per gap in priority order, scheduled at a weekly pace (?weekly_hours=&start=); undeclared skills are assess_first, with no hours", "responses": ok("TrainingPlan") } },
            "/api/workforce-intelligence/training-demand": { "get": { "tags": ["intelligence"], "summary": "Training hours the workforce's skill gaps would take, by skill and department (?department=); hours and counts only", "responses": ok("TrainingDemand") } },
            "/api/workers/{pid}/movements": { "get": { "tags": ["hr-core"], "summary": "A person's joiner and leaver records", "responses": ok("Movements") }, "post": { "tags": ["hr-core"], "summary": "Open a joiner or leaver record with its dated checklist (a leaver needs a last day and a reason)", "responses": ok("Pid") } },
            "/api/movements": { "get": { "tags": ["hr-core"], "summary": "Joiner and leaver records in the caller's organizations, soonest first, with checklist progress (?kind=&status=)", "responses": ok("Movements") } },
            "/api/movements/{pid}": { "get": { "tags": ["hr-core"], "summary": "A movement with its dated checklist (done / skipped / overdue / due today / upcoming)", "responses": ok("Movement") } },
            "/api/movements/{pid}/items": { "post": { "tags": ["hr-core"], "summary": "Add an ad-hoc dated checklist item", "responses": ok("Pid") } },
            "/api/movements/{pid}/complete": { "post": { "tags": ["hr-core"], "summary": "Complete the record; refused while checklist items are open or a leaver still holds anything", "responses": ok("Ok") } },
            "/api/movements/{pid}/cancel": { "post": { "tags": ["hr-core"], "summary": "Cancel the record", "responses": ok("Ok") } },
            "/api/movement-items/{pid}/done": { "post": { "tags": ["hr-core"], "summary": "Tick a checklist item off", "responses": ok("Ok") } },
            "/api/movement-items/{pid}/skip": { "post": { "tags": ["hr-core"], "summary": "Set a checklist item aside, with a reason", "responses": ok("Ok") } },
            "/api/movement-items/{pid}/reopen": { "post": { "tags": ["hr-core"], "summary": "Undo a tick or a skip", "responses": ok("Ok") } },
            "/api/movement-items/{pid}/assign": { "post": { "tags": ["hr-core"], "summary": "Say who is responsible for an item", "responses": ok("Ok") } },
            "/api/movements/{pid}/handover": { "get": { "tags": ["hr-core"], "summary": "Everything a leaver still holds as of their last day: direct reports, dotted-line reports, group lead, rota seat and swaps, backup roles, mentorships, shifts, tasks, access", "responses": ok("Handover") }, "post": { "tags": ["hr-core"], "summary": "Reassign (or close) one thing the leaver holds; a direct report needs a new manager, access can only be revoked; audited, new holder notified", "responses": ok("Reassigned") } },
            "/api/movements/{pid}/handover/all": { "post": { "tags": ["hr-core"], "summary": "Hand everything that can be handed over to one successor and revoke access; each item audited, failures reported", "responses": ok("HandedOver") } },
            "/api/movements/{pid}/handover/actions": { "get": { "tags": ["hr-core"], "summary": "The audit trail of the handover, oldest first", "responses": ok("HandoverActions") } },
            "/api/directory": { "get": { "tags": ["hr-core"], "summary": "Employee directory: employed workers in the caller's organizations, searchable (?q=&department=&limit=&offset=); no salary or dates", "responses": ok("Directory entries") } },
            "/api/workers/{pid}/job-level": { "get": { "tags": ["hr-core"], "summary": "The worker's job level on a reference ladder, or null; readable by the worker and HR only", "responses": ok("JobLevel") }, "put": { "tags": ["hr-core"], "summary": "Set the worker's level (framework, level, optional effective_on not in the future); the audit entry names no level", "responses": ok("JobLevel") }, "delete": { "tags": ["hr-core"], "summary": "Clear the worker's level", "responses": ok("Cleared") } },
            "/api/role-profiles/{pid}/grade": { "get": { "tags": ["roles"], "summary": "A role's job level and pay band (either may be null)", "responses": ok("Grade") }, "put": { "tags": ["roles"], "summary": "Set a role's job level and/or pay band; absent parts are cleared; a link made by the editor, never derived", "responses": ok("Grade") }, "delete": { "tags": ["roles"], "summary": "Clear a role's grade", "responses": ok("Cleared") } },
            "/api/workers/{pid}/pay-position": { "get": { "tags": ["hr-core"], "summary": "The worker's band and step on a reference pay scale, what it pays, and when they become eligible for the next step; the worker and HR only", "responses": ok("PayPosition") }, "put": { "tags": ["hr-core"], "summary": "Set the worker's scale, band and step (and step_since, not in the future); the audit entry names none of them", "responses": ok("PayPosition") }, "delete": { "tags": ["hr-core"], "summary": "Clear the worker's pay position", "responses": ok("Cleared") } },
            "/api/workers/{pid}/expense-claims": { "get": { "tags": ["payroll"], "summary": "The worker's expense claims with totals; seen by the claimant, their manager and HR", "responses": ok("ExpenseClaims") }, "post": { "tags": ["payroll"], "summary": "Start a draft expense claim (title, ISO 4217 currency, optional description)", "responses": ok("ExpenseClaim") } },
            "/api/expense-claims": { "get": { "tags": ["payroll"], "summary": "Claims the caller may decide, by status (default submitted): their direct reports' and, for HR, their organization's — never their own", "responses": ok("ExpenseClaims") } },
            "/api/expense-claims/{pid}": { "get": { "tags": ["payroll"], "summary": "One claim with its items, duplicate flags and what the caller may do", "responses": ok("ExpenseClaim") } },
            "/api/expense-claims/{pid}/items": { "post": { "tags": ["payroll"], "summary": "Add an item (date not in the future, category, positive amount) to a draft claim", "responses": ok("Item") } },
            "/api/expense-items/{pid}": { "delete": { "tags": ["payroll"], "summary": "Remove an item from a draft claim", "responses": ok("Removed") } },
            "/api/expense-claims/{pid}/submit": { "post": { "tags": ["payroll"], "summary": "Submit a draft claim (needs an item); tells the manager", "responses": ok("ExpenseClaim") } },
            "/api/expense-claims/{pid}/withdraw": { "post": { "tags": ["payroll"], "summary": "Take a submitted claim back to draft", "responses": ok("ExpenseClaim") } },
            "/api/expense-claims/{pid}/cancel": { "post": { "tags": ["payroll"], "summary": "Cancel a draft or submitted claim", "responses": ok("ExpenseClaim") } },
            "/api/expense-claims/{pid}/approve": { "post": { "tags": ["payroll"], "summary": "Approve a submitted claim — by the manager or HR, never the claimant", "responses": ok("ExpenseClaim") } },
            "/api/expense-claims/{pid}/reject": { "post": { "tags": ["payroll"], "summary": "Reject a submitted claim; a note saying why is required", "responses": ok("ExpenseClaim") } },
            "/api/expense-claims/{pid}/reimburse": { "post": { "tags": ["payroll"], "summary": "Mark an approved claim reimbursed (reimbursed_on, not in the future)", "responses": ok("ExpenseClaim") } },
            "/api/job-levels": { "get": { "tags": ["hr-core"], "summary": "Job-level frameworks the service knows (Google technical levels L3–L11): id, organization, track, source", "responses": ok("Job-level frameworks") } },
            "/api/job-levels/{id}": { "get": { "tags": ["hr-core"], "summary": "One framework with every level: code, title, summary, typical experience and management equivalent where the source states them (no pay)", "responses": ok("LevelFramework") } },
            "/api/job-levels/{id}/levels/{code}": { "get": { "tags": ["hr-core"], "summary": "One level (L5, l5 or 5) with the next level up, or null at the top", "responses": ok("Level") } },
            "/api/pay-scales": { "get": { "tags": ["hr-core"], "summary": "Pay scales the service knows (the national pay scale 2026/27): id, nation, effective date, source circular, band codes", "responses": ok("Pay scales") } },
            "/api/pay-scales/{id}": { "get": { "tags": ["hr-core"], "summary": "One pay scale with every band, step (pence, years until eligible for progression) and allowance", "responses": ok("PayScale") } },
            "/api/pay-scales/{id}/position": { "get": { "tags": ["hr-core"], "summary": "Where a full-time-equivalent salary sits on a band (?band=&salary_minor=), and whether a step up is due (?step=&months_on_step=); stateless, nothing stored", "responses": ok("Position") } },
            "/api/me/contact-details": { "get": { "tags": ["me"], "summary": "The caller's own home address, telephone numbers and personal e-mail, or null; the person is the token's, never a path parameter", "responses": ok("ContactDetails") }, "put": { "tags": ["me"], "summary": "Replace the caller's own contact details (a field left out is cleared; shapes validated, nothing verified); a self-service write allowed past the HR-only write policy; the audit entry says they changed, never what they were", "responses": ok("ContactDetails") }, "delete": { "tags": ["me"], "summary": "Remove them all", "responses": ok("Cleared") } },
            "/api/me/emergency-contacts": { "get": { "tags": ["me"], "summary": "The caller's own emergency contacts, first-to-call first", "responses": ok("Contacts") }, "post": { "tags": ["me"], "summary": "Add one (at most five); a self-service write", "responses": ok("Pid") } },
            "/api/me/emergency-contacts/{pid}": { "put": { "tags": ["me"], "summary": "Change one of the caller's own contacts; anyone else's is a 404", "responses": ok("Contact") }, "delete": { "tags": ["me"], "summary": "Remove one of the caller's own contacts", "responses": ok("Removed") } },
            "/api/me/time-off": { "get": { "tags": ["me"], "summary": "The caller's own allowances, days taken, booked, requested and remaining per kind (?year=), the year's requests and past absence by year, in calendar days; no reason for sick leave, no score; a kind with no allowance has no remaining figure", "responses": ok("TimeOff") } },
            "/api/me/leave-requests": { "post": { "tags": ["me"], "summary": "Request leave for the caller (kind, start_on, end_on, optional reason); the same balance check as HR's route and no reason on sick leave; a person with authority still decides it", "responses": ok("LeaveRequest") } },
            "/api/me/leave-requests/{pid}/cancel": { "post": { "tags": ["me"], "summary": "Cancel one of the caller's own requests while it awaits a decision, or once approved if it has not started (approved days come back); anyone else's is a 404; leave that has started is HR's to change", "responses": ok("LeaveRequest") } },
            "/api/me/resignation": { "get": { "tags": ["me"], "summary": "The caller's latest resignation with the reason they gave, or null", "responses": ok("Resignation") }, "post": { "tags": ["me"], "summary": "Log an intent to resign: a proposed last day that allows the notice (calendar days, default 28, WPM_RESIGNATION_NOTICE_CALENDAR_DAYS) and an optional reason from a closed list; the manager is told that it was logged, not why; nothing ends here", "responses": ok("Resignation") } },
            "/api/me/resignation/withdraw": { "post": { "tags": ["me"], "summary": "Take the resignation back while it awaits acceptance; once accepted only HR can record it as rescinded", "responses": ok("Resignation") } },
            "/api/resignations": { "get": { "tags": ["resignations"], "summary": "Resignations in the caller's organizations (?status=, default logged), soonest first; privileged callers only; never the reason", "responses": ok("Resignations") } },
            "/api/resignations/summary": { "get": { "tags": ["resignations"], "summary": "Resignations by department and, where every cell reaches the floor, by reason; small groups are withheld; privileged callers only", "responses": ok("ResignationSummary") } },
            "/api/resignations/{pid}/accept": { "post": { "tags": ["resignations"], "summary": "A person accepts it: records the agreed last day (default the proposed one, not before it was logged), opens the leaver process and tells the worker, in one transaction; employment is not ended by this", "responses": ok("Accepted") } },
            "/api/resignations/{pid}/rescind": { "post": { "tags": ["resignations"], "summary": "A person records that an accepted resignation was set aside by agreement; the open leaver record is cancelled", "responses": ok("Resignation") } },
            "/api/workers/{pid}/resignation": { "get": { "tags": ["resignations"], "summary": "For the worker's line managers and HR: whether a resignation is open and its dates; never the reason", "responses": ok("Resignation") } },
            "/api/me/flexible-working": { "get": { "tags": ["me"], "summary": "The caller's own flexible working requests, newest first, each with its decide-by date, whether it is overdue, the decision, whether an appeal is still open, and the hours HR is to apply", "responses": ok("FlexibleRequests") }, "post": { "tags": ["me"], "summary": "Ask for a different working arrangement (kind, proposed_start not in the past, optional proposed_fte_percent, the worker's own words on the effect and how it might be managed); the software computes the decide-by date (calendar months, default 2) and limits requests (default 2 in 12 calendar months); a person decides", "responses": ok("FlexibleRequest") } },
            "/api/me/flexible-working/{pid}/withdraw": { "post": { "tags": ["me"], "summary": "Withdraw while waiting or while a counter-proposal is open", "responses": ok("FlexibleRequest") } },
            "/api/me/flexible-working/{pid}/respond": { "post": { "tags": ["me"], "summary": "Accept or decline a counter-proposal (response: accept or decline)", "responses": ok("FlexibleRequest") } },
            "/api/me/flexible-working/{pid}/appeal": { "post": { "tags": ["me"], "summary": "Appeal a refusal, once, within the window (default 14 calendar days)", "responses": ok("FlexibleRequest") } },
            "/api/flexible-working": { "get": { "tags": ["flexible-working"], "summary": "Requests the caller may decide (?status=&overdue=): a manager's reports' anywhere down the chain, or HR's; soonest decide-by date first; overdue ones flagged", "responses": ok("FlexibleRequests") } },
            "/api/flexible-working/{pid}/decide": { "post": { "tags": ["flexible-working"], "summary": "Approve (optional trial_until), refuse (a reason from the deployer's list) or counter (counter_note, optional counter_fte_percent); never the worker's own request; an agreed change of hours is a proposal for HR to apply, the contract is not changed here", "responses": ok("FlexibleRequest") } },
            "/api/flexible-working/{pid}/appeal-decision": { "post": { "tags": ["flexible-working"], "summary": "Uphold (the request is approved) or dismiss an appeal; decided by someone other than who refused", "responses": ok("FlexibleRequest") } },
            "/api/workers/{pid}/flexible-working": { "get": { "tags": ["flexible-working"], "summary": "A worker's requests, for the worker, their line managers and HR", "responses": ok("FlexibleRequests") } },
            "/api/me/equality-monitoring": { "get": { "tags": ["me"], "summary": "Whether equality monitoring is on (it is off until the deployer records a lawful basis), the categories on offer (each with prefer_not_to_say), the deployer's stated basis and the caller's own answers; only the worker can read their own", "responses": ok("EqualityMonitoring") }, "put": { "tags": ["me"], "summary": "Declare or change answers for the categories given (voluntary); 404 while monitoring is off; the audit entry says that answers changed, never which or what", "responses": ok("EqualityMonitoring") }, "delete": { "tags": ["me"], "summary": "Take back every answer", "responses": ok("EqualityMonitoring") } },
            "/api/me/equality-monitoring/{category}": { "delete": { "tags": ["me"], "summary": "Take back one answer", "responses": ok("EqualityMonitoring") } },
            "/api/equality-monitoring/summary": { "get": { "tags": ["equality-monitoring"], "summary": "One category across the caller's organizations (?category=): completeness (names no value), the organization's counts with cells below the floor withheld, and a department breakdown only when every department and cell reaches the floor; privileged callers only; the read is audited; 404 while monitoring is off", "responses": ok("EqualitySummary") } },
            "/api/me/health-requirements": { "get": { "tags": ["me"], "summary": "Each workplace health requirement that applies to the caller with their own standing, dates and whether a reminder is due; off (enabled false) unless the deployer records a lawful basis; only they and occupational health see the detail", "responses": ok("HealthRequirements") } },
            "/api/health-requirements": { "get": { "tags": ["health-requirements"], "summary": "What is required, and of whom (not personal data); 404 while the feature is off", "responses": ok("HealthRequirementList") }, "post": { "tags": ["health-requirements"], "summary": "Define a requirement: name, description (what is required, not why), departments (empty is everyone), recheck_calendar_days, reminder_calendar_days; HR", "responses": ok("HealthRequirement") } },
            "/api/health-requirements/{pid}": { "put": { "tags": ["health-requirements"], "summary": "Change a requirement (HR)", "responses": ok("HealthRequirement") }, "delete": { "tags": ["health-requirements"], "summary": "Soft-delete a requirement (HR)", "responses": ok("HealthRequirement") } },
            "/api/health-requirements/clearance": { "get": { "tags": ["health-requirements"], "summary": "For each worker the caller manages (or HR, in their organizations): cleared or not cleared per requirement that applies (?requirement=&not_cleared=). The status is never shown", "responses": ok("Clearance") } },
            "/api/health-requirements/records": { "get": { "tags": ["health-requirements"], "summary": "(?worker=) The detail for one worker; occupational health only; each read is audited", "responses": ok("HealthRecords") } },
            "/api/health-requirements/compliance": { "get": { "tags": ["health-requirements"], "summary": "Per requirement and department, people it applies to and how many are cleared; small groups withheld; HR and occupational health", "responses": ok("Compliance") } },
            "/api/workers/{pid}/health-requirements/{req_pid}": { "put": { "tags": ["health-requirements"], "summary": "Record a worker's status against a requirement (status up_to_date, exempt_recorded or declined; recorded_on; next_due): occupational health only, past the HR-only write policy by the token's occupational_health attribute; no reason is asked for or held; the response says only whether the worker is cleared", "responses": ok("Cleared") }, "delete": { "tags": ["health-requirements"], "summary": "Remove an entry made by mistake (occupational health only)", "responses": ok("Removed") } },
            "/api/workers/{pid}/contact-details": { "get": { "tags": ["me"], "summary": "A worker's contact details for HR and payroll (and the worker); each read by someone else is audited without values", "responses": ok("ContactDetails") } },
            "/api/workers/{pid}/engagement": { "get": { "tags": ["engagements"], "summary": "The worker's basis, engagement end date (the last day), where it stands (open_ended, missing_end_date, running, ending_soon, past_end_undecided, past_end_decided), the extensions and the decisions so far; the worker and HR only", "responses": ok("Engagement") } },
            "/api/workers/{pid}/engagement/extensions": { "post": { "tags": ["engagements"], "summary": "Move the end date later (new_end, optional reason); kept as a dated entry; the audit entry carries no reason", "responses": ok("Extension") } },
            "/api/workers/{pid}/engagement/decision": { "post": { "tags": ["engagements"], "summary": "Record what a person decides about an engagement that is ending (decision: extend, convert or end), against the end date it settles; the worker's line manager or a privileged caller, never the worker; it records a decision and changes nothing else; the audit entry does not carry the decision", "responses": ok("Decision") } },
            "/api/workers/{pid}/engagement/status-assessments": { "get": { "tags": ["engagements"], "summary": "A contractor's employment-status assessments, newest first", "responses": ok("Assessments") }, "post": { "tags": ["engagements"], "summary": "Record an assessment (outcome contractor, employee or undetermined; assessed_on not in the future); the reviewer is the caller", "responses": ok("Pid") } },
            "/api/workers/{pid}/contractor-details": { "get": { "tags": ["engagements"], "summary": "A contractor's supplier, route and rate, or null; the rate is masked like salary and an unmasked read is audited", "responses": ok("ContractorDetails") }, "put": { "tags": ["engagements"], "summary": "Set supplier_ref (organization URN), route (agency, own_company, statement_of_work, direct) and a rate (rate_minor, rate_currency, rate_basis day or hour, all together or none); contractors only; never echoes the rate", "responses": ok("ContractorDetails") }, "delete": { "tags": ["engagements"], "summary": "Clear the details", "responses": ok("Cleared") } },
            "/api/workers/{pid}/conversion-plans": { "get": { "tags": ["engagements"], "summary": "A worker's conversion plans, oldest first, with flags (past_end_without_approved_plan, review_due); HR only; the read is audited", "responses": ok("ConversionPlans") }, "post": { "tags": ["engagements"], "summary": "Propose a plan (intent convert, extend, end or undecided; target_on before the contract end; a conversion needs post_funding_kind or a reason; no pay figure); the worker's line manager or a privileged caller, never the worker; one plan is open at a time; the audit entry carries no reason", "responses": ok("ConversionPlan") } },
            "/api/workforce-intelligence/contingent": { "get": { "tags": ["engagements"], "summary": "The share of FTE that is contractor and that is fixed-term (each apart) and the funding horizon for the next twelve months: people whose funding ends in each month, or already has. Aggregate only; groups below the floor withheld, no total given; HR only; the read is audited", "responses": { "200": { "description": "The shares and the horizon", "content": { "application/json": { "schema": { "$ref": "#/components/schemas/ContingentWorkforce" } } } } } } },
            "/api/workforce-intelligence/programmes": { "get": { "tags": ["engagements"], "summary": "Per programme and month for twelve months: FTE funded, the contractor and fixed-term share of it, engagements ending and funding ending. Aggregate only; a programme with fewer people than the floor is left out, a figure for one to nine is withheld, no total across programmes; HR only; the read is audited", "responses": { "200": { "description": "The programmes and months", "content": { "application/json": { "schema": { "$ref": "#/components/schemas/ProgrammeRollup" } } } } } } },
            "/api/workers/{pid}/funding": { "get": { "tags": ["engagements"], "summary": "Where the worker's post is funded from (kind, programme, end date; never an amount) beside the contract end, and how they compare (matched, contract_outlasts_funding, funding_outlasts_contract, funding_open_ended, unknown); HR only", "responses": { "200": { "description": "The funding beside the contract end", "content": { "application/json": { "schema": { "$ref": "#/components/schemas/PostFunding" } } } } } }, "put": { "tags": ["engagements"], "summary": "Set the funding (kind core, time_limited, external or programme; time_limited and external need ends_on; optional programme_ref thing: URN)", "requestBody": { "required": true, "content": { "application/json": { "schema": { "$ref": "#/components/schemas/PostFundingPayload" } } } }, "responses": { "200": { "description": "The funding beside the contract end", "content": { "application/json": { "schema": { "$ref": "#/components/schemas/PostFunding" } } } } } }, "delete": { "tags": ["engagements"], "summary": "Clear it", "responses": { "200": { "description": "The funding beside the contract end", "content": { "application/json": { "schema": { "$ref": "#/components/schemas/PostFunding" } } } } } } },
            "/api/conversion-plans": { "get": { "tags": ["engagements"], "summary": "Open plans and engagements past their end with no approved plan, with flags (?organization=); HR only; names workers", "responses": ok("ConversionPlans") } },
            "/api/conversion-plans/{pid}/approve": { "post": { "tags": ["engagements"], "summary": "Approve a proposed plan; never the proposer; also records the matching engagement decision against the current end date in the same transaction", "responses": ok("ConversionPlan") } },
            "/api/conversion-plans/{pid}/abandon": { "post": { "tags": ["engagements"], "summary": "Drop a proposed or approved plan; it stays as history", "responses": ok("ConversionPlan") } },
            "/api/conversion-plans/{pid}/done": { "post": { "tags": ["engagements"], "summary": "Carry out an approved conversion: the worker becomes permanent and the contract end is cleared in one transaction; the plan, extensions and decisions are kept", "responses": ok("ConversionPlan") } },
            "/api/engagements/contingent-list": { "get": { "tags": ["engagements"], "summary": "The people behind one figure of the contingent view, by worker number, department, basis and dates (?list=ending_within_30, ending_31_to_60, ending_61_to_90, past_end_undecided, much_extended or long_running_contractors); no pay, no floor; HR only; the read is audited", "responses": { "200": { "description": "The list and the people on it" } } } },
            "/api/engagements/missing-end-date": { "get": { "tags": ["engagements"], "summary": "Fixed-term and contractor workers with no end date recorded (?organization=); none is ever invented", "responses": ok("MissingEndDate") } },
            "/api/skills": { "get": { "tags": ["learning"], "summary": "The skills catalogue", "responses": ok("Skills") }, "post": { "tags": ["learning"], "summary": "Add a skill (name, category)", "responses": ok("Skill") } },
            "/api/workers/{pid}/skills": { "get": { "tags": ["learning"], "summary": "A worker's declared skills and proficiency; seen by the worker, their line managers and HR", "responses": ok("WorkerSkills") }, "put": { "tags": ["learning"], "summary": "Declare a skill and proficiency", "responses": ok("WorkerSkill") } },
            "/api/learning-paths": { "get": { "tags": ["learning"], "summary": "Learning paths", "responses": ok("LearningPaths") }, "post": { "tags": ["learning"], "summary": "Create a learning path", "responses": ok("LearningPath") } },
            "/api/learning-paths/{pid}/enrollments": { "post": { "tags": ["learning"], "summary": "Enrol a worker on a path", "responses": ok("Enrollment") } },
            "/api/learning-paths/{pid}/progress": { "get": { "tags": ["learning"], "summary": "Progress on a path; privileged callers only", "responses": ok("PathProgress") } },
            "/api/mentorships": { "post": { "tags": ["learning"], "summary": "Start a mentorship", "responses": ok("Mentorship") } },
            "/api/mentorships/{pid}": { "get": { "tags": ["learning"], "summary": "One mentorship; privileged callers only", "responses": ok("Mentorship") } },
            "/api/mentorships/{pid}/status": { "post": { "tags": ["learning"], "summary": "Move a mentorship to a new status", "responses": ok("Mentorship") } },
            "/api/mentorships/{pid}/sessions": { "post": { "tags": ["learning"], "summary": "Log a mentoring session", "responses": ok("Session") } },
            "/api/learning/skills-matrix": { "get": { "tags": ["learning"], "summary": "Coverage over declared proficiencies; aggregate", "responses": ok("SkillsMatrix") } },
            "/api/learning/mentorship-overview": { "get": { "tags": ["learning"], "summary": "Mentorships in aggregate", "responses": ok("MentorshipOverview") } },
            "/api/learning/training-analytics": { "get": { "tags": ["learning"], "summary": "Training in aggregate", "responses": ok("TrainingAnalytics") } },
            "/api/skill-pools": { "get": { "tags": ["capacity"], "summary": "Skill pools in the caller's organizations (?organization=) with their members; a pool groups role profiles and skills into a plannable unit", "responses": ok("SkillPools") }, "post": { "tags": ["capacity"], "summary": "Create a pool (organization_ref, name unique per organization, operations_reservation_bp 0-10000: the share of its FTE kept for running live services)", "responses": ok("Pid") } },
            "/api/skill-pools/{pid}": { "get": { "tags": ["capacity"], "summary": "One pool with its members", "responses": ok("SkillPool") }, "put": { "tags": ["capacity"], "summary": "Rename a pool or change its reservation", "responses": ok("Pid") }, "delete": { "tags": ["capacity"], "summary": "Soft-delete a pool; it leaves every view", "responses": ok("Pid") } },
            "/api/skill-pools/{pid}/members": { "post": { "tags": ["capacity"], "summary": "Add a member: a role_profile_pid, or a skill_pid with min_proficiency 1-5 (never both)", "responses": ok("Pid") } },
            "/api/skill-pools/{pid}/members/{member_pid}": { "delete": { "tags": ["capacity"], "summary": "Remove a member (soft delete)", "responses": ok("Pid") } },
            "/api/programme-demands": { "get": { "tags": ["capacity"], "summary": "Programme claims on pools (?pool=&programme=): FTE in hundredths per month", "responses": ok("ProgrammeDemands") }, "put": { "tags": ["capacity"], "summary": "Set a programme's claim on a pool for a month (programme_ref is a thing: URN owned by project-portfolio-management, month is a first-of-month, fte_centi 1-1000000); status may be proposed or closed, never active: a claim becomes active through a recorded start decision", "responses": ok("Pid") } },
            "/api/programme-demands/{pid}": { "delete": { "tags": ["capacity"], "summary": "Remove a claim (soft delete)", "responses": ok("Pid") } },
            "/api/partner-commitments": { "get": { "tags": ["capacity"], "summary": "Partner capacity per pool and month (?pool=&partner=); aggregate FTE, never partner staff", "responses": ok("PartnerCommitments") }, "put": { "tags": ["capacity"], "summary": "Record a partner organization's capacity for a pool and month as requested, committed or confirmed; a month with none is unknown, not zero", "responses": ok("Pid") } },
            "/api/partner-commitments/{pid}": { "delete": { "tags": ["capacity"], "summary": "Remove a commitment (soft delete)", "responses": ok("Pid") } },
            "/api/capacity": { "get": { "tags": ["capacity"], "summary": "Pool x month supply against demand (?organization=&from=&months=, default 12, at most 36): supply is employed FTE by available days less approved leave and the operations reservation; flags over-commitment, names the constraint pool, shows each partner (unknown where silent) and the derivation", "responses": ok("Capacity") } },
            "/api/capacity/settings": { "get": { "tags": ["capacity"], "summary": "The organization's limit on active programmes drawing on the constraint pool (?organization=)", "responses": ok("CapacitySettings") }, "put": { "tags": ["capacity"], "summary": "Set or clear the work-in-progress limit", "responses": ok("CapacitySettings") } },
            "/api/capacity/start-check": { "post": { "tags": ["capacity"], "summary": "Can this programme start? Checks its proposed claims (or what-if lines; stores nothing) against every pool: failing months and by how much, the earliest delay that fits, and the work-in-progress limit. A suggestion with its derivation; a person decides", "responses": ok("StartCheck") } },
            "/api/capacity/start-decisions": { "get": { "tags": ["capacity"], "summary": "Recorded start decisions (?programme=&organization=), newest first", "responses": ok("StartDecisions") }, "post": { "tags": ["capacity"], "summary": "Record a person's decision (proceed or defer) with a required reason; the check is re-run on the server; proceed makes the programme's proposed claims active; the audit entry does not carry the reason", "responses": ok("StartDecision") } },
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
            "/api/workforce-plans/{pid}/demand-lines": { "put": { "tags": ["planning"], "summary": "Set planned headcount for a department (optionally a role) at a date, with an optional FTE target (target_fte_percent; not stated is unknown), basis (permanent, fixed_term, contractor or intern; expected_ends_on required for the first two) and programme (a thing: URN). Upsert on department, role, date, basis and programme", "responses": ok("Pid") } },
            "/api/workforce-plans/{pid}/demand-lines/{line_pid}": { "delete": { "tags": ["planning"], "summary": "Remove a demand line", "responses": ok("Ok") } },
            "/api/workforce-plans/{pid}/demand-lines/{line_pid}/objectives": { "put": { "tags": ["planning"], "summary": "Replace the objectives a demand line serves", "responses": ok("Ok") } },
            "/api/workforce-plans/{pid}/objectives": { "post": { "tags": ["planning"], "summary": "Add a strategic objective", "responses": ok("Pid") } },
            "/api/workforce-plans/{pid}/forecast": { "get": { "tags": ["planning"], "summary": "Supply projection (each basis on its own terms), headcount and FTE gap, demand by basis and by programme, and competency gaps per department and date, with assumptions", "responses": ok("Forecast") } },
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
            "/api/retention": { "get": { "tags": ["privacy"], "summary": "Retention report: soft-deleted rows past their kind's horizon per table + expired-consent candidates (horizons per record kind, floor 30 calendar days)", "responses": ok("Report") } },
            "/api/retention/schedule": { "get": { "tags": ["privacy"], "summary": "Retention schedule: each record kind, its tables, its horizon in calendar days and where it came from (default, WPM_RETENTION_DAYS or WPM_RETENTION_<KIND>_DAYS)", "responses": ok("Schedule") } },
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
            "/api/audits/verify": { "get": { "tags": ["audit"], "summary": "Check the audit trail's hash chain: ok, the number of entries, the head hash (record it elsewhere) and the first entry that does not match; privileged callers only", "responses": ok("ChainReport") } },
            "/api/audits/{entity_pid}": { "get": { "tags": ["audit"], "summary": "One record's audit trail", "responses": ok("Audit entries") } },
            "/api/events/recent": { "get": { "tags": ["events"], "summary": "Recent events (memory ring or outbox)", "responses": ok("Events") } },
            "/metrics.prom": { "get": { "tags": ["ops"], "summary": "Prometheus metrics (public)", "responses": ok("Exposition text") } }
        }
    });
    add_governance_paths(&mut document);
    decorate(&mut document);
    document
}

/// The governance-load paths (WPM-R64), added here because the literal above is at the macro's
/// recursion limit.
fn add_governance_paths(document: &mut Value) {
    let ok = |what: &str| json!({ "200": { "description": what } });
    let paths = json!({
        "/api/oversight/packs": { "get": { "tags": ["capacity"], "summary": "The frozen monthly packs of an organization (?organization=), newest first, without their payloads", "responses": ok("The packs") }, "post": { "tags": ["capacity"], "summary": "Freeze the oversight position, criteria, measures and capacity view as of today; written once, never changed", "responses": ok("The pack") } },
        "/api/oversight/packs/{pid}/withdraw": { "post": { "tags": ["capacity"], "summary": "Withdraw a mistaken pack with a reason; the pack is not changed or removed and every reader sees the mark. For an administrator of the organization", "responses": ok("The pack with its mark") } },
        "/api/oversight/packs/{pid}": { "get": { "tags": ["capacity"], "summary": "One frozen pack, the same payload for every audience (aggregates only; a partner-role caller may read this and nothing else)", "responses": ok("The pack") } },
        "/api/cutovers": { "get": { "tags": ["capacity"], "summary": "The cutovers of an organization (?organization=): a programme's go-live at a site over a window of dates", "responses": ok("The cutovers") }, "post": { "tags": ["capacity"], "summary": "Add a cutover: programme, site, window and optionally the on-call rota for hypercare", "responses": ok("The cutover") } },
        "/api/cutovers/{pid}": { "delete": { "tags": ["capacity"], "summary": "Remove a cutover with its roles and pools", "responses": ok("The pid removed") } },
        "/api/cutovers/{pid}/roles": { "post": { "tags": ["capacity"], "summary": "A role that must be staffed at the site, with the course that trains it and the headcount", "responses": ok("The role") } },
        "/api/cutovers/{pid}/pools": { "post": { "tags": ["capacity"], "summary": "A skill pool the cutover depends on", "responses": ok("The pool") } },
        "/api/cutovers/{pid}/readiness": { "get": { "tags": ["capacity"], "summary": "The people side of the go-live: trained staff per role, hypercare cover and key-person risk, each ready, at_risk or unknown with reasons, and the worst of them. The go/no-go stays a human decision", "responses": ok("The readiness") } },
        "/api/early-warnings": { "get": { "tags": ["capacity"], "summary": "The early warnings of an organization (?organization=), newest first, with days open, days to acknowledge and the time to report (the longest, null while any is unacknowledged)", "responses": ok("The warnings") } },
        "/api/early-warnings/sweep": { "post": { "tags": ["capacity"], "summary": "Look now (?organization=): open a warning for each pool over-committed in the next six months or measure in breach for the last complete month, close those that have stopped, and notify the HR administrators", "responses": ok("What the sweep did") } },
        "/api/early-warnings/{pid}/acknowledge": { "post": { "tags": ["capacity"], "summary": "A person acknowledges a warning, once; the calendar days since it was first seen are the time to report", "responses": ok("The warning") } },
        "/api/oversight/derivations": { "get": { "tags": ["capacity"], "summary": "The computations a derived measure may name, each with its unit and its definition: what is counted, what is left out, the period and when it is absent", "responses": ok("The derivations") } },
        "/api/oversight/status": { "get": { "tags": ["capacity"], "summary": "The organization's own scale, recorded positions and, for the criteria of the level it is at, whether each sustain rule is met as of the last complete month (?organization=). Evidence, never a verdict: the service does not change a level", "responses": ok("The oversight status") } },
        "/api/oversight/levels": { "post": { "tags": ["capacity"], "summary": "Add a level to the organization's scale (ordinal 1 to 20, a name)", "responses": ok("The level") } },
        "/api/oversight/levels/{pid}": { "delete": { "tags": ["capacity"], "summary": "Remove a level that has no position and no criterion", "responses": ok("The pid removed") } },
        "/api/oversight/positions": { "post": { "tags": ["capacity"], "summary": "Record the organization's position: the level, the effective date, the deciding body (free text) and the published reason", "responses": ok("The position") } },
        "/api/oversight/criteria": { "post": { "tags": ["capacity"], "summary": "Add an exit criterion to a level", "responses": ok("The criterion") } },
        "/api/oversight/criteria/{pid}": { "delete": { "tags": ["capacity"], "summary": "Remove a criterion with its measures and readings", "responses": ok("The pid removed") } },
        "/api/oversight/measures": { "post": { "tags": ["capacity"], "summary": "Add a measure to a criterion (origin reported, or derived with a derivation from GET /api/oversight/derivations; threshold, direction at_most or at_least, sustain 1 to 120 months)", "responses": ok("The measure") } },
        "/api/oversight/measures/{pid}": { "delete": { "tags": ["capacity"], "summary": "Remove a measure with its readings", "responses": ok("The pid removed") } },
        "/api/cpd-requirements/{pid}": { "put": { "tags": ["cpd"], "summary": "Correct a CPD requirement: name, amount, period, job title. The unit cannot change; entries are untouched and are counted afresh", "responses": ok("The pid") }, "delete": { "tags": ["cpd"], "summary": "Withdraw a CPD requirement (soft delete, audited)", "responses": ok("Withdrawn") } },
        "/api/workforce-plans/{pid}/tolerances": { "get": { "tags": ["planning"], "summary": "The plan's tolerances in time, cost, resources and risk: for each, the deviation now, the limit, and red, amber, green, unknown or not_set, with its basis; and the worst of them. An unknown is never green. Nothing is escalated", "responses": ok("The tolerances") } },
        "/api/workforce-plans/{pid}/tolerances/{dimension}": { "put": { "tags": ["planning"], "summary": "Set or change a tolerance: limit (0 or more, in calendar days, basis points or roles) and optionally amber_percent (1 to 99, default 75). Audited", "responses": ok("The tolerances") }, "delete": { "tags": ["planning"], "summary": "Remove a tolerance. Audited", "responses": ok("Removed") } },
        "/api/workforce-plans/{pid}/escalations": { "get": { "tags": ["planning"], "summary": "A plan's escalations, open first then decided, with the measure at each step", "responses": ok("The escalations") }, "post": { "tags": ["planning"], "summary": "Take an amber or red dimension to level 1 (plan_owner): dimension and an optional note of at most 500 characters, with no personal details. One may be open per dimension. Notifies the organization's administrators", "responses": ok("The escalation") } },
        "/api/escalations": { "get": { "tags": ["planning"], "summary": "The open escalations of an organization (?organization=), oldest first, with calendar days open", "responses": ok("The open escalations") } },
        "/api/escalations/{pid}/raise-level": { "post": { "tags": ["planning"], "summary": "One level up: plan_owner, board, executive. Notifies the administrators", "responses": ok("The escalation") } },
        "/api/escalations/{pid}/decide": { "post": { "tags": ["planning"], "summary": "An administrator of the organization closes it: decision accept_deviation, change_plan or stop, and a required note. Records the plan's status and deviation at that moment; changes neither the tolerance nor the plan", "responses": ok("The escalation") } },
        "/api/audits/export": { "get": { "tags": ["audit"], "summary": "The audit trail as JSON Lines for a security monitoring system: oldest first, resumable (?after=<chain_seq>&limit=), no snapshots; X-Next-After names the last chain_seq. Privileged callers only", "responses": ok("Audit entries, one JSON object per line") } },
        "/api/oversight/measures/{pid}/readings": { "put": { "tags": ["capacity"], "summary": "Enter or replace a month's reading of a reported measure (a derived one is refused); a value needs an evidence note and a source document; null says the data is missing", "responses": ok("The reading") } },
        "/api/workforce-intelligence/key-person": {
            "get": { "tags": ["capacity"], "summary": "For each skill marked critical on a role profile: employed holders at the highest critical minimum, qualified backups, ready successors and a status (no_holder, single_point_of_failure, covered_single_holder, shared), overall and per department. Counts only; names no one", "responses": ok("The key-person counts") }
        },
        "/api/workforce-intelligence/key-person/named": {
            "get": { "tags": ["capacity"], "summary": "The sole holder behind each single point of failure, by worker number; HR only; the read is audited", "responses": ok("The people") }
        },
        "/api/governance-commitments": {
            "get": { "tags": ["capacity"], "summary": "The governance commitments of an organization (?organization=): a role, never a person, with hours a month for each", "responses": ok("The commitments") },
            "post": { "tags": ["capacity"], "summary": "Record a recurring commitment of a role (kind meeting, review, return or self_assessment; frequency weekly to annually; minutes one occurrence takes one person; holders = people in the role who take part)", "responses": ok("The commitment") }
        },
        "/api/governance-commitments/load": {
            "get": { "tags": ["capacity"], "summary": "Hours a month per role and the senior total (?organization=); an estimate from the entered commitments, not a timesheet", "responses": ok("The load") }
        },
        "/api/governance-commitments/{pid}": {
            "put": { "tags": ["capacity"], "summary": "Change a commitment", "responses": ok("The commitment") },
            "delete": { "tags": ["capacity"], "summary": "Remove a commitment", "responses": ok("The pid removed") }
        }
    });
    if let (Some(target), Some(extra)) = (document["paths"].as_object_mut(), paths.as_object()) {
        target.extend(extra.clone());
    }
}

/// The methods that change something, and so take an `Idempotency-Key` (WPM-R132).
const MUTATING: [&str; 4] = ["post", "put", "patch", "delete"];

/// Add `components` and give every operation that changes something the `Idempotency-Key`
/// parameter and the two answers a key can earn. Done in one place, by code, so that no operation
/// can be missed and a new one gets it without being edited.
fn decorate(document: &mut Value) {
    document["components"] = components();
    let Some(paths) = document["paths"].as_object_mut() else {
        return;
    };
    for item in paths.values_mut() {
        let Some(item) = item.as_object_mut() else {
            continue;
        };
        for method in MUTATING {
            let Some(operation) = item.get_mut(method).and_then(Value::as_object_mut) else {
                continue;
            };
            let parameters = operation.entry("parameters").or_insert_with(|| json!([]));
            if let Some(list) = parameters.as_array_mut() {
                list.push(json!({ "$ref": "#/components/parameters/IdempotencyKey" }));
            }
            let responses = operation.entry("responses").or_insert_with(|| json!({}));
            if let Some(map) = responses.as_object_mut() {
                map.entry("409").or_insert_with(
                    || json!({ "$ref": "#/components/responses/IdempotencyInProgress" }),
                );
                if !map.contains_key("422") {
                    map.insert(
                        "422".into(),
                        json!({ "$ref": "#/components/responses/IdempotencyRefused" }),
                    );
                }
                if let Some(success) = map.get_mut("200").and_then(Value::as_object_mut) {
                    success.entry("headers").or_insert_with(|| {
                        json!({ "Idempotent-Replayed": { "$ref": "#/components/headers/IdempotentReplayed" } })
                    });
                }
            }
        }
    }
}

/// Reusable parts: the key, the replay header, the two answers a key can earn, and the shapes of
/// the funding and contingent-workforce answers.
#[allow(clippy::too_many_lines)] // one literal
fn components() -> Value {
    json!({
        "parameters": {
            "IdempotencyKey": {
                "name": "Idempotency-Key", "in": "header", "required": false,
                "description": "Names one action so that a repeat of it (a double click, a retry after a lost reply) runs once. 16 to 64 letters, digits, hyphens or underscores. The first request under a key runs and, if it succeeds, its response is kept for one calendar day and returned again (with Idempotent-Replayed: true) for the same method, path, query and body from the same caller. The same key with a different request is refused (422). A failure is not kept, so the request can be corrected and sent again under the same key. A key belongs to the caller who made it. Ignored on a read.",
                "schema": { "type": "string", "minLength": 16, "maxLength": 64, "pattern": "^[A-Za-z0-9_-]+$" }
            }
        },
        "headers": {
            "IdempotentReplayed": {
                "description": "true when this response is the stored one for a repeated Idempotency-Key; absent otherwise",
                "schema": { "type": "string", "enum": ["true"] }
            }
        },
        "responses": {
            "IdempotencyInProgress": {
                "description": "A request with this Idempotency-Key is still running; try again after Retry-After seconds",
                "headers": { "Retry-After": { "schema": { "type": "integer" } } },
                "content": { "text/plain": { "schema": { "$ref": "#/components/schemas/PlainTextError" } } }
            },
            "IdempotencyRefused": {
                "description": "Validation failure, or the Idempotency-Key is malformed or was used for a different request",
                "content": { "text/plain": { "schema": { "$ref": "#/components/schemas/PlainTextError" } } }
            }
        },
        "schemas": {
            "PlainTextError": { "type": "string", "description": "A sentence saying what to change" },
            "PostFundingKind": { "type": "string", "enum": ["core", "time_limited", "external", "programme"] },
            "PostFundingPayload": {
                "type": "object", "required": ["kind"],
                "properties": {
                    "kind": { "$ref": "#/components/schemas/PostFundingKind" },
                    "programme_ref": { "type": "string", "nullable": true, "description": "A thing: URN", "example": "thing:7d0e2c7e-8f55-4e0a-b1a7-0c2b6a3a9d11" },
                    "ends_on": { "type": "string", "format": "date", "nullable": true, "description": "Required for time_limited and external; not before the engagement started" }
                }
            },
            "PostFunding": {
                "type": "object", "required": ["funding", "contract_ends_on", "alignment", "needs_attention"],
                "description": "Where a post is funded from beside the contract end. Never an amount of money.",
                "properties": {
                    "funding": { "type": "object", "nullable": true, "properties": {
                        "kind": { "$ref": "#/components/schemas/PostFundingKind" },
                        "programme_ref": { "type": "string", "nullable": true },
                        "ends_on": { "type": "string", "format": "date", "nullable": true }
                    } },
                    "contract_ends_on": { "type": "string", "format": "date", "nullable": true },
                    "alignment": { "type": "string", "enum": ["matched", "contract_outlasts_funding", "funding_outlasts_contract", "funding_open_ended", "unknown"], "description": "An unknown is never a match" },
                    "needs_attention": { "type": "boolean", "description": "True for the two mismatches" }
                }
            },
            "FundingMonth": {
                "type": "object", "required": ["month", "people", "fte"],
                "properties": {
                    "month": { "type": "string", "description": "YYYY-MM, or already_ended", "example": "2026-12" },
                    "people": { "type": "integer", "nullable": true, "description": "null when above zero and below the small-group floor" },
                    "fte": { "type": "number", "nullable": true, "description": "null when people is null" }
                }
            },
            "DepartmentContingent": {
                "type": "object",
                "properties": {
                    "department": { "type": "string" },
                    "headcount": { "type": "integer", "nullable": true },
                    "contractor_percent_of_fte": { "type": "number", "nullable": true },
                    "fixed_term_percent_of_fte": { "type": "number", "nullable": true }
                }
            },
            "ProgrammeRollup": {
                "type": "object", "description": "Aggregate only. A programme below the floor is left out and counted in withheld_programmes; a figure for one to nine people is null; no total across programmes.",
                "properties": {
                    "small_group_floor": { "type": "integer" },
                    "withheld_programmes": { "type": "integer" },
                    "programmes": { "type": "array", "items": { "type": "object", "properties": {
                        "programme": { "type": "string" },
                        "months": { "type": "array", "items": { "type": "object", "properties": {
                            "month": { "type": "string" },
                            "funded_fte": { "type": "number", "nullable": true },
                            "contractor_percent_of_fte": { "type": "number", "nullable": true },
                            "fixed_term_percent_of_fte": { "type": "number", "nullable": true },
                            "engagements_ending": { "type": "integer", "nullable": true },
                            "funding_ending": { "type": "integer", "nullable": true }
                        } } }
                    } } },
                    "derivation": { "type": "string" }
                }
            },
            "ContingentWorkforce": {
                "type": "object", "description": "Aggregate only: no one is named. Groups below the floor are withheld and no total is given for the funding months.",
                "properties": {
                    "small_group_floor": { "type": "integer" },
                    "headcount": { "type": "integer" },
                    "fte": { "type": "number" },
                    "contractor_percent_of_fte": { "type": "number", "nullable": true },
                    "fixed_term_percent_of_fte": { "type": "number", "nullable": true },
                    "departments_shown": { "type": "boolean" },
                    "by_department": { "type": "array", "items": { "$ref": "#/components/schemas/DepartmentContingent" } },
                    "funding_ends": { "type": "array", "items": { "$ref": "#/components/schemas/FundingMonth" } },
                    "trend": { "type": "object", "description": "The contingent share over the snapshots recorded with the split by basis: status ok, or insufficient_history (fewer than three usable dates over at least 60 calendar days) with no points. Older snapshots are unknown, not zero.", "additionalProperties": true },
                    "engagements": { "type": "object", "description": "Engagements ending in separate 30-calendar-day bands, past their end with no decision, extended more than the threshold, and long-running contractors. A count of one to nine is null; no total is given.", "additionalProperties": true },
                    "derivation": { "type": "string" }
                }
            }
        }
    })
}

#[cfg(test)]
mod tests {
    use super::{MUTATING, spec};
    use serde_json::Value;

    /// Resolve a local `$ref` such as `#/components/schemas/X` in a document.
    fn resolves(document: &Value, reference: &str) -> bool {
        reference.strip_prefix("#/").is_some_and(|path| {
            path.split('/')
                .try_fold(document, |v, k| v.get(k))
                .is_some()
        })
    }

    fn refs(value: &Value, out: &mut Vec<String>) {
        match value {
            Value::Object(map) => {
                for (k, v) in map {
                    if k == "$ref" {
                        if let Some(r) = v.as_str() {
                            out.push(r.to_string());
                        }
                    } else {
                        refs(v, out);
                    }
                }
            }
            Value::Array(items) => items.iter().for_each(|v| refs(v, out)),
            _ => {}
        }
    }

    /// Every `$ref` in the document points at something that exists.
    #[test]
    fn every_reference_resolves() {
        let document = spec();
        let mut found = Vec::new();
        refs(&document, &mut found);
        assert!(found.len() > 100, "{}", found.len());
        for r in found {
            assert!(resolves(&document, &r), "{r} does not resolve");
        }
    }

    /// Every operation that changes something takes the key and documents what a key can earn; no read does.
    #[test]
    fn only_a_change_takes_an_idempotency_key() {
        let document = spec();
        let mut changes = 0;
        for (path, item) in document["paths"].as_object().unwrap() {
            for (method, operation) in item.as_object().unwrap() {
                let takes_key = operation["parameters"].as_array().is_some_and(|list| {
                    list.iter()
                        .any(|p| p["$ref"] == "#/components/parameters/IdempotencyKey")
                });
                if MUTATING.contains(&method.as_str()) {
                    changes += 1;
                    assert!(takes_key, "{method} {path} takes no Idempotency-Key");
                    assert!(
                        operation["responses"]["409"].is_object(),
                        "{method} {path}: 409"
                    );
                    assert!(
                        operation["responses"]["422"].is_object(),
                        "{method} {path}: 422"
                    );
                } else {
                    assert!(!takes_key, "{method} {path} is a read");
                }
            }
        }
        assert!(changes > 100, "{changes}");
    }

    /// The parameter says what the middleware enforces.
    #[test]
    fn the_key_parameter_matches_the_rules() {
        let schema = &spec()["components"]["parameters"]["IdempotencyKey"]["schema"];
        assert_eq!(schema["minLength"], crate::rules::idempotency::KEY_MIN);
        assert_eq!(schema["maxLength"], crate::rules::idempotency::KEY_MAX);
        let pattern = schema["pattern"].as_str().unwrap();
        assert_eq!(pattern, "^[A-Za-z0-9_-]+$");
    }

    /// The funding and contingent answers are described by schemas, not only by words.
    #[test]
    fn the_funding_and_contingent_answers_have_schemas() {
        let document = spec();
        for (path, method) in [
            ("/api/workers/{pid}/funding", "get"),
            ("/api/workers/{pid}/funding", "put"),
            ("/api/workforce-intelligence/contingent", "get"),
            ("/api/workforce-intelligence/programmes", "get"),
        ] {
            let schema = &document["paths"][path][method]["responses"]["200"]["content"]["application/json"]
                ["schema"];
            assert!(schema["$ref"].is_string(), "{method} {path}");
        }
        let body = &document["paths"]["/api/workers/{pid}/funding"]["put"]["requestBody"];
        assert_eq!(
            body["content"]["application/json"]["schema"]["$ref"],
            "#/components/schemas/PostFundingPayload"
        );
    }

    /// Every route the controllers register under `/api` has a path in the document. This is what
    /// stops the document drifting behind the routes, or silently losing entries.
    #[test]
    fn every_registered_route_is_documented() {
        let spec = std::fs::read_to_string(
            std::path::Path::new(env!("CARGO_MANIFEST_DIR")).join("src/openapi.rs"),
        )
        .unwrap();
        let mut missing = Vec::new();
        for entry in std::fs::read_dir(
            std::path::Path::new(env!("CARGO_MANIFEST_DIR")).join("src/controllers"),
        )
        .unwrap()
        {
            let mut path = entry.unwrap().path();
            if path.is_dir() {
                // a controller split into sections keeps its routes in `mod.rs`
                path.push("mod.rs");
            }
            if path.extension().and_then(|e| e.to_str()) != Some("rs") {
                continue;
            }
            let flat = std::fs::read_to_string(&path)
                .unwrap()
                .split_whitespace()
                .collect::<Vec<_>>()
                .join(" ");
            let Some((_, rest)) = flat.split_once(".prefix(\"") else {
                continue;
            };
            let Some((prefix, body)) = rest.split_once('"') else {
                continue;
            };
            if !prefix.starts_with("/api") {
                continue;
            }
            for call in body.split(".add(").skip(1) {
                let Some(call) = call.trim_start().strip_prefix('"') else {
                    continue;
                };
                let Some((route, _)) = call.split_once('"') else {
                    continue;
                };
                let full = format!("{prefix}{route}");
                if !spec.contains(&format!("\"{full}\"")) {
                    missing.push(full);
                }
            }
        }
        missing.sort();
        missing.dedup();
        assert!(
            missing.is_empty(),
            "routes with no OpenAPI path: {missing:?}"
        );
    }

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
            "/api/capacity",
            "/api/workers/{pid}/engagement",
            "/api/workers/{pid}/contractor-details",
            "/api/engagements/contingent-list",
            "/api/workforce-intelligence/programmes",
            "/api/engagements/missing-end-date",
            "/api/me/contact-details",
            "/api/me/time-off",
            "/api/me/resignation",
            "/api/me/health-requirements",
            "/api/health-requirements",
            "/api/me/equality-monitoring",
            "/api/me/flexible-working",
            "/api/flexible-working",
            "/api/resignations",
            "/api/capacity/start-check",
            "/api/capacity/start-decisions",
            "/api/skill-pools",
            "/api/programme-demands",
            "/api/partner-commitments",
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
