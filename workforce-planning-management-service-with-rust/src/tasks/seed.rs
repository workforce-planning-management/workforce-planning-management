//! `cargo loco task seed` — create a synthetic demo organization so
//! the HR views demo instantly. **Synthetic data only** (spec
//! `regulatory.md`): person/worker URNs are random UUIDs, names are
//! obviously fictional.

use loco_rs::prelude::*;
use loco_rs::task::{TaskInfo, Vars};
use sea_orm::ActiveValue;
use uuid::Uuid;

use crate::models::_entities::{
    benchmarks, benefit_plans, leave_entitlements, oversight_criteria, oversight_levels,
    oversight_measures, oversight_positions, requisitions, workers,
};

/// The demo-organization seed task.
pub struct Seed;

#[async_trait]
impl Task for Seed {
    fn task(&self) -> TaskInfo {
        TaskInfo {
            name: "seed".to_string(),
            detail:
                "Seed synthetic demo organizations (~40 workers paid in GBP, 3 in JPY; requisitions, plans, benchmarks)"
                    .to_string(),
        }
    }

    #[allow(clippy::too_many_lines)] // one linear walk building the demo org
    async fn run(&self, ctx: &AppContext, _vars: &Vars) -> Result<()> {
        let db = &ctx.db;
        let org = format!("organization:{}", Uuid::new_v4());
        let departments = ["engineering", "clinical", "operations", "finance"];
        let titles = [
            ("Engineer", 3_600_000_i64),
            ("Senior Engineer", 4_800_000),
            ("Nurse", 3_200_000),
            ("Analyst", 3_000_000),
            ("Coordinator", 2_800_000),
        ];
        let mut n = 0;
        for department in departments {
            // One department head (no manager), then nine reports.
            let head = Uuid::new_v4();
            n += 1;
            workers::ActiveModel {
                pid: ActiveValue::set(head),
                person_ref: ActiveValue::set(format!("person:{}", Uuid::new_v4())),
                upstream_worker_ref: ActiveValue::set(Some(format!("worker:{}", Uuid::new_v4()))),
                organization_ref: ActiveValue::set(org.clone()),
                work_jurisdiction: ActiveValue::set(Some("GB".to_string())),
                worker_number: ActiveValue::set(format!("E-{n:04}")),
                display_name: ActiveValue::set(format!("Test Head {n:03}")),
                status: ActiveValue::set("active".to_string()),
                employment_type: ActiveValue::set("permanent".to_string()),
                fte_percent: ActiveValue::set(100),
                department: ActiveValue::set(department.to_string()),
                job_title: ActiveValue::set("Head of Department".to_string()),
                manager_pid: ActiveValue::set(None),
                salary_minor: ActiveValue::set(Some(6_500_000)),
                salary_currency: ActiveValue::set(Some("GBP".to_string())),
                hired_on: ActiveValue::set(
                    chrono::NaiveDate::from_ymd_opt(2020, 1, 6).ok_or_else(|| {
                        Error::string("seed data: invalid department-head hired_on date")
                    })?,
                ),
                terminated_on: ActiveValue::set(None),
                deleted_at: ActiveValue::set(None),
                ..Default::default()
            }
            .insert(db)
            .await?;
            for i in 0..9 {
                n += 1;
                let (title, salary) = titles[usize::try_from(n).unwrap_or(0) % titles.len()];
                let pid = Uuid::new_v4();
                workers::ActiveModel {
                    pid: ActiveValue::set(pid),
                    person_ref: ActiveValue::set(format!("person:{}", Uuid::new_v4())),
                    upstream_worker_ref: ActiveValue::set(None),
                    organization_ref: ActiveValue::set(org.clone()),
                    work_jurisdiction: ActiveValue::set(Some("GB".to_string())),
                    worker_number: ActiveValue::set(format!("E-{n:04}")),
                    display_name: ActiveValue::set(format!("Test Worker {n:03}")),
                    status: ActiveValue::set(
                        if i == 8 { "onboarding" } else { "active" }.to_string(),
                    ),
                    employment_type: ActiveValue::set(
                        if i % 4 == 3 {
                            "fixed_term"
                        } else {
                            "permanent"
                        }
                        .to_string(),
                    ),
                    fte_percent: ActiveValue::set(if i % 5 == 4 { 60 } else { 100 }),
                    department: ActiveValue::set(department.to_string()),
                    job_title: ActiveValue::set(title.to_string()),
                    manager_pid: ActiveValue::set(Some(head)),
                    salary_minor: ActiveValue::set(Some(salary)),
                    salary_currency: ActiveValue::set(Some("GBP".to_string())),
                    hired_on: ActiveValue::set(
                        chrono::NaiveDate::from_ymd_opt(2023, 3, 1).ok_or_else(|| {
                            Error::string("seed data: invalid worker hired_on date")
                        })?,
                    ),
                    terminated_on: ActiveValue::set(None),
                    // A fixed-term engagement needs an end date (WPM-R79).
                    engagement_ends_on: ActiveValue::set(
                        (i % 4 == 3)
                            .then(|| chrono::NaiveDate::from_ymd_opt(2028, 3, 1))
                            .flatten(),
                    ),
                    deleted_at: ActiveValue::set(None),
                    ..Default::default()
                }
                .insert(db)
                .await?;
                // Annual + sick entitlements for the current year.
                for (kind, days) in [("annual", 25), ("sick", 10)] {
                    leave_entitlements::ActiveModel {
                        pid: ActiveValue::set(Uuid::new_v4()),
                        worker_pid: ActiveValue::set(pid),
                        kind: ActiveValue::set(kind.to_string()),
                        year: ActiveValue::set(2026),
                        entitled_days: ActiveValue::set(days),
                        used_days: ActiveValue::set(0),
                        deleted_at: ActiveValue::set(None),
                        ..Default::default()
                    }
                    .insert(db)
                    .await?;
                }
            }
        }
        // Requisitions across the pipeline.
        for (department, title, status) in [
            ("engineering", "Platform Engineer", "open"),
            ("clinical", "Ward Nurse", "interviewing"),
            ("operations", "Scheduler", "draft"),
            ("finance", "Payroll Analyst", "offer"),
        ] {
            requisitions::ActiveModel {
                pid: ActiveValue::set(Uuid::new_v4()),
                organization_ref: ActiveValue::set(org.clone()),
                department: ActiveValue::set(department.to_string()),
                job_title: ActiveValue::set(title.to_string()),
                headcount: ActiveValue::set(1),
                salary_min_minor: ActiveValue::set(Some(2_800_000)),
                salary_max_minor: ActiveValue::set(Some(5_000_000)),
                salary_currency: ActiveValue::set(Some("GBP".to_string())),
                status: ActiveValue::set(status.to_string()),
                opened_on: ActiveValue::set(Some(
                    chrono::NaiveDate::from_ymd_opt(2026, 6, 1).ok_or_else(|| {
                        Error::string("seed data: invalid requisition opened_on date")
                    })?,
                )),
                deleted_at: ActiveValue::set(None),
                ..Default::default()
            }
            .insert(db)
            .await?;
        }
        // Benefit plans + benchmarks.
        for (name, kind, worker_cost, employer_cost) in [
            ("Pension 5%", "pension", 15_000_i64, 30_000_i64),
            ("Health Cover", "health", 8_000, 20_000),
        ] {
            benefit_plans::ActiveModel {
                pid: ActiveValue::set(Uuid::new_v4()),
                name: ActiveValue::set(name.to_string()),
                kind: ActiveValue::set(kind.to_string()),
                provider: ActiveValue::set("Demo Provider Ltd".to_string()),
                worker_cost_minor: ActiveValue::set(worker_cost),
                employer_cost_minor: ActiveValue::set(employer_cost),
                currency: ActiveValue::set("GBP".to_string()),
                deleted_at: ActiveValue::set(None),
                ..Default::default()
            }
            .insert(db)
            .await?;
        }
        for (title, min, median, max) in [
            ("Engineer", 3_200_000_i64, 3_800_000_i64, 4_600_000_i64),
            ("Nurse", 2_900_000, 3_300_000, 3_900_000),
            ("Analyst", 2_600_000, 3_100_000, 3_700_000),
        ] {
            benchmarks::ActiveModel {
                pid: ActiveValue::set(Uuid::new_v4()),
                job_title: ActiveValue::set(title.to_string()),
                currency: ActiveValue::set("GBP".to_string()),
                min_minor: ActiveValue::set(min),
                median_minor: ActiveValue::set(median),
                max_minor: ActiveValue::set(max),
                source: ActiveValue::set("Demo Salary Survey 2026".to_string()),
                as_of: ActiveValue::set(
                    chrono::NaiveDate::from_ymd_opt(2026, 4, 1)
                        .ok_or_else(|| Error::string("seed data: invalid benchmark as_of date"))?,
                ),
                deleted_at: ActiveValue::set(None),
                ..Default::default()
            }
            .insert(db)
            .await?;
        }
        // Not only GBP (WPM-T385): money in currencies of other ISO 4217 exponents beside the
        // pounds (exponent 2). A second organization pays in yen (exponent 0, so 6,000,000 is
        // 6,000,000 yen, not 60,000), with a requisition and a benchmark in yen; a benchmark and a
        // benefit plan in Kuwaiti dinars (exponent 3, so 12_500 is 12.500 dinars). The payroll stub is
        // written for GBP, so a run for the yen organization is refused until a deployer sets a stub for
        // yen (WPM-T383); the demo runs payroll for the GBP organization only.
        let yen_org = format!("organization:{}", Uuid::new_v4());
        for (title, salary) in [
            ("Engineer", 6_000_000_i64),
            ("Analyst", 5_200_000),
            ("Coordinator", 4_400_000),
        ] {
            n += 1;
            workers::ActiveModel {
                pid: ActiveValue::set(Uuid::new_v4()),
                person_ref: ActiveValue::set(format!("person:{}", Uuid::new_v4())),
                upstream_worker_ref: ActiveValue::set(None),
                organization_ref: ActiveValue::set(yen_org.clone()),
                work_jurisdiction: ActiveValue::set(Some("JP".to_string())),
                worker_number: ActiveValue::set(format!("E-{n:04}")),
                display_name: ActiveValue::set(format!("Test Worker {n:03}")),
                status: ActiveValue::set("active".to_string()),
                employment_type: ActiveValue::set("permanent".to_string()),
                fte_percent: ActiveValue::set(100),
                department: ActiveValue::set("engineering".to_string()),
                job_title: ActiveValue::set(title.to_string()),
                manager_pid: ActiveValue::set(None),
                salary_minor: ActiveValue::set(Some(salary)),
                salary_currency: ActiveValue::set(Some("JPY".to_string())),
                hired_on: ActiveValue::set(
                    chrono::NaiveDate::from_ymd_opt(2024, 4, 1).ok_or_else(|| {
                        Error::string("seed data: invalid yen worker hired_on date")
                    })?,
                ),
                terminated_on: ActiveValue::set(None),
                deleted_at: ActiveValue::set(None),
                ..Default::default()
            }
            .insert(db)
            .await?;
        }
        requisitions::ActiveModel {
            pid: ActiveValue::set(Uuid::new_v4()),
            organization_ref: ActiveValue::set(yen_org.clone()),
            department: ActiveValue::set("engineering".to_string()),
            job_title: ActiveValue::set("Platform Engineer".to_string()),
            headcount: ActiveValue::set(1),
            salary_min_minor: ActiveValue::set(Some(5_500_000)),
            salary_max_minor: ActiveValue::set(Some(8_000_000)),
            salary_currency: ActiveValue::set(Some("JPY".to_string())),
            status: ActiveValue::set("open".to_string()),
            opened_on: ActiveValue::set(Some(
                chrono::NaiveDate::from_ymd_opt(2026, 6, 1).ok_or_else(|| {
                    Error::string("seed data: invalid yen requisition opened_on date")
                })?,
            )),
            deleted_at: ActiveValue::set(None),
            ..Default::default()
        }
        .insert(db)
        .await?;
        for (title, currency, min, median, max) in [
            (
                "Engineer",
                "JPY",
                5_200_000_i64,
                6_200_000_i64,
                7_600_000_i64,
            ),
            ("Analyst", "KWD", 7_800_000, 9_000_000, 10_800_000),
        ] {
            benchmarks::ActiveModel {
                pid: ActiveValue::set(Uuid::new_v4()),
                job_title: ActiveValue::set(title.to_string()),
                currency: ActiveValue::set(currency.to_string()),
                min_minor: ActiveValue::set(min),
                median_minor: ActiveValue::set(median),
                max_minor: ActiveValue::set(max),
                source: ActiveValue::set("Demo Salary Survey 2026".to_string()),
                as_of: ActiveValue::set(
                    chrono::NaiveDate::from_ymd_opt(2026, 4, 1)
                        .ok_or_else(|| Error::string("seed data: invalid benchmark as_of date"))?,
                ),
                deleted_at: ActiveValue::set(None),
                ..Default::default()
            }
            .insert(db)
            .await?;
        }
        benefit_plans::ActiveModel {
            pid: ActiveValue::set(Uuid::new_v4()),
            name: ActiveValue::set("Health Cover (dinars)".to_string()),
            kind: ActiveValue::set("health".to_string()),
            provider: ActiveValue::set("Demo Provider Ltd".to_string()),
            worker_cost_minor: ActiveValue::set(12_500),
            employer_cost_minor: ActiveValue::set(30_000),
            currency: ActiveValue::set("KWD".to_string()),
            deleted_at: ActiveValue::set(None),
            ..Default::default()
        }
        .insert(db)
        .await?;
        // A synthetic oversight scale (WPM-D48): the product ships no real framework, so this one is
        // labelled synthetic in its names and its flag, and exists only to make the views demonstrable.
        let mut level_pids = Vec::new();
        for (ordinal, name) in [
            (1, "Synthetic level 1 (routine)"),
            (2, "Synthetic level 2 (enhanced)"),
            (3, "Synthetic level 3 (intervention)"),
        ] {
            let pid = Uuid::new_v4();
            level_pids.push(pid);
            oversight_levels::ActiveModel {
                pid: ActiveValue::set(pid),
                organization_ref: ActiveValue::set(org.clone()),
                ordinal: ActiveValue::set(ordinal),
                name: ActiveValue::set(name.to_string()),
                synthetic: ActiveValue::set(true),
                ..Default::default()
            }
            .insert(db)
            .await?;
        }
        oversight_positions::ActiveModel {
            pid: ActiveValue::set(Uuid::new_v4()),
            organization_ref: ActiveValue::set(org.clone()),
            level_pid: ActiveValue::set(level_pids[1]),
            effective_on: ActiveValue::set(
                chrono::NaiveDate::from_ymd_opt(2026, 1, 1)
                    .ok_or_else(|| Error::string("seed data: invalid position date"))?,
            ),
            deciding_body: ActiveValue::set("Synthetic oversight panel".to_string()),
            reason: ActiveValue::set("Synthetic demonstration position.".to_string()),
            ..Default::default()
        }
        .insert(db)
        .await?;
        let criterion = Uuid::new_v4();
        oversight_criteria::ActiveModel {
            pid: ActiveValue::set(criterion),
            level_pid: ActiveValue::set(level_pids[1]),
            name: ActiveValue::set("Synthetic criterion: staffing is stable".to_string()),
            synthetic: ActiveValue::set(true),
            ..Default::default()
        }
        .insert(db)
        .await?;
        oversight_measures::ActiveModel {
            pid: ActiveValue::set(Uuid::new_v4()),
            criterion_pid: ActiveValue::set(criterion),
            name: ActiveValue::set("Synthetic vacancy rate".to_string()),
            origin: ActiveValue::set("reported".to_string()),
            unit: ActiveValue::set("basis points".to_string()),
            threshold: ActiveValue::set(800),
            direction: ActiveValue::set("at_most".to_string()),
            sustain: ActiveValue::set(3),
            ..Default::default()
        }
        .insert(db)
        .await?;
        seed_profile(db, &org, &example_profile_gb()).await?;
        seed_profile(db, &yen_org, &example_profile_jp()).await?;
        // A third organization (WPM-T385), paid in rupees, works under the example India profile, in a state with no rows of its own,
        // so its limits come from the country's layer through the chain.
        let rupee_org = format!("organization:{}", Uuid::new_v4());
        for (title, salary) in [
            ("Engineer", 180_000_000_i64),
            ("Analyst", 120_000_000),
            ("Coordinator", 90_000_000),
        ] {
            n += 1;
            workers::ActiveModel {
                pid: ActiveValue::set(Uuid::new_v4()),
                person_ref: ActiveValue::set(format!("person:{}", Uuid::new_v4())),
                organization_ref: ActiveValue::set(rupee_org.clone()),
                work_jurisdiction: ActiveValue::set(Some("IN-MH".to_string())),
                worker_number: ActiveValue::set(format!("E-{n:04}")),
                display_name: ActiveValue::set(format!("Test Worker {n:03}")),
                status: ActiveValue::set("active".to_string()),
                employment_type: ActiveValue::set("permanent".to_string()),
                fte_percent: ActiveValue::set(100),
                department: ActiveValue::set("engineering".to_string()),
                job_title: ActiveValue::set(title.to_string()),
                salary_minor: ActiveValue::set(Some(salary)),
                salary_currency: ActiveValue::set(Some("INR".to_string())),
                hired_on: ActiveValue::set(
                    chrono::NaiveDate::from_ymd_opt(2025, 4, 1).ok_or_else(|| {
                        Error::string("seed data: invalid rupee worker hired_on date")
                    })?,
                ),
                ..Default::default()
            }
            .insert(db)
            .await?;
        }
        seed_profile(db, &rupee_org, &example_profile_in()).await?;
        tracing::info!(organization = %org, yen_organization = %yen_org, workers = n, "seeded synthetic demo organizations");
        println!(
            "seeded: organization {org} (GBP, example GB profile), {yen_org} (JPY, example JP profile) and {rupee_org} (INR, example IN profile), {n} workers in all"
        );
        Ok(())
    }
}

