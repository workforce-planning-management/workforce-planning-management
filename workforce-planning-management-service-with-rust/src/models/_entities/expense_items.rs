//! Row type — `expense_items`. One dated, categorised line on an expense claim (WPM-R55).

#![allow(missing_docs)]

use crate::sql_time::{SqlDate, SqlTimestamp};
use serde::{Deserialize, Serialize};
use uuid::Uuid;

#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
pub struct Model {
    pub created_at: SqlTimestamp,
    pub updated_at: SqlTimestamp,
    pub id: i32,
    pub pid: Uuid,
    pub claim_pid: Uuid,
    pub worker_pid: Uuid,
    pub incurred_on: SqlDate,
    pub category: String,
    pub amount_minor: i64,
    pub description: Option<String>,
    pub receipt_ref: Option<String>,
}
