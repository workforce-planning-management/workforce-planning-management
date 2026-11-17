//! Row type — `handover_actions`. The audit trail of a leaver handover: what was held and where it went.

#![allow(missing_docs)]

use crate::sql_time::SqlTimestamp;
use serde::{Deserialize, Serialize};
use uuid::Uuid;

#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
pub struct Model {
    pub id: i32,
    pub pid: Uuid,
    pub movement_pid: Uuid,
    pub kind: String,
    pub subject_pid: Uuid,
    pub subject_label: Option<String>,
    pub from_worker: Uuid,
    pub to_worker: Option<Uuid>,
    pub action: String,
    pub note: Option<String>,
    pub performed_by: Option<String>,
    pub performed_at: SqlTimestamp,
}
