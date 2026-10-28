//! Public (government) holidays generated per country or region (WPM-R5, WPM-D116). DB-free and clock-free.
//!
//! A region's holidays are **rules**, not a list typed in year by year: a fixed date, the nth or last weekday of a month, or a day
//! counted from Easter, each with a policy for when it falls on a weekend. [`generate`] turns a region's rules into the dates for a
//! year. **The shipped rule packs are worked examples written from the author's knowledge, not an official source, and are marked
//! unverified**: an employer confirms a region against its government's published list before relying on it, and a day the pack does
//! not know is added by hand. Regional and local days (a Scottish or a state holiday, a one-off royal event) are not in any pack.

use std::collections::BTreeSet;

use chrono::{Datelike, Days, NaiveDate, Weekday};

/// How a holiday's date is found in a year.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum When {
    /// The same month and day every year.
    Fixed {
        /// Month, 1 to 12.
        month: u32,
        /// Day of the month.
        day: u32,
    },
    /// The `n`th (1 to 4) given weekday of a month.
    Nth {
        /// Month, 1 to 12.
        month: u32,
        /// The weekday.
        weekday: Weekday,
        /// Which one, 1 to 4.
        n: u32,
    },
    /// The last given weekday of a month.
    Last {
        /// Month, 1 to 12.
        month: u32,
        /// The weekday.
        weekday: Weekday,
    },
    /// A number of days after (or, if negative, before) Easter Sunday.
    Easter {
        /// Days from Easter Sunday: Good Friday is -2, Easter Monday 1.
        offset: i64,
    },
}

/// What happens when the date falls on a Saturday or Sunday.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Weekend {
    /// Nothing: the day is the day, weekend or not.
    Stays,
    /// The next weekday that is not already a holiday (a substitute day).
    NextWeekday,
    /// Saturday is observed on the Friday before and Sunday on the Monday after.
    Nearest,
}

/// One holiday rule.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct Rule {
    /// The holiday's name.
    pub name: &'static str,
    /// How its date is found.
    pub when: When,
    /// What a weekend date does.
    pub weekend: Weekend,
}

/// One generated holiday.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Holiday {
    /// The holiday's name.
    pub name: &'static str,
    /// The date the rule gives.
    pub natural_on: NaiveDate,
    /// The day it is taken: the same, or the substitute when the natural date is a weekend. May be in the adjoining year.
    pub observed_on: NaiveDate,
}

/// Easter Sunday in the Gregorian calendar (the anonymous Gregorian algorithm). `None` outside the years chrono can hold.
#[must_use]
#[allow(clippy::many_single_char_names)] // the published algorithm's own names
pub fn easter(year: i32) -> Option<NaiveDate> {
    let a = year % 19;
    let b = year / 100;
    let c = year % 100;
    let d = b / 4;
    let e = b % 4;
    let f = (b + 8) / 25;
    let g = (b - f + 1) / 3;
    let h = (19 * a + b - d - g + 15) % 30;
    let i = c / 4;
    let k = c % 4;
    let l = (32 + 2 * e + 2 * i - h - k) % 7;
    let m = (a + 11 * h + 22 * l) / 451;
    let month = (h + l - 7 * m + 114) / 31;
    let day = (h + l - 7 * m + 114) % 31 + 1;
    NaiveDate::from_ymd_opt(year, u32::try_from(month).ok()?, u32::try_from(day).ok()?)
}

fn nth_weekday(year: i32, month: u32, weekday: Weekday, n: u32) -> Option<NaiveDate> {
    NaiveDate::from_weekday_of_month_opt(year, month, weekday, u8::try_from(n).ok()?)
}

fn last_weekday(year: i32, month: u32, weekday: Weekday) -> Option<NaiveDate> {
    let first_next = if month == 12 {
        NaiveDate::from_ymd_opt(year + 1, 1, 1)?
    } else {
        NaiveDate::from_ymd_opt(year, month + 1, 1)?
    };
    let mut d = first_next.checked_sub_days(Days::new(1))?;
    while d.weekday() != weekday {
        d = d.checked_sub_days(Days::new(1))?;
    }
    Some(d)
}

