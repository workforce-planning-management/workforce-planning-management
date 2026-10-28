//! Absence categories (WPM-R5, WPM-R118, WPM-D73, WPM-D121). DB-free.
//!
//! Each kind of leave is a category with three properties: whether it is **paid, unpaid or set by the employer's policy**, how it
//! is held against a balance, and whether a reason may be given. The table is the one place those are written down; the balance
//! check and the sick-leave reason rule read it.
//!
//! **There is no category for a condition.** `sick` is illness or unwellness *as a fact and its dates*, never why (WPM-D73), and
//! there is no workplace-stress category: stress is assessed in the work, not recorded in the person (WPM-D121).

/// Whether time off in the category is paid.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Pay {
    /// Paid leave.
    Paid,
    /// Unpaid leave.
    Unpaid,
    /// Paid or not as the employer's policy, scheme or contract says; WPM does not decide it.
    Policy,
}

impl Pay {
    /// The token the API uses.
    #[must_use]
    pub fn token(self) -> &'static str {
        match self {
            Pay::Paid => "paid",
            Pay::Unpaid => "unpaid",
            Pay::Policy => "policy",
        }
    }
}

/// How a request is held against the worker's balance.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Balance {
    /// A request over the remaining balance is refused.
    Enforced,
    /// A request over the remaining balance is allowed and flagged.
    MayGoNegative,
    /// There is no balance.
    None,
}

impl Balance {
    /// The token the API uses.
    #[must_use]
    pub fn token(self) -> &'static str {
        match self {
            Balance::Enforced => "enforced",
            Balance::MayGoNegative => "may_go_negative",
            Balance::None => "none",
        }
    }
}

/// One category of absence.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct Category {
    /// The leave kind token.
    pub kind: &'static str,
    /// Paid, unpaid or by policy.
    pub pay: Pay,
    /// How it is held against a balance.
    pub balance: Balance,
    /// Whether a request may carry a reason. `false` for sick leave (WPM-D73).
    pub reason_allowed: bool,
}

/// Every category. `annual` is vacation and `public_holiday` is a government holiday; `sick` is illness or unwellness.
pub const CATEGORIES: &[Category] = &[
    Category {
        kind: "annual",
        pay: Pay::Paid,
        balance: Balance::Enforced,
        reason_allowed: true,
    },
    Category {
        kind: "public_holiday",
        pay: Pay::Paid,
        balance: Balance::None,
        reason_allowed: true,
    },
    Category {
        kind: "sick",
        pay: Pay::Policy,
        balance: Balance::MayGoNegative,
        reason_allowed: false,
    },
    Category {
        kind: "parental",
        pay: Pay::Policy,
        balance: Balance::Enforced,
        reason_allowed: true,
    },
    Category {
        kind: "unpaid",
        pay: Pay::Unpaid,
        balance: Balance::None,
        reason_allowed: true,
    },
    Category {
        kind: "other",
        pay: Pay::Policy,
        balance: Balance::Enforced,
        reason_allowed: true,
    },
];

/// The category for a kind.
#[must_use]
pub fn find(kind: &str) -> Option<&'static Category> {
    CATEGORIES.iter().find(|c| c.kind == kind)
}

/// How a kind is held against a balance. A kind that is not a category is enforced, the strict reading.
#[must_use]
pub fn balance_of(kind: &str) -> Balance {
    find(kind).map_or(Balance::Enforced, |c| c.balance)
}

/// Whether a request of this kind may carry a reason. A kind that is not a category may.
#[must_use]
pub fn reason_allowed(kind: &str) -> bool {
    find(kind).is_none_or(|c| c.reason_allowed)
}

/// The categories with the given pay, in table order.
#[must_use]
pub fn with_pay(pay: Pay) -> Vec<&'static Category> {
    CATEGORIES.iter().filter(|c| c.pay == pay).collect()
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::rules::tokens::LEAVE_KINDS;

    #[test]
    fn every_leave_kind_is_a_category_and_the_other_way_round() {
        let kinds: Vec<_> = CATEGORIES.iter().map(|c| c.kind).collect();
        assert_eq!(kinds, LEAVE_KINDS.to_vec(), "same kinds in the same order");
    }

    #[test]
    fn the_examples_are_there_with_the_pay_they_should_have() {
        assert_eq!(find("annual").map(|c| c.pay), Some(Pay::Paid), "vacation");
        assert_eq!(
            find("public_holiday").map(|c| c.pay),
            Some(Pay::Paid),
            "government holiday"
        );
        assert_eq!(
            find("sick").map(|c| c.pay),
            Some(Pay::Policy),
            "illness: sick pay is a policy, not WPM's call"
        );
        assert_eq!(find("unpaid").map(|c| c.pay), Some(Pay::Unpaid));
    }

    #[test]
    fn paid_and_unpaid_are_separate_lists_and_policy_is_not_forced_into_either() {
        let names = |p| with_pay(p).iter().map(|c| c.kind).collect::<Vec<_>>();
        assert_eq!(names(Pay::Paid), ["annual", "public_holiday"]);
        assert_eq!(names(Pay::Unpaid), ["unpaid"]);
        assert_eq!(names(Pay::Policy), ["sick", "parental", "other"]);
        assert_eq!(CATEGORIES.len(), 6);
    }

    #[test]
    fn only_sick_leave_refuses_a_reason_and_only_unpaid_and_holidays_have_no_balance() {
        let refuses: Vec<_> = CATEGORIES
            .iter()
            .filter(|c| !c.reason_allowed)
            .map(|c| c.kind)
            .collect();
        assert_eq!(refuses, ["sick"]);
        let none: Vec<_> = CATEGORIES
            .iter()
            .filter(|c| c.balance == Balance::None)
            .map(|c| c.kind)
            .collect();
        assert_eq!(none, ["public_holiday", "unpaid"]);
        assert_eq!(balance_of("sick"), Balance::MayGoNegative);
        assert_eq!(balance_of("made_up"), Balance::Enforced);
        assert!(reason_allowed("made_up"));
    }

    /// WPM-D73 and WPM-D121: no category names a condition, a diagnosis or stress.
    #[test]
    fn no_category_records_a_condition() {
        for c in CATEGORIES {
            for word in [
                "stress",
                "diagnos",
                "condition",
                "mental",
                "cancer",
                "burnout",
                "depress",
                "anxiety",
            ] {
                assert!(
                    !c.kind.contains(word),
                    "{} names a condition: {word}",
                    c.kind
                );
            }
        }
    }

    #[test]
    fn tokens_are_the_api_words() {
        assert_eq!(
            (Pay::Paid.token(), Pay::Unpaid.token(), Pay::Policy.token()),
            ("paid", "unpaid", "policy")
        );
        assert_eq!(Balance::MayGoNegative.token(), "may_go_negative");
    }
}