/// One example jurisdiction profile for the demo: the codes and the parameter rows, every row labelled `example` (D144 question 1:
/// the demo always loads example profiles; none is the default and none is claimed as read).
pub struct ExampleProfile {
    /// `(code, layer, parent, name)`.
    pub codes: &'static [(
        &'static str,
        &'static str,
        Option<&'static str>,
        &'static str,
    )],
    /// `(code, key, value as JSON, effective_from, source)`.
    pub rows: &'static [(
        &'static str,
        &'static str,
        &'static str,
        &'static str,
        &'static str,
    )],
}

/// The example `GB` working-time profile: the shape of the Working Time Regulations 1998 (a 17-week reference period, an average of
/// 48 hours a week, 11 hours' daily rest), the values the service used to hard-code. **Example, unverified**: not read against the
/// Regulations in this work.
#[must_use]
pub const fn example_profile_gb() -> ExampleProfile {
    const SOURCE: &str = "Example in the shape of the Working Time Regulations 1998 (SI 1998/1833), regs 4 and 10; not read against \
                          the text (WPM-T380)";
    ExampleProfile {
        codes: &[("GB", "national", None, "United Kingdom (example)")],
        rows: &[
            (
                "GB",
                "working_time.reference_weeks",
                "17",
                "1998-10-01",
                SOURCE,
            ),
            (
                "GB",
                "working_time.max_average_weekly_minutes",
                "2880",
                "1998-10-01",
                SOURCE,
            ),
            (
                "GB",
                "working_time.min_daily_rest_minutes",
                "660",
                "1998-10-01",
                SOURCE,
            ),
        ],
    }
}

