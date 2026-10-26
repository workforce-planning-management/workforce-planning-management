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
            "/api/org-chart": { "get": { "tags": ["hr-core"], "summary": "The manager forest for one organization (?organization=&limit=; at most limit people are drawn, roots first then each level; default 5000, at most 20000)", "responses": ok("OrgNodes") } },
            "/api/org-chart/size": { "get": { "tags": ["hr-core"], "summary": "How many people the organization has in all (?organization=), to say when the chart is only part of it", "responses": ok("People count") } },
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
            "/api/workers/{pid}/downline": { "get": { "tags": ["reporting"], "summary": "Everyone below a manager: direct and indirect reports with depth, nearest first (?limit=, default 5000, at most 20000; the summary counts everyone)", "responses": ok("Downline") } },
            "/api/workers/{pid}/reports": { "get": { "tags": ["reporting"], "summary": "Direct or indirect reports (?kind=direct|indirect&limit=); total and truncated say whether the list is whole", "responses": ok("Reports") } },
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
    add_integration_paths(&mut document);
    typed_answers(&mut document);
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
        },
        "/api/pay-categories": {
            "get": { "tags": ["pay-transparency"], "summary": "The categories of workers an organization reports pay gaps by (?organization=), each with the basis on which its roles count as equal value and the job titles in it", "responses": ok("The categories") },
            "post": { "tags": ["pay-transparency"], "summary": "Set up a category: name, the stated basis (required), and 1 to 50 job titles; a title is in one category per organization. Holds no personal data", "responses": ok("The pid of the category") }
        },
        "/api/pay-categories/{pid}": {
            "put": { "tags": ["pay-transparency"], "summary": "Replace a category's name, basis and job titles; the organization does not change", "responses": ok("The pid of the category") },
            "delete": { "tags": ["pay-transparency"], "summary": "Take a category away; later reports count its workers as in no category", "responses": ok("Removed") }
        },
        "/api/pay-transparency/gap-report": {
            "get": { "tags": ["pay-transparency"], "summary": "The seven pay gap indicators of the EU Pay Transparency Directive for an organization (?organization=&as_of=&currency=&variable=): aggregates with a cell under the floor withheld, a side with nobody unknown, no head count and no name, and the workers left out counted. Needs unmasked salary read and equality monitoring switched on; the read is audited", "responses": ok("The report, its basis and the reporting tier") }
        },
        "/api/me/pay-information-requests": {
            "get": { "tags": ["pay-transparency"], "summary": "The caller's own pay information requests, newest first, each with its due date and standing (open, overdue or answered)", "responses": ok("The requests") },
            "post": { "tags": ["pay-transparency"], "summary": "Ask for your own pay level and the average pay, by sex, of your category of workers (no body); the answer is due a configured number of calendar months later (default 2) and one request waits at a time", "responses": ok("The pid and the due date") }
        },
        "/api/pay-information-requests": {
            "get": { "tags": ["pay-transparency"], "summary": "Requests to answer (?organization=&status=open|overdue|answered), oldest due first, with who asked and no figure", "responses": ok("The requests") }
        },
        "/api/pay-information-requests/{pid}/answer": {
            "get": { "tags": ["pay-transparency"], "summary": "Work out the answer to give: the worker's own pay level and the average pay of their category by sex, each average withheld under the floor. Not stored. Needs unmasked salary read; the read is audited with no figure", "responses": ok("The answer") }
        },
        "/api/pay-information-requests/{pid}/answered": {
            "post": { "tags": ["pay-transparency"], "summary": "Record that the answer was given, and when; the figures are not recorded", "responses": ok("The request") }
        },
        "/api/pay-transparency/joint-assessment": {
            "get": { "tags": ["pay-transparency"], "summary": "For each category of workers, whether a joint pay assessment with workers' representatives is due, pending or not due (?organization=&reported_on=&as_of=&currency=): the category's gap in basic pay at or over the threshold (default 5%, either way), not justified, not remedied within the period (default six calendar months) from the date of reporting. Reports only; a person records. Needs unmasked salary read; audited with no figure", "responses": ok("The signals") }
        },
        "/api/pay-transparency/assessment-records": {
            "get": { "tags": ["pay-transparency"], "summary": "What has been recorded for categories and report dates (?organization=), newest report first", "responses": ok("The records") },
            "put": { "tags": ["pay-transparency"], "summary": "Record, for a category and a report date, whether the gap was justified (and on what), when it was remedied, and whether the joint assessment was carried out and how it ended (outcome from a closed list); replaces what was there; never a verdict on a person", "responses": ok("The pid of the record") }
        },
        "/api/pay-transparency/gap-report/csv": {
            "get": { "tags": ["pay-transparency"], "summary": "The same report as CSV, one row per figure with the two figures each gap comes from; the export is audited", "responses": ok("A CSV file") }
        },
    });
    if let (Some(target), Some(extra)) = (document["paths"].as_object_mut(), paths.as_object()) {
        target.extend(extra.clone());
    }
}

