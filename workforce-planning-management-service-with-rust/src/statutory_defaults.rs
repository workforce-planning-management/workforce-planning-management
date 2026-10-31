//! **Whose each default is** (WPM-T384; WPM-R214, WPM-D143).
//!
//! The rules take their limits as parameters, and the controllers read each from a `WPM_*` setting
//! with a default when the setting is absent. Several defaults are one regime's value. This table
//! names, for every such setting, whose value its default is: a jurisdiction code in the shape of
//! WPM-D144 (`EU`, `GB`), what it rests on, and whether that source was read here. A default that is
//! no regime's says so (`regime: None`), rather than passing as one.
//!
//! It lives outside `src/rules/` because it names jurisdictions, which the pure core does not
//! (WPM-T369). It is a label, not the registry D144 proposes: moving these defaults into dated,
//! sourced profiles waits for that registry to be confirmed.

use serde::Serialize;

/// Where a default came from.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize)]
pub struct Origin {
    /// The jurisdiction code whose rule the default is (`EU`, `GB`), or `None` when it is no regime's.
    pub regime: Option<&'static str>,
    /// What the default rests on, in words.
    pub reference: &'static str,
    /// `read` (the source was checked, as `reference` says when), `unverified` (modelled on the
    /// source, not checked against its text), `example` (a worked example in a regime's shape, not
    /// that regime's rule) or `starting_point` (no regime's: a value to replace).
    pub status: &'static str,
}

/// One setting with a default.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize)]
pub struct Default {
    /// The environment variable.
    pub setting: &'static str,
    /// The area it belongs to (a stable lowercase token).
    pub area: &'static str,
    /// The default, as text (the unit is in the setting's name or the area's documentation).
    pub default: &'static str,
    /// Whose it is.
    pub origin: Origin,
}

const FLEXIBLE_GB: Origin = Origin {
    regime: Some("GB"),
    reference: "Great Britain: Employment Rights Act 1996, Part 8A, as amended by the Employment Relations \
                (Flexible Working) Act 2023 from 6 April 2024",
    status: "unverified",
};
/// A reference to one Article of the Pay Transparency Directive, read in WPM-T268.
macro_rules! directive {
    ($article:literal) => {
        Origin {
            regime: Some("EU"),
            reference: concat!(
                "European Union: Pay Transparency Directive (EU) 2023/970, ",
                $article,
                "; read against the Directive's text in WPM-T268, 2026-10-24"
            ),
            status: "read",
        }
    };
}
const STARTING_POINT: Origin = Origin {
    regime: None,
    reference: "a starting point, not taken from a regime; the deployer replaces it",
    status: "starting_point",
};
const RETENTION: Origin = Origin {
    regime: None,
    reference: "a cautious starting point, no regime's (spec/governance/retention-schedule.md)",
    status: "starting_point",
};
const AI_ACT: Origin = Origin {
    regime: Some("EU"),
    reference: "European Union: AI Act, Regulation (EU) 2024/1689, Article 113 as amended by the Digital Omnibus on AI \
                (the Annex III duties from 2 December 2027), as the Commission's AI Act Service Desk showed it in \
                WPM-T268, 2026-10-24; neither the amending act's text nor its number was read",
    status: "unverified",
};
const SEX_CATEGORY: Origin = Origin {
    regime: None,
    reference: "the name of an equality category the deployment monitors (WPM_EQUALITY_CATEGORIES), no regime's; the \
                report compares its female and male answers, which is the shape of the EU Directive's Article 9 \
                indicators, not a category any regime names",
    status: "starting_point",
};
const PAYROLL_STUB: Origin = Origin {
    regime: Some("GB"),
    reference: "an example in the shape of the UK's income tax for 2025-26 (the personal allowance of 12,570 pounds \
                a year divided by 12, the basic rate of 20%), written for GBP; a stub, not that tax",
    status: "example",
};

