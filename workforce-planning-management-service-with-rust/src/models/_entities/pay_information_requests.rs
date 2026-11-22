//! Row type — `pay_information_requests`. A worker's request for their own pay level and their category's
//! average pay by sex (WPM-R145). Dates only; the figures are not stored.

#![expect(
    missing_docs,
    reason = "a generated row type: the module header names the table"
)]

use crate::sql_time::{SqlDate, SqlTimestamp};
use serde::{Deserialize, Serialize};
use uuid::Uuid;

#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub struct Model {
    pub created_at: SqlTimestamp,
    pub updated_at: SqlTimestamp,
    pub id: i32,
    pub pid: Uuid,
    pub worker_pid: Uuid,
    pub asked_on: SqlDate,
    pub due_on: SqlDate,
    pub answered_on: Option<SqlDate>,
    pub answered_by: Option<String>,
    pub deleted_at: Option<SqlTimestamp>,
}