/// The webhook, MCP, AI register and report paths (WPM-R147 to WPM-R151), in their own function for the same reason.
#[allow(clippy::too_many_lines)] // one literal of paths; splitting it would only move the lines
fn add_integration_paths(document: &mut Value) {
    let ok = |what: &str| json!({ "200": { "description": what } });
    let paths = json!({
        "/api/webhook-subscriptions": {
            "get": { "tags": ["webhooks"], "summary": "Administrators only. The webhook subscriptions (endpoint, event kinds, the name of the environment variable holding the signing secret, whether that variable is set, whether on) with how many deliveries wait, were delivered or gave up, and the kinds on offer", "responses": ok("The subscriptions") },
            "post": { "tags": ["webhooks"], "summary": "Administrators only. Subscribe an https endpoint (or http to this machine; no credentials, query or internal address) to event kinds, signing with the secret named by secret_ref (WPM_WEBHOOK_SECRET_<NAME>); the secret itself is never stored or returned. Audited in the same transaction", "responses": ok("The pid of the subscription") }
        },
        "/api/webhook-subscriptions/{pid}": {
            "put": { "tags": ["webhooks"], "summary": "Administrators only. Replace a subscription's name, endpoint, kinds, secret reference and whether it is on", "responses": ok("The pid") },
            "delete": { "tags": ["webhooks"], "summary": "Administrators only. Remove a subscription and its delivery log", "responses": ok("The pid") }
        },
        "/api/webhook-subscriptions/{pid}/deliveries": {
            "get": { "tags": ["webhooks"], "summary": "Administrators only. The delivery log, newest first (?limit=): event id, kind, the URN of the record, status, attempts, last HTTP status and a short reason. No payload body is kept, so none is shown", "responses": ok("The deliveries") }
        },
        "/api/positions": {
            "get": { "tags": ["positions"], "summary": "Positions of an organization (?organization=&state=&cost_centre=): cost centre, title, budget in FTE percent, funding kind and end, state, and links to the requisition and the plan demand line. The holder is shown only to HR, payroll, service and administrators; others see whether the seat is held. No pay", "responses": ok("The positions") },
            "post": { "tags": ["positions"], "summary": "Plan a seat (starts planned). Budget 1 to 100 FTE percent; a time-limited or external funding names its end; a programme funding names a thing: reference. An optional demand_line_pid records where it came from, only when a person chooses it", "responses": ok("The pid") }
        },
        "/api/positions/establishment": {
            "get": { "tags": ["positions"], "summary": "The establishment per cost centre (?organization=): positions, budget, held, vacant, over, the vacancy sitting in frozen positions and positions by state. Approved, open, filled and frozen positions count. Over-establishment is never netted against a vacancy elsewhere; an unknown holder FTE is unknown; no holder is named, and a caller who may not see holders gets held, vacant and over withheld for a centre under the floor", "responses": ok("The establishment") }
        },
        "/api/positions/{pid}": {
            "put": { "tags": ["positions"], "summary": "Change what a position is made of; not its organization, state or links. An abolished position is not changed", "responses": ok("The pid") }
        },
        "/api/positions/{pid}/status": {
            "post": { "tags": ["positions"], "summary": "Move a position (planned, approved, open, filled, frozen, abolished). Open needs a linked requisition, filled a linked holder; a held position is not made vacant or abolished until its holder is unlinked. 422 names the state it is in", "responses": ok("The new state") }
        },
        "/api/positions/{pid}/holder": {
            "put": { "tags": ["positions"], "summary": "Link the worker who holds the position (same organization), or null to unlink. Not for a planned or abolished position", "responses": ok("The pid") }
        },
        "/api/positions/{pid}/requisition": {
            "put": { "tags": ["positions"], "summary": "Link the requisition that fills the position (same organization), or null to unlink; an open position keeps its requisition", "responses": ok("The pid") }
        },
        "/api/workforce-plans/{pid}/target-organization": {
            "get": { "tags": ["planning"], "summary": "The roles a plan intends the organization to have: key, parent role, department, title and headcount. Roles, never people", "responses": ok("The target roles") },
            "put": { "tags": ["planning"], "summary": "Replace the plan's target organization: 1 to 2000 roles, unique keys, present parents, no loop, headcount 1 to 10000. There is no way to name a person. Audited in the same transaction", "responses": ok("The plan pid") }
        },
        "/api/workforce-plans/{pid}/organization-diff": {
            "get": { "tags": ["planning"], "summary": "The live structure of the plan's organization against its target: layers, span of control and each role added, removed, moved or resized. Counts of people under the floor are withheld with the change that would give them back; no person is named; cost and skills are not compared", "responses": ok("The comparison") }
        },
        "/mcp": {
            "post": { "tags": ["mcp"], "summary": "A read-only Model Context Protocol server (Streamable HTTP, revision 2025-11-25, JSON replies, no session): initialize, ping, tools/list and tools/call over ten existing reads. Each tool is one GET route called in process under the caller's own token, so the API's policy decides and a caller without access is refused as on the route. No write tool, no service account. Every tool call is audited with the tool name first; if the audit cannot be written the call is refused. A browser Origin must be listed in WPM_MCP_ALLOWED_ORIGINS", "responses": ok("A JSON-RPC reply") }
        },
        "/api/ai-register": {
            "get": { "tags": ["ai-register"], "summary": "Every feature that scores or ranks people (S1 to S8) with what the deployer recorded under Article 26 of the AI Act: Annex III category, oversight role, when representatives and workers were informed, log retention. A field with nothing recorded is unknown. Records what is said; does not classify, certify or evidence compliance. Any signed-in caller", "responses": ok("The register") }
        },
        "/api/ai-register/{feature}": {
            "put": { "tags": ["ai-register"], "summary": "Replace what is recorded for a feature: category, oversight role (a role, not a person), the two informed dates (not in the future), log retention in months (under six needs a note naming the other law). A field left out is cleared. Audited in the same transaction", "responses": ok("The pid of the entry") }
        },
        "/api/recognised-bodies": {
            "get": { "tags": ["bargaining"], "summary": "The bodies the organization recognises (?organization=): a union, works council or staff forum, with a kind, a basis, a date and a contact role, and how many units each has. HR. No member or representative is held", "responses": ok("The bodies") },
            "post": { "tags": ["bargaining"], "summary": "Record a recognised body: a name unique within the organization, a kind (union, works_council, staff_forum), a basis, the date recognised and a contact role (never a person)", "responses": ok("The pid of the body") }
        },
        "/api/recognised-bodies/{pid}/units": {
            "get": { "tags": ["bargaining"], "summary": "A body's bargaining units with their rules and how many workers in work today they cover: a count from the floor, nothing under it, never a list of who. HR", "responses": ok("The units") },
            "post": { "tags": ["bargaining"], "summary": "Add a bargaining unit: a name and rules over department, location, employment_type and job_title (at least one). Nothing is recorded about who is covered", "responses": ok("The pid of the unit") }
        },
        "/api/bargaining-units/{pid}/agreements": {
            "get": { "tags": ["bargaining"], "summary": "Every version of every collective agreement of a unit, oldest first, saying which is in force today. HR", "responses": ok("The agreements") },
            "post": { "tags": ["bargaining"], "summary": "Record a collective agreement for a unit, or a new version of one by giving the same title: dated, sourced and transcribed, with optional terms (a known pay scale id, max weekly hours, notice weeks, an agreed consultation period in days). A new version must take effect after the newest", "responses": ok("The pid and version") }
        },
        "/api/recognised-bodies/{pid}/facility-time": {
            "get": { "tags": ["bargaining"], "summary": "The time released for representative duties for a body, per period, newest first, with any budget and whether it was exceeded (unknown without a budget), and the total in minutes. Per body, never per person. HR", "responses": ok("The periods") },
            "post": { "tags": ["bargaining"], "summary": "Record minutes released for representative duties for the body over a period (at most 366 days, not overlapping another), with an optional budget in minutes. No person and no time-entry kind", "responses": ok("The pid") }
        },
        "/api/decision-gates": {
            "get": { "tags": ["bargaining"], "summary": "The kinds of change that need consultation with a recognised body (?organization=), and with which. HR", "responses": ok("The gates") },
            "post": { "tags": ["bargaining"], "summary": "Say that a kind of change (redundancy_proposal, shift_pattern_change, new_monitoring, new_or_changed_score, pay_structure_change, other) needs consultation with a body of the organization", "responses": ok("The pid") }
        },
        "/api/decision-gates/check": {
            "get": { "tags": ["bargaining"], "summary": "What a change would be pointed at (?organization=&change=&consultation_pid=): the bodies to consult and whether a consultation reference is still needed. Advice, not a block. HR", "responses": ok("The pointer") }
        },
        "/api/bargaining-units/{pid}/information-pack": {
            "get": { "tags": ["bargaining"], "summary": "The aggregates a body is given to bargain on, for the people a unit's rules cover: headcount and FTE by department and employment type, pay bands, starters and leavers in the last year, those not permanent, and open vacancy posts. Counts from the floor, complementary suppression, a unit under the floor gives nothing; no name, salary or individual band. Reading it is logged. HR", "responses": ok("The pack") }
        },
        "/api/workers/{pid}/collective-agreements": {
            "get": { "tags": ["bargaining"], "summary": "The agreements in force on a day (?on=, default today) for the units whose rules cover this worker's job facts. HR and payroll, audited. Derived, and says nothing about membership: no body or unit is named", "responses": ok("The agreements") }
        },
        "/api/consultations": {
            "get": { "tags": ["bargaining"], "summary": "The consultation register (?organization=): one record per proposal, newest termination date first. HR. No attendee, representative or minute is held", "responses": ok("The consultations") },
            "post": { "tags": ["bargaining"], "summary": "Open a record for a proposal: the body, the establishment, the threshold profile (great_britain or eu_directive_98_59), establishment size, dismissals, the earliest termination date, topics and a document reference (no spaces). Counts and dates only", "responses": ok("The pid") }
        },
        "/api/consultations/{pid}/begin": {
            "post": { "tags": ["bargaining"], "summary": "Record the day consultation began (once, not in the future), and optionally the reference to the documents shared", "responses": ok("The pid") }
        },
        "/api/consultations/{pid}/close": {
            "post": { "tags": ["bargaining"], "summary": "End a consultation as agreed (optionally linking an agreement of the body's units), no_agreement or withdrawn, on a day not in the future and not before it began", "responses": ok("The pid") }
        },
        "/api/consultations/{pid}/timeline": {
            "get": { "tags": ["bargaining"], "summary": "The record with the consultation rules read over it: whether a duty may apply and the evidence, the latest days to begin consultation and notify the authority, any longer period an agreement in force sets, the checklist, and whether consultation began in time. A suggestion, not a verdict; the thresholds are the deployer's to confirm", "responses": ok("The timeline") }
        },
        "/api/assets": {
            "get": { "tags": ["equipment"], "summary": "The organization's assets (?organization=), each with whether it is out. Kind, tag and owner role only: no location, cost or reading is held", "responses": ok("The assets") },
            "post": { "tags": ["equipment"], "summary": "Add an asset: kind (laptop, phone, pass, key, other), a tag unique within the organization, and the role that owns it", "responses": ok("The pid of the asset") }
        },
        "/api/assets/{pid}/issue": {
            "post": { "tags": ["equipment"], "summary": "Give an asset to a worker of its organization who is employed today, from a day not in the future and not before their first day. One holder at a time; audited in the same transaction", "responses": ok("The pid of the issue") }
        },
        "/api/asset-issues/{pid}/return": {
            "post": { "tags": ["equipment"], "summary": "Close an open issue as returned, or written_off with a reason, on a day not before it was issued and not in the future", "responses": ok("The outcome") }
        },
        "/api/workers/{pid}/equipment": {
            "get": { "tags": ["equipment"], "summary": "What a worker holds and has held, and how many pieces are still out. The worker themself and HR", "responses": ok("The issues") }
        },
        "/api/policies": {
            "get": { "tags": ["policies"], "summary": "The organization's policies (?organization=) and their versions: title, a reference to the document (WPM holds no text), effective day, audience and calendar days to acknowledge", "responses": ok("The policies") },
            "post": { "tags": ["policies"], "summary": "Name a policy; a name is unique within the organization", "responses": ok("The pid of the policy") }
        },
        "/api/policies/{pid}": {
            "patch": { "tags": ["policies"], "summary": "Correct a policy's name (body: name); unique within the organization. Versions and acknowledgements are unchanged. Audited in the same transaction", "responses": ok("The pid and the new name") }
        },
        "/api/policies/{pid}/versions": {
            "post": { "tags": ["policies"], "summary": "Add the next version (never an edit): title, document_ref with no spaces, effective_on, audience (all, department, job_title) with its value, acknowledge_within_days 1 to 365 (default 30). Every worker in its audience must acknowledge it again", "responses": ok("The pid and number of the version") }
        },
        "/api/workers/{pid}/policies": {
            "get": { "tags": ["policies"], "summary": "The newest version in force of each policy aimed at the worker and where they stand: done, waiting or overdue (flagged, never forced), with a count waiting. The worker themself and HR", "responses": ok("The policies") }
        },
        "/api/me/policy-acknowledgements": {
            "post": { "tags": ["policies"], "summary": "The caller says they have read the newest version in force aimed at them (body: version_pid). Their own act: the worker is the caller, so no one acknowledges for another, HR included. Idempotent: the first day is kept. Audited in the same transaction", "responses": ok("The day acknowledged") }
        },
        "/api/me/policies": {
            "get": { "tags": ["policies"], "summary": "The caller's own policies to acknowledge, as /api/workers/{pid}/policies", "responses": ok("The policies") }
        },
        "/api/workers/{pid}/team-policy-counts": {
            "get": { "tags": ["policies"], "summary": "For a manager (?version_pid=): how many of their direct reports in the audience have acknowledged and how many have not, from the floor; under it, nothing, since two numbers would name them", "responses": ok("The counts") }
        },
        "/api/workers/{pid}/team-equipment-count": {
            "get": { "tags": ["equipment"], "summary": "For a manager: how many of their direct reports in work hold equipment and how many hold none, from the floor; under it, nothing. No item, kind or name", "responses": ok("The count") }
        },
        "/api/policy-versions/{pid}/acknowledgements": {
            "get": { "tags": ["policies"], "summary": "The named record: everyone in the version's audience with the day they acknowledged or null. HR and the like only; the read is audited", "responses": ok("The record") }
        },
        "/api/me/credentials": {
            "get": { "tags": ["credentials"], "summary": "The caller's credential issuances: that each was issued, when, its place on the revocation list and whether it is revoked. Not the credentials", "responses": ok("The issuances") },
            "post": { "tags": ["credentials"], "summary": "Issue an Open Badges 3.0 credential (signed vc+jwt, EdDSA) from the caller's own verified CPD entry (body: cpd_entry_pid). Returned once and not stored; it names no person. One live credential per entry", "responses": ok("The pid, its status index and the signed credential") }
        },
        "/api/me/credentials/{pid}/revoke": {
            "post": { "tags": ["credentials"], "summary": "The worker withdraws one of their own credentials; the revocation list changes at once", "responses": ok("Revoked") }
        },
        "/api/credentials/{pid}/revoke": {
            "post": { "tags": ["credentials"], "summary": "HR or an administrator revokes a credential, for instance when its CPD entry is withdrawn", "responses": ok("Revoked") }
        },
        "/api/credentials/issuer": {
            "get": { "tags": ["credentials"], "summary": "PUBLIC (no sign-in): who issues, and the Ed25519 public key (JWK) to check a credential against. Holds no personal data", "responses": ok("The issuer and key") }
        },
        "/api/credentials/status-list": {
            "get": { "tags": ["credentials"], "summary": "PUBLIC (no sign-in): the revocation list as a signed Bitstring Status List credential (131 072 bits). Positions only; holds no personal data", "responses": ok("The list") }
        },
        "/api/skill-mappings": {
            "get": { "tags": ["skills"], "summary": "The skills crosswalk: each mapping from a declared skill to a needed one, with its confidence (exact, close, broader, narrower), its basis and source, who stated it and when, and whether it counts in gap analysis (exact and close do)", "responses": ok("The mappings") },
            "post": { "tags": ["skills"], "summary": "State a mapping: from_skill_pid, to_skill_pid, confidence, basis (authority or judgement) and a source naming the published crosswalk or whose judgement. The pair is unique; audited", "responses": ok("The pid of the mapping") }
        },
        "/api/skill-mappings/{pid}": {
            "delete": { "tags": ["skills"], "summary": "Withdraw a mapping; gap analysis stops using it at once", "responses": ok("Removed") }
        },
        "/api/reports/pay-range": {
            "get": { "tags": ["reports"], "summary": "Pay position analysis (?organization=&as_of=&cutoff=): compa-ratio (pay over the band midpoint) and range penetration per band, level, department and location, and the groups far from everyone (cut-off in percentage points, default 10). Aggregates: groups under the floor are withheld with no count, a total that would reveal one is not shown, workers with no pay or no band are counted not zeroed. Needs an unmasked salary read; proposes no pay change. Audited", "responses": ok("The analysis") }
        },
        "/api/reports/human-capital": {
            "get": { "tags": ["reports"], "summary": "The human capital report (?organization=&as_of=): headcount and FTE by employment type, starters, leavers, turnover, time to fill, training completion and succession coverage, each with its definition, source, period and exclusions, laid beside ISO 30414 areas and ESRS S1 requirements by name. Groups under the floor are withheld; what WPM cannot compute is unknown with a reason; the named metrics WPM does not cover are listed. Audited", "responses": ok("The report") }
        }
    });
    if let (Some(target), Some(extra)) = (document["paths"].as_object_mut(), paths.as_object()) {
        target.extend(extra.clone());
    }
}