/// The date a rule gives in `year`, before any weekend policy. `None` for a date that does not exist (29 February in a common year).
#[must_use]
pub fn natural_date(when: When, year: i32) -> Option<NaiveDate> {
    match when {
        When::Fixed { month, day } => NaiveDate::from_ymd_opt(year, month, day),
        When::Nth { month, weekday, n } => nth_weekday(year, month, weekday, n),
        When::Last { month, weekday } => last_weekday(year, month, weekday),
        When::Easter { offset } => easter(year)?.checked_add_signed(chrono::Duration::days(offset)),
    }
}

fn is_weekend(d: NaiveDate) -> bool {
    matches!(d.weekday(), Weekday::Sat | Weekday::Sun)
}

fn next_free_weekday(from: NaiveDate, taken: &BTreeSet<NaiveDate>) -> Option<NaiveDate> {
    let mut d = from;
    loop {
        d = d.checked_add_days(Days::new(1))?;
        if !is_weekend(d) && !taken.contains(&d) {
            return Some(d);
        }
    }
}

/// The holidays a set of rules gives for `year`, by observed date. A rule whose date does not exist that year gives nothing. Days
/// that already fall on a weekday are placed first, so a substitute never lands on another holiday.
#[must_use]
pub fn generate(rules: &[Rule], year: i32) -> Vec<Holiday> {
    let dated: Vec<(&Rule, NaiveDate)> = rules
        .iter()
        .filter_map(|r| natural_date(r.when, year).map(|d| (r, d)))
        .collect();
    let mut taken: BTreeSet<NaiveDate> = dated
        .iter()
        .filter(|(r, d)| !is_weekend(*d) || r.weekend == Weekend::Stays)
        .map(|(_, d)| *d)
        .collect();
    let mut out = Vec::new();
    for (rule, natural) in dated {
        let observed = if !is_weekend(natural) || rule.weekend == Weekend::Stays {
            Some(natural)
        } else if rule.weekend == Weekend::Nearest {
            let shift = if natural.weekday() == Weekday::Sat {
                -1
            } else {
                1
            };
            natural.checked_add_signed(chrono::Duration::days(shift))
        } else {
            next_free_weekday(natural, &taken)
        };
        if let Some(observed_on) = observed {
            taken.insert(observed_on);
            out.push(Holiday {
                name: rule.name,
                natural_on: natural,
                observed_on,
            });
        }
    }
    out.sort_by_key(|h| (h.observed_on, h.name));
    out
}

/// Whether `date` is taken as a holiday in the list.
#[must_use]
pub fn is_holiday(holidays: &[Holiday], date: NaiveDate) -> bool {
    holidays.iter().any(|h| h.observed_on == date)
}

/// A business day: Monday to Friday and not a holiday in the list.
#[must_use]
pub fn is_business_day(holidays: &[Holiday], date: NaiveDate) -> bool {
    !is_weekend(date) && !is_holiday(holidays, date)
}

/// A region's rule pack.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct Pack {
    /// The region code (ISO 3166-1 alpha-2, or ISO 3166-2 for a part of a country).
    pub region: &'static str,
    /// The region's name.
    pub name: &'static str,
    /// The rules.
    pub rules: &'static [Rule],
    /// Whether anyone has checked the pack against the government's published list. **Every shipped pack is `false`.**
    pub verified: bool,
}

const fn fixed(name: &'static str, month: u32, day: u32, weekend: Weekend) -> Rule {
    Rule {
        name,
        when: When::Fixed { month, day },
        weekend,
    }
}

const fn nth(name: &'static str, month: u32, weekday: Weekday, n: u32) -> Rule {
    Rule {
        name,
        when: When::Nth { month, weekday, n },
        weekend: Weekend::Stays,
    }
}

const fn last(name: &'static str, month: u32, weekday: Weekday) -> Rule {
    Rule {
        name,
        when: When::Last { month, weekday },
        weekend: Weekend::Stays,
    }
}

