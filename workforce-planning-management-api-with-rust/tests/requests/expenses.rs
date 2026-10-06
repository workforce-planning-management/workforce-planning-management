//! Expense claims: a draft built from validated items, submitted, decided,
//! reimbursed; duplicates flagged; the manager told and the claimant told of the
//! outcome without any amount; exported, and on erasure the text goes but the
//! amounts stay. (Who may do what under enforcement is in `tests/enforcement.rs`.)

use chrono::{Duration, Utc};
use loco_rs::testing::prelude::*;
use sea_orm::{ConnectionTrait, Statement};
use serde_json::{Value, json};
use serial_test::serial;
use workforce_planning_management_service::app::App;

use super::{activate, an_org, seed_worker};

#[tokio::test]
#[serial]
#[ignore = "requires PostgreSQL (config/test.yaml); run with `cargo test -- --ignored`"]
async fn a_claim_goes_from_draft_to_reimbursed() {
    request::<App, _, _>(|request, _ctx| async move {
        let org = an_org();
        let boss = seed_worker!(&request, &org, "EX-M", None).await;
        let worker = seed_worker!(&request, &org, "EX-1", None).await;
        for w in [&boss, &worker] {
            activate!(&request, w).await;
        }
        request
            .put(&format!("/api/workers/{worker}"))
            .json(&json!({ "manager_pid": boss }))
            .await
            .assert_status_ok();
        let today = Utc::now().date_naive();
        let url = format!("/api/workers/{worker}/expense-claims");

        // Bad claims: no title, a lower-case or short currency.
        for bad in [
            json!({ "title": " ", "currency": "GBP" }),
            json!({ "title": "Trip", "currency": "gbp" }),
            json!({ "title": "Trip", "currency": "GB" }),
        ] {
            assert_eq!(
                request.post(&url).json(&bad).await.status_code(),
                422,
                "{bad}"
            );
        }
        let claim: Value = request
            .post(&url)
            .json(&json!({ "title": "Leeds client visit", "currency": "GBP",
                           "description": "train and lunch" }))
            .await
            .json();
        let pid = claim["pid"].as_str().unwrap().to_string();
        assert_eq!(claim["status"], "draft");
        assert_eq!(claim["total_minor"], 0);
        let items = format!("/api/expense-claims/{pid}/items");

        // An empty claim cannot be submitted.
        assert_eq!(
            request
                .post(&format!("/api/expense-claims/{pid}/submit"))
                .await
                .status_code(),
            422
        );

        // Items are validated: category, amount, future date.
        for bad in [
            json!({ "incurred_on": today.to_string(), "category": "yacht", "amount_minor": 500 }),
            json!({ "incurred_on": today.to_string(), "category": "travel", "amount_minor": 0 }),
            json!({ "incurred_on": today.to_string(), "category": "travel", "amount_minor": -5 }),
            json!({ "incurred_on": (today + Duration::days(1)).to_string(),
                    "category": "travel", "amount_minor": 500 }),
        ] {
            assert_eq!(
                request.post(&items).json(&bad).await.status_code(),
                422,
                "{bad}"
            );
        }
        let day = (today - Duration::days(3)).to_string();
        for (cat, amount) in [("travel", 4_250), ("meals", 1_250), ("meals", 1_250)] {
            request
                .post(&items)
                .json(
                    &json!({ "incurred_on": day, "category": cat, "amount_minor": amount,
                               "description": "ticket", "receipt_ref": "R-100" }),
                )
                .await
                .assert_status_ok();
        }

        // The total is the sum; the repeated lunch is *flagged*, not refused.
        let full: Value = request
            .get(&format!("/api/expense-claims/{pid}"))
            .await
            .json();
        assert_eq!(full["total_minor"], 6_750);
        assert_eq!(full["item_count"], 3);
        let flags: Vec<bool> = full["items"]
            .as_array()
            .unwrap()
            .iter()
            .map(|i| i["possible_duplicate"].as_bool().unwrap())
            .collect();
        assert_eq!(flags.iter().filter(|f| **f).count(), 2, "{flags:?}");
        assert_eq!(full["can"]["submit"], true);
        assert_eq!(full["can"]["decide"], false, "a draft is not yet decidable");

        // Remove one lunch, then submit.
        let dup = full["items"]
            .as_array()
            .unwrap()
            .iter()
            .rfind(|i| i["category"] == "meals")
            .unwrap()["pid"]
            .as_str()
            .unwrap()
            .to_string();
        request
            .delete(&format!("/api/expense-items/{dup}"))
            .await
            .assert_status_success();
        let submitted: Value = request
            .post(&format!("/api/expense-claims/{pid}/submit"))
            .await
            .json();
        assert_eq!(submitted["status"], "submitted");
        assert_eq!(submitted["total_minor"], 5_500);

        // Frozen once submitted: no more items, no second submit.
        assert_eq!(
            request
                .post(&items)
                .json(&json!({ "incurred_on": day,
            "category": "travel", "amount_minor": 100 }))
                .await
                .status_code(),
            422
        );
        assert_eq!(
            request
                .post(&format!("/api/expense-claims/{pid}/submit"))
                .await
                .status_code(),
            422
        );

        // The manager was told, without any amount or title.
        let told: Value = request
            .get(&format!("/api/workers/{boss}/notifications"))
            .await
            .json();
        let note = told
            .as_array()
            .unwrap()
            .iter()
            .find(|n| n["kind"] == "expense_submitted")
            .expect("the manager is told");
        let all = serde_json::to_string(note).unwrap();
        for leak in ["5500", "55.00", "Leeds", "£", "train"] {
            assert!(!all.contains(leak), "{leak} leaked: {all}");
        }

        // The decision queue lists it; rejecting needs a reason; approve it.
        let queue: Value = request
            .get("/api/expense-claims?status=submitted")
            .await
            .json();
        assert!(
            queue
                .as_array()
                .unwrap()
                .iter()
                .any(|c| c["pid"] == pid.as_str())
        );
        assert_eq!(
            request
                .get("/api/expense-claims?status=nope")
                .await
                .status_code(),
            422
        );
        assert_eq!(
            request
                .post(&format!("/api/expense-claims/{pid}/reject"))
                .await
                .status_code(),
            422,
            "a rejection needs a note"
        );
        // Reimbursing before approval is refused.
        assert_eq!(
            request
                .post(&format!("/api/expense-claims/{pid}/reimburse"))
                .await
                .status_code(),
            422
        );
        let approved: Value = request
            .post(&format!("/api/expense-claims/{pid}/approve"))
            .json(&json!({ "note": "fine" }))
            .await
            .json();
        assert_eq!(approved["status"], "approved");
        // A decided claim cannot be decided again or cancelled.
        for verb in ["approve", "reject", "cancel", "withdraw"] {
            assert_eq!(
                request
                    .post(&format!("/api/expense-claims/{pid}/{verb}"))
                    .json(&json!({ "note": "x" }))
                    .await
                    .status_code(),
                422,
                "{verb}"
            );
        }
        // A future reimbursement date is refused; then reimburse.
        assert_eq!(
            request
                .post(&format!("/api/expense-claims/{pid}/reimburse"))
                .json(&json!({ "reimbursed_on": (today + Duration::days(2)).to_string() }))
                .await
                .status_code(),
            422
        );
        let paid: Value = request
            .post(&format!("/api/expense-claims/{pid}/reimburse"))
            .await
            .json();
        assert_eq!(paid["status"], "reimbursed");
        assert_eq!(paid["reimbursed_on"], today.to_string());

        // The claimant was told each outcome, naming no amount.
        let own: Value = request
            .get(&format!("/api/workers/{worker}/notifications"))
            .await
            .json();
        let decided: Vec<&Value> = own
            .as_array()
            .unwrap()
            .iter()
            .filter(|n| n["kind"] == "expense_decided")
            .collect();
        assert_eq!(decided.len(), 2, "approved and reimbursed");
        assert!(!serde_json::to_string(&decided).unwrap().contains("5500"));

        // Audit: events recorded, no amount or title.
        let audit = serde_json::to_string(
            &request
                .get(&format!("/api/audits/{pid}"))
                .await
                .json::<Value>(),
        )
        .unwrap();
        assert!(audit.contains("expense_claim_reimbursed"), "{audit}");
        assert!(
            !audit.contains("5500") && !audit.contains("Leeds"),
            "{audit}"
        );

        // Rejection path on a second claim, and a cancelled draft.
        let second: Value = request
            .post(&url)
            .json(&json!({ "title": "Conference", "currency": "EUR" }))
            .await
            .json();
        let p2 = second["pid"].as_str().unwrap().to_string();
        request
            .post(&format!("/api/expense-claims/{p2}/items"))
            .json(&json!({ "incurred_on": day, "category": "training", "amount_minor": 90_000 }))
            .await
            .assert_status_ok();
        request
            .post(&format!("/api/expense-claims/{p2}/submit"))
            .await
            .assert_status_ok();
        let rejected: Value = request
            .post(&format!("/api/expense-claims/{p2}/reject"))
            .json(&json!({ "note": "not in budget" }))
            .await
            .json();
        assert_eq!(rejected["status"], "rejected");
        assert_eq!(rejected["decision_note"], "not in budget");
        let third: Value = request
            .post(&url)
            .json(&json!({ "title": "Oops", "currency": "GBP" }))
            .await
            .json();
        let cancelled: Value = request
            .post(&format!(
                "/api/expense-claims/{}/cancel",
                third["pid"].as_str().unwrap()
            ))
            .await
            .json();
        assert_eq!(cancelled["status"], "cancelled");

        // The list shows all three with their totals, newest first.
        let list: Value = request.get(&url).await.json();
        let statuses: Vec<&str> = list
            .as_array()
            .unwrap()
            .iter()
            .map(|c| c["status"].as_str().unwrap())
            .collect();
        assert_eq!(statuses, ["cancelled", "rejected", "reimbursed"]);
        assert_eq!(list[1]["total_minor"], 90_000);
        assert_eq!(list[1]["currency"], "EUR");
    })
    .await;
}

