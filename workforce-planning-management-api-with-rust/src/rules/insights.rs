//! Pure rules for **workforce analytics and insights** — the narrative
//! layer over the shared metrics (WPM-T44): which figures deserve
//! attention, and what to look at next.
//!
//! DB-free and clock-free. The thresholds are published heuristics
//! ([`THRESHOLDS`]), not benchmarks: they say "look here", never "this is
//! wrong". A metric that is absent yields no insight — nothing is
//! imputed.

/// The heuristics, returned verbatim so a reader sees what triggered a
/// finding: `(name, value, meaning)`.
pub const THRESHOLDS: &[(&str, f64, &str)] = &[
    (
        "turnover_high",
        0.20,
        "Turnover rate at or above this is flagged for review.",
    ),
    (
        "headcount_change",
        0.10,
        "Closing headcount at least this fraction away from opening is flagged.",
    ),
    (
        "span_wide",
        12.0,
        "Mean direct reports per manager above this suggests overloaded managers.",
    ),
    (
        "span_narrow",
        3.0,
        "Mean direct reports per manager below this suggests excess layers.",
    ),
    (
        "time_to_fill_slow_days",
        60.0,
        "Median days to fill above this suggests a hiring bottleneck.",
    ),
];

/// How much attention a finding deserves.
#[derive(Debug, Clone, Copy, PartialEq, Eq, serde::Serialize)]
#[serde(rename_all = "snake_case")]
pub enum Severity {
    /// Context worth knowing.
    Info,
    /// Worth a look.
    Attention,
}

/// The metric values an insight is derived from; `None` is unknown.
#[derive(Debug, Clone, Copy, Default)]
pub struct Inputs {
    /// Headcount at the start of the period.
    pub opening: usize,
    /// Headcount at the end of the period.
    pub closing: usize,
    /// Turnover rate, when defined.
    pub turnover_rate: Option<f64>,
    /// Mean direct reports per manager, when any manager has reports.
    pub span_mean: Option<f64>,
    /// Median days-to-fill, when any requisition filled in the period.
    pub fill_median_days: Option<f64>,
}

/// One finding: what the numbers show, and a next step.
#[derive(Debug, Clone, PartialEq, serde::Serialize)]
pub struct Insight {
    /// Stable machine code.
    pub code: &'static str,
    /// How much attention it deserves.
    pub severity: Severity,
    /// What the numbers show (English).
    pub observation: String,
    /// The figures the observation is built from, by name (`pct`,
    /// `opening`, `closing`, `mean`, `days`), so a client can render the
    /// finding in its own language from `code` instead of this English
    /// `observation`/`suggestion`.
    pub params: serde_json::Value,
    /// What to look at next.
    pub suggestion: &'static str,
}

fn threshold(name: &str) -> f64 {
    THRESHOLDS
        .iter()
        .find(|(n, _, _)| *n == name)
        .map_or(f64::NAN, |(_, v, _)| *v)
}

/// Derive the findings for `inputs`, most pressing first. Empty when
/// nothing crosses a threshold (or nothing is known).
#[must_use]
pub fn derive(inputs: &Inputs) -> Vec<Insight> {
    let mut out = Vec::new();

    if let Some(rate) = inputs.turnover_rate
        && rate >= threshold("turnover_high")
    {
        out.push(Insight {
            code: "turnover_high",
            severity: Severity::Attention,
            observation: format!("Turnover was {:.0}% over the period.", rate * 100.0),
            params: serde_json::json!({ "pct": (rate * 100.0).round() }),
            suggestion: "Break leavers down by department and tenure, and compare with \
                         pulse-survey and exit feedback.",
        });
    }

    if inputs.opening > 0 {
        #[allow(clippy::cast_precision_loss)] // ratio over small counts
        let change = (inputs.closing as f64 - inputs.opening as f64) / inputs.opening as f64;
        if change.abs() >= threshold("headcount_change") {
            let (code, word, suggestion) = if change < 0.0 {
                (
                    "headcount_shrinking",
                    "fell",
                    "Check whether the reduction was planned; compare with open requisitions \
                     and the draft plan.",
                )
            } else {
                (
                    "headcount_growing",
                    "grew",
                    "Check the growth against the budgeted plan and onboarding capacity.",
                )
            };
            out.push(Insight {
                code,
                severity: if change < 0.0 {
                    Severity::Attention
                } else {
                    Severity::Info
                },
                observation: format!(
                    "Headcount {word} {:.0}% ({} to {}).",
                    change.abs() * 100.0,
                    inputs.opening,
                    inputs.closing
                ),
                params: serde_json::json!({
                    "pct": (change.abs() * 100.0).round(),
                    "opening": inputs.opening,
                    "closing": inputs.closing,
                }),
                suggestion,
            });
        }
    }

    if let Some(mean) = inputs.span_mean {
        if mean > threshold("span_wide") {
            out.push(Insight {
                code: "span_wide",
                severity: Severity::Attention,
                observation: format!("Managers average {mean:.1} direct reports."),
                params: serde_json::json!({ "mean": (mean * 10.0).round() / 10.0 }),
                suggestion: "Look for managers with the widest spans and consider adding a \
                             layer or a team lead.",
            });
        } else if mean < threshold("span_narrow") {
            out.push(Insight {
                code: "span_narrow",
                severity: Severity::Info,
                observation: format!("Managers average {mean:.1} direct reports."),
                params: serde_json::json!({ "mean": (mean * 10.0).round() / 10.0 }),
                suggestion: "Look for layers that could be merged.",
            });
        }
    }

    if let Some(median) = inputs.fill_median_days
        && median > threshold("time_to_fill_slow_days")
    {
        out.push(Insight {
            code: "time_to_fill_slow",
            severity: Severity::Attention,
            observation: format!("Median time to fill was {median:.0} days."),
            params: serde_json::json!({ "days": median.round() }),
            suggestion: "Find which stage of hiring the requisitions wait in.",
        });
    }

    out.sort_by_key(|i| i.severity != Severity::Attention);
    out
}

