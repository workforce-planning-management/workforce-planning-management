//! **Second-profile tests** (WPM-R214, WPM-D143, WPM-T369): every rule that already takes a jurisdiction's limits as
//! parameters is run here with a second set of values of a *different shape* from the worked example it ships with.
//!
//! The values are synthetic: they are not any country's law, and no country is named (the source-text check in
//! `jurisdiction_neutrality.rs` covers this file too). Their only job is to prove that the rule accepts another shape,
//! so that a deployer elsewhere can configure it without a code change.
//!
//! The working-time limits (WPM-T380) and the holiday rules' weekend days (WPM-T379) are parameters since 2026-10-31 and are
//! tested with a second shape in their own modules and below; the payroll stub's is WPM-T383.

use chrono::{NaiveDate, Weekday};

use super::{
    consultation, cost, expenses, flexible_working, holiday, joint_assessment, pay_gap,
    pay_information, privacy, resignation,
};

fn day(y: i32, m: u32, d: u32) -> NaiveDate {
    NaiveDate::from_ymd_opt(y, m, d).unwrap()
}

/// A consultation profile with only a percentage test, a size floor, a 60-day window, two consultation tiers that
/// start at one dismissal, and **no** notice to an authority: none of which the shipped examples have together.
fn second_consultation_profile() -> consultation::Profile {
    consultation::Profile {
        id: "second_shape",
        source: "synthetic test profile",
        tests: vec![consultation::Test {
            size_from: 50,
            size_to: None,
            threshold: consultation::Threshold::Percent(5),
            window_days: 60,
        }],
        consultation: vec![
            consultation::Period {
                from_dismissals: 1,
                days: 15,
            },
            consultation::Period {
                from_dismissals: 50,
                days: 75,
            },
        ],
        authority_notice: vec![],
        checklist: vec!["consult_the_works_council"],
    }
}

fn proposal(size: u32, dismissals: u32) -> consultation::Proposal {
    consultation::Proposal {
        establishment_size: size,
        dismissals,
        earliest_termination: day(2027, 6, 30),
        others: vec![],
        agreed_period_days: None,
    }
}

#[test]
fn consultation_takes_a_percentage_only_profile_with_no_authority_notice() {
    let p = second_consultation_profile();
    let today = day(2027, 1, 1);
    let under = consultation::assess(&p, &proposal(200, 9), today);
    assert!(!under.may_apply);
    assert_eq!(under.evidence.needed, Some(10), "5% of 200, rounded up");
    let met = consultation::assess(&p, &proposal(200, 10), today);
    assert!(met.may_apply);
    assert_eq!(met.profile, "second_shape");
    assert_eq!(met.consultation_days, Some(15));
    assert_eq!(met.latest_consultation_start, Some(day(2027, 6, 15)));
    assert_eq!(
        met.latest_authority_notice, None,
        "this profile has no authority notice"
    );
    assert_eq!(met.checklist, ["consult_the_works_council"]);
    let large = consultation::assess(&p, &proposal(1_000, 50), today);
    assert_eq!(large.consultation_days, Some(75));
    let small = consultation::assess(&p, &proposal(40, 40), today);
    assert!(!small.may_apply);
    assert_eq!(
        small.evidence.needed, None,
        "no test applies below the size floor"
    );
}

#[test]
fn consultation_counts_together_in_the_profiles_own_window() {
    let p = second_consultation_profile();
    let mut within = proposal(200, 6);
    within.others = vec![(day(2027, 5, 2), 4)]; // 59 days before: inside a 60-day window
    assert!(consultation::assess(&p, &within, day(2027, 1, 1)).may_apply);
    let mut apart = proposal(200, 6);
    apart.others = vec![(day(2027, 4, 30), 4)]; // 61 days before
    assert!(!consultation::assess(&p, &apart, day(2027, 1, 1)).may_apply);
}

/// Holiday rules with no Easter and no substitute day: each day stays where it falls, even on a weekend.
#[test]
fn holiday_rules_of_another_shape_keep_a_weekend_day_where_it_falls() {
    let stays = [
        holiday::Rule {
            name: "New year",
            when: holiday::When::Fixed { month: 1, day: 1 },
            weekend: holiday::Weekend::Stays,
        },
        holiday::Rule {
            name: "Spring day",
            when: holiday::When::Nth {
                month: 3,
                weekday: Weekday::Thu,
                n: 2,
            },
            weekend: holiday::Weekend::Stays,
        },
    ];
    let h = holiday::generate(&stays, 2028, holiday::SATURDAY_SUNDAY);
    // 1 January 2028 is a Saturday; with this policy it is not moved.
    assert_eq!(
        h.iter().map(|x| x.observed_on).collect::<Vec<_>>(),
        [day(2028, 1, 1), day(2028, 3, 9)]
    );
    let nearest = [holiday::Rule {
        weekend: holiday::Weekend::Nearest,
        ..stays[0]
    }];
    assert_eq!(
        holiday::generate(&nearest, 2028, holiday::SATURDAY_SUNDAY)[0].observed_on,
        day(2027, 12, 31),
        "the nearest weekday may be in the year before"
    );
    // A Thursday and Friday weekend (a shape no shipped pack has): 1 January 2026 is a Thursday, so the nearest working day is the
    // Wednesday before, and the substitute policy gives the Saturday after.
    let thu_fri = [Weekday::Thu, Weekday::Fri];
    assert_eq!(
        holiday::generate(&nearest, 2026, &thu_fri)[0].observed_on,
        day(2025, 12, 31)
    );
    let substitute = [holiday::Rule {
        weekend: holiday::Weekend::NextWeekday,
        ..stays[0]
    }];
    assert_eq!(
        holiday::generate(&substitute, 2026, &thu_fri)[0].observed_on,
        day(2026, 1, 3)
    );
    assert!(
        holiday::is_business_day(&[], day(2026, 1, 4), &thu_fri),
        "a Sunday works there"
    );
}