/// The methods that change something, and so take an `Idempotency-Key` (WPM-R132).
const MUTATING: [&str; 4] = ["post", "put", "patch", "delete"];

/// What an answer is, for the operations whose handler was read to find out (WPM-T263).
enum Answer {
    /// `200` with no body at all (`format::empty()`).
    NoBody,
    /// `200` with a body of this component schema.
    Schema(&'static str),
}

/// Operations whose answer shape is stated, so the UI can be typed from this document instead of by
/// hand. Paths use `{}` for every parameter, whatever the document calls it. The rest still say only
/// "200"; the UI types those as `unknown` until their handlers are read.
const ANSWERS: &[(&str, &str, Answer)] = &[
    ("/api/workers/{}/job-level", "delete", Answer::NoBody),
    ("/api/expense-items/{}", "delete", Answer::NoBody),
    ("/api/workers/{}/pay-position", "delete", Answer::NoBody),
    ("/api/role-profiles/{}/grade", "delete", Answer::NoBody),
    ("/api/me/contact-details", "delete", Answer::NoBody),
    (
        "/api/role-profiles/{}/requirements",
        "put",
        Answer::Schema("EmptyObject"),
    ),
    (
        "/api/role-profiles/{}/requirements/{}",
        "delete",
        Answer::Schema("EmptyObject"),
    ),
    (
        "/api/cpd-requirements/{}",
        "delete",
        Answer::Schema("EmptyObject"),
    ),
    (
        "/api/workers/{}/framework-roles/{}",
        "delete",
        Answer::Schema("EmptyObject"),
    ),
    (
        "/api/workers/{}/dotted-line-managers/{}",
        "delete",
        Answer::Schema("EmptyObject"),
    ),
    (
        "/api/emergency-contacts/{}",
        "delete",
        Answer::Schema("EmptyObject"),
    ),
    ("/api/backups/{}", "delete", Answer::Schema("EmptyObject")),
    (
        "/api/announcements/{}",
        "delete",
        Answer::Schema("EmptyObject"),
    ),
    ("/api/rotas/{}", "delete", Answer::Schema("EmptyObject")),
    (
        "/api/rota-overrides/{}",
        "delete",
        Answer::Schema("EmptyObject"),
    ),
    (
        "/api/skill-courses/{}",
        "delete",
        Answer::Schema("EmptyObject"),
    ),
    (
        "/api/groups/{}/members/{}",
        "delete",
        Answer::Schema("EmptyObject"),
    ),
    (
        "/api/aspirations/{}",
        "delete",
        Answer::Schema("EmptyObject"),
    ),
    (
        "/api/mobility-interests/{}",
        "delete",
        Answer::Schema("EmptyObject"),
    ),
    (
        "/api/change-initiatives/{}/role-impacts",
        "put",
        Answer::Schema("EmptyObject"),
    ),
    (
        "/api/change-initiatives/{}/skill-shifts",
        "put",
        Answer::Schema("EmptyObject"),
    ),
    (
        "/api/change-initiatives/{}/status",
        "post",
        Answer::Schema("PidRef"),
    ),
    (
        "/api/workers/{}/skill-history/past",
        "post",
        Answer::Schema("PidRef"),
    ),
    (
        "/api/announcements/{}/read",
        "post",
        Answer::Schema("ReadReceipt"),
    ),
    (
        "/api/movement-items/{}/done",
        "post",
        Answer::Schema("ItemState"),
    ),
    (
        "/api/movement-items/{}/reopen",
        "post",
        Answer::Schema("ItemState"),
    ),
    (
        "/api/movement-items/{}/skip",
        "post",
        Answer::Schema("ItemState"),
    ),
    (
        "/api/movements/{}/complete",
        "post",
        Answer::Schema("MovementClosed"),
    ),
    (
        "/api/movements/{}/cancel",
        "post",
        Answer::Schema("MovementClosed"),
    ),
    (
        "/api/wellbeing-entitlements/{}",
        "delete",
        Answer::Schema("Deleted"),
    ),
    ("/api/cutovers/{}", "delete", Answer::Schema("Removed")),
    (
        "/api/cutovers/{}/pools",
        "post",
        Answer::Schema("CutoverPool"),
    ),
    (
        "/api/cutovers/{}/roles",
        "post",
        Answer::Schema("CutoverRole"),
    ),
    (
        "/api/oversight/criteria",
        "post",
        Answer::Schema("OversightCriterionCreated"),
    ),
    (
        "/api/oversight/measures/{}/readings",
        "put",
        Answer::Schema("OversightReading"),
    ),
    (
        "/api/me/flexible-working",
        "post",
        Answer::Schema("FlexibleRequested"),
    ),
    (
        "/api/me/resignation",
        "post",
        Answer::Schema("ResignationLogged"),
    ),
    (
        "/api/aspirations/{}",
        "put",
        Answer::Schema("AspirationChanged"),
    ),
    ("/api/skills/{}", "put", Answer::Schema("PidRef")),
    ("/api/skills/{}", "delete", Answer::Schema("EmptyObject")),
    (
        "/api/skills/{}/refs/{}",
        "delete",
        Answer::Schema("EmptyObject"),
    ),
    (
        "/api/workers/{}/skills",
        "put",
        Answer::Schema("WorkerSkill"),
    ),
    (
        "/api/cpd-entries/{}/verify",
        "post",
        Answer::Schema("CpdEntry"),
    ),
    (
        "/api/workers/{}/framework-roles/{}/past",
        "post",
        Answer::Schema("FrameworkRoleHeld"),
    ),
    (
        "/api/workers/{}/dotted-line-managers",
        "post",
        Answer::Schema("DottedLineManager"),
    ),
    (
        "/api/groups/{}/members",
        "post",
        Answer::Schema("GroupMember"),
    ),
    (
        "/api/workforce-plans/{}/status",
        "post",
        Answer::Schema("PidRef"),
    ),
    (
        "/api/workforce-plans/{}/demand-lines/{}",
        "delete",
        Answer::Schema("EmptyObject"),
    ),
    (
        "/api/workforce-plans/{}/demand-lines/{}/objectives",
        "put",
        Answer::Schema("EmptyObject"),
    ),
    (
        "/api/workforce-plans/{}/tolerances/{}",
        "put",
        Answer::Schema("PlanTolerances"),
    ),
    (
        "/api/workforce-plans/{}/tolerances/{}",
        "delete",
        Answer::Schema("EmptyObject"),
    ),
    (
        "/api/mentorships/{}/status",
        "post",
        Answer::Schema("Mentorship"),
    ),
    (
        "/api/appraisals/{}/status",
        "post",
        Answer::Schema("Appraisal"),
    ),
    (
        "/api/workers/{}/wellbeing-acknowledgements",
        "post",
        Answer::Schema("EntitlementAcknowledgement"),
    ),
    (
        "/api/oversight/packs/{}/withdraw",
        "post",
        Answer::Schema("OversightPack"),
    ),
    (
        "/api/skill-pools/{}/members",
        "post",
        Answer::Schema("PidRef"),
    ),
    ("/api/programme-demands", "put", Answer::Schema("PidRef")),
    (
        "/api/oversight/levels",
        "post",
        Answer::Schema("OversightLevel"),
    ),
    (
        "/api/oversight/positions",
        "post",
        Answer::Schema("OversightPosition"),
    ),
    (
        "/api/oversight/measures",
        "post",
        Answer::Schema("OversightMeasure"),
    ),
    (
        "/api/development-plan-items/{}",
        "put",
        Answer::Schema("DevelopmentPlanItem"),
    ),
    (
        "/api/me/leave-requests",
        "post",
        Answer::Schema("LeaveRequested"),
    ),
    (
        "/api/me/leave-requests/{}/cancel",
        "post",
        Answer::Schema("LeaveRequest"),
    ),
    (
        "/api/me/flexible-working/{}/withdraw",
        "post",
        Answer::Schema("FlexibleRequest"),
    ),
    (
        "/api/me/flexible-working/{}/respond",
        "post",
        Answer::Schema("FlexibleRequest"),
    ),
    (
        "/api/me/flexible-working/{}/appeal",
        "post",
        Answer::Schema("FlexibleRequest"),
    ),
    (
        "/api/me/resignation/withdraw",
        "post",
        Answer::Schema("ResignationOwn"),
    ),
    (
        "/api/me/equality-monitoring",
        "put",
        Answer::Schema("MyEquality"),
    ),
    (
        "/api/me/equality-monitoring",
        "delete",
        Answer::Schema("MyEquality"),
    ),
    (
        "/api/me/equality-monitoring/{}",
        "delete",
        Answer::Schema("MyEquality"),
    ),
];

/// `{name}` to `{}`, so a table can ignore what the document calls a parameter.
fn blank_params(path: &str) -> String {
    let mut out = String::new();
    let mut depth = 0;
    for c in path.chars() {
        match c {
            '{' => {
                depth += 1;
                out.push('{');
            }
            '}' => {
                depth -= 1;
                out.push('}');
            }
            _ if depth > 0 => {}
            _ => out.push(c),
        }
    }
    out
}

/// The shapes `ANSWERS` points at.
fn answer_schemas() -> Value {
    let uuid = json!({ "type": "string", "format": "uuid" });
    let date = json!({ "type": "string", "format": "date" });
    json!({
        "EmptyObject": { "type": "object", "description": "An object with nothing in it (`{}`).", "additionalProperties": false },
        "PidRef": { "type": "object", "required": ["pid"], "properties": { "pid": uuid } },
        "ReadReceipt": { "type": "object", "required": ["read"], "properties": { "read": { "type": "boolean", "enum": [true] } } },
        "ItemState": { "type": "object", "required": ["state"], "properties": { "state": { "type": "string", "enum": ["done", "reopened", "skipped"] } } },
        "MovementClosed": { "type": "object", "required": ["status"], "properties": { "status": { "type": "string", "enum": ["completed", "cancelled"] } } },
        "Deleted": { "type": "object", "required": ["deleted"], "properties": { "deleted": uuid } },
        "Removed": { "type": "object", "required": ["removed"], "properties": { "removed": uuid } },
        "CutoverPool": { "type": "object", "required": ["pid", "cutover_pid", "pool_pid"], "properties": { "pid": uuid, "cutover_pid": uuid, "pool_pid": uuid } },
        "CutoverRole": {
            "type": "object",
            "required": ["pid", "cutover_pid", "role_profile_pid", "course_ref", "lms_url", "required_headcount"],
            "properties": { "pid": uuid, "cutover_pid": uuid, "role_profile_pid": uuid, "course_ref": { "type": "string" }, "lms_url": { "type": "string", "nullable": true }, "required_headcount": { "type": "integer" } }
        },
        "OversightCriterionCreated": { "type": "object", "required": ["pid", "level_pid", "name"], "properties": { "pid": uuid, "level_pid": uuid, "name": { "type": "string" } } },
        "OversightReading": {
            "type": "object",
            "required": ["pid", "measure_pid", "period_on", "value", "evidence", "source_document"],
            "properties": { "pid": uuid, "measure_pid": uuid, "period_on": date, "value": { "type": "integer", "format": "int64", "nullable": true, "description": "Unknown is null, never zero." }, "evidence": { "type": "string", "nullable": true }, "source_document": { "type": "string", "nullable": true } }
        },
        "FlexibleRequested": { "type": "object", "required": ["pid", "decide_by"], "properties": { "pid": uuid, "decide_by": date } },
        "ResignationLogged": {
            "type": "object",
            "required": ["pid", "earliest_last_day", "notice_calendar_days"],
            "properties": { "pid": uuid, "earliest_last_day": { "type": "string", "format": "date", "nullable": true }, "notice_calendar_days": { "type": "integer", "format": "int64" } }
        },
        "AspirationChanged": { "type": "object", "required": ["pid", "status", "visibility"], "properties": { "pid": uuid, "status": { "type": "string" }, "visibility": { "type": "string" } } },
    })
}

/// The answers that are a stored row, a record built by a helper, or a view: shapes read off the handlers (WPM-T263).
/// A row from `row_json` is the `SeaORM` model without `id` and `deleted_at`, so it keeps both timestamps.
#[allow(clippy::too_many_lines)] // one literal
fn record_schemas() -> Value {
    let uuid = json!({ "type": "string", "format": "uuid" });
    let uuid_or_null = json!({ "type": "string", "format": "uuid", "nullable": true });
    let date = json!({ "type": "string", "format": "date" });
    let date_or_null = json!({ "type": "string", "format": "date", "nullable": true });
    let time = json!({ "type": "string", "format": "date-time" });
    let time_or_null = json!({ "type": "string", "format": "date-time", "nullable": true });
    let text = json!({ "type": "string" });
    let text_or_null = json!({ "type": "string", "nullable": true });
    let int = json!({ "type": "integer" });
    let int_or_null = json!({ "type": "integer", "nullable": true });
    let int64 = json!({ "type": "integer", "format": "int64" });
    let int64_or_null = json!({ "type": "integer", "format": "int64", "nullable": true });
    let boolean = json!({ "type": "boolean" });
    let rag = json!({ "type": "string", "enum": ["not_set", "green", "unknown", "amber", "red"] });
    let rag_or_null = json!({ "type": "string", "nullable": true });
    let escalation = json!({
        "type": "object",
        "required": ["pid", "plan_pid", "plan_name", "dimension", "status", "level", "level_number", "raised_by", "raised_on", "level_raised_on", "days_open", "note", "at_raise", "decision", "decided_by", "decided_on", "decision_note", "at_decision"],
        "properties": {
            "pid": uuid, "plan_pid": uuid, "plan_name": text_or_null, "dimension": text, "status": text,
            "level": { "type": "string", "nullable": true, "enum": ["plan_owner", "board", "executive", null] },
            "level_number": int, "raised_by": text_or_null, "raised_on": date, "level_raised_on": date_or_null,
            "days_open": { "type": "integer", "format": "int64", "nullable": true, "description": "Only while the escalation is open." },
            "note": text_or_null,
            "at_raise": { "type": "object", "required": ["rag", "deviation", "limit"], "properties": { "rag": text, "deviation": int64_or_null, "limit": int64 } },
            "decision": text_or_null, "decided_by": text_or_null, "decided_on": date_or_null, "decision_note": text_or_null,
            "at_decision": { "type": "object", "nullable": true, "required": ["rag", "deviation"], "properties": { "rag": rag_or_null, "deviation": int64_or_null } }
        }
    });
    let flexible = json!({
        "type": "object",
        "required": ["pid", "kind", "requested_on", "proposed_start", "proposed_fte_percent", "effect_on_team", "how_to_manage", "status", "decide_by", "overdue", "decided_on", "decision_reason", "decision_note", "counter_note", "counter_fte_percent", "trial_until", "appealed_on", "appeal_note", "can_appeal", "hr_to_apply_fte_percent"],
        "properties": {
            "pid": uuid, "kind": text, "requested_on": date, "proposed_start": date,
            "proposed_fte_percent": int_or_null, "effect_on_team": text_or_null, "how_to_manage": text_or_null,
            "status": text, "decide_by": date, "overdue": boolean, "decided_on": date_or_null,
            "decision_reason": text_or_null, "decision_note": text_or_null, "counter_note": text_or_null,
            "counter_fte_percent": int_or_null, "trial_until": date_or_null, "appealed_on": date_or_null,
            "appeal_note": text_or_null, "can_appeal": boolean,
            "hr_to_apply_fte_percent": { "type": "integer", "nullable": true, "description": "What HR applies once a change of hours is agreed; the software never changes a contract." }
        }
    });
    json!({
        "WorkerSkill": {
            "type": "object",
            "required": ["created_at", "updated_at", "pid", "worker_pid", "skill_pid", "proficiency", "target", "assessed_on"],
            "properties": { "created_at": time, "updated_at": time, "pid": uuid, "worker_pid": uuid, "skill_pid": uuid, "proficiency": int, "target": int_or_null, "assessed_on": date }
        },
        "Mentorship": {
            "type": "object",
            "required": ["created_at", "updated_at", "pid", "mentor_pid", "mentee_pid", "focus", "status", "started_on", "ended_on"],
            "properties": { "created_at": time, "updated_at": time, "pid": uuid, "mentor_pid": uuid, "mentee_pid": uuid, "focus": text, "status": text, "started_on": date_or_null, "ended_on": date_or_null }
        },
        "Appraisal": {
            "type": "object",
            "required": ["created_at", "updated_at", "pid", "worker_pid", "competencies", "status", "shared_on"],
            "properties": { "created_at": time, "updated_at": time, "pid": uuid, "worker_pid": uuid, "competencies": { "type": "array", "items": text }, "status": text, "shared_on": date_or_null }
        },
        "EntitlementAcknowledgement": {
            "type": "object",
            "required": ["created_at", "updated_at", "pid", "entitlement_pid", "worker_pid", "response", "responded_on", "reminded_on"],
            "properties": { "created_at": time, "updated_at": time, "pid": uuid, "entitlement_pid": uuid, "worker_pid": uuid, "response": text, "responded_on": date, "reminded_on": date_or_null }
        },
        "DevelopmentPlanItem": {
            "type": "object",
            "required": ["created_at", "updated_at", "pid", "plan_pid", "skill_pid", "current_level", "target_level", "method", "course_ref", "due_on", "status"],
            "properties": { "created_at": time, "updated_at": time, "pid": uuid, "plan_pid": uuid, "skill_pid": uuid, "current_level": int, "target_level": int, "method": text, "course_ref": text_or_null, "due_on": date_or_null, "status": text }
        },
        "LeaveRequest": {
            "type": "object",
            "required": ["created_at", "updated_at", "pid", "worker_pid", "kind", "start_on", "end_on", "days", "status", "negative_balance", "reason", "decided_by", "decided_at"],
            "properties": { "created_at": time, "updated_at": time, "pid": uuid, "worker_pid": uuid, "kind": text, "start_on": date, "end_on": date, "days": int, "status": text, "negative_balance": boolean, "reason": text_or_null, "decided_by": text_or_null, "decided_at": time_or_null }
        },
        "LeaveRequested": { "type": "object", "required": ["pid", "days", "negative_balance"], "properties": { "pid": uuid, "days": int, "negative_balance": boolean } },
        "CpdEntry": {
            "type": "object",
            "required": ["pid", "entry_date", "activity", "category", "unit", "amount", "amount_hundredths", "evidence_note", "evidence_url", "source", "lms_url", "verified_on", "verified_by"],
            "properties": {
                "pid": uuid, "entry_date": date, "activity": text, "category": text, "unit": text,
                "amount": { "type": "number", "format": "double", "description": "`amount_hundredths` as units." },
                "amount_hundredths": int64, "evidence_note": text_or_null, "evidence_url": text_or_null, "source": text,
                "lms_url": text_or_null, "verified_on": date_or_null, "verified_by": text_or_null
            }
        },
        "FrameworkRoleHeld": {
            "type": "object",
            "required": ["pid", "framework", "role_label", "role_profile_pid", "occupation_uri", "started_at", "ended_at", "current", "recorded_by", "on_behalf"],
            "properties": { "pid": uuid, "framework": text, "role_label": text, "role_profile_pid": uuid_or_null, "occupation_uri": text_or_null, "started_at": time, "ended_at": time_or_null, "current": boolean, "recorded_by": text_or_null, "on_behalf": boolean }
        },
        "DottedLineManager": {
            "type": "object",
            "description": "The manager's `pid`, `display_name`, `job_title` and `department` are present when the manager is known.",
            "required": ["note", "started_at", "ended_at", "current", "on_behalf"],
            "properties": {
                "pid": uuid, "display_name": text, "job_title": text, "department": text,
                "note": text_or_null, "started_at": time, "ended_at": time_or_null, "current": boolean, "on_behalf": boolean
            }
        },
        "GroupMember": {
            "type": "object",
            "required": ["worker_pid", "display_name", "job_title", "department", "role", "joined_at", "left_at", "current", "recorded_by", "on_behalf"],
            "properties": { "worker_pid": uuid, "display_name": text_or_null, "job_title": text_or_null, "department": text_or_null, "role": text, "joined_at": time, "left_at": time_or_null, "current": boolean, "recorded_by": text_or_null, "on_behalf": boolean }
        },
        "PlanTolerances": {
            "type": "object",
            "required": ["derivation", "plan", "as_of", "overall", "breaches_without_escalation", "dimensions"],
            "properties": {
                "derivation": text,
                "plan": { "type": "object", "required": ["pid", "name", "status"], "properties": { "pid": uuid, "name": text, "status": text } },
                "as_of": date, "overall": rag, "breaches_without_escalation": int64,
                "dimensions": {
                    "type": "array",
                    "items": {
                        "type": "object",
                        "required": ["dimension", "unit", "set", "limit", "amber_percent", "deviation", "rag", "basis", "reason", "escalation"],
                        "properties": {
                            "dimension": { "type": "string", "enum": ["time", "cost", "resources", "risk"] },
                            "unit": text, "set": boolean, "limit": int64_or_null, "amber_percent": int_or_null,
                            "deviation": { "type": "integer", "format": "int64", "nullable": true, "description": "Unknown is null, never zero." },
                            "rag": rag, "basis": text,
                            "reason": { "type": "string", "nullable": true, "description": "Why the deviation cannot be worked out, when it cannot." },
                            "escalation": { "nullable": true, "allOf": [escalation] }
                        }
                    }
                }
            }
        },
        "OversightLevel": {
            "type": "object",
            "required": ["pid", "organization_ref", "ordinal", "name", "synthetic"],
            "properties": { "pid": uuid, "organization_ref": text, "ordinal": int, "name": text, "synthetic": boolean }
        },
        "OversightPosition": {
            "type": "object",
            "required": ["pid", "level_pid", "level", "ordinal", "effective_on", "deciding_body", "reason"],
            "properties": { "pid": uuid, "level_pid": uuid, "level": text_or_null, "ordinal": int_or_null, "effective_on": date, "deciding_body": text, "reason": text }
        },
        "OversightMeasure": {
            "type": "object",
            "required": ["pid", "criterion_pid", "name", "origin", "derivation", "unit", "threshold", "direction", "sustain"],
            "properties": { "pid": uuid, "criterion_pid": uuid, "name": text, "origin": text, "derivation": text_or_null, "unit": text, "threshold": int64, "direction": text, "sustain": int }
        },
        "OversightPack": {
            "type": "object",
            "required": ["pid", "organization_ref", "as_of", "month", "withdrawn", "pack"],
            "properties": {
                "pid": uuid, "organization_ref": text, "as_of": date, "month": date,
                "withdrawn": { "type": "object", "nullable": true, "required": ["on", "reason"], "properties": { "on": date, "reason": text }, "description": "Null unless the pack was withdrawn." },
                "pack": { "type": "object", "description": "The frozen payload, as it was written." }
            }
        },
        "FlexibleRequest": flexible,
        "ResignationOwn": {
            "type": "object",
            "required": ["pid", "status", "logged_on", "proposed_last_day", "agreed_last_day", "reason", "notice_calendar_days"],
            "properties": { "pid": uuid, "status": text, "logged_on": date, "proposed_last_day": date, "agreed_last_day": date_or_null, "reason": text_or_null, "notice_calendar_days": int64 }
        },
        "MyEquality": {
            "oneOf": [
                { "type": "object", "required": ["enabled"], "properties": { "enabled": { "type": "boolean", "enum": [false] } } },
                {
                    "type": "object",
                    "required": ["enabled", "lawful_basis", "categories", "declarations", "small_group_floor", "notice"],
                    "properties": {
                        "enabled": { "type": "boolean", "enum": [true] },
                        "lawful_basis": text,
                        "categories": { "type": "object", "additionalProperties": { "type": "array", "items": text } },
                        "declarations": { "type": "object", "additionalProperties": text },
                        "small_group_floor": int64,
                        "notice": text
                    }
                }
            ]
        },
    })
}

/// Give the operations in `ANSWERS` their answer. The shapes are added to the components by `decorate`.
fn typed_answers(document: &mut Value) {
    if let Some(paths) = document["paths"].as_object_mut() {
        for (path, item) in paths.iter_mut() {
            let blank = blank_params(path);
            for (table_path, method, answer) in ANSWERS {
                if *table_path != blank {
                    continue;
                }
                let Some(operation) = item.get_mut(*method) else {
                    continue;
                };
                operation["responses"]["200"] = match answer {
                    Answer::NoBody => json!({ "description": "Done. The answer has no body." }),
                    Answer::Schema(name) => json!({
                        "description": "Done",
                        "content": { "application/json": { "schema": { "$ref": format!("#/components/schemas/{name}") } } }
                    }),
                };
            }
        }
    }
}

/// Add `components` and give every operation that changes something the `Idempotency-Key`
/// parameter and the two answers a key can earn. Done in one place, by code, so that no operation
/// can be missed and a new one gets it without being edited.
fn decorate(document: &mut Value) {
    document["components"] = components();
    if let (Some(into), Some(from)) = (
        document["components"]["schemas"].as_object_mut(),
        answer_schemas().as_object(),
    ) {
        into.extend(from.iter().map(|(k, v)| (k.clone(), v.clone())));
    }
    if let (Some(into), Some(from)) = (
        document["components"]["schemas"].as_object_mut(),
        record_schemas().as_object(),
    ) {
        into.extend(from.iter().map(|(k, v)| (k.clone(), v.clone())));
    }
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
    /// The UI types its calls from a copy of this document. `WPM_WRITE_OPENAPI=1 cargo test --lib
    /// the_ui_copy` rewrites it; without that, the copy must match what the service serves now.
    #[test]
    fn the_ui_copy_is_current() {
        let path = std::path::Path::new(env!("CARGO_MANIFEST_DIR"))
            .join("../workforce-planning-management-ui-with-svelte/src/lib/api/openapi.json");
        let mut now = serde_json::to_string_pretty(&super::spec()).unwrap();
        now.push('\n');
        if std::env::var_os("WPM_WRITE_OPENAPI").is_some() {
            std::fs::write(&path, &now).unwrap();
            return;
        }
        let have = std::fs::read_to_string(&path).unwrap_or_default();
        assert!(
            have == now,
            "the UI's openapi.json is out of date: run `WPM_WRITE_OPENAPI=1 cargo test --lib the_ui_copy` and then `pnpm api:types` in the UI"
        );
    }

    /// Every stated answer names an operation that exists, and every schema it points at exists.
    #[test]
    fn the_stated_answers_name_real_operations() {
        let doc = super::spec();
        let paths = doc["paths"].as_object().unwrap();
        for (path, method, answer) in super::ANSWERS {
            let hit = paths
                .iter()
                .find(|(p, item)| super::blank_params(p) == *path && item.get(*method).is_some());
            let (name, item) = hit.unwrap_or_else(|| panic!("{method} {path} is not documented"));
            let ok = &item[*method]["responses"]["200"];
            match answer {
                super::Answer::NoBody => assert!(ok.get("content").is_none(), "{name}"),
                super::Answer::Schema(schema) => {
                    assert!(
                        doc["components"]["schemas"].get(*schema).is_some(),
                        "{schema}"
                    );
                    assert_eq!(
                        ok["content"]["application/json"]["schema"]["$ref"],
                        format!("#/components/schemas/{schema}"),
                        "{name}"
                    );
                }
            }
            // The key's replay header is still there.
            assert!(
                item[*method]["responses"]["200"].get("headers").is_some()
                    || !super::MUTATING.contains(method),
                "{name}"
            );
        }
    }

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
