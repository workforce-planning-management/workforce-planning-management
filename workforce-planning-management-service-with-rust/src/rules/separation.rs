//! The separation record (WPM-R222). DB-free and clock-free.
//!
//! An authority's separation paper is built from the leaver record: the last day and the reason are read from it and never stored
//! twice. HR adds the final pay date and a reference to the pay summary, which is a pointer and never an amount. A field not yet
//! recorded is named as missing, so a gap is never read as a finished record. WPM submits nothing.

use chrono::NaiveDate;

use super::csv::{field, row};

/// The longest pay summary reference.
pub const MAX_REF_CHARS: usize = 200;

/// The export's header.
const CSV_HEADER: &str =
    "worker_number,display_name,last_day,reason_code,final_pay_on,pay_summary_ref,missing\r\n";

/// Tidy a pay summary reference: trimmed, blank is none, one line and not too long.
///
/// # Errors
/// A sentence saying what is wrong.
pub fn clean_ref(reference: Option<&str>) -> Result<Option<String>, String> {
    let reference = reference.map(str::trim).filter(|r| !r.is_empty());
    if reference.is_some_and(|r| r.chars().count() > MAX_REF_CHARS || r.contains(['\n', '\r'])) {
        return Err(format!(
            "the pay summary reference is one line of at most {MAX_REF_CHARS} characters"
        ));
    }
    Ok(reference.map(String::from))
}

/// What the record still lacks.
#[must_use]
pub fn missing(
    final_pay_on: Option<NaiveDate>,
    pay_summary_ref: Option<&str>,
) -> Vec<&'static str> {
    let mut gaps = Vec::new();
    if final_pay_on.is_none() {
        gaps.push("final_pay_on");
    }
    if pay_summary_ref.is_none() {
        gaps.push("pay_summary_ref");
    }
    gaps
}

/// The period the export covers must run forwards.
///
/// # Errors
/// A sentence saying what is wrong.
pub fn valid_period(from: NaiveDate, to: NaiveDate) -> Result<(), String> {
    if to < from {
        return Err("the period ends before it starts".to_string());
    }
    Ok(())
}

/// One leaver in the export.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Line {
    /// The worker's number.
    pub worker_number: String,
    /// Their name.
    pub display_name: String,
    /// The last day, from the leaver record.
    pub last_day: NaiveDate,
    /// The reason, from the leaver record. `None` is shown blank and is listed as missing.
    pub reason_code: Option<String>,
    /// The final pay date.
    pub final_pay_on: Option<NaiveDate>,
    /// The pay summary reference.
    pub pay_summary_ref: Option<String>,
}

/// The export. Every text field goes through [`field`], so a name cannot open as a spreadsheet formula.
#[must_use]
pub fn to_csv(lines: &[Line]) -> String {
    let mut text = CSV_HEADER.to_string();
    for line in lines {
        let mut gaps = missing(line.final_pay_on, line.pay_summary_ref.as_deref());
        if line.reason_code.is_none() {
            gaps.insert(0, "reason_code");
        }
        text.push_str(&row(&[
            field(&line.worker_number),
            field(&line.display_name),
            line.last_day.to_string(),
            field(line.reason_code.as_deref().unwrap_or_default()),
            line.final_pay_on.map(|d| d.to_string()).unwrap_or_default(),
            field(line.pay_summary_ref.as_deref().unwrap_or_default()),
            field(&gaps.join(" ")),
        ]));
    }
    text
}

#[cfg(test)]
mod tests {
    use super::*;

    fn day(month: u32, date: u32) -> NaiveDate {
        NaiveDate::from_ymd_opt(2026, month, date).expect("a date")
    }

    fn line() -> Line {
        Line {
            worker_number: "W-1".to_string(),
            display_name: "Ann Report".to_string(),
            last_day: day(3, 31),
            reason_code: Some("resignation".to_string()),
            final_pay_on: Some(day(4, 25)),
            pay_summary_ref: Some("payroll:run-42".to_string()),
        }
    }

    #[test]
    fn a_reference_is_trimmed_blank_is_none_and_it_is_one_short_line() {
        assert_eq!(clean_ref(Some("  a  ")).unwrap().as_deref(), Some("a"));
        assert_eq!(clean_ref(Some("   ")).unwrap(), None);
        assert_eq!(clean_ref(None).unwrap(), None);
        assert!(clean_ref(Some(&"x".repeat(MAX_REF_CHARS))).is_ok());
        assert!(clean_ref(Some(&"x".repeat(MAX_REF_CHARS + 1))).is_err());
        assert!(clean_ref(Some("a\nb")).is_err());
    }

    #[test]
    fn each_gap_is_named() {
        assert_eq!(missing(None, None), vec!["final_pay_on", "pay_summary_ref"]);
        assert_eq!(missing(Some(day(1, 1)), None), vec!["pay_summary_ref"]);
        assert_eq!(missing(None, Some("r")), vec!["final_pay_on"]);
        assert_eq!(missing(Some(day(1, 1)), Some("r")), Vec::<&str>::new());
    }

    #[test]
    fn a_period_runs_forwards() {
        assert!(valid_period(day(1, 1), day(1, 1)).is_ok());
        assert!(valid_period(day(1, 2), day(1, 1)).is_err());
    }

    #[test]
    fn the_export_has_a_row_for_a_complete_and_for_an_empty_record() {
        let empty = Line {
            reason_code: None,
            final_pay_on: None,
            pay_summary_ref: None,
            ..line()
        };
        let text = to_csv(&[line(), empty]);
        let rows: Vec<&str> = text.split("\r\n").collect();
        assert_eq!(format!("{}\r\n", rows[0]), CSV_HEADER);
        assert_eq!(
            rows[1],
            "W-1,Ann Report,2026-03-31,resignation,2026-04-25,payroll:run-42,"
        );
        assert_eq!(
            rows[2],
            "W-1,Ann Report,2026-03-31,,,,reason_code final_pay_on pay_summary_ref"
        );
    }

    #[test]
    fn a_name_cannot_open_as_a_formula() {
        let hostile = Line {
            display_name: "=HYPERLINK(1)".to_string(),
            pay_summary_ref: Some("+1".to_string()),
            ..line()
        };
        let text = to_csv(&[hostile]);
        assert!(text.contains("'=HYPERLINK(1)"));
        assert!(text.contains("'+1"));
    }
}
