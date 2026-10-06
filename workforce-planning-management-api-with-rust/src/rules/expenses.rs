//! Employee **expense claims** (WPM-R55), DB-free: the claim lifecycle, line-item
//! validation, totals, duplicate detection and who may see, edit and decide.
//!
//! A claim is a draft the claimant builds from dated, categorised line items; they
//! submit it; someone **other than the claimant** — their manager or HR —
//! approves or rejects it; approved claims are marked reimbursed. Money is minor
//! units in the claim's single currency: mixed currencies never silently add.
//! Dates are arguments; nothing here reads a clock.

use chrono::NaiveDate;

use super::lifecycle;

/// The claim statuses.
pub const STATUSES: &[&str] = &[
    "draft",
    "submitted",
    "approved",
    "rejected",
    "reimbursed",
    "cancelled",
];

/// The legal claim transitions. `rejected`, `reimbursed` and `cancelled` are
/// terminal; a submitted claim can be withdrawn back to `draft` to be edited.
pub const TRANSITIONS: &[(&str, &str)] = &[
    ("draft", "submitted"),
    ("submitted", "draft"),
    ("submitted", "approved"),
    ("submitted", "rejected"),
    ("approved", "reimbursed"),
    ("draft", "cancelled"),
    ("submitted", "cancelled"),
];

/// Check a status move, with the family `422` message on refusal.
///
/// # Errors
/// A refusal naming the current state.
pub fn check_transition(from: &str, to: &str) -> Result<(), String> {
    lifecycle::check("expense claim", TRANSITIONS, from, to)
}

/// What an item was for.
pub const CATEGORIES: &[&str] = &[
    "travel",
    "accommodation",
    "meals",
    "equipment",
    "training",
    "subscriptions",
    "other",
];

/// The most items on one claim.
pub const MAX_ITEMS: usize = 50;

/// The largest single item, in minor units (a sanity cap, not a policy limit).
pub const MAX_ITEM_MINOR: i64 = 100_000_000;

/// An ISO 4217-shaped currency: three upper-case ASCII letters.
///
/// # Errors
/// A message when it is not.
pub fn validate_currency(currency: &str) -> Result<(), String> {
    if currency.len() == 3 && currency.bytes().all(|b| b.is_ascii_uppercase()) {
        Ok(())
    } else {
        Err("currency must be a three-letter ISO 4217 code such as GBP".to_string())
    }
}

/// One expense line, validated: a known category, a positive bounded amount, and
/// an incurred date that is not in the future.
///
/// # Errors
/// A message naming the first problem.
pub fn validate_item(
    category: &str,
    amount_minor: i64,
    incurred_on: NaiveDate,
    today: NaiveDate,
) -> Result<(), String> {
    if !CATEGORIES.contains(&category) {
        return Err(format!(
            "category must be one of: {}",
            CATEGORIES.join(", ")
        ));
    }
    if amount_minor <= 0 {
        return Err("amount_minor must be greater than zero".to_string());
    }
    if amount_minor > MAX_ITEM_MINOR {
        return Err(format!("amount_minor is over the {MAX_ITEM_MINOR} cap"));
    }
    if incurred_on > today {
        return Err("incurred_on cannot be in the future".to_string());
    }
    Ok(())
}

/// The total of a claim's items, in minor units — checked, so an overflow is an
/// error rather than a wrapped number.
///
/// # Errors
/// A message on overflow.
pub fn total_minor(amounts: &[i64]) -> Result<i64, String> {
    amounts
        .iter()
        .try_fold(0_i64, |sum, a| sum.checked_add(*a))
        .ok_or_else(|| "total is too large".to_string())
}

/// A claim needs at least one item to be submitted.
///
/// # Errors
/// A message when there are none.
pub fn can_submit(item_count: usize) -> Result<(), String> {
    if item_count == 0 {
        Err("add at least one item before submitting".to_string())
    } else {
        Ok(())
    }
}

/// The identity of an item for duplicate detection: the same day, category and
/// amount.
pub type ItemKey<'a> = (NaiveDate, &'a str, i64);

/// Which of `items` look like duplicates — the same day, category and amount as
/// another item on the claim, or as an item on one of the claimant's `others`.
/// Returns the indexes into `items`. A **flag, not a refusal**: two identical
/// coffees are sometimes two coffees.
#[must_use]
pub fn possible_duplicates(items: &[ItemKey<'_>], others: &[ItemKey<'_>]) -> Vec<usize> {
    items
        .iter()
        .enumerate()
        .filter(|(i, key)| {
            others.contains(key)
                || items
                    .iter()
                    .enumerate()
                    .any(|(j, other)| j != *i && other == *key)
        })
        .map(|(i, _)| i)
        .collect()
}

/// Whether the caller is the claimant, the claimant's manager, or HR in the
/// claimant's organization.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct Standing {
    /// The claim is the caller's own.
    pub claimant: bool,
    /// The caller manages the claimant.
    pub manager: bool,
    /// The caller has the HR role in the claimant's organization.
    pub hr: bool,
}

/// Who may **see** a claim: the claimant, their manager, HR.
#[must_use]
pub fn may_view(s: Standing) -> bool {
    s.claimant || s.manager || s.hr
}

