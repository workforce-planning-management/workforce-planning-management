//! Absence categories (WPM-R5, WPM-D73, WPM-D121): the kinds of leave, each paid, unpaid or by policy, how it is held against a
//! balance, and whether a reason may be given. Reference data from the pure core, open to a signed-in caller. There is no category
//! for a condition or for stress.

use loco_rs::prelude::*;
use serde::Deserialize;
use serde_json::json;

use super::{ensure_valid, unprocessable};
use crate::rules::absence_category::{self as rules, Pay};

#[derive(Debug, Deserialize)]
struct Query {
    pay: Option<String>,
}

/// `GET /api/absence-categories?pay=paid|unpaid|policy` — the categories, optionally only those with the given pay.
#[debug_handler]
async fn list(axum::extract::Query(q): axum::extract::Query<Query>) -> Result<Response> {
    let wanted = match q.pay.as_deref() {
        None => None,
        Some("paid") => Some(Pay::Paid),
        Some("unpaid") => Some(Pay::Unpaid),
        Some("policy") => Some(Pay::Policy),
        Some(_) => {
            ensure_valid(&["pay must be one of: paid, unpaid, policy".to_string()])?;
            return Err(unprocessable("pay must be one of: paid, unpaid, policy"));
        }
    };
    let categories: Vec<_> = rules::CATEGORIES
        .iter()
        .filter(|c| wanted.is_none_or(|p| c.pay == p))
        .map(|c| {
            json!({
                "kind": c.kind,
                "pay": c.pay.token(),
                "balance": c.balance.token(),
                "reason_allowed": c.reason_allowed,
            })
        })
        .collect();
    format::json(json!({ "categories": categories }))
}

/// The routes.
pub fn routes() -> Routes {
    Routes::new()
        .prefix("/api")
        .add("/absence-categories", get(list))
}
