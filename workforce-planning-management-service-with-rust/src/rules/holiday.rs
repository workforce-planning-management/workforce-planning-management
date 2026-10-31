//! Public (government) holidays generated per country or region (WPM-R5, WPM-D116). DB-free and clock-free.
//!
//! A region's holidays are **rules**, not a list typed in year by year: a fixed date, the nth or last weekday of a month, a day
//! counted from Easter, or a fixed day of the tabular Islamic calendar, each with a policy for when it falls on a weekend. The weekend
//! is a **parameter** (D144's `holidays.weekend_days`, WPM-T379): any set of weekdays, so a Friday and Saturday, a Friday-only or a
//! Sunday-only weekend is expressed as readily as a Saturday and Sunday one, and an empty set means no day ever moves. [`generate`] turns a region's rules into the dates for a
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
    /// A fixed day of the **tabular** (arithmetic) Islamic calendar, civil epoch, the 30-year cycle with leap years 2, 5, 7, 10, 13,
    /// 16, 18, 21, 24, 26 and 29. It is an approximation: where a day is fixed by a sighting or an announced calendar (such as Umm
    /// al-Qura), the tabular date can differ by a day or two, and the deployer adds the announced day by hand. A lunar year is about
    /// eleven days shorter than a Gregorian one, so a Gregorian year can hold the day twice, or (rarely) not at all.
    Hijri {
        /// Month, 1 (Muharram) to 12 (Dhu al-Hijja).
        month: u32,
        /// Day of the month, 1 to 30 (a day the month lacks that year gives nothing).
        day: u32,
    },
}

/// What happens when the date falls on a weekend day (one of the weekend days passed to [`generate`]).
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Weekend {
    /// Nothing: the day is the day, weekend or not.
    Stays,
    /// The next working day that is not already a holiday (a substitute day).
    NextWeekday,
    /// The nearest working day, the earlier on a tie: with a Saturday and Sunday weekend, Saturday is observed on the Friday before
    /// and Sunday on the Monday after; with a Friday and Saturday one, Friday on the Thursday and Saturday on the Sunday.
    Nearest,
}

/// A Saturday and Sunday weekend, the shape of the three Western example packs (not a default: [`generate`] always takes one).
pub const SATURDAY_SUNDAY: &[Weekday] = &[Weekday::Sat, Weekday::Sun];

/// The weekend from ISO weekday numbers (1 Monday to 7 Sunday), as `holidays.weekend_days` stores them. `None` for a number out of
/// range.
#[must_use]
pub fn weekend_from_iso(days: &[u8]) -> Option<Vec<Weekday>> {
    days.iter()
        .map(|n| match n {
            1 => Some(Weekday::Mon),
            2 => Some(Weekday::Tue),
            3 => Some(Weekday::Wed),
            4 => Some(Weekday::Thu),
            5 => Some(Weekday::Fri),
            6 => Some(Weekday::Sat),
            7 => Some(Weekday::Sun),
            _ => None,
        })
        .collect()
}

/// The ISO weekday numbers of a weekend, sorted.
#[must_use]
pub fn weekend_to_iso(weekend: &[Weekday]) -> Vec<u32> {
    let mut days: Vec<u32> = weekend.iter().map(Weekday::number_from_monday).collect();
    days.sort_unstable();
    days.dedup();
    days
}

/// The Julian day number of a tabular Islamic date (civil epoch: 1 Muharram 1 is 16 July 622 Julian, day 1,948,440).
const fn hijri_jdn(year: i64, month: i64, day: i64) -> i64 {
    day + (59 * (month - 1) + 1) / 2 + (year - 1) * 354 + (3 + 11 * year) / 30 + 1_948_439
}

/// Whether a tabular Islamic year is a leap year (Dhu al-Hijja has 30 days).
const fn hijri_leap(year: i64) -> bool {
    (14 + 11 * year).rem_euclid(30) < 11
}