/// The example `IN` profile, the first of the two D144 question 5 named (India, then Japan): a federal shape (a state, `IN-MH`,
/// under the country, with no rows of its own), a one-week window, a 48-hour week, a Sunday-only weekend and **no** daily rest floor
/// (none was taken, so the view says "no limit configured" for it). In the shape of the Factories Act 1948, sections 51 and 52.
/// **Example, unverified**: not read against the Act in this work; a state's shops and establishments act is not modelled.
#[must_use]
pub const fn example_profile_in() -> ExampleProfile {
    const SOURCE: &str = "Example in the shape of India's Factories Act 1948, sections 51 (48 hours in a week) and 52 (a weekly \
                          holiday, here Sunday); not read against the text (WPM-T385)";
    ExampleProfile {
        codes: &[
            ("IN", "national", None, "India (example)"),
            ("IN-MH", "subnational", Some("IN"), "Maharashtra (example)"),
        ],
        rows: &[
            (
                "IN",
                "working_time.reference_weeks",
                "1",
                "1948-09-23",
                SOURCE,
            ),
            (
                "IN",
                "working_time.max_average_weekly_minutes",
                "2880",
                "1948-09-23",
                SOURCE,
            ),
            ("IN", "holidays.weekend_days", "[7]", "1948-09-23", SOURCE),
        ],
    }
}

