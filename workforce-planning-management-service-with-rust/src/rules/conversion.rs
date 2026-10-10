//! **Conversion plans** (WPM-R98, WPM-D66): pure, database-free and clock-free rules about a
//! recorded intent for a fixed-term or contractor engagement.
//!
//! A plan is a decision a person makes and records. Nothing here decides one, and length of
//! service never creates one. A plan holds no pay figure.

use chrono::NaiveDate;

/// What a plan intends for the engagement.
pub const INTENTS: &[&str] = &["convert", "extend", "end", "undecided"];

/// Where a plan stands.
pub const STATUSES: &[&str] = &["proposed", "approved", "done", "abandoned"];

/// Where the permanent post's money comes from (the kinds of WPM-R62 and WPM-R83).
pub const FUNDING_KINDS: &[&str] = &["core", "time_limited", "external", "programme"];

/// The longest reason, in characters.
pub const MAX_REASON_CHARS: usize = 500;

/// Whether a plan is still open: proposed or approved.
#[must_use]
pub fn is_open(status: &str) -> bool {
    matches!(status, "proposed" | "approved")
}

/// What a person proposes.
#[derive(Debug, Clone, Copy)]
pub struct Proposal<'a> {
    /// One of [`INTENTS`].
    pub intent: &'a str,
    /// The date by which the decision is to be taken.
    pub target_on: NaiveDate,
    /// The funding of the permanent post, if it has one.
    pub post_funding_kind: Option<&'a str>,
    /// The funding's own end, if it has one.
    pub post_funding_ends_on: Option<NaiveDate>,
    /// Why, in the proposer's words.
    pub reason: Option<&'a str>,
    /// The date to look at the plan again.
    pub review_on: Option<NaiveDate>,
    /// The last day of the contract, when known.
    pub contract_ends_on: Option<NaiveDate>,
}

/// Check a proposal.
///
/// # Errors
///
/// A refusal that says what to change.
pub fn validate_proposal(p: &Proposal<'_>, today: NaiveDate) -> Result<(), String> {
    if !INTENTS.contains(&p.intent) {
        return Err(format!("intent must be one of {}", INTENTS.join(", ")));
    }
    if let Some(kind) = p.post_funding_kind
        && !FUNDING_KINDS.contains(&kind)
    {
        return Err(format!(
            "post_funding_kind must be one of {}",
            FUNDING_KINDS.join(", ")
        ));
    }
    if p.post_funding_ends_on.is_some() && p.post_funding_kind.is_none() {
        return Err("a funding end needs a funding kind".to_string());
    }
    if let Some(reason) = p.reason
        && reason.chars().count() > MAX_REASON_CHARS
    {
        return Err(format!(
            "the reason is at most {MAX_REASON_CHARS} characters"
        ));
    }
    if p.target_on < today {
        return Err("the decision date is in the past".to_string());
    }
    if let Some(end) = p.contract_ends_on
        && p.target_on >= end
    {
        return Err(format!(
            "the decision date must fall before the contract ends on {end}"
        ));
    }
    if let Some(review) = p.review_on
        && review < today
    {
        return Err("the review date is in the past".to_string());
    }
    if p.intent == "convert" {
        let has_reason = p.reason.is_some_and(|r| !r.trim().is_empty());
        if p.post_funding_kind.is_none() && !has_reason {
            return Err(
                "a conversion needs a funded permanent post, or a reason stating why it has none"
                    .to_string(),
            );
        }
        if let Some(funding_end) = p.post_funding_ends_on
            && funding_end <= p.target_on
        {
            return Err(
                "the permanent post's funding ends before the conversion could happen".to_string(),
            );
        }
    }
    Ok(())
}

/// Whether a plan may move from one status to another.
#[must_use]
pub fn may_move(from: &str, to: &str) -> bool {
    matches!(
        (from, to),
        ("proposed", "approved" | "abandoned") | ("approved", "done" | "abandoned")
    )
}

/// Check an approval: the approver is never the proposer. Two unknown actors cannot be told
/// apart, so they are treated as the same person.
///
/// # Errors
///
/// The plan may not be approved, or the approver proposed it.
pub fn validate_approval(
    intent: &str,
    status: &str,
    proposer: Option<&str>,
    approver: Option<&str>,
) -> Result<(), String> {
    if !may_move(status, "approved") {
        return Err(format!("a plan that is {status} cannot be approved"));
    }
    if intent == "undecided" {
        return Err(
            "an undecided plan records that no decision is taken yet; change the intent first"
                .to_string(),
        );
    }
    if let (Some(a), Some(b)) = (proposer, approver)
        && a == b
    {
        return Err("nobody approves their own plan".to_string());
    }
    Ok(())
}