#[cfg(test)]
mod tests {
    use super::*;

    fn codes(i: &Inputs) -> Vec<&'static str> {
        derive(i).iter().map(|x| x.code).collect()
    }

    #[test]
    fn unknown_metrics_yield_nothing() {
        assert_eq!(codes(&Inputs::default()), [] as [&str; 0]);
    }

    #[test]
    fn calm_numbers_yield_nothing() {
        let i = Inputs {
            opening: 100,
            closing: 103,
            turnover_rate: Some(0.08),
            span_mean: Some(6.0),
            fill_median_days: Some(35.0),
        };
        assert_eq!(codes(&i), [] as [&str; 0]);
    }

    #[test]
    fn high_turnover_is_flagged_at_the_threshold() {
        let i = Inputs {
            opening: 100,
            closing: 100,
            turnover_rate: Some(0.20),
            ..Inputs::default()
        };
        assert_eq!(codes(&i), ["turnover_high"]);
    }

    #[test]
    fn shrinking_is_attention_and_growing_is_info() {
        let shrink = Inputs {
            opening: 100,
            closing: 85,
            ..Inputs::default()
        };
        let grow = Inputs {
            opening: 100,
            closing: 120,
            ..Inputs::default()
        };
        assert_eq!(derive(&shrink)[0].code, "headcount_shrinking");
        assert_eq!(derive(&shrink)[0].severity, Severity::Attention);
        assert_eq!(derive(&grow)[0].code, "headcount_growing");
        assert_eq!(derive(&grow)[0].severity, Severity::Info);
    }

    #[test]
    fn no_opening_headcount_means_no_growth_claim() {
        let i = Inputs {
            opening: 0,
            closing: 5,
            ..Inputs::default()
        };
        assert_eq!(codes(&i), [] as [&str; 0]);
    }

    #[test]
    fn span_and_fill_time_are_flagged() {
        let i = Inputs {
            opening: 10,
            closing: 10,
            turnover_rate: None,
            span_mean: Some(14.0),
            fill_median_days: Some(90.0),
        };
        assert_eq!(codes(&i), ["span_wide", "time_to_fill_slow"]);
        let narrow = Inputs {
            span_mean: Some(2.0),
            ..Inputs::default()
        };
        assert_eq!(codes(&narrow), ["span_narrow"]);
    }

    #[test]
    fn attention_sorts_before_info() {
        let i = Inputs {
            opening: 100,
            closing: 120,
            turnover_rate: Some(0.3),
            ..Inputs::default()
        };
        assert_eq!(codes(&i), ["turnover_high", "headcount_growing"]);
    }

    #[test]
    fn params_carry_the_figures_for_client_side_rendering() {
        let i = Inputs {
            opening: 100,
            closing: 85,
            turnover_rate: Some(0.254),
            span_mean: None,
            fill_median_days: Some(72.4),
        };
        let found = derive(&i);
        let by = |code: &str| found.iter().find(|x| x.code == code).unwrap().params.clone();
        assert_eq!(by("turnover_high")["pct"], 25.0);
        assert_eq!(by("headcount_shrinking")["pct"], 15.0);
        assert_eq!(by("headcount_shrinking")["opening"], 100);
        assert_eq!(by("headcount_shrinking")["closing"], 85);
        assert_eq!(by("time_to_fill_slow")["days"], 72.0);
    }
}
