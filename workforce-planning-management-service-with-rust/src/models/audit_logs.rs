//! `audit_logs` model — the audit trail's chain report and the guard that keeps each audit row in its change's transaction.
//! The row is written by [`crate::db::audit`] and the chain is checked by `controllers::audits::verify_chain`.
//!
//! Every mutation **and every sensitive read** (salary-bearing worker reads,
//! payslips, review content, succession plans) writes one row. The `snapshot` JSON carries the action
//! detail (old/new state, override reasons, the `department` where
//! relevant — which is what the department-scoped query filters on).

/// The result of checking the audit chain.
#[derive(Debug, Clone, PartialEq, Eq, serde::Serialize)]
pub struct ChainReport {
    /// Whether every entry matched its hash and its place.
    pub ok: bool,
    /// How many entries there are.
    pub entries: i64,
    /// The newest entry's hash: record it somewhere the database cannot reach, and compare it
    /// after a restore.
    pub head_hash: Option<String>,
    /// The `chain_seq` of the first entry that does not match, or `None`.
    pub first_break: Option<i64>,
}

/// A source guard (WPM-T250): an audit row commits with the change it records, so every
/// `db::audit` outside a transaction must be on [`READ_AUDITS`], the audits of reads.
#[cfg(test)]
mod tests {
    /// `(file under src/, action)`: audits of a read, which have no write to share a transaction with.
    const READ_AUDITS: &[(&str, &str)] = &[
        ("controllers/engagements.rs", "rate_read"),
        ("controllers/pay_transparency.rs", "pay_gap_report_read"),
        ("controllers/human_capital.rs", "human_capital_report_read"),
        ("controllers/pay_range.rs", "pay_range_report_read"),
        ("controllers/policies.rs", "policy_acknowledgements_read"),
        ("controllers/mcp.rs", "mcp_tool_called"),
        ("controllers/pay_assessment.rs", "joint_assessment_read"),
        (
            "controllers/pay_information.rs",
            "pay_information_answer_read",
        ),
        ("controllers/pay_transparency.rs", "pay_gap_report_exported"),
        ("controllers/adjustments.rs", "adjustments_read"),
        ("controllers/my_payslips.rs", "my_payslips_read"),
        ("controllers/my_payslips.rs", "payslip_queries_read"),
        ("controllers/development.rs", "review_content_read"),
        ("controllers/padr.rs", "padr_read"),
        ("controllers/separation.rs", "separation_read"),
        ("controllers/movements.rs", "movement_why_read"),
        ("controllers/injury_records.rs", "injury_records_read"),
        ("controllers/separation.rs", "separation_exported"),
        ("controllers/development.rs", "succession_read"),
        ("controllers/conversion_plans.rs", "conversion_plans_read"),
        ("controllers/conversion_plans.rs", "conversion_plans_listed"),
        ("controllers/me.rs", "contact_details_read"),
        ("controllers/appraisals.rs", "report_read"),
        ("controllers/funding.rs", "contingent_list_read"),
        ("controllers/funding.rs", "contingent_workforce_read"),
        ("controllers/key_person.rs", "key_person_read"),
        ("controllers/hr_core.rs", "salary_read"),
        ("controllers/individual_goals.rs", "goals_read"),
        ("controllers/training_economics.rs", "salary_read"),
        ("controllers/health_requirements.rs", "health_record_read"),
        ("controllers/privacy.rs", "subject_access_exported"),
        ("controllers/assessments.rs", "results_read"),
        (
            "controllers/equality_monitoring.rs",
            "equality_monitoring_summary_read",
        ),
        ("controllers/programmes.rs", "programme_rollup_read"),
        ("controllers/payroll.rs", "payslips_read"),
        // A read: the minimum-wage check changes nothing, and its entry holds counts (WPM-D156).
        ("controllers/payroll.rs", "minimum_wage_checked"),
        // A read: the overtime check changes nothing, and its entry holds counts (WPM-D157).
        ("controllers/payroll.rs", "overtime_checked"),
        ("controllers/payroll.rs", "comparison_read"),
    ];

