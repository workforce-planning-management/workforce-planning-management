//! Row type — `candidates`. One retained applicant profile, consent-bounded (WPM-R2, WPM-D8).

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
    pub person_ref: Option<String>,
    pub display_name: String,
    pub email: String,
    pub source: String,
    pub consent_until: Option<SqlDate>,
    pub deleted_at: Option<SqlTimestamp>,
}
