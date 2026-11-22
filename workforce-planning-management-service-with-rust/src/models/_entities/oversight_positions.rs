//! Row type — `oversight_positions`. A recorded position: the level, when it took effect, the deciding body (free text) and the published reason (WPM-R65).

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
    pub organization_ref: String,
    pub level_pid: Uuid,
    pub effective_on: SqlDate,
    pub deciding_body: String,
    pub reason: String,
    pub recorded_by: Option<String>,
}
