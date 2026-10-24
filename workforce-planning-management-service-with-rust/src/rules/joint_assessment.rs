//! **The joint pay assessment signal** (WPM-R146, WPM-D99): pure rules that say, for one category of workers, whether the
//! pay report makes a joint pay assessment with workers' representatives due (Article 10(1) of the EU Pay Transparency
//! Directive). Spec: `spec/pay-transparency.md`.
//!
//! It **reports; a person decides.** The Directive makes an assessment due when a category shows an average pay gap of at
//! least a threshold (5%), the employer has not justified it on objective, gender-neutral criteria, and has not remedied it
//! within a period (six months) of the date of reporting. WPM computes the clock from the indicators and what a person has
//! recorded; it never concludes that anyone was paid unfairly, and a gap it cannot see is `unknown`, not "no assessment due".
//! The threshold and the period are configuration whose defaults are the Directive's own.

use chrono::{Months, NaiveDate};
use serde::Serialize;

use super::pay_gap::{Figure, Gap};

/// The default threshold: five percent, in basis points.
pub const DEFAULT_THRESHOLD_BP: i64 = 500;
/// The default time to remedy a gap, in calendar months, from the date of reporting.
pub const DEFAULT_REMEDY_MONTHS: u32 = 6;
/// The longest remedy period a deployment may set.
pub const MAX_REMEDY_MONTHS: u32 = 24;
/// The highest threshold a deployment may set (50%).
pub const MAX_THRESHOLD_BP: i64 = 5_000;
/// How an assessment ended, as a person recorded it. A closed list: never a verdict on a person.
pub const OUTCOMES: &[&str] = &[
    "no_unjustified_difference",
    "difference_found_remedy_planned",
    "difference_remedied",
];

/// Threshold and period.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct Config {
    /// A gap of at least this many basis points, either way, counts.
    pub threshold_bp: i64,
    /// Calendar months from the date of reporting that the employer has to remedy.
    pub remedy_months: u32,
}

impl Default for Config {
    fn default() -> Self {
        Self {
            threshold_bp: DEFAULT_THRESHOLD_BP,
            remedy_months: DEFAULT_REMEDY_MONTHS,
        }
    }
}

/// What a person has recorded for a category and a report.
#[derive(Debug, Clone, PartialEq, Eq, Default)]
pub struct Record {
    /// The employer has justified the gap on objective, gender-neutral criteria.
    pub justified: bool,
    /// The day the gap was remedied, if it was.
    pub remedied_on: Option<NaiveDate>,
    /// The day the joint assessment was carried out, if it was.
    pub assessed_on: Option<NaiveDate>,
}

/// Why no assessment is due.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize)]
#[serde(rename_all = "snake_case")]
pub enum NotDue {
    /// The gap is under the threshold.
    BelowThreshold,
    /// The employer recorded a justification.
    Justified,
    /// The gap was remedied within the period.
    Remedied,
    /// The assessment has been carried out.
    Assessed,
}

/// Why the signal cannot be given.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize)]
#[serde(rename_all = "snake_case")]
pub enum Unknown {
    /// The gap is withheld because a sex has fewer workers than the floor.
    GapWithheld,
    /// The gap cannot be worked out (nobody on a side, or no pay recorded).
    GapUnknown,
}

/// The signal for one category.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize)]
#[serde(tag = "status", rename_all = "snake_case")]
pub enum Signal {
    /// Cannot be said.
    Unknown {
        /// Why.
        why: Unknown,
    },
    /// No assessment is due.
    NotDue {
        /// Why not.
        why: NotDue,
    },
    /// The gap is at or over the threshold and the employer has until `remedy_by` (none until it has reported).
    Pending {
        /// The last day to justify or remedy, six months after reporting by default.
        remedy_by: Option<NaiveDate>,
    },
    /// The period has run out with the gap neither justified nor remedied: a joint assessment is due.
    Due {
        /// The last day the employer had.
        remedy_by: NaiveDate,
    },
}

/// The last day to remedy: `remedy_months` calendar months after the date of reporting.
#[must_use]
pub fn remedy_by(reported_on: NaiveDate, config: Config) -> Option<NaiveDate> {
    reported_on.checked_add_months(Months::new(config.remedy_months))
}