    /// The audit writer (WPM-D184).
    const WRITER: &str = "db::audit(";

    /// The connection arguments that are, or are handed, a transaction.
    const IN_TRANSACTION: &[&str] = &["&txn", "txn", "db", "&mut tx", "connection"];

    /// The top-level arguments of the call that opens at `open` (the index after its `(`).
    fn arguments(source: &str, open: usize) -> Vec<String> {
        let mut out = vec![String::new()];
        let mut depth = 0usize;
        let mut in_string = false;
        let mut previous = '\0';
        for c in source[open..].chars() {
            if in_string {
                out.last_mut().unwrap().push(c);
                if c == '"' && previous != '\\' {
                    in_string = false;
                }
            } else {
                match c {
                    '"' => {
                        in_string = true;
                        out.last_mut().unwrap().push(c);
                    }
                    '(' | '[' | '{' => {
                        depth += 1;
                        out.last_mut().unwrap().push(c);
                    }
                    ')' | ']' | '}' if depth == 0 => break,
                    ')' | ']' | '}' => {
                        depth -= 1;
                        out.last_mut().unwrap().push(c);
                    }
                    ',' if depth == 0 => out.push(String::new()),
                    _ => out.last_mut().unwrap().push(c),
                }
            }
            previous = c;
        }
        out.into_iter().map(|a| a.trim().to_string()).collect()
    }

    fn sources(dir: &std::path::Path, out: &mut Vec<std::path::PathBuf>) {
        for entry in std::fs::read_dir(dir).unwrap() {
            let path = entry.unwrap().path();
            if path.is_dir() {
                sources(&path, out);
            } else if path.extension().is_some_and(|e| e == "rs") {
                out.push(path);
            }
        }
    }

    #[test]
    fn every_audit_of_a_change_is_in_its_transaction() {
        let root = std::path::Path::new(env!("CARGO_MANIFEST_DIR")).join("src");
        let mut files = Vec::new();
        sources(&root, &mut files);
        let mut outside = Vec::new();
        let mut seen_reads = Vec::new();
        for path in files {
            let file = path
                .strip_prefix(&root)
                .unwrap()
                .to_string_lossy()
                .replace('\\', "/");
            if file == "models/audit_logs.rs" {
                continue;
            }
            let source = std::fs::read_to_string(&path).unwrap();
            for (at, _) in source.match_indices(WRITER) {
                let args = arguments(&source, at + WRITER.len());
                let action = args.get(3).map_or("", |a| a.trim_matches('"')).to_string();
                if IN_TRANSACTION.contains(&args[0].as_str()) {
                    continue;
                }
                if READ_AUDITS.contains(&(file.as_str(), action.as_str())) {
                    seen_reads.push((file.clone(), action));
                } else {
                    let line = source[..at].lines().count();
                    outside.push(format!("{file}:{line} {} {action}", args[0]));
                }
            }
        }
        assert!(
            outside.is_empty(),
            "audit rows written outside the change's transaction:\n{}",
            outside.join("\n")
        );
        for (file, action) in READ_AUDITS {
            assert!(
                seen_reads.iter().any(|(f, a)| f == file && a == action),
                "READ_AUDITS lists {file} {action}, which no longer exists"
            );
        }
    }

    #[test]
    fn arguments_split_at_the_top_level_only() {
        let source = r#"db::audit(&mut *db::pool(&ctx).acquire().await?, "worker", w.pid, "moved, twice", caller.actor(), Some(json!({ "a": [1, 2] }))).await"#;
        let args = arguments(source, "db::audit(".len());
        assert_eq!(args.len(), 6);
        assert_eq!(args[0], "&mut *db::pool(&ctx).acquire().await?");
        assert_eq!(args[3], "\"moved, twice\"");
        assert_eq!(args[5], r#"Some(json!({ "a": [1, 2] }))"#);
    }
}
