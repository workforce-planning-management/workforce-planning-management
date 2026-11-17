//! Row type — `applications`. One candidate's application to one requisition (WPM-R2).

#![allow(missing_docs)]

use crate::sql_time::SqlTimestamp;
use serde::{Deserialize, Serialize};
use uuid::Uuid;

#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
pub struct Model {
    pub created_at: SqlTimestamp,
    pub updated_at: SqlTimestamp,
    pub id: i32,
    pub pid: Uuid,
    pub requisition_pid: Uuid,
    pub candidate_pid: Uuid,
    pub stage: String,
    pub notes: Option<String>,
    pub deleted_at: Option<SqlTimestamp>,
}
