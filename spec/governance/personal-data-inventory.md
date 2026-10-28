# Personal-data inventory

Every table that is keyed to a person, and what happens to it on a subject-access request and on erasure. **Generated** by
`workforce-planning-management-service-with-rust/tests/personal_data_inventory.rs` from the migrations and `src/controllers/privacy.rs`;
do not edit by hand. Rewrite it with `WPM_WRITE_INVENTORY=1 cargo test --test personal_data_inventory`; the test fails when it is out of date.

- **Export**: the table is in `GET /api/workers/{pid}/subject-access`.
- **Erasure**: a statement in `erasure_statements` deletes or scrubs it when a worker is erased (WPM-D22).
- **Retention**: the table is on the retention schedule ([retention-schedule.md](retention-schedule.md)); a soft-deleted row is removed for good after its horizon.
- A table with neither erasure nor retention, or no export, carries the reason in the last column.

**What this does not cover.** A table is listed because it has a column that names a person (`worker_pid`, `person_ref` and the others in the test). Personal data in a
free-text column of a table that is not keyed to a person, a table linked by some other name, and whether an export or erasure statement is *right* are for a reviewer.
Applicants (`candidates`, `applications`) have no subject-access route yet.

| Table | Linked by | Export | Erasure | Retention | Reason, where something is missing |
| --- | --- | --- | --- | --- | --- |
| `adjustment_requests` | `worker_pid` | yes | yes | wellbeing |  |
| `announcement_reads` | `worker_pid` | yes | yes | none |  |
| `applications` | `candidate_pid` | no | no | recruitment | Not exported: Applicants are data subjects too, but there is no subject-access route for them yet: a gap (WPM-T353). Removed by the retention horizon. |
| `appraisal_nominations` | `rater_pid` | no | no | none | Not exported: Says who was asked to rate whom. Raters' answers are third-party content withheld from the subject by design, and a nominee's export carries the report aggregate instead. Not erased or retained: The row (that a person was asked to rate) stays with the appraisal it belongs to; erasure scrubs the rater's free-text comments on it. Not yet on the retention schedule: a gap (WPM-T353). |
| `appraisals` | `worker_pid` | yes | yes | performance |  |
| `asset_issues` | `worker_pid` | yes | yes | none |  |
| `assignments` | `worker_pid` | yes | yes | none |  |
| `benefit_enrollments` | `worker_pid` | yes | no | pay |  |
| `candidates` | `person_ref` | no | yes | recruitment | Not exported: As for applications: no subject-access route for applicants yet (WPM-T353). Removed when consent expires and by the retention horizon. |
| `conversion_plans` | `worker_pid` | yes | yes | none |  |
| `cpd_entries` | `worker_pid` | yes | yes | learning |  |
| `credential_issuances` | `worker_pid` | yes | yes | none |  |
| `development_plans` | `worker_pid` | yes | no | performance |  |
| `dotted_line_reports` | `report_pid`, `manager_pid` | yes | yes | none |  |
| `emergency_contacts` | `worker_pid` | yes | yes | employment |  |
| `engagement_decisions` | `worker_pid` | yes | yes | none |  |
| `engagement_extensions` | `worker_pid` | yes | yes | none |  |
| `engagement_reminders` | `worker_pid` | yes | yes | none |  |
| `engagement_status_assessments` | `worker_pid` | yes | yes | none |  |
| `entitlement_acknowledgements` | `worker_pid` | yes | yes | none |  |
| `equality_declarations` | `worker_pid` | yes | yes | none |  |
| `erasure_ledger` | `worker_pid` | no | no | none | Not exported: Records only that an erasure happened (a pid and a time, WPM-D63); it is not data about the person. Not erased or retained: Must stay: it is what makes a restore erase the person again (WPM-D63). It holds a pid and a time. |
| `ergonomic_assessments` | `worker_pid` | yes | yes | wellbeing |  |
| `expense_claims` | `worker_pid` | yes | yes | pay |  |
| `expense_items` | `worker_pid` | yes | yes | none |  |
| `flexible_working_requests` | `worker_pid` | yes | yes | none |  |
| `group_members` | `worker_pid` | yes | yes | none |  |
| `handover_actions` | `from_worker`, `to_worker` | yes | yes | none |  |
| `leave_entitlements` | `worker_pid` | yes | no | timeandleave |  |
| `leave_requests` | `worker_pid` | yes | no | timeandleave |  |
| `mobility_interests` | `worker_pid` | yes | yes | performance |  |
| `movement_items` | `assignee_pid` | no | yes | none | Not exported: A task assigned to a worker about someone else's move; the worker's own joiner and leaver record is exported. Erasure unassigns it. |
| `movements` | `worker_pid` | yes | yes | none |  |
| `notifications` | `worker_pid` | yes | yes | none |  |
| `onboarding_items` | `worker_pid` | yes | yes | recruitment |  |
| `organization_memberships` | `worker_pid`, `person_ref` | yes | yes | none |  |
| `path_enrollments` | `worker_pid` | yes | no | learning |  |
| `pay_information_requests` | `worker_pid` | yes | yes | pay |  |
| `payslips` | `worker_pid` | yes | no | pay |  |
| `policy_acknowledgements` | `worker_pid` | yes | yes | none |  |
| `positions` | `holder_worker_pid` | yes | yes | none |  |
| `post_fundings` | `worker_pid` | yes | yes | none |  |
| `professional_registrations` | `worker_pid` | yes | yes | learning |  |
| `program_placements` | `worker_pid` | yes | no | performance |  |
| `resignations` | `worker_pid` | yes | yes | none |  |
| `reviews` | `worker_pid` | yes | no | performance |  |
| `rota_members` | `worker_pid` | yes | yes | none |  |
| `rota_overrides` | `worker_pid` | yes | yes | none |  |
| `rota_swap_requests` | `requester_pid`, `taker_pid` | yes | yes | none |  |
| `scim_pushes` | `worker_pid` | yes | yes | none |  |
| `shift_assignments` | `worker_pid` | yes | no | timeandleave |  |
| `succession_candidates` | `worker_pid` | no | yes | performance | Not exported: Readiness and rank are withheld from the person by design (WPM-D128); a redacted view of the person's own entry is not built yet. Erased with the worker. |
| `time_entries` | `worker_pid` | yes | yes | timeandleave |  |
| `time_entry_weeks` | `worker_pid` | yes | yes | none |  |
| `training_enrollments` | `worker_pid` | yes | no | learning |  |
| `worker_aspirations` | `worker_pid` | yes | yes | employment |  |
| `worker_backups` | `worker_pid`, `backup_pid` | yes | yes | employment |  |
| `worker_contact_details` | `worker_pid` | yes | yes | none |  |
| `worker_contractor_details` | `worker_pid` | yes | yes | none |  |
| `worker_framework_roles` | `worker_pid` | yes | yes | none |  |
| `worker_health_records` | `worker_pid` | yes | yes | none |  |
| `worker_job_levels` | `worker_pid` | yes | yes | none |  |
| `worker_pay_positions` | `worker_pid` | yes | yes | none |  |
| `worker_skill_history` | `worker_pid` | yes | yes | none |  |
| `worker_skills` | `worker_pid` | yes | no | learning |  |
| `workers` | `person_ref`, `manager_pid` | yes | yes | employment |  |