/// Who may **build or change** a claim (add items, submit, withdraw, cancel):
/// the claimant, or HR on their behalf. A manager reviews; they do not write it.
#[must_use]
pub fn may_edit(s: Standing) -> bool {
    s.claimant || s.hr
}

/// Who may **decide** a claim (approve, reject, mark reimbursed): the claimant's
/// manager or HR — **never the claimant**, even if they are also HR.
#[must_use]
pub fn may_decide(s: Standing) -> bool {
    !s.claimant && (s.manager || s.hr)
}

#[cfg(test)]
mod tests {
    use super::*;

    fn d(y: i32, m: u32, day: u32) -> NaiveDate {
        NaiveDate::from_ymd_opt(y, m, day).unwrap()
    }

    /// The machine: the happy path, withdrawal, and every terminal state stays
    /// terminal; skips are refused; the tokens match the status vocabulary.
    #[test]
    fn the_claim_lifecycle() {
        for (from, to) in TRANSITIONS {
            assert!(
                STATUSES.contains(from) && STATUSES.contains(to),
                "{from}->{to}"
            );
        }
        for (from, to) in [
            ("draft", "submitted"),
            ("submitted", "approved"),
            ("approved", "reimbursed"),
            ("submitted", "draft"),
            ("submitted", "rejected"),
            ("draft", "cancelled"),
        ] {
            assert!(check_transition(from, to).is_ok(), "{from}->{to}");
        }
        for (from, to) in [
            ("draft", "approved"),
            ("draft", "reimbursed"),
            ("submitted", "reimbursed"),
            ("approved", "submitted"),
            ("approved", "rejected"),
            ("rejected", "submitted"),
            ("reimbursed", "cancelled"),
            ("cancelled", "draft"),
        ] {
            assert!(check_transition(from, to).is_err(), "{from}->{to}");
        }
        let message = check_transition("approved", "rejected").unwrap_err();
        assert!(message.contains("approved") && message.contains("expense claim"));
        for terminal in ["rejected", "reimbursed", "cancelled"] {
            assert!(TRANSITIONS.iter().all(|(from, _)| *from != terminal));
        }
    }

    #[test]
    fn an_item_is_validated() {
        let today = d(2026, 10, 6);
        assert!(validate_item("travel", 1, d(2026, 10, 6), today).is_ok());
        assert!(validate_item("travel", MAX_ITEM_MINOR, d(2020, 1, 1), today).is_ok());
        assert!(
            validate_item("yacht", 100, today, today)
                .unwrap_err()
                .contains("category")
        );
        assert!(validate_item("travel", 0, today, today).is_err());
        assert!(validate_item("travel", -5, today, today).is_err());
        assert!(validate_item("travel", MAX_ITEM_MINOR + 1, today, today).is_err());
        assert!(
            validate_item("travel", 100, d(2026, 10, 7), today)
                .unwrap_err()
                .contains("future")
        );
    }

    #[test]
    fn currency_and_totals() {
        assert!(validate_currency("GBP").is_ok());
        for bad in ["gbp", "GB", "GBPP", "G1P", ""] {
            assert!(validate_currency(bad).is_err(), "{bad}");
        }
        assert_eq!(total_minor(&[]), Ok(0));
        assert_eq!(total_minor(&[1250, 799, 1]), Ok(2050));
        assert!(total_minor(&[i64::MAX, 1]).is_err(), "overflow is an error");
        assert!(can_submit(0).is_err());
        assert!(can_submit(1).is_ok());
    }

    /// A repeat within the claim, or of an earlier claim's item, is flagged on
    /// the repeating items; different day, category or amount is not.
    #[test]
    fn duplicates_are_flagged_not_refused() {
        let a: ItemKey = (d(2026, 10, 1), "meals", 1250);
        let b: ItemKey = (d(2026, 10, 1), "meals", 1250);
        let c: ItemKey = (d(2026, 10, 2), "meals", 1250);
        let e: ItemKey = (d(2026, 10, 1), "travel", 1250);
        assert_eq!(possible_duplicates(&[a, b, c, e], &[]), vec![0, 1]);
        assert_eq!(possible_duplicates(&[c, e], &[]), Vec::<usize>::new());
        // Against another claim of the same person.
        assert_eq!(possible_duplicates(&[c, e], &[e]), vec![1]);
        assert_eq!(possible_duplicates(&[], &[a]), Vec::<usize>::new());
    }

    /// The claimant never decides their own claim — even as HR — and a manager
    /// decides but does not write the claim.
    #[test]
    fn who_may_do_what() {
        let s = |claimant, manager, hr| Standing {
            claimant,
            manager,
            hr,
        };
        assert!(
            may_view(s(true, false, false))
                && may_view(s(false, true, false))
                && may_view(s(false, false, true))
        );
        assert!(!may_view(s(false, false, false)));
        assert!(may_edit(s(true, false, false)) && may_edit(s(false, false, true)));
        assert!(
            !may_edit(s(false, true, false)),
            "a manager reviews, not writes"
        );
        assert!(may_decide(s(false, true, false)) && may_decide(s(false, false, true)));
        assert!(!may_decide(s(true, false, false)));
        assert!(
            !may_decide(s(true, false, true)),
            "HR cannot approve their own claim"
        );
        assert!(!may_decide(s(true, true, true)));
        assert!(!may_decide(s(false, false, false)));
    }
}