const fn easter_day(name: &'static str, offset: i64) -> Rule {
    Rule {
        name,
        when: When::Easter { offset },
        weekend: Weekend::Stays,
    }
}

/// The shipped example packs. Unverified; see the module note.
pub const PACKS: &[Pack] = &[
    Pack {
        region: "GB-ENG",
        name: "England and Wales",
        verified: false,
        rules: &[
            fixed("New Year's Day", 1, 1, Weekend::NextWeekday),
            easter_day("Good Friday", -2),
            easter_day("Easter Monday", 1),
            nth("Early May bank holiday", 5, Weekday::Mon, 1),
            last("Spring bank holiday", 5, Weekday::Mon),
            last("Summer bank holiday", 8, Weekday::Mon),
            fixed("Christmas Day", 12, 25, Weekend::NextWeekday),
            fixed("Boxing Day", 12, 26, Weekend::NextWeekday),
        ],
    },
    Pack {
        region: "US",
        name: "United States (federal)",
        verified: false,
        rules: &[
            fixed("New Year's Day", 1, 1, Weekend::Nearest),
            nth("Birthday of Martin Luther King, Jr.", 1, Weekday::Mon, 3),
            nth("Washington's Birthday", 2, Weekday::Mon, 3),
            last("Memorial Day", 5, Weekday::Mon),
            fixed("Juneteenth", 6, 19, Weekend::Nearest),
            fixed("Independence Day", 7, 4, Weekend::Nearest),
            nth("Labor Day", 9, Weekday::Mon, 1),
            nth("Columbus Day", 10, Weekday::Mon, 2),
            fixed("Veterans Day", 11, 11, Weekend::Nearest),
            nth("Thanksgiving Day", 11, Weekday::Thu, 4),
            fixed("Christmas Day", 12, 25, Weekend::Nearest),
        ],
    },
    Pack {
        region: "DE",
        name: "Germany (nationwide)",
        verified: false,
        rules: &[
            fixed("Neujahr", 1, 1, Weekend::Stays),
            easter_day("Karfreitag", -2),
            easter_day("Ostermontag", 1),
            fixed("Tag der Arbeit", 5, 1, Weekend::Stays),
            easter_day("Christi Himmelfahrt", 39),
            easter_day("Pfingstmontag", 50),
            fixed("Tag der Deutschen Einheit", 10, 3, Weekend::Stays),
            fixed("Erster Weihnachtstag", 12, 25, Weekend::Stays),
            fixed("Zweiter Weihnachtstag", 12, 26, Weekend::Stays),
        ],
    },
];

/// The pack for a region code.
#[must_use]
pub fn pack(region: &str) -> Option<&'static Pack> {
    PACKS.iter().find(|p| p.region == region)
}

/// Problems with a manually added holiday: a name of 1 to 120 characters.
#[must_use]
pub fn manual_problems(name: &str) -> Vec<String> {
    if (1..=120).contains(&name.trim().chars().count()) {
        Vec::new()
    } else {
        vec!["name must be 1 to 120 characters".to_string()]
    }
}

