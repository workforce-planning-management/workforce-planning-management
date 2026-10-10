//! **My time-off** (WPM-R129): pure arithmetic over a worker's leave entitlements and requests.
//!
//! Every figure is in **calendar days**, the unit the leave data is kept in. Nothing here is a
//! score: there is no absence rate, no trigger point and no ranking, because a number about a
//! person's sickness is a score about them and would need an equality impact assessment first
//! (WPM-R99). A balance for a kind with no entitlement recorded is **unknown**, never zero.

use chrono::{Datelike, NaiveDate};
use std::collections::BTreeMap;

/// A recorded entitlement for a kind of leave in a year.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Entitlement {
    /// The kind of leave.
    pub kind: String,
    /// The leave year.
    pub year: i32,
    /// Calendar days allowed.
    pub entitled: i32,
    /// Calendar days the service has already subtracted (approved requests).
    pub used: i32,
}

/// A leave request.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Request {
    /// The kind of leave.
    pub kind: String,
    /// First day.
    pub start_on: NaiveDate,
    /// Last day.
    pub end_on: NaiveDate,
    /// Calendar days it spans.
    pub days: i32,
    /// `requested`, `approved`, `rejected` or `cancelled`.
    pub status: String,
}

/// One kind of leave in one year.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Balance {
    /// The kind of leave.
    pub kind: String,
    /// The year (by the day a request starts, as the service counts it).
    pub year: i32,
    /// Calendar days allowed; `None` when no entitlement is recorded.
    pub entitled: Option<i32>,
    /// Calendar days subtracted by approvals.
    pub used: i32,
    /// Approved and already over (ended before today).
    pub taken: i32,
    /// Approved and still to come or under way.
    pub booked: i32,
    /// Asked for and awaiting a decision.
    pub requested: i32,
    /// `entitled - used`; `None` when there is no entitlement (unknown, not zero).
    pub remaining: Option<i32>,
    /// What would remain if everything awaiting a decision were approved.
    pub remaining_if_requested_approved: Option<i32>,
}

/// Balances per kind and year, from entitlements and requests, as of `today`.
#[must_use]
pub fn balances(
    entitlements: &[Entitlement],
    requests: &[Request],
    today: NaiveDate,
) -> Vec<Balance> {
    let mut rows: BTreeMap<(i32, String), Balance> = BTreeMap::new();
    let blank = |kind: &str, year: i32| Balance {
        kind: kind.to_string(),
        year,
        entitled: None,
        used: 0,
        taken: 0,
        booked: 0,
        requested: 0,
        remaining: None,
        remaining_if_requested_approved: None,
    };
    for e in entitlements {
        let row = rows
            .entry((e.year, e.kind.clone()))
            .or_insert_with(|| blank(&e.kind, e.year));
        row.entitled = Some(e.entitled);
        row.used = e.used;
    }
    for r in requests {
        let year = r.start_on.year();
        let row = rows
            .entry((year, r.kind.clone()))
            .or_insert_with(|| blank(&r.kind, year));
        match r.status.as_str() {
            "approved" if r.end_on < today => row.taken += r.days,
            "approved" => row.booked += r.days,
            "requested" => row.requested += r.days,
            _ => {}
        }
    }
    rows.into_values()
        .map(|mut b| {
            b.remaining = b.entitled.map(|e| e - b.used);
            b.remaining_if_requested_approved = b.remaining.map(|r| r - b.requested);
            b
        })
        .collect()
}

/// Absence in a year, by kind: how many absences and how many calendar days. Only approved
/// requests that are over count. Reasons are not part of it.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Absence {
    /// The kind of leave.
    pub kind: String,
    /// How many separate absences.
    pub absences: i32,
    /// Their total length in calendar days.
    pub days: i32,
}

/// Historical absence by year (the year a request starts in), oldest first.
#[must_use]
pub fn history(requests: &[Request], today: NaiveDate) -> Vec<(i32, Vec<Absence>)> {
    let mut by: BTreeMap<i32, BTreeMap<String, (i32, i32)>> = BTreeMap::new();
    for r in requests
        .iter()
        .filter(|r| r.status == "approved" && r.end_on < today)
    {
        let entry = by
            .entry(r.start_on.year())
            .or_default()
            .entry(r.kind.clone())
            .or_insert((0, 0));
        entry.0 += 1;
        entry.1 += r.days;
    }
    by.into_iter()
        .map(|(year, kinds)| {
            (
                year,
                kinds
                    .into_iter()
                    .map(|(kind, (absences, days))| Absence {
                        kind,
                        absences,
                        days,
                    })
                    .collect(),
            )
        })
        .collect()
}

/// Whether a worker may cancel their own request: one awaiting a decision, at any time, or one
/// approved that has not started. Leave that has started or is over, or one already refused or
/// cancelled, is HR's to change.
///
/// # Errors
///
/// A request the worker may not cancel, with the reason.
pub fn can_worker_cancel(
    status: &str,
    start_on: NaiveDate,
    today: NaiveDate,
) -> Result<(), String> {
    match status {
        "requested" => Ok(()),
        "approved" if start_on > today => Ok(()),
        "approved" => Err("this leave has started or is over; ask HR to change it".to_string()),
        other => Err(format!("a {other} request cannot be cancelled")),
    }
}