/// The signal for a category on `today`.
///
/// `gap` is the category's gap in basic pay; `reported_on` the date of reporting (`None` before the report is made).
#[must_use]
pub fn signal(
    gap: &Figure<Gap>,
    record: Option<&Record>,
    reported_on: Option<NaiveDate>,
    config: Config,
    today: NaiveDate,
) -> Signal {
    let gap_bp = match gap {
        Figure::Shown(g) => g.gap_bp,
        Figure::Withheld => {
            return Signal::Unknown {
                why: Unknown::GapWithheld,
            };
        }
        Figure::Unknown(_) => {
            return Signal::Unknown {
                why: Unknown::GapUnknown,
            };
        }
    };
    if gap_bp.abs() < config.threshold_bp {
        return Signal::NotDue {
            why: NotDue::BelowThreshold,
        };
    }
    let by = reported_on.and_then(|on| remedy_by(on, config));
    if let Some(record) = record {
        if record.assessed_on.is_some() {
            return Signal::NotDue {
                why: NotDue::Assessed,
            };
        }
        if record.justified {
            return Signal::NotDue {
                why: NotDue::Justified,
            };
        }
        // Remedied in time: on or before the last day. A remedy after it does not undo that the period ran out.
        if let Some(done) = record.remedied_on
            && by.is_none_or(|last| done <= last)
        {
            return Signal::NotDue {
                why: NotDue::Remedied,
            };
        }
    }
    match by {
        None => Signal::Pending { remedy_by: None },
        Some(last) if today <= last => Signal::Pending {
            remedy_by: Some(last),
        },
        Some(last) => Signal::Due { remedy_by: last },
    }
}

/// Check what a person recorded.
///
/// # Errors
///
/// A justification stated in too few words, a remedy or an assessment dated before the report, an outcome not on the
/// closed list, or an outcome with no assessment date.
pub fn validate_record(
    reported_on: NaiveDate,
    justified: bool,
    justification: Option<&str>,
    remedied_on: Option<NaiveDate>,
    assessed_on: Option<NaiveDate>,
    outcome: Option<&str>,
) -> Result<(), String> {
    let words = justification.map_or(0, |t| t.trim().chars().count());
    if justified && !(10..=1_000).contains(&words) {
        return Err(
            "say on what objective, gender-neutral criteria the gap is justified (10 to 1,000 characters)"
                .to_string(),
        );
    }
    if !justified && words > 0 {
        return Err("a justification is only recorded with justified: true".to_string());
    }
    if remedied_on.is_some_and(|d| d < reported_on) || assessed_on.is_some_and(|d| d < reported_on)
    {
        return Err("a remedy or an assessment cannot be dated before the report".to_string());
    }
    match (assessed_on, outcome) {
        (Some(_), Some(o)) if OUTCOMES.contains(&o) => Ok(()),
        (Some(_), Some(_)) => Err(format!("outcome is one of {}", OUTCOMES.join(", "))),
        (Some(_), None) => Err("record the outcome of the assessment".to_string()),
        (None, Some(_)) => Err("an outcome needs the date of the assessment".to_string()),
        (None, None) => Ok(()),
    }
}

