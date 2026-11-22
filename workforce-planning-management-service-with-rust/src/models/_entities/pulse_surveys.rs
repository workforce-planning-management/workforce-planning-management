//! Row type — `pulse_surveys`. One anonymous pulse survey (WPM-R28): a name, one question, an active window.

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
    pub name: String,
    pub question: String,
    pub active_from: Option<SqlDate>,
    pub active_until: Option<SqlDate>,
    pub deleted_at: Option<SqlTimestamp>,
}
