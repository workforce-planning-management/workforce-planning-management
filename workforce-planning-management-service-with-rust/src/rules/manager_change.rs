//! Pure rules for **changing manager** (WPM-R188): the dated "getting to know
//! your team" item a new line manager sees for each new report.
//!
//! The item carries *what is open*, as counts and one date, and a single
//! yes/no: whether the person has an adjustments passport they may choose to
//! share. It carries **no adjustment words** and no free text of any kind
//! (WPM-D124, WPM-D126): the output type has no string field at all, so there
//! is nothing to hand over but the person's reference. The person shares the
//! words themselves.
//!
//! The caller supplies the facts (each owned by its own record) and the as-of
//! day; this module reads no database and no clock. The facts come from the
//! records WPM-T326–T331 and T334 will own; until those exist the controller
//! layer is not wired (see the task entry).

use chrono::{Duration, NaiveDate};
use serde::Serialize;
use uuid::Uuid;

/// How many days after the change the item stays on the list.
pub const WINDOW_DAYS: i64 = 60;

/// What is known about one report at the moment the manager changed.
/// Counts and dates only; never a reason, never a word of an adjustment.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ReportFacts {
    /// The person, by reference.
    pub person_ref: Uuid,
    /// The day the person became this manager's report.
    pub manager_since: NaiveDate,
    /// Absences with a plan still open (not why).
    pub open_absence_plans: u32,
    /// Review points reached and not yet concluded.
    pub open_review_points: u32,
    /// When the PADR is due, if set.
    pub padr_due: Option<NaiveDate>,
    /// Statutory or mandatory learning expiring within the horizon.
    pub learning_expiring: u32,
    /// The person has an adjustments passport. A yes/no only.
    pub has_passport: bool,
}

/// One "getting to know your team" item. Deliberately no text fields.
#[derive(Debug, Clone, PartialEq, Eq, Serialize)]
pub struct GetToKnowItem {
    /// The person, by reference.
    pub person_ref: Uuid,
    /// First day the item shows.
    pub from: NaiveDate,
    /// Last day the item shows.
    pub until: NaiveDate,
    /// Absences with a plan still open (not why).
    pub open_absence_plans: u32,
    /// Review points reached and not concluded.
    pub open_review_points: u32,
    /// When the PADR is due, if set.
    pub padr_due: Option<NaiveDate>,
    /// Learning expiring within the horizon.
    pub learning_expiring: u32,
    /// "They have a passport they may choose to share"; the words are not here.
    pub passport_may_be_shared: bool,
}

/// Is this a change of manager that should raise an item? True when there is
/// a new manager who differs from the previous one (including a first manager).
#[must_use]
pub fn is_manager_change(previous: Option<Uuid>, new: Option<Uuid>) -> bool {
    match new {
        Some(n) => previous != Some(n),
        None => false,
    }
}

/// The items to show on `as_of`: those whose window has started and not ended,
/// ordered by start day then person reference (stable, no ranking of people).
#[must_use]
pub fn items(facts: &[ReportFacts], as_of: NaiveDate) -> Vec<GetToKnowItem> {
    let mut out: Vec<GetToKnowItem> = facts
        .iter()
        .filter_map(|f| {
            let until = f.manager_since + Duration::days(WINDOW_DAYS);
            (as_of >= f.manager_since && as_of <= until).then_some(GetToKnowItem {
                person_ref: f.person_ref,
                from: f.manager_since,
                until,
                open_absence_plans: f.open_absence_plans,
                open_review_points: f.open_review_points,
                padr_due: f.padr_due,
                learning_expiring: f.learning_expiring,
                passport_may_be_shared: f.has_passport,
            })
        })
        .collect();
    out.sort_by_key(|i| (i.from, i.person_ref));
    out
}

#[cfg(test)]
mod tests {
    use super::*;

    fn d(n: i64) -> NaiveDate {
        NaiveDate::from_ymd_opt(2026, 1, 1).unwrap() + Duration::days(n)
    }

    fn facts(id: u128, since: i64) -> ReportFacts {
        ReportFacts {
            person_ref: Uuid::from_u128(id),
            manager_since: d(since),
            open_absence_plans: 1,
            open_review_points: 2,
            padr_due: Some(d(90)),
            learning_expiring: 3,
            has_passport: true,
        }
    }

    #[test]
    fn change_detection() {
        let a = Uuid::from_u128(1);
        let b = Uuid::from_u128(2);
        assert!(is_manager_change(None, Some(a)));
        assert!(is_manager_change(Some(a), Some(b)));
        assert!(!is_manager_change(Some(a), Some(a)));
        assert!(!is_manager_change(Some(a), None));
        assert!(!is_manager_change(None, None));
    }

    #[test]
    fn window_is_inclusive_at_both_ends() {
        let f = [facts(1, 10)];
        assert_eq!(items(&f, d(9)).len(), 0);
        assert_eq!(items(&f, d(10)).len(), 1);
        assert_eq!(items(&f, d(10 + WINDOW_DAYS)).len(), 1);
        assert_eq!(items(&f, d(10 + WINDOW_DAYS + 1)).len(), 0);
    }

    #[test]
    fn carries_counts_and_the_passport_flag() {
        let it = &items(&[facts(1, 0)], d(0))[0];
        assert_eq!(it.open_absence_plans, 1);
        assert_eq!(it.open_review_points, 2);
        assert_eq!(it.padr_due, Some(d(90)));
        assert_eq!(it.learning_expiring, 3);
        assert!(it.passport_may_be_shared);
        let mut none = facts(2, 0);
        none.has_passport = false;
        assert!(!items(&[none], d(0))[0].passport_may_be_shared);
    }

    #[test]
    fn ordered_by_start_then_ref_and_empty_is_empty() {
        assert_eq!(items(&[], d(0)).len(), 0);
        let f = [facts(3, 5), facts(2, 5), facts(9, 1)];
        let ids: Vec<u128> = items(&f, d(6))
            .iter()
            .map(|i| i.person_ref.as_u128())
            .collect();
        assert_eq!(ids, vec![9, 2, 3]);
    }

    /// WPM-R188 / WPM-D126: no adjustment words in the payload. Every JSON
    /// field is a known count, date, flag or reference; nothing else.
    #[test]
    fn payload_has_only_the_allowed_fields_and_no_free_text() {
        let json = serde_json::to_value(&items(&[facts(1, 0)], d(0))[0]).unwrap();
        let allowed = [
            "person_ref",
            "from",
            "until",
            "open_absence_plans",
            "open_review_points",
            "padr_due",
            "learning_expiring",
            "passport_may_be_shared",
        ];
        let obj = json.as_object().unwrap();
        for k in obj.keys() {
            assert!(allowed.contains(&k.as_str()), "unexpected field {k}");
        }
        assert_eq!(obj.len(), allowed.len());
    }

    /// The source of the output type must never grow a text field: that is
    /// where adjustment words would have to travel.
    #[test]
    fn output_type_has_no_text_field() {
        let src = include_str!("manager_change.rs");
        let start = src.find("pub struct GetToKnowItem").unwrap();
        let end = start + src[start..].find("\n}\n").unwrap();
        let body = &src[start..end];
        for bad in ["String", "&str", "Vec<", "Value", "Cow"] {
            assert!(!body.contains(bad), "GetToKnowItem must not contain {bad}");
        }
    }
}
