//! Public-sector **pay scales** (WPM-R51), DB-free: a national banded pay scale,
//! and the arithmetic of "where does this salary sit on the scale, and is a step
//! up due?".
//!
//! The figures are **reference data transcribed from the pay circular** named
//! in [`PayScale::source`], not computed: the government issues a circular each
//! year, so a new year is a new scale added here with its source. Salaries are annual, full-time (37.5 hours a week), in minor units
//! (pence). Nothing here is stored against a person — a lookup takes a salary and
//! returns a position, and forgets it.
//!
//! The structure: bands 2–4 have an entry and a top step, bands 5–9 have entry, intermediate and top, and each step carries the
//! **years until eligible for progression**. Band 1 is closed to new entrants and
//! band 2 is a single rate.

use serde::Serialize;

/// One pay point on a band.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize)]
pub struct Step {
    /// Annual full-time salary in pence.
    pub annual_minor: i64,
    /// Whole years on this step before eligible for the next; `None` on the top
    /// step (and on a single-rate band), where there is nowhere to go.
    pub years_to_next: Option<u8>,
}

/// One band: its steps in ascending order.
#[derive(Debug, Clone, PartialEq, Eq, Serialize)]
pub struct Band {
    /// `"1"`, `"2"`, … `"8a"`–`"8d"`, `"9"`.
    pub code: &'static str,
    /// Closed to new entrants (band 1).
    pub closed: bool,
    /// Entry first, top last. Never empty.
    pub steps: Vec<Step>,
}

/// A weekly/nightly allowance set by the same circular.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize)]
pub struct Allowance {
    /// Stable machine code.
    pub code: &'static str,
    /// Display name.
    pub name: &'static str,
    /// Amount in pence.
    pub amount_minor: i64,
}

/// A whole scale for one framework, nation and year.
#[derive(Debug, Clone, PartialEq, Eq, Serialize)]
pub struct PayScale {
    /// Stable id, e.g. `national-2026-27`.
    pub id: &'static str,
    /// Display name.
    pub name: &'static str,
    /// The framework, e.g. `banded-pay`.
    pub framework: &'static str,
    /// The jurisdiction whose circular this is (`national` for a national scale).
    pub nation: &'static str,
    /// ISO 4217 currency.
    pub currency: &'static str,
    /// ISO date the scale applies from.
    pub effective_from: &'static str,
    /// The uplift this year applied to every pay point, in tenths of a percent
    /// (33 = 3.3%).
    pub uplift_tenths_percent: u16,
    /// The pay circular the figures come from.
    pub source: &'static str,
    /// The full-time week the annual figures assume, in minutes (2250 = 37.5
    /// hours); whole minutes, never a float.
    pub minutes_per_week: u32,
    /// Bands, lowest first.
    pub bands: Vec<Band>,
    /// Allowances set by the same circular.
    pub allowances: Vec<Allowance>,
}

/// A band from its steps, each `(annual pay in pence, years to the next step)`.
fn band(code: &'static str, closed: bool, steps: &[(i64, Option<u8>)]) -> Band {
    Band {
        code,
        closed,
        steps: steps
            .iter()
            .map(|&(annual_minor, years_to_next)| Step {
                annual_minor,
                years_to_next,
            })
            .collect(),
    }
}