/// Check that a plan may be marked done: only an approved plan to convert.
///
/// # Errors
///
/// The plan is not an approved conversion.
pub fn validate_done(intent: &str, status: &str) -> Result<(), String> {
    if status != "approved" {
        return Err(format!("a plan that is {status} cannot be marked done"));
    }
    if intent != "convert" {
        return Err("only a plan to convert is carried out here; extend an engagement through its extensions".to_string());
    }
    Ok(())
}

/// What is wrong with an engagement's plan on `as_of`, if anything.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Flag {
    /// The contract has ended and no approved plan covers it.
    PastEndWithoutApprovedPlan,
    /// The plan's review date has arrived.
    ReviewDue,
}

impl Flag {
    /// The reported token.
    #[must_use]
    pub const fn as_str(self) -> &'static str {
        match self {
            Self::PastEndWithoutApprovedPlan => "past_end_without_approved_plan",
            Self::ReviewDue => "review_due",
        }
    }
}

/// The flags for an engagement. `plan_status` and `review_on` describe its open plan, if any.
#[must_use]
pub fn flags(
    contract_ends_on: Option<NaiveDate>,
    plan_status: Option<&str>,
    review_on: Option<NaiveDate>,
    as_of: NaiveDate,
) -> Vec<Flag> {
    let mut out = Vec::new();
    if let Some(end) = contract_ends_on
        && end < as_of
        && plan_status != Some("approved")
    {
        out.push(Flag::PastEndWithoutApprovedPlan);
    }
    if let (Some(status), Some(review)) = (plan_status, review_on)
        && is_open(status)
        && review <= as_of
    {
        out.push(Flag::ReviewDue);
    }
    out
}

#[cfg(test)]
mod tests {
    use super::*;

    fn d(y: i32, m: u32, day: u32) -> NaiveDate {
        NaiveDate::from_ymd_opt(y, m, day).unwrap()
    }

