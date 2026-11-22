//! Row type — `requisitions`. One funded job opening with its hiring pipeline (WPM-R1).

#![expect(
    missing_docs,
    reason = "a generated row type: the module header names the table"
)]

use crate::sql_time::{SqlDate, SqlTimestamp};
use serde::{Deserialize, Serialize};
use uuid::Uuid;

#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
pub struct Model {
    pub created_at: SqlTimestamp,
    pub updated_at: SqlTimestamp,
    pub id: i32,
    pub pid: Uuid,
    pub organization_ref: String,
    pub department: String,
    pub job_title: String,
    pub headcount: i32,
    pub salary_min_minor: Option<i64>,
    pub salary_max_minor: Option<i64>,
    pub salary_currency: Option<String>,
    pub status: String,
    pub opened_on: Option<SqlDate>,
    pub filled_on: Option<SqlDate>,
    pub deleted_at: Option<SqlTimestamp>,
    /// The programme the post serves (`thing:` URN), if any.
    pub programme_ref: Option<String>,
    /// The engagement basis of the post.
    pub basis: String,
    /// How long a basis that ends is expected to last, in calendar months.
    pub expected_duration_months: Option<i32>,
}