#[cfg(test)]
mod tests {
    #[test]
    fn a_worker_cancels_what_has_not_started() {
        use super::can_worker_cancel;
        let today = super::NaiveDate::from_ymd_opt(2026, 6, 15).unwrap();
        let day = |m, d| super::NaiveDate::from_ymd_opt(2026, m, d).unwrap();
        assert!(
            can_worker_cancel("requested", day(1, 5), today).is_ok(),
            "awaiting a decision, even if the dates passed"
        );
        assert!(
            can_worker_cancel("approved", day(6, 16), today).is_ok(),
            "starts tomorrow"
        );
        assert!(
            can_worker_cancel("approved", day(6, 15), today).is_err(),
            "starts today"
        );
        assert!(
            can_worker_cancel("approved", day(3, 1), today).is_err(),
            "already over"
        );
        assert!(can_worker_cancel("rejected", day(8, 1), today).is_err());
        assert!(can_worker_cancel("cancelled", day(8, 1), today).is_err());
    }

    use super::*;

    fn d(y: i32, m: u32, day: u32) -> NaiveDate {
        NaiveDate::from_ymd_opt(y, m, day).unwrap()
    }
    fn req(kind: &str, s: NaiveDate, e: NaiveDate, days: i32, status: &str) -> Request {
        Request {
            kind: kind.into(),
            start_on: s,
            end_on: e,
            days,
            status: status.into(),
        }
    }
    fn ent(kind: &str, year: i32, entitled: i32, used: i32) -> Entitlement {
        Entitlement {
            kind: kind.into(),
            year,
            entitled,
            used,
        }
    }

    #[test]
    fn a_balance_splits_taken_booked_and_requested() {
        let today = d(2026, 6, 15);
        let b = balances(
            &[ent("annual", 2026, 25, 8)],
            &[
                req("annual", d(2026, 3, 2), d(2026, 3, 6), 5, "approved"), // over
                req("annual", d(2026, 6, 15), d(2026, 6, 15), 1, "approved"), // today: still booked
                req("annual", d(2026, 8, 3), d(2026, 8, 4), 2, "approved"), // to come
                req("annual", d(2026, 9, 1), d(2026, 9, 3), 3, "requested"),
                req("annual", d(2026, 10, 1), d(2026, 10, 2), 2, "rejected"),
                req("annual", d(2026, 11, 1), d(2026, 11, 2), 2, "cancelled"),
            ],
            today,
        );
        assert_eq!(b.len(), 1);
        let a = &b[0];
        assert_eq!((a.taken, a.booked, a.requested), (5, 3, 3));
        assert_eq!(a.entitled, Some(25));
        assert_eq!(
            a.remaining,
            Some(17),
            "25 allowed less 8 the service has subtracted"
        );
        assert_eq!(a.remaining_if_requested_approved, Some(14));
    }

    #[test]
    fn no_entitlement_is_unknown_not_zero() {
        let b = balances(
            &[],
            &[req("unpaid", d(2026, 2, 2), d(2026, 2, 3), 2, "approved")],
            d(2026, 6, 1),
        );
        assert_eq!(b.len(), 1);
        assert_eq!(b[0].entitled, None);
        assert_eq!(b[0].remaining, None, "unknown, never 0");
        assert_eq!(b[0].remaining_if_requested_approved, None);
        assert_eq!(b[0].taken, 2);
    }

    #[test]
    fn an_entitlement_with_no_requests_is_all_remaining() {
        let b = balances(&[ent("sick", 2026, 30, 0)], &[], d(2026, 6, 1));
        assert_eq!(b[0].remaining, Some(30));
        assert_eq!((b[0].taken, b[0].booked, b[0].requested), (0, 0, 0));
    }

    #[test]
    fn requests_are_counted_in_the_year_they_start() {
        let b = balances(
            &[ent("annual", 2026, 25, 0), ent("annual", 2027, 25, 0)],
            &[req(
                "annual",
                d(2026, 12, 30),
                d(2027, 1, 3),
                5,
                "requested",
            )],
            d(2026, 6, 1),
        );
        let y26 = b.iter().find(|x| x.year == 2026).unwrap();
        let y27 = b.iter().find(|x| x.year == 2027).unwrap();
        assert_eq!(y26.requested, 5);
        assert_eq!(y27.requested, 0);
        assert!(
            b.windows(2)
                .all(|w| (w[0].year, &w[0].kind) <= (w[1].year, &w[1].kind)),
            "ordered by year"
        );
    }

    #[test]
    fn history_counts_finished_approved_absences_by_year_and_kind() {
        let h = history(
            &[
                req("sick", d(2025, 2, 3), d(2025, 2, 5), 3, "approved"),
                req("sick", d(2025, 9, 1), d(2025, 9, 1), 1, "approved"),
                req("annual", d(2025, 8, 4), d(2025, 8, 15), 12, "approved"),
                req("sick", d(2026, 1, 12), d(2026, 1, 13), 2, "approved"),
                req("sick", d(2026, 7, 1), d(2026, 7, 2), 2, "approved"), // not over yet
                req("sick", d(2025, 5, 5), d(2025, 5, 6), 2, "rejected"),
                req("sick", d(2025, 6, 6), d(2025, 6, 6), 1, "requested"),
            ],
            d(2026, 6, 15),
        );
        assert_eq!(h.len(), 2);
        assert_eq!(h[0].0, 2025);
        let sick25 = h[0].1.iter().find(|a| a.kind == "sick").unwrap();
        assert_eq!((sick25.absences, sick25.days), (2, 4));
        let annual25 = h[0].1.iter().find(|a| a.kind == "annual").unwrap();
        assert_eq!((annual25.absences, annual25.days), (1, 12));
        assert_eq!(h[1].0, 2026);
        assert_eq!(h[1].1.len(), 1);
        assert_eq!(
            h[1].1[0].days, 2,
            "the absence still to come is not history"
        );
    }
}