    fn base() -> Proposal<'static> {
        Proposal {
            intent: "convert",
            target_on: d(2026, 11, 1),
            post_funding_kind: Some("core"),
            post_funding_ends_on: None,
            reason: None,
            review_on: None,
            contract_ends_on: Some(d(2026, 12, 31)),
        }
    }

    const TODAY: NaiveDate = NaiveDate::from_ymd_opt(2026, 10, 10).unwrap();

    #[test]
    fn a_funded_conversion_before_the_end_is_accepted() {
        assert!(validate_proposal(&base(), TODAY).is_ok());
    }

    #[test]
    fn an_unknown_intent_is_refused() {
        let p = Proposal {
            intent: "promote",
            ..base()
        };
        assert!(validate_proposal(&p, TODAY).unwrap_err().contains("intent"));
    }

    #[test]
    fn a_conversion_with_no_post_funding_needs_a_reason() {
        let p = Proposal {
            post_funding_kind: None,
            ..base()
        };
        assert!(
            validate_proposal(&p, TODAY)
                .unwrap_err()
                .contains("funded permanent post")
        );
        let blank = Proposal {
            post_funding_kind: None,
            reason: Some("   "),
            ..base()
        };
        assert!(validate_proposal(&blank, TODAY).is_err());
        let ok = Proposal {
            post_funding_kind: None,
            reason: Some("funding bid pending"),
            ..base()
        };
        assert!(validate_proposal(&ok, TODAY).is_ok());
    }

    #[test]
    fn other_intents_need_no_funding() {
        for intent in ["extend", "end", "undecided"] {
            let p = Proposal {
                intent,
                post_funding_kind: None,
                ..base()
            };
            assert!(validate_proposal(&p, TODAY).is_ok(), "{intent}");
        }
    }

    #[test]
    fn the_decision_date_falls_before_the_contract_end() {
        let on_the_day = Proposal {
            target_on: d(2026, 12, 31),
            ..base()
        };
        assert!(
            validate_proposal(&on_the_day, TODAY)
                .unwrap_err()
                .contains("before the contract")
        );
        let after = Proposal {
            target_on: d(2027, 1, 5),
            ..base()
        };
        assert!(validate_proposal(&after, TODAY).is_err());
        let day_before = Proposal {
            target_on: d(2026, 12, 30),
            ..base()
        };
        assert!(validate_proposal(&day_before, TODAY).is_ok());
    }

    #[test]
    fn no_known_end_leaves_the_date_open() {
        let p = Proposal {
            contract_ends_on: None,
            target_on: d(2030, 1, 1),
            ..base()
        };
        assert!(validate_proposal(&p, TODAY).is_ok());
    }

    #[test]
    fn dates_in_the_past_are_refused() {
        let p = Proposal {
            target_on: d(2026, 10, 9),
            ..base()
        };
        assert!(validate_proposal(&p, TODAY).unwrap_err().contains("past"));
        let r = Proposal {
            review_on: Some(d(2026, 10, 9)),
            ..base()
        };
        assert!(validate_proposal(&r, TODAY).unwrap_err().contains("review"));
    }

    #[test]
    fn funding_kind_must_be_known_and_a_funding_end_needs_a_kind() {
        let bad = Proposal {
            post_funding_kind: Some("lottery"),
            ..base()
        };
        assert!(
            validate_proposal(&bad, TODAY)
                .unwrap_err()
                .contains("post_funding_kind")
        );
        let orphan = Proposal {
            post_funding_kind: None,
            post_funding_ends_on: Some(d(2028, 1, 1)),
            reason: Some("x"),
            ..base()
        };
        assert!(
            validate_proposal(&orphan, TODAY)
                .unwrap_err()
                .contains("funding kind")
        );
    }

    #[test]
    fn a_post_whose_funding_ends_before_the_conversion_is_refused() {
        let p = Proposal {
            post_funding_ends_on: Some(d(2026, 11, 1)),
            ..base()
        };
        assert!(
            validate_proposal(&p, TODAY)
                .unwrap_err()
                .contains("funding ends")
        );
        let fine = Proposal {
            post_funding_ends_on: Some(d(2027, 11, 2)),
            ..base()
        };
        assert!(validate_proposal(&fine, TODAY).is_ok());
    }

    #[test]
    fn a_long_reason_is_refused() {
        let long = "x".repeat(MAX_REASON_CHARS + 1);
        let p = Proposal {
            reason: Some(&long),
            ..base()
        };
        assert!(validate_proposal(&p, TODAY).unwrap_err().contains("500"));
        let exact = "x".repeat(MAX_REASON_CHARS);
        let ok = Proposal {
            reason: Some(&exact),
            ..base()
        };
        assert!(validate_proposal(&ok, TODAY).is_ok());
    }

    #[test]
    fn the_transitions_are_exactly_the_four() {
        let all = ["proposed", "approved", "done", "abandoned"];
        let allowed = [
            ("proposed", "approved"),
            ("proposed", "abandoned"),
            ("approved", "done"),
            ("approved", "abandoned"),
        ];
        for from in all {
            for to in all {
                assert_eq!(
                    may_move(from, to),
                    allowed.contains(&(from, to)),
                    "{from}->{to}"
                );
            }
        }
    }

    #[test]
    fn nobody_approves_their_own_plan() {
        assert!(validate_approval("convert", "proposed", Some("a"), Some("a")).is_err());
        assert!(validate_approval("convert", "proposed", Some("a"), Some("b")).is_ok());
        assert!(validate_approval("convert", "proposed", Some("a"), None).is_ok());
        assert!(validate_approval("convert", "proposed", None, None).is_ok());
    }

    #[test]
    fn only_a_proposed_decided_plan_is_approved() {
        assert!(validate_approval("convert", "approved", Some("a"), Some("b")).is_err());
        assert!(validate_approval("convert", "done", Some("a"), Some("b")).is_err());
        assert!(validate_approval("convert", "abandoned", Some("a"), Some("b")).is_err());
        assert!(validate_approval("undecided", "proposed", Some("a"), Some("b")).is_err());
        for intent in ["convert", "extend", "end"] {
            assert!(validate_approval(intent, "proposed", None, Some("b")).is_ok());
        }
    }

    #[test]
    fn only_an_approved_conversion_is_done() {
        assert!(validate_done("convert", "approved").is_ok());
        assert!(validate_done("convert", "proposed").is_err());
        assert!(validate_done("convert", "done").is_err());
        assert!(validate_done("extend", "approved").is_err());
        assert!(validate_done("end", "approved").is_err());
    }

    #[test]
    fn past_the_end_with_no_approved_plan_is_flagged() {
        let end = Some(d(2026, 9, 30));
        let flags_for = |status| flags(end, status, None, TODAY);
        assert_eq!(flags_for(None), vec![Flag::PastEndWithoutApprovedPlan]);
        assert_eq!(
            flags_for(Some("proposed")),
            vec![Flag::PastEndWithoutApprovedPlan]
        );
        assert_eq!(flags_for(Some("approved")), Vec::<Flag>::new());
        // The last day itself is not past the end.
        assert_eq!(flags(Some(TODAY), None, None, TODAY), Vec::<Flag>::new());
        assert_eq!(flags(None, None, None, TODAY), Vec::<Flag>::new());
    }

    #[test]
    fn a_review_date_that_has_arrived_is_flagged_for_an_open_plan_only() {
        let due = Some(d(2026, 10, 10));
        assert_eq!(
            flags(None, Some("proposed"), due, TODAY),
            vec![Flag::ReviewDue]
        );
        assert_eq!(
            flags(None, Some("approved"), due, TODAY),
            vec![Flag::ReviewDue]
        );
        assert_eq!(flags(None, Some("done"), due, TODAY), Vec::<Flag>::new());
        assert_eq!(
            flags(None, Some("abandoned"), due, TODAY),
            Vec::<Flag>::new()
        );
        assert_eq!(
            flags(None, Some("proposed"), Some(d(2026, 10, 11)), TODAY),
            Vec::<Flag>::new()
        );
    }
}