/// The dates in Gregorian `year` that are `day` `month` in the tabular Islamic calendar (zero, one or two of them).
#[must_use]
pub fn hijri_dates(month: u32, day: u32, year: i32) -> Vec<NaiveDate> {
    if !(1..=12).contains(&month) || day == 0 {
        return Vec::new();
    }
    let (month, day) = (i64::from(month), i64::from(day));
    let first = (i64::from(year) - 622) * 33 / 32;
    (first - 1..=first + 2)
        .filter(|hijri_year| *hijri_year >= 1)
        .filter(|hijri_year| {
            let length = if month % 2 == 1 || (month == 12 && hijri_leap(*hijri_year)) {
                30
            } else {
                29
            };
            day <= length
        })
        .filter_map(|hijri_year| {
            let days_from_ce = i32::try_from(hijri_jdn(hijri_year, month, day) - 1_721_425).ok()?;
            NaiveDate::from_num_days_from_ce_opt(days_from_ce)
        })
        .filter(|date| date.year() == year)
        .collect()
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

/// The date a rule gives in `year`, before any weekend policy. `None` for a date that does not exist (29 February in a common year);
/// for a [`When::Hijri`] rule, the first of the year's dates (see [`natural_dates`]).
#[must_use]
pub fn natural_date(when: When, year: i32) -> Option<NaiveDate> {
    natural_dates(when, year).into_iter().next()
}

/// Every date a rule gives in `year`, before any weekend policy: one for a Gregorian rule, zero to two for a [`When::Hijri`] one.
#[must_use]
pub fn natural_dates(when: When, year: i32) -> Vec<NaiveDate> {
    match when {
        When::Fixed { month, day } => NaiveDate::from_ymd_opt(year, month, day)
            .into_iter()
            .collect(),
        When::Nth { month, weekday, n } => {
            nth_weekday(year, month, weekday, n).into_iter().collect()
        }
        When::Last { month, weekday } => last_weekday(year, month, weekday).into_iter().collect(),
        When::Easter { offset } => easter(year)
            .and_then(|e| e.checked_add_signed(chrono::Duration::days(offset)))
            .into_iter()
            .collect(),
        When::Hijri { month, day } => hijri_dates(month, day, year),
    }
}

fn is_weekend(d: NaiveDate, weekend: &[Weekday]) -> bool {
    weekend.contains(&d.weekday())
}

fn next_free_weekday(
    from: NaiveDate,
    taken: &BTreeSet<NaiveDate>,
    weekend: &[Weekday],
) -> Option<NaiveDate> {
    // At most six weekend days, so a working day is found within a week; a full-week weekend has none.
    if weekend_to_iso(weekend).len() >= 7 {
        return None;
    }
    let mut d = from;
    loop {
        d = d.checked_add_days(Days::new(1))?;
        if !is_weekend(d, weekend) && !taken.contains(&d) {
            return Some(d);
        }
    }
}

fn nearest_working_day(from: NaiveDate, weekend: &[Weekday]) -> Option<NaiveDate> {
    (1..7).find_map(|step| {
        let before = from.checked_sub_days(Days::new(step))?;
        if !is_weekend(before, weekend) {
            return Some(Some(before));
        }
        let after = from.checked_add_days(Days::new(step))?;
        (!is_weekend(after, weekend)).then_some(Some(after))
    })?
}

/// The holidays a set of rules gives for `year`, by observed date, with `weekend` as the weekly rest days. A rule whose date does not
/// exist that year gives nothing. Days that already fall on a working day are placed first, so a substitute never lands on another
/// holiday.
#[must_use]
pub fn generate(rules: &[Rule], year: i32, weekend: &[Weekday]) -> Vec<Holiday> {
    let dated: Vec<(&Rule, NaiveDate)> = rules
        .iter()
        .flat_map(|r| natural_dates(r.when, year).into_iter().map(move |d| (r, d)))
        .collect();
    let mut taken: BTreeSet<NaiveDate> = dated
        .iter()
        .filter(|(r, d)| !is_weekend(*d, weekend) || r.weekend == Weekend::Stays)
        .map(|(_, d)| *d)
        .collect();
    let mut out = Vec::new();
    for (rule, natural) in dated {
        let observed = if !is_weekend(natural, weekend) || rule.weekend == Weekend::Stays {
            Some(natural)
        } else if rule.weekend == Weekend::Nearest {
            nearest_working_day(natural, weekend)
        } else {
            next_free_weekday(natural, &taken, weekend)
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

/// A business day: not one of the `weekend` days and not a holiday in the list.
#[must_use]
pub fn is_business_day(holidays: &[Holiday], date: NaiveDate, weekend: &[Weekday]) -> bool {
    !is_weekend(date, weekend) && !is_holiday(holidays, date)
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
    /// The weekend the example was written for, used only when the organization's profile has no `holidays.weekend_days` for the
    /// region (the answer then says so). Part of the unverified example, like the rules.
    pub weekend: &'static [Weekday],
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

const fn hijri(name: &'static str, month: u32, day: u32) -> Rule {
    Rule {
        name,
        when: When::Hijri { month, day },
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
        weekend: SATURDAY_SUNDAY,
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
        weekend: SATURDAY_SUNDAY,
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
        weekend: SATURDAY_SUNDAY,
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
    Pack {
        region: "SA",
        name: "Saudi Arabia (example of another shape)",
        verified: false,
        weekend: &[Weekday::Fri, Weekday::Sat],
        rules: &[
            fixed("Founding Day", 2, 22, Weekend::Stays),
            hijri("Eid al-Fitr", 10, 1),
            fixed("National Day", 9, 23, Weekend::Stays),
            hijri("Eid al-Adha", 12, 10),
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
        generate(
            pack(region).unwrap().rules,
            year,
            pack(region).unwrap().weekend,
        )
        .into_iter()
        .map(|h| (h.name.to_string(), h.natural_on, h.observed_on))
        .collect()
    }

    fn observed(region: &str, year: i32) -> Vec<NaiveDate> {
        generate(
            pack(region).unwrap().rules,
            year,
            pack(region).unwrap().weekend,
        )
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
                o.iter().all(|x| !is_weekend(*x, SATURDAY_SUNDAY)),
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
            is_weekend(d(2026, 10, 3), SATURDAY_SUNDAY),
            "3 October 2026 is a Saturday and stays"
        );
    }

    #[test]
    fn a_rule_for_a_day_that_does_not_exist_gives_nothing() {
        let leap = [fixed("Leap day", 2, 29, Weekend::Stays)];
        assert_eq!(generate(&leap, 2024, SATURDAY_SUNDAY).len(), 1);
        assert_eq!(
            generate(&leap, 2025, SATURDAY_SUNDAY),
            Vec::<Holiday>::new()
        );
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
        let h = generate(pack("GB-ENG").unwrap().rules, 2026, SATURDAY_SUNDAY);
        assert!(
            !is_business_day(&h, d(2026, 4, 3), SATURDAY_SUNDAY),
            "Good Friday"
        );
        assert!(
            !is_business_day(&h, d(2026, 4, 4), SATURDAY_SUNDAY),
            "Saturday"
        );
        assert!(is_business_day(&h, d(2026, 4, 7), SATURDAY_SUNDAY));
        // Under a Friday and Saturday weekend the Saturday is still off, the Sunday is a business day and the Friday is not.
        let fri_sat = [Weekday::Fri, Weekday::Sat];
        assert!(is_business_day(&[], d(2026, 4, 5), &fri_sat));
        assert!(!is_business_day(&[], d(2026, 4, 10), &fri_sat));
        assert!(
            is_business_day(&[], d(2026, 4, 4), &[]),
            "no weekend at all"
        );
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

    /// The tabular Islamic calendar: its epoch, known tabular dates, two in one Gregorian year, a day the month lacks, out of range.
    #[test]
    fn hijri_days_follow_the_tabular_calendar() {
        assert_eq!(
            hijri_dates(1, 1, 622),
            vec![d(622, 7, 19)],
            "1 Muharram 1 is 16 July 622 Julian"
        );
        assert_eq!(hijri_dates(10, 1, 2024), vec![d(2024, 4, 10)]);
        assert_eq!(
            hijri_dates(12, 10, 2024),
            vec![d(2024, 6, 17)],
            "tabular; an announced date can differ by a day"
        );
        assert_eq!(hijri_dates(10, 1, 2026), vec![d(2026, 3, 20)]);
        // A lunar year is shorter: 1 Muharram falls twice in 2008.
        assert_eq!(hijri_dates(1, 1, 2008).len(), 2);
        // The second month has 29 days; Dhu al-Hijja has 30 only in a leap year (1446 is not one, 1447 is).
        assert_eq!(hijri_dates(2, 30, 2026), Vec::<NaiveDate>::new());
        assert!(hijri_leap(1447) && !hijri_leap(1446));
        assert!(hijri_dates(13, 1, 2026).is_empty() && hijri_dates(1, 0, 2026).is_empty());
        // Every Gregorian year from 1970 to 2100 holds each day of a 30-day month at least once or twice, never three times.
        for year in 1970..=2100 {
            let n = hijri_dates(9, 1, year).len();
            assert!((1..=2).contains(&n), "{year}: {n}");
        }
    }

    /// The weekend is a parameter: the same rule moves differently under a Friday and Saturday weekend, a Friday-only one, and
    /// none at all, and `Nearest` takes the nearest working day, the earlier on a tie.
    #[test]
    fn the_weekend_is_a_parameter() {
        let fri_sat = [Weekday::Fri, Weekday::Sat];
        let next = [fixed("Day", 1, 2, Weekend::NextWeekday)]; // 2 January 2026 is a Friday
        assert_eq!(
            generate(&next, 2026, SATURDAY_SUNDAY)[0].observed_on,
            d(2026, 1, 2),
            "a weekday there"
        );
        assert_eq!(
            generate(&next, 2026, &fri_sat)[0].observed_on,
            d(2026, 1, 4),
            "the Sunday is the next working day"
        );
        assert_eq!(
            generate(&next, 2026, &[Weekday::Fri])[0].observed_on,
            d(2026, 1, 3),
            "a Friday-only weekend"
        );
        assert_eq!(
            generate(&next, 2026, &[])[0].observed_on,
            d(2026, 1, 2),
            "no weekend: nothing moves"
        );
        let nearest = [
            fixed("Day", 1, 2, Weekend::Nearest),
            fixed("Other", 1, 3, Weekend::Nearest),
        ];
        let got: Vec<NaiveDate> = generate(&nearest, 2026, &fri_sat)
            .iter()
            .map(|h| h.observed_on)
            .collect();
        assert_eq!(
            got,
            [d(2026, 1, 1), d(2026, 1, 4)],
            "Friday to Thursday, Saturday to Sunday"
        );
        // A substitute skips another holiday under any weekend.
        let pair = [
            fixed("A", 1, 2, Weekend::NextWeekday),
            fixed("B", 1, 4, Weekend::Stays),
        ];
        assert_eq!(
            generate(&pair, 2026, &fri_sat)
                .iter()
                .find(|h| h.name == "A")
                .unwrap()
                .observed_on,
            d(2026, 1, 5)
        );
        assert_eq!(
            weekend_from_iso(&[5, 6]),
            Some(vec![Weekday::Fri, Weekday::Sat])
        );
        assert_eq!(weekend_from_iso(&[0]), None);
        assert_eq!(
            weekend_to_iso(&[Weekday::Sun, Weekday::Sat, Weekday::Sun]),
            [6, 7]
        );
    }

    /// The pack of another shape: a Friday and Saturday weekend, two tabular Islamic days and two fixed Gregorian ones; a
    /// Gregorian year can hold an Eid twice, and the fixed days stay where they fall.
    #[test]
    fn the_pack_of_another_shape_mixes_calendars_and_its_own_weekend() {
        let sa = pack("SA").unwrap();
        assert_eq!(weekend_to_iso(sa.weekend), [5, 6]);
        let days = generate(sa.rules, 2026, sa.weekend);
        let names: Vec<&str> = days.iter().map(|h| h.name).collect();
        assert_eq!(
            names,
            ["Founding Day", "Eid al-Fitr", "Eid al-Adha", "National Day"]
        );
        assert_eq!(days[1].observed_on, d(2026, 3, 20));
        assert_eq!(days[2].observed_on, d(2026, 5, 27));
        // Each Eid falls once or twice in a Gregorian year, so a year holds four to six days.
        let counts: Vec<usize> = (2026..=2060)
            .map(|y| generate(sa.rules, y, sa.weekend).len())
            .collect();
        assert!(counts.iter().all(|n| (4..=6).contains(n)), "{counts:?}");
        assert!(
            counts.iter().any(|n| *n > 4),
            "some year holds an Eid twice"
        );
    }
}