/// The bands of [`national_2026_27`], lowest first.
fn national_bands() -> Vec<Band> {
    vec![
        band("1", true, &[(2_630_000, None)]),
        band("2", false, &[(2_630_000, None)]),
        band("3", false, &[(2_630_000, Some(2)), (2_789_000, None)]),
        band("4", false, &[(2_881_900, Some(3)), (3_162_600, None)]),
        band(
            "5",
            false,
            &[
                (3_255_700, Some(2)),
                (3_511_400, Some(2)),
                (3_963_100, None),
            ],
        ),
        band(
            "6",
            false,
            &[
                (4_055_900, Some(2)),
                (4_280_500, Some(3)),
                (4_884_100, None),
            ],
        ),
        band(
            "7",
            false,
            &[
                (5_012_900, Some(2)),
                (5_271_200, Some(3)),
                (5_736_500, None),
            ],
        ),
        band(
            "8a",
            false,
            &[
                (5_837_900, Some(2)),
                (6_131_700, Some(3)),
                (6_572_300, None),
            ],
        ),
        band(
            "8b",
            false,
            &[
                (6_758_300, Some(2)),
                (7_195_200, Some(3)),
                (7_853_000, None),
            ],
        ),
        band(
            "8c",
            false,
            &[
                (8_069_800, Some(2)),
                (8_561_100, Some(3)),
                (9_298_400, None),
            ],
        ),
        band(
            "8d",
            false,
            &[
                (9_577_300, Some(2)),
                (10_164_300, Some(3)),
                (11_044_800, None),
            ],
        ),
        band(
            "9",
            false,
            &[
                (11_447_500, Some(2)),
                (12_137_700, Some(3)),
                (13_173_200, None),
            ],
        ),
    ]
}

/// The national banded pay scale, from 1 April 2026 (the government's pay
/// circular for 2026/27, Annex 1).
#[must_use]
pub fn national_2026_27() -> PayScale {
    PayScale {
        id: "national-2026-27",
        name: "Public-sector pay scale 2026/27",
        framework: "banded-pay",
        nation: "national",
        currency: "GBP",
        effective_from: "2026-04-01",
        uplift_tenths_percent: 33,
        source: "Government pay circular for 2026/27, Annex 1",
        minutes_per_week: 2250,
        bands: national_bands(),
        allowances: vec![
            Allowance {
                code: "sleeping_in",
                name: "Sleeping-in allowance",
                amount_minor: 4_482,
            },
            Allowance {
                code: "on_call_weekday_weekend",
                name: "On-call, weekday or weekend",
                amount_minor: 2_605,
            },
            Allowance {
                code: "on_call_public_holiday",
                name: "On-call, public holiday",
                amount_minor: 5_208,
            },
        ],
    }
}

/// Every scale the service knows, newest first within a framework.
#[must_use]
pub fn all() -> Vec<PayScale> {
    vec![national_2026_27()]
}

/// The scale with this id, if any.
#[must_use]
pub fn find(id: &str) -> Option<PayScale> {
    all().into_iter().find(|s| s.id == id)
}

impl PayScale {
    /// The band with this code (case-insensitive: `8A` is `8a`).
    #[must_use]
    pub fn band(&self, code: &str) -> Option<&Band> {
        self.bands
            .iter()
            .find(|b| b.code.eq_ignore_ascii_case(code))
    }
}

/// Where a salary sits on a band.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize)]
#[serde(tag = "kind", rename_all = "snake_case")]
pub enum Position {
    /// Under the entry step: a pay problem, not a position.
    BelowEntry {
        /// Pence short of the entry step.
        shortfall_minor: i64,
    },
    /// Exactly on a step (1-based).
    OnStep {
        /// The step, 1-based.
        step: usize,
    },
    /// Between two steps — not a valid scale salary for full-time pay, but common
    /// for protected or pro-rata figures, so it is reported, not rejected.
    BetweenSteps {
        /// The step just below, 1-based.
        below_step: usize,
    },
    /// Above the top step.
    AboveTop {
        /// Pence over the top step.
        excess_minor: i64,
    },
}

/// Locate `salary_minor` on `band`. Pure: callers normalise part-time pay to a
/// full-time equivalent first. `None` when the distance from a step does not
/// fit an `i64`.
#[must_use]
pub fn locate(band: &Band, salary_minor: i64) -> Option<Position> {
    let steps = &band.steps;
    let entry = steps.first().map_or(0, |s| s.annual_minor);
    let top = steps.last().map_or(0, |s| s.annual_minor);
    if salary_minor < entry {
        return Some(Position::BelowEntry {
            shortfall_minor: entry.checked_sub(salary_minor)?,
        });
    }
    if salary_minor > top {
        return Some(Position::AboveTop {
            excess_minor: salary_minor.checked_sub(top)?,
        });
    }
    if let Some(i) = steps.iter().position(|s| s.annual_minor == salary_minor) {
        return Some(Position::OnStep {
            step: i.checked_add(1)?,
        });
    }
    // Between: the last step strictly below the salary.
    let below = steps
        .iter()
        .rposition(|s| s.annual_minor < salary_minor)
        .map_or(Some(1), |i| i.checked_add(1))?;
    Some(Position::BetweenSteps { below_step: below })
}