/// Problems with a region code: 2 to 6 characters of upper-case letters, digits and one hyphen (`US`, `GB-ENG`).
#[must_use]
pub fn region_problems(region: &str) -> Vec<String> {
    let ok = (2..=6).contains(&region.len())
        && region
            .chars()
            .all(|c| c.is_ascii_uppercase() || c.is_ascii_digit() || c == '-')
        && region.matches('-').count() <= 1
        && !region.starts_with('-')
        && !region.ends_with('-');
    if ok {
        Vec::new()
    } else {
        vec!["region must be an upper-case code such as US or GB-ENG".to_string()]
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn d(y: i32, m: u32, day: u32) -> NaiveDate {
        NaiveDate::from_ymd_opt(y, m, day).unwrap()
    }

    fn dates(region: &str, year: i32) -> Vec<(String, NaiveDate, NaiveDate)> {
        generate(pack(region).unwrap().rules, year)
            .into_iter()
            .map(|h| (h.name.to_string(), h.natural_on, h.observed_on))
            .collect()
    }

    fn observed(region: &str, year: i32) -> Vec<NaiveDate> {
        generate(pack(region).unwrap().rules, year)
            .into_iter()
            .map(|h| h.observed_on)
            .collect()
    }

    #[test]
    fn easter_is_right_for_known_years() {
        for (y, m, day) in [
            (2000, 4, 23),
            (2024, 3, 31),
            (2025, 4, 20),
            (2026, 4, 5),
            (2027, 3, 28),
            (2038, 4, 25),
        ] {
            assert_eq!(easter(y), Some(d(y, m, day)), "{y}");
        }
    }

    #[test]
    fn england_and_wales_2026_has_eight_days_with_the_boxing_day_substitute() {
        assert_eq!(
            observed("GB-ENG", 2026),
            [
                d(2026, 1, 1),
                d(2026, 4, 3),
                d(2026, 4, 6),
                d(2026, 5, 4),
                d(2026, 5, 25),
                d(2026, 8, 31),
                d(2026, 12, 25),
                d(2026, 12, 28)
            ]
        );
    }

    #[test]
    fn christmas_on_a_weekend_gives_substitutes_that_never_collide() {
        // 2027: Christmas Saturday, Boxing Day Sunday -> Monday 27 and Tuesday 28.
        let xmas27 = observed("GB-ENG", 2027);
        assert!(
            xmas27.contains(&d(2027, 12, 27)) && xmas27.contains(&d(2027, 12, 28)),
            "{xmas27:?}"
        );
        // 2022: Christmas Sunday -> Tuesday 27, because Boxing Day is Monday 26.
        let xmas22 = observed("GB-ENG", 2022);
        assert!(
            xmas22.contains(&d(2022, 12, 26)) && xmas22.contains(&d(2022, 12, 27)),
            "{xmas22:?}"
        );
        // 2021: Christmas Saturday and Boxing Day Sunday -> Monday 27 and Tuesday 28.
        let xmas21 = observed("GB-ENG", 2021);
        assert!(xmas21.contains(&d(2021, 12, 27)) && xmas21.contains(&d(2021, 12, 28)));
        for year in 2015..2040 {
            let o = observed("GB-ENG", year);
            let unique: BTreeSet<_> = o.iter().collect();
            assert_eq!(unique.len(), o.len(), "{year} has two holidays on one day");
            assert_eq!(o.len(), 8, "{year}");
            assert!(
                o.iter().all(|x| !is_weekend(*x)),
                "{year}: a substitute day is a weekday"
            );
        }
    }

    #[test]
    fn new_years_day_on_a_saturday_moves_to_monday_in_england() {
        let all = dates("GB-ENG", 2022);
        let ny = all.iter().find(|(n, _, _)| n == "New Year's Day").unwrap();
        assert_eq!((ny.1, ny.2), (d(2022, 1, 1), d(2022, 1, 3)));
    }

    #[test]
    fn the_us_pack_observes_a_saturday_on_friday_and_a_sunday_on_monday() {
        let all = dates("US", 2026);
        let day = |n: &str| {
            all.iter()
                .find(|(name, _, _)| name == n)
                .map(|(_, nat, obs)| (*nat, *obs))
                .unwrap()
        };
        assert_eq!(
            day("Independence Day"),
            (d(2026, 7, 4), d(2026, 7, 3)),
            "Saturday to Friday"
        );
        assert_eq!(day("Thanksgiving Day").1, d(2026, 11, 26));
        assert_eq!(day("Memorial Day").1, d(2026, 5, 25));
        assert_eq!(day("Labor Day").1, d(2026, 9, 7));
        assert_eq!(day("Columbus Day").1, d(2026, 10, 12));
        assert_eq!(day("Birthday of Martin Luther King, Jr.").1, d(2026, 1, 19));
        assert_eq!(all.len(), 11);
        let sunday = dates("US", 2027);
        let july = sunday
            .iter()
            .find(|(n, _, _)| n == "Independence Day")
            .unwrap();
        assert_eq!(
            (july.1, july.2),
            (d(2027, 7, 4), d(2027, 7, 5)),
            "Sunday to Monday"
        );
        let new_year_2022 = dates("US", 2022);
        let ny = new_year_2022
            .iter()
            .find(|(n, _, _)| n == "New Year's Day")
            .unwrap();
        assert_eq!(
            ny.2,
            d(2021, 12, 31),
            "the observed day can fall in the year before"
        );
    }

    #[test]
    fn germany_counts_from_easter_and_does_not_move_a_weekend_day() {
        assert_eq!(
            observed("DE", 2026),
            [
                d(2026, 1, 1),
                d(2026, 4, 3),
                d(2026, 4, 6),
                d(2026, 5, 1),
                d(2026, 5, 14),
                d(2026, 5, 25),
                d(2026, 10, 3),
                d(2026, 12, 25),
                d(2026, 12, 26)
            ]
        );
        assert!(
            is_weekend(d(2026, 10, 3)),
            "3 October 2026 is a Saturday and stays"
        );
    }

    #[test]
    fn a_rule_for_a_day_that_does_not_exist_gives_nothing() {
        let leap = [fixed("Leap day", 2, 29, Weekend::Stays)];
        assert_eq!(generate(&leap, 2024).len(), 1);
        assert_eq!(generate(&leap, 2025), Vec::<Holiday>::new());
        assert_eq!(
            natural_date(
                When::Nth {
                    month: 2,
                    weekday: Weekday::Mon,
                    n: 5
                },
                2026
            ),
            None,
            "no fifth Monday in February 2026"
        );
    }

    #[test]
    fn last_weekday_handles_december_and_a_month_that_ends_on_it() {
        assert_eq!(
            natural_date(
                When::Last {
                    month: 12,
                    weekday: Weekday::Fri
                },
                2027
            ),
            Some(d(2027, 12, 31))
        );
        assert_eq!(
            natural_date(
                When::Last {
                    month: 8,
                    weekday: Weekday::Mon
                },
                2026
            ),
            Some(d(2026, 8, 31))
        );
        assert_eq!(
            natural_date(
                When::Last {
                    month: 5,
                    weekday: Weekday::Mon
                },
                2026
            ),
            Some(d(2026, 5, 25))
        );
    }

    #[test]
    fn a_business_day_is_neither_a_weekend_nor_a_holiday() {
        let h = generate(pack("GB-ENG").unwrap().rules, 2026);
        assert!(!is_business_day(&h, d(2026, 4, 3)), "Good Friday");
        assert!(!is_business_day(&h, d(2026, 4, 4)), "Saturday");
        assert!(is_business_day(&h, d(2026, 4, 7)));
        assert!(is_holiday(&h, d(2026, 12, 28)));
        assert!(
            !is_holiday(&h, d(2026, 12, 26)),
            "the natural Boxing Day is a Saturday and is not the day taken"
        );
    }

    #[test]
    fn every_shipped_pack_says_it_is_unverified_and_has_a_clean_code() {
        for p in PACKS {
            assert!(!p.verified, "{} must not claim to be checked", p.region);
            assert_eq!(region_problems(p.region), Vec::<String>::new());
            assert_ne!(p.rules.len(), 0);
        }
        assert!(pack("FR").is_none());
    }

    #[test]
    fn region_codes_and_names_are_checked() {
        for good in ["US", "GB-ENG", "DE", "AU-NSW"] {
            assert_eq!(region_problems(good), Vec::<String>::new(), "{good}");
        }
        for bad in ["", "u", "us", "GB_ENG", "-US", "US-", "A-B-C", "TOOLONGX"] {
            assert_eq!(region_problems(bad).len(), 1, "{bad}");
        }
        assert_eq!(manual_problems("Founders' day"), Vec::<String>::new());
        assert_eq!(manual_problems(" ").len(), 1);
    }
}