/// Every `WPM_*` setting whose default is, or might be read as, one regime's value. Sorted by
/// area, then setting.
pub const DEFAULTS: &[Default] = &[
    Default {
        setting: "WPM_AI_ACT_APPLIES_FROM",
        area: "ai_register",
        default: "2027-12-02",
        origin: AI_ACT,
    },
    Default {
        setting: "WPM_FLEXIBLE_APPEAL_CALENDAR_DAYS",
        area: "flexible_working",
        default: "14",
        origin: STARTING_POINT,
    },
    Default {
        setting: "WPM_FLEXIBLE_DECISION_MONTHS",
        area: "flexible_working",
        default: "2",
        origin: FLEXIBLE_GB,
    },
    Default {
        setting: "WPM_FLEXIBLE_REFUSAL_REASONS",
        area: "flexible_working",
        default: "the eight business reasons of that Part, and other_recorded",
        origin: FLEXIBLE_GB,
    },
    Default {
        setting: "WPM_FLEXIBLE_REQUESTS_PER_YEAR",
        area: "flexible_working",
        default: "2",
        origin: FLEXIBLE_GB,
    },
    Default {
        setting: "WPM_JOINT_ASSESSMENT_MONTHS",
        area: "pay_transparency",
        default: "6",
        origin: directive!("Article 10(1)"),
    },
    Default {
        setting: "WPM_JOINT_ASSESSMENT_THRESHOLD_PERCENT",
        area: "pay_transparency",
        default: "5",
        origin: directive!("Article 10(1)"),
    },
    Default {
        setting: "WPM_PAY_GAP_SEX_CATEGORY",
        area: "pay_transparency",
        default: "sex",
        origin: SEX_CATEGORY,
    },
    Default {
        setting: "WPM_PAY_GAP_TIERS",
        area: "pay_transparency",
        default: "250:1:2027-06-07,150:3:2027-06-07,100:3:2031-06-07",
        origin: directive!("Article 9"),
    },
    Default {
        setting: "WPM_PAY_INFORMATION_MONTHS",
        area: "pay_transparency",
        default: "2",
        origin: directive!("Article 7"),
    },
    Default {
        setting: "WPM_REQUIRE_PAY_RANGE",
        area: "pay_transparency",
        default: "on",
        origin: directive!("Article 5"),
    },
    Default {
        setting: "WPM_CONTRACTED_DAY_MINUTES",
        area: "payroll",
        default: "450",
        origin: STARTING_POINT,
    },
    Default {
        setting: "WPM_PAYROLL_MONTH_MINUTES",
        area: "payroll",
        default: "9750",
        origin: STARTING_POINT,
    },
    Default {
        setting: "WPM_PAYROLL_STUB_ALLOWANCE_MINOR",
        area: "payroll",
        default: "104750",
        origin: PAYROLL_STUB,
    },
    Default {
        setting: "WPM_PAYROLL_STUB_CURRENCY",
        area: "payroll",
        default: "GBP",
        origin: PAYROLL_STUB,
    },
    Default {
        setting: "WPM_PAYROLL_STUB_RATE_PERCENT",
        area: "payroll",
        default: "20",
        origin: PAYROLL_STUB,
    },
    Default {
        setting: "WPM_RESIGNATION_NOTICE_CALENDAR_DAYS",
        area: "resignation",
        default: "28",
        origin: STARTING_POINT,
    },
    Default {
        setting: "WPM_RETENTION_EMPLOYMENT_DAYS",
        area: "retention",
        default: "2190",
        origin: RETENTION,
    },
    Default {
        setting: "WPM_RETENTION_LEARNING_DAYS",
        area: "retention",
        default: "1095",
        origin: RETENTION,
    },
    Default {
        setting: "WPM_RETENTION_PAY_DAYS",
        area: "retention",
        default: "2190",
        origin: RETENTION,
    },
    Default {
        setting: "WPM_RETENTION_PERFORMANCE_DAYS",
        area: "retention",
        default: "730",
        origin: RETENTION,
    },
    Default {
        setting: "WPM_RETENTION_PLANNING_DAYS",
        area: "retention",
        default: "365",
        origin: RETENTION,
    },
    Default {
        setting: "WPM_RETENTION_RECRUITMENT_DAYS",
        area: "retention",
        default: "180",
        origin: RETENTION,
    },
    Default {
        setting: "WPM_RETENTION_TIME_AND_LEAVE_DAYS",
        area: "retention",
        default: "730",
        origin: RETENTION,
    },
    Default {
        setting: "WPM_RETENTION_WELLBEING_DAYS",
        area: "retention",
        default: "365",
        origin: RETENTION,
    },
];

/// The entry for one setting.
#[must_use]
pub fn find(setting: &str) -> Option<&'static Default> {
    DEFAULTS.iter().find(|d| d.setting == setting)
}