/// A retention profile that keeps pay far longer than the cautious default and recruitment shorter than the floor.
#[test]
fn retention_takes_a_profile_of_other_horizons_and_keeps_its_floor() {
    use privacy::{HorizonSource, RecordKind, horizon_for};
    let pay = horizon_for(RecordKind::Pay, Some("3650"), Some("1000"));
    assert_eq!((pay.days, pay.source), (3650, HorizonSource::Override));
    let recruitment = horizon_for(RecordKind::Recruitment, Some("14"), None);
    assert_eq!(
        (recruitment.days, recruitment.source),
        (privacy::RETENTION_FLOOR_DAYS, HorizonSource::Override),
        "below the floor is raised to it"
    );
    let learning = horizon_for(RecordKind::Learning, None, Some("1000"));
    assert_eq!(
        (learning.days, learning.source),
        (1000, HorizonSource::Legacy)
    );
}

/// A joint-assessment configuration with a higher threshold and a shorter remedy period than the default.
#[test]
fn joint_assessment_takes_another_threshold_and_period() {
    use joint_assessment::{NotDue, Signal, config, signal};
    let second = config(1_000, 3).unwrap();
    let gap = pay_gap::Figure::Shown(pay_gap::Gap {
        female_minor: 1,
        male_minor: 2,
        gap_bp: 600,
    });
    let reported = Some(day(2027, 6, 7));
    let today = day(2027, 9, 8);
    assert_eq!(
        signal(&gap, None, reported, second, today),
        Signal::NotDue {
            why: NotDue::BelowThreshold
        },
        "6% is under this profile's 10%"
    );
    let wide = pay_gap::Figure::Shown(pay_gap::Gap {
        female_minor: 1,
        male_minor: 2,
        gap_bp: -1_200,
    });
    assert_eq!(
        signal(&wide, None, reported, second, today),
        Signal::Due {
            remedy_by: day(2027, 9, 7)
        },
        "three months, not six, and a gap either way counts"
    );
    assert!(config(0, 3).is_err() && config(1_000, 0).is_err());
}

#[test]
fn pay_information_takes_another_answer_period() {
    assert_eq!(
        pay_information::due_on(day(2027, 1, 31), 1),
        Some(day(2027, 2, 28))
    );
    assert_eq!(
        pay_information::due_on(day(2027, 1, 31), 3),
        Some(day(2027, 4, 30))
    );
}

/// Pay-gap reporting tiers of one band from 50 workers every two years, against the three shipped tiers.
#[test]
fn pay_gap_takes_another_set_of_reporting_tiers() {
    let tiers = [pay_gap::Obligation {
        from_workers: 50,
        every_years: 2,
        first_report_on: day(2029, 3, 31),
    }];
    assert_eq!(pay_gap::tier_for(49, &tiers), None);
    let tier = pay_gap::tier_for(80, &tiers).unwrap();
    assert_eq!(pay_gap::next_due(tier, day(2029, 4, 1)), day(2031, 3, 31));
}

/// Flexible working with a three-month decision, one request a year, a 30-day appeal and a refusal list of its own.
#[test]
fn flexible_working_takes_other_periods_limits_and_reasons() {
    use flexible_working::{Status, can_appeal, decide_by, validate_refusal_reason, within_limit};
    assert_eq!(decide_by(day(2027, 11, 30), 3), Some(day(2028, 2, 29)));
    let today = day(2027, 12, 1);
    let one = [(day(2027, 6, 1), Status::Refused)];
    assert!(!within_limit(&one, today, 1));
    assert!(within_limit(&one, today, 0), "0 is no limit");
    assert!(can_appeal(day(2027, 1, 1), day(2027, 1, 31), 30));
    assert!(!can_appeal(day(2027, 1, 1), day(2027, 2, 1), 30));
    let reasons = ["operational_continuity", "safety"];
    assert!(validate_refusal_reason("safety", &reasons).is_ok());
    assert!(validate_refusal_reason("burden_of_additional_costs", &reasons).is_err());
}

#[test]
fn resignation_takes_another_notice_period() {
    assert!(resignation::validate_notice(90).is_ok() && resignation::validate_notice(0).is_ok());
    assert_eq!(
        resignation::earliest_last_day(day(2027, 1, 1), 90),
        Some(day(2027, 4, 1))
    );
    assert_eq!(
        resignation::earliest_last_day(day(2027, 1, 1), 0),
        Some(day(2027, 1, 1))
    );
}

/// The API holds money as minor units and a code, and never divides by 100: a currency with no minor unit or with
/// three is as valid as one with two, and an amount is compared as it is.
#[test]
fn money_rules_do_not_assume_two_decimals() {
    for code in ["JPY", "KWD", "EUR"] {
        assert!(expenses::validate_currency(code).is_ok() && cost::valid_currency(code));
    }
    assert_eq!(
        cost::budget_in(Some(1_500), Some("JPY"), "JPY"),
        Some(1_500)
    );
    assert_eq!(cost::budget_in(Some(1_500), Some("KWD"), "JPY"), None);
}
