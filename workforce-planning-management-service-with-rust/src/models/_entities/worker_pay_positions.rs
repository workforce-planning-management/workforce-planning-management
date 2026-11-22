//! Row type — `worker_pay_positions`. A worker's band and step on a reference pay scale (WPM-R54).

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
    pub worker_pid: Uuid,
    pub scale_id: String,
    pub band: String,
    pub step: i32,
    pub step_since: SqlDate,
    pub recorded_by: Option<String>,
    pub on_behalf: bool,
}
