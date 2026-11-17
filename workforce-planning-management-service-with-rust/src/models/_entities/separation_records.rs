//! Row type — `separation_records`. What a leaver's separation record adds to the leaver record: the final pay date and a reference to the pay summary (WPM-R222).

#![allow(missing_docs)]

use crate::sql_time::{SqlDate, SqlTimestamp};
use serde::{Deserialize, Serialize};
use uuid::Uuid;

#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub struct Model {
    pub created_at: SqlTimestamp,
    pub updated_at: SqlTimestamp,
    pub id: i32,
    pub pid: Uuid,
    pub movement_pid: Uuid,
    pub worker_pid: Uuid,
    pub final_pay_on: Option<SqlDate>,
    pub pay_summary_ref: Option<String>,
    pub recorded_by: Option<String>,
}
