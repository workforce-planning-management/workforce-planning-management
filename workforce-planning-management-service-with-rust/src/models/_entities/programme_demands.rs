//! Row type — `programme_demands`. A programme's claim on a pool for one month, in hundredths of an FTE (WPM-R57).

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
    pub programme_ref: String,
    pub pool_pid: Uuid,
    pub month: SqlDate,
    pub fte_centi: i64,
    pub status: String,
    pub source: String,
    pub peer_event_id: Option<String>,
    pub received_at: Option<SqlTimestamp>,
    pub peer_sent_at: Option<SqlTimestamp>,
    pub peer_withdrawn_at: Option<SqlTimestamp>,
    pub deleted_at: Option<SqlTimestamp>,
}
