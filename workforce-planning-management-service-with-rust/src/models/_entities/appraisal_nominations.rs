//! Row type — `appraisal_nominations`. One invited rater on one 360° appraisal (WPM-R29): who, and in which group (`self | manager | peer | report`).

#![expect(
    missing_docs,
    reason = "a generated row type: the module header names the table"
)]

use crate::sql_time::SqlTimestamp;
use serde::{Deserialize, Serialize};
use uuid::Uuid;

#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub struct Model {
    pub created_at: SqlTimestamp,
    pub updated_at: SqlTimestamp,
    pub id: i32,
    pub pid: Uuid,
    pub appraisal_pid: Uuid,
    pub rater_pid: Uuid,
    pub rater_group: String,
}