/// Whether a person on a step can move up yet.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize)]
#[serde(tag = "kind", rename_all = "snake_case")]
pub enum Progression {
    /// On the top step (or a single-rate band): nowhere to go.
    AtTop,
    /// Eligible now; `next_annual_minor` is the pay on the next step.
    Due {
        /// Annual pay on the next step, in pence.
        next_annual_minor: i64,
    },
    /// Not yet; `months_remaining` until eligible.
    NotYet {
        /// Whole months until eligible.
        months_remaining: u32,
        /// Annual pay on the next step, in pence.
        next_annual_minor: i64,
    },
}

/// Progression for someone `months_on_step` months into step `step` (1-based).
/// `None` when `step` is not on the band. Eligibility is a fixed number of
/// whole years on the step (the circular's "years until eligible for pay
/// progression"); it says nothing about whether the employer's own conditions
/// for progression are met, which is not modelled.
#[must_use]
pub fn progression(band: &Band, step: usize, months_on_step: u32) -> Option<Progression> {
    let current = band.steps.get(step.checked_sub(1)?)?;
    let (Some(years), Some(next)) = (current.years_to_next, band.steps.get(step)) else {
        return Some(Progression::AtTop);
    };
    let needed = u32::from(years).checked_mul(12)?;
    Some(match needed.checked_sub(months_on_step) {
        None | Some(0) => Progression::Due {
            next_annual_minor: next.annual_minor,
        },
        Some(months_remaining) => Progression::NotYet {
            months_remaining,
            next_annual_minor: next.annual_minor,
        },
    })
}

#[cfg(test)]
mod tests {
    use super::*;

    fn national() -> PayScale {
        national_2026_27()
    }

    /// Every band is non-empty, strictly ascending, and only the top step has no
    /// progression period — the shape the circular's table has.
    #[test]
    fn the_scale_is_well_formed() {
        let scale = national();
        assert_eq!(scale.bands.len(), 12);
        for b in &scale.bands {
            assert!(!b.steps.is_empty(), "{}", b.code);
            for pair in b.steps.windows(2) {
                assert!(pair[0].annual_minor < pair[1].annual_minor, "{}", b.code);
                assert!(pair[0].years_to_next.is_some(), "{}", b.code);
            }
            assert_eq!(b.steps.last().unwrap().years_to_next, None, "{}", b.code);
        }
        // Band 1 is closed; every other band is open.
        assert_eq!(
            scale.bands.iter().filter(|b| b.closed).count(),
            1,
            "only band 1 is closed"
        );
        assert!(scale.band("1").unwrap().closed);
        // Bands bottom out where the circular says (spot checks, in pence).
        assert_eq!(scale.band("5").unwrap().steps[0].annual_minor, 3_255_700);
        assert_eq!(scale.band("9").unwrap().steps[2].annual_minor, 13_173_200);
        assert_eq!(
            scale.band("2").unwrap().steps.len(),
            1,
            "band 2 is one rate"
        );
        assert_eq!(scale.band("7").unwrap().steps.len(), 3);
        // Bands rise: each band's top is below the next band's top.
        for pair in scale.bands.windows(2) {
            assert!(
                pair[0].steps.last().unwrap().annual_minor
                    <= pair[1].steps.last().unwrap().annual_minor,
                "{} vs {}",
                pair[0].code,
                pair[1].code
            );
        }
    }

    #[test]
    fn lookup_is_case_insensitive_and_unknowns_are_none() {
        let scale = national();
        assert!(scale.band("8A").is_some());
        assert!(scale.band("10").is_none());
        assert!(find("national-2026-27").is_some());
        assert!(find("national-2025-26").is_none());
    }