/// The example `JP` profile, the second D144 question 5 named: a one-week window, a 40-hour week, a Saturday and Sunday weekend (a
/// common practice, not a statute: the Act asks for one rest day a week) and no daily rest floor. In the shape of the Labour
/// Standards Act 1947, Articles 32 and 35. **Example, unverified**: not read against the Act in this work.
#[must_use]
pub const fn example_profile_jp() -> ExampleProfile {
    const SOURCE: &str = "Example in the shape of Japan's Labour Standards Act 1947, Articles 32 (40 hours a week) and 35 (a \
                          weekly rest day; the Saturday and Sunday weekend is common practice); not read against the text (WPM-T385)";
    ExampleProfile {
        codes: &[("JP", "national", None, "Japan (example)")],
        rows: &[
            (
                "JP",
                "working_time.reference_weeks",
                "1",
                "1947-09-01",
                SOURCE,
            ),
            (
                "JP",
                "working_time.max_average_weekly_minutes",
                "2400",
                "1947-09-01",
                SOURCE,
            ),
            (
                "JP",
                "holidays.weekend_days",
                "[6, 7]",
                "1947-09-01",
                SOURCE,
            ),
        ],
    }
}

/// Load `profile` for `organization`: its codes, then its rows (validated against the registry; a row that does not fit is an
/// error, so the seed cannot ship a value the API would refuse).
///
/// # Errors
///
/// A database error, or a row the registry refuses.
pub async fn seed_profile<C: sea_orm::ConnectionTrait>(
    db: &C,
    organization: &str,
    profile: &ExampleProfile,
) -> Result<()> {
    use sea_orm::{DbBackend, Statement};
    for (code, layer, parent, name) in profile.codes {
        db.execute_raw(Statement::from_sql_and_values(
            DbBackend::Postgres,
            "INSERT INTO jurisdictions (organization_ref, code, layer, parent_code, name) VALUES ($1, $2, $3, $4, $5)
             ON CONFLICT (organization_ref, code) DO NOTHING",
            [
                organization.into(),
                (*code).into(),
                (*layer).into(),
                parent.map(str::to_string).into(),
                (*name).into(),
            ],
        ))
        .await?;
    }
    for (code, key, value, from, source) in profile.rows {
        let raw: serde_json::Value =
            serde_json::from_str(value).map_err(|e| Error::Message(e.to_string()))?;
        let value = crate::rules::jurisdiction::validate(key, &raw).map_err(Error::Message)?;
        let from = chrono::NaiveDate::parse_from_str(from, "%Y-%m-%d")
            .map_err(|e| Error::Message(e.to_string()))?;
        db.execute_raw(Statement::from_sql_and_values(
            DbBackend::Postgres,
            "INSERT INTO jurisdiction_parameters (pid, organization_ref, code, key, value, effective_from, source, status)
             VALUES ($1, $2, $3, $4, $5, $6, $7, 'example')",
            [
                Uuid::new_v4().into(),
                organization.into(),
                (*code).into(),
                (*key).into(),
                serde_json::to_value(value).unwrap_or_default().into(),
                from.into(),
                (*source).into(),
            ],
        ))
        .await?;
    }
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;

    /// Every example row fits the registry and every code is well formed, so the seed cannot fail on its own data.
    #[test]
    fn the_example_profiles_fit_the_registry() {
        for profile in [
            example_profile_gb(),
            example_profile_in(),
            example_profile_jp(),
        ] {
            for (code, layer, _, _) in profile.codes {
                let layer = crate::rules::jurisdiction::Layer::parse(layer).unwrap();
                assert!(
                    crate::rules::jurisdiction::layer_fits(code, layer),
                    "{code}"
                );
            }
            for (code, key, value, from, source) in profile.rows {
                assert!(crate::rules::jurisdiction::valid_code(code));
                let raw: serde_json::Value = serde_json::from_str(value).unwrap();
                assert!(
                    crate::rules::jurisdiction::validate(key, &raw).is_ok(),
                    "{key}"
                );
                assert!(chrono::NaiveDate::parse_from_str(from, "%Y-%m-%d").is_ok());
                assert!(
                    source.contains("not read"),
                    "an example says it was not read: {key}"
                );
            }
        }
    }
}