#[tokio::test]
#[serial]
#[ignore = "requires PostgreSQL (config/test.yaml); run with `cargo test -- --ignored`"]
async fn a_claim_is_exported_and_erased_keeping_the_amounts() {
    request::<App, _, _>(|request, ctx| async move {
        let org = an_org();
        let worker = seed_worker!(&request, &org, "EX-2", None).await;
        activate!(&request, &worker).await;
        let day = (Utc::now().date_naive() - Duration::days(2)).to_string();
        let paid: Value = request
            .post(&format!("/api/workers/{worker}/expense-claims"))
            .json(&json!({ "title": "Secret errand", "currency": "GBP", "description": "private detail" }))
            .await
            .json();
        let pid = paid["pid"].as_str().unwrap().to_string();
        request
            .post(&format!("/api/expense-claims/{pid}/items"))
            .json(&json!({ "incurred_on": day, "category": "equipment", "amount_minor": 12_345,
                           "description": "a thing", "receipt_ref": "RCPT-9" }))
            .await
            .assert_status_ok();
        for verb in ["submit", "approve", "reimburse"] {
            request.post(&format!("/api/expense-claims/{pid}/{verb}")).await.assert_status_ok();
        }
        // A second claim left as a draft.
        let draft: Value = request
            .post(&format!("/api/workers/{worker}/expense-claims"))
            .json(&json!({ "title": "Unfinished", "currency": "GBP" }))
            .await
            .json();
        let draft_pid = draft["pid"].as_str().unwrap().to_string();

        let export: Value = request.get(&format!("/api/workers/{worker}/subject-access")).await.json();
        assert_eq!(export["expense_claims"].as_array().unwrap().len(), 2);
        assert_eq!(export["expense_items"][0]["amount_minor"], 12_345);

        for to in ["offboarding", "terminated"] {
            request
                .post(&format!("/api/workers/{worker}/status"))
                .json(&json!({ "to": to }))
                .await
                .assert_status_ok();
        }
        assert_eq!(request.post(&format!("/api/workers/{worker}/erase")).await.status_code(), 200);

        let query = |sql: String| {
            let ctx = &ctx;
            async move {
                ctx.db
                    .query_one_raw(Statement::from_string(ctx.db.get_database_backend(), sql))
                    .await
                    .unwrap()
                    .unwrap()
            }
        };
        // Free text is gone; the amount and the reimbursed claim stay.
        let item = query(format!(
            "SELECT amount_minor, description, receipt_ref FROM expense_items WHERE claim_pid = '{pid}'"
        ))
        .await;
        assert_eq!(item.try_get_by_index::<i64>(0).unwrap(), 12_345);
        assert_eq!(item.try_get_by_index::<Option<String>>(1).unwrap(), None);
        assert_eq!(item.try_get_by_index::<Option<String>>(2).unwrap(), None);
        let claim = query(format!(
            "SELECT title, description, status FROM expense_claims WHERE pid = '{pid}'"
        ))
        .await;
        assert_eq!(claim.try_get_by_index::<String>(0).unwrap(), "erased");
        assert_eq!(claim.try_get_by_index::<Option<String>>(1).unwrap(), None);
        assert_eq!(claim.try_get_by_index::<String>(2).unwrap(), "reimbursed");
        // The unfinished draft is closed, not left dangling.
        let closed = query(format!("SELECT status FROM expense_claims WHERE pid = '{draft_pid}'")).await;
        assert_eq!(closed.try_get_by_index::<String>(0).unwrap(), "cancelled");
    })
    .await;
}