    /// Below, on each step, between, above — and the boundaries themselves.
    #[test]
    fn locate_covers_every_case() {
        let scale = national();
        let b5 = scale.band("5").unwrap();
        assert_eq!(
            locate(b5, 3_255_600).unwrap(),
            Position::BelowEntry {
                shortfall_minor: 100
            }
        );
        assert_eq!(locate(b5, 3_255_700).unwrap(), Position::OnStep { step: 1 });
        assert_eq!(
            locate(b5, 3_400_000).unwrap(),
            Position::BetweenSteps { below_step: 1 }
        );
        assert_eq!(locate(b5, 3_511_400).unwrap(), Position::OnStep { step: 2 });
        assert_eq!(
            locate(b5, 3_600_000).unwrap(),
            Position::BetweenSteps { below_step: 2 }
        );
        assert_eq!(locate(b5, 3_963_100).unwrap(), Position::OnStep { step: 3 });
        assert_eq!(
            locate(b5, 3_963_101).unwrap(),
            Position::AboveTop { excess_minor: 1 }
        );
        // A single-rate band: only on, below or above.
        let b2 = scale.band("2").unwrap();
        assert_eq!(locate(b2, 2_630_000).unwrap(), Position::OnStep { step: 1 });
        assert_eq!(
            locate(b2, 2_700_000).unwrap(),
            Position::AboveTop {
                excess_minor: 70_000
            }
        );
    }

    /// A distance that does not fit is unknown, not wrapped.
    #[test]
    fn locate_at_the_edges_of_the_type() {
        let scale = national();
        let b5 = scale.band("5").unwrap();
        assert_eq!(locate(b5, i64::MIN), None);
        let odd = band("x", false, &[(-1, None)]);
        assert_eq!(locate(&odd, i64::MAX), None);
        assert_eq!(
            locate(&odd, i64::MAX - 1),
            Some(Position::AboveTop {
                excess_minor: i64::MAX
            })
        );
        let empty = band("y", false, &[]);
        assert_eq!(
            locate(&empty, 1),
            Some(Position::AboveTop { excess_minor: 1 })
        );
        assert!(progression(b5, 1, u32::MAX).is_some());
    }

    /// Eligibility is whole years on the step, exact at the boundary; the top
    /// step and a single rate have nothing to progress to; a step not on the
    /// band is `None`, not a guess.
    #[test]
    fn progression_rules() {
        let scale = national();
        let b6 = scale.band("6").unwrap();
        // Entry step: 2 years.
        assert_eq!(
            progression(b6, 1, 23),
            Some(Progression::NotYet {
                months_remaining: 1,
                next_annual_minor: 4_280_500
            })
        );
        assert_eq!(
            progression(b6, 1, 24),
            Some(Progression::Due {
                next_annual_minor: 4_280_500
            })
        );
        // Intermediate step: 3 years in band 6 (2 in band 5).
        assert_eq!(
            progression(b6, 2, 35),
            Some(Progression::NotYet {
                months_remaining: 1,
                next_annual_minor: 4_884_100
            })
        );
        assert_eq!(
            progression(scale.band("5").unwrap(), 2, 24),
            Some(Progression::Due {
                next_annual_minor: 3_963_100
            })
        );
        assert_eq!(progression(b6, 3, 999), Some(Progression::AtTop));
        assert_eq!(
            progression(scale.band("2").unwrap(), 1, 999),
            Some(Progression::AtTop)
        );
        assert_eq!(progression(b6, 0, 0), None, "steps are 1-based");
        assert_eq!(progression(b6, 4, 0), None, "no fourth step");
    }

    /// The allowances are the circular's.
    #[test]
    fn allowances_are_the_circulars() {
        let scale = national();
        let amount = |code: &str| {
            scale
                .allowances
                .iter()
                .find(|a| a.code == code)
                .map(|a| a.amount_minor)
        };
        assert_eq!(amount("sleeping_in"), Some(4_482));
        assert_eq!(amount("on_call_weekday_weekend"), Some(2_605));
        assert_eq!(amount("on_call_public_holiday"), Some(5_208));
    }
}