/// The configuration, checked.
///
/// # Errors
///
/// A threshold outside 1 to 5,000 basis points or a period outside 1 to 24 calendar months.
pub fn config(threshold_bp: i64, remedy_months: u32) -> Result<Config, String> {
    if !(1..=MAX_THRESHOLD_BP).contains(&threshold_bp) {
        return Err(format!(
            "the threshold is 1 to {MAX_THRESHOLD_BP} basis points"
        ));
    }
    if !(1..=MAX_REMEDY_MONTHS).contains(&remedy_months) {
        return Err(format!(
            "the period is 1 to {MAX_REMEDY_MONTHS} calendar months"
        ));
    }
    Ok(Config {
        threshold_bp,
        remedy_months,
    })
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::rules::pay_gap::Reason;

    fn day(y: i32, m: u32, d: u32) -> NaiveDate {
        NaiveDate::from_ymd_opt(y, m, d).unwrap()
    }

    fn gap(bp: i64) -> Figure<Gap> {
        Figure::Shown(Gap {
            female_minor: 1,
            male_minor: 2,
            gap_bp: bp,
        })
    }

    const REPORT: (i32, u32, u32) = (2027, 6, 7);
    #[allow(clippy::unnecessary_wraps)] // the argument it stands for is an `Option`
    fn reported() -> Option<NaiveDate> {
        Some(day(REPORT.0, REPORT.1, REPORT.2))
    }

    #[test]
    fn a_gap_that_cannot_be_seen_is_unknown_not_no_assessment() {
        let c = Config::default();
        let t = day(2030, 1, 1);
        assert_eq!(
            signal(&Figure::Withheld, None, reported(), c, t),
            Signal::Unknown {
                why: Unknown::GapWithheld
            }
        );
        assert_eq!(
            signal(&Figure::Unknown(Reason::OneSided), None, reported(), c, t),
            Signal::Unknown {
                why: Unknown::GapUnknown
            }
        );
        // Even a long overdue report says unknown when the gap itself is hidden.
        assert!(matches!(
            signal(&Figure::Withheld, None, None, c, t),
            Signal::Unknown { .. }
        ));
    }

    #[test]
    fn the_threshold_is_at_or_over_and_either_way() {
        let c = Config::default();
        let t = day(2027, 7, 1);
        let at = |bp| signal(&gap(bp), None, reported(), c, t);
        assert_eq!(
            at(499),
            Signal::NotDue {
                why: NotDue::BelowThreshold
            }
        );
        assert!(
            matches!(at(500), Signal::Pending { .. }),
            "exactly the threshold counts"
        );
        assert!(matches!(at(501), Signal::Pending { .. }));
        assert_eq!(
            at(-499),
            Signal::NotDue {
                why: NotDue::BelowThreshold
            }
        );
        assert!(
            matches!(at(-500), Signal::Pending { .. }),
            "women paid more by the same margin counts too"
        );
        assert_eq!(
            at(0),
            Signal::NotDue {
                why: NotDue::BelowThreshold
            }
        );
    }

    #[test]
    fn it_is_pending_for_six_calendar_months_after_reporting_then_due() {
        let c = Config::default();
        let last = day(2027, 12, 7);
        let g = gap(800);
        assert_eq!(
            signal(&g, None, reported(), c, day(2027, 6, 8)),
            Signal::Pending {
                remedy_by: Some(last)
            }
        );
        assert_eq!(
            signal(&g, None, reported(), c, last),
            Signal::Pending {
                remedy_by: Some(last)
            },
            "the last day is still in time"
        );
        assert_eq!(
            signal(&g, None, reported(), c, day(2027, 12, 8)),
            Signal::Due { remedy_by: last }
        );
        assert_eq!(
            signal(&g, None, reported(), c, day(2031, 1, 1)),
            Signal::Due { remedy_by: last },
            "and stays due"
        );
    }

    #[test]
    fn before_a_report_is_made_the_clock_has_not_started() {
        let s = signal(&gap(800), None, None, Config::default(), day(2030, 1, 1));
        assert_eq!(s, Signal::Pending { remedy_by: None });
    }

    #[test]
    fn a_month_end_report_clamps_the_last_day() {
        let c = Config::default();
        assert_eq!(
            remedy_by(day(2027, 8, 31), c),
            Some(day(2028, 2, 29)),
            "to a leap day when there is one"
        );
        assert_eq!(remedy_by(day(2026, 8, 31), c), Some(day(2027, 2, 28)));
        assert_eq!(remedy_by(NaiveDate::MAX, c), None);
    }

    #[test]
    fn a_recorded_justification_or_assessment_means_none_is_due() {
        let c = Config::default();
        let late = day(2029, 1, 1);
        let just = Record {
            justified: true,
            ..Record::default()
        };
        assert_eq!(
            signal(&gap(900), Some(&just), reported(), c, late),
            Signal::NotDue {
                why: NotDue::Justified
            }
        );
        let assessed = Record {
            assessed_on: Some(day(2028, 3, 1)),
            ..Record::default()
        };
        assert_eq!(
            signal(&gap(900), Some(&assessed), reported(), c, late),
            Signal::NotDue {
                why: NotDue::Assessed
            }
        );
        // An assessment outranks a justification: it was carried out.
        let both = Record {
            justified: true,
            assessed_on: Some(day(2028, 3, 1)),
            remedied_on: None,
        };
        assert_eq!(
            signal(&gap(900), Some(&both), reported(), c, late),
            Signal::NotDue {
                why: NotDue::Assessed
            }
        );
        // A record for a gap under the threshold changes nothing.
        assert_eq!(
            signal(&gap(100), Some(&just), reported(), c, late),
            Signal::NotDue {
                why: NotDue::BelowThreshold
            }
        );
    }

    #[test]
    fn a_remedy_counts_only_if_it_was_in_time() {
        let c = Config::default();
        let late = day(2029, 1, 1);
        let in_time = Record {
            remedied_on: Some(day(2027, 12, 7)),
            ..Record::default()
        };
        assert_eq!(
            signal(&gap(900), Some(&in_time), reported(), c, late),
            Signal::NotDue {
                why: NotDue::Remedied
            }
        );
        let too_late = Record {
            remedied_on: Some(day(2027, 12, 8)),
            ..Record::default()
        };
        assert_eq!(
            signal(&gap(900), Some(&too_late), reported(), c, late),
            Signal::Due {
                remedy_by: day(2027, 12, 7)
            },
            "the period ran out first"
        );
        // Before a report is made any remedy is simply recorded: no last day yet to be late for.
        assert_eq!(
            signal(&gap(900), Some(&too_late), None, c, late),
            Signal::NotDue {
                why: NotDue::Remedied
            }
        );
    }

    #[test]
    fn the_threshold_and_period_are_configuration() {
        let strict = config(300, 3).unwrap();
        assert!(matches!(
            signal(&gap(350), None, reported(), strict, day(2027, 7, 1)),
            Signal::Pending { .. }
        ));
        assert_eq!(
            signal(&gap(350), None, reported(), strict, day(2027, 9, 8)),
            Signal::Due {
                remedy_by: day(2027, 9, 7)
            }
        );
        for (bp, months) in [(0, 6), (5_001, 6), (500, 0), (500, 25), (-1, 6)] {
            assert!(config(bp, months).is_err(), "{bp}/{months}");
        }
        assert_eq!(config(500, 6).unwrap(), Config::default());
    }

    #[test]
    fn what_a_person_records_is_checked() {
        let on = day(2027, 6, 7);
        let ok = |j: bool,
                  t: Option<&str>,
                  r: Option<NaiveDate>,
                  a: Option<NaiveDate>,
                  o: Option<&str>| validate_record(on, j, t, r, a, o);
        assert!(ok(false, None, None, None, None).is_ok());
        assert!(
            ok(
                true,
                Some("Different grades, set by a published job evaluation"),
                None,
                None,
                None
            )
            .is_ok()
        );
        assert!(
            ok(true, None, None, None, None)
                .unwrap_err()
                .contains("objective"),
            "a justification must be stated"
        );
        assert!(ok(true, Some("short"), None, None, None).is_err());
        assert!(
            ok(
                false,
                Some("a reason without a justification"),
                None,
                None,
                None
            )
            .is_err()
        );
        assert!(
            ok(false, None, Some(day(2027, 6, 6)), None, None)
                .unwrap_err()
                .contains("before the report")
        );
        assert!(
            ok(
                false,
                None,
                None,
                Some(day(2027, 6, 6)),
                Some("difference_remedied")
            )
            .is_err()
        );
        assert!(
            ok(false, None, None, Some(on), Some("difference_remedied")).is_ok(),
            "on the day of the report is fine"
        );
        assert!(
            ok(false, None, None, Some(on), Some("unfair"))
                .unwrap_err()
                .contains("outcome is one of")
        );
        assert!(
            ok(false, None, None, Some(on), None)
                .unwrap_err()
                .contains("outcome")
        );
        assert!(
            ok(false, None, None, None, Some("difference_remedied"))
                .unwrap_err()
                .contains("date")
        );
        for o in OUTCOMES {
            assert!(ok(false, None, None, Some(on), Some(o)).is_ok(), "{o}");
        }
    }
}
