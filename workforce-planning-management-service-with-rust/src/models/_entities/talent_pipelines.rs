//! Row type — `talent_pipelines`. A named pool of people being grown toward a target (WPM-R22).

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
    pub name: String,
    pub purpose: String,
    pub target_job_title: Option<String>,
    pub target_department: Option<String>,
    pub deleted_at: Option<SqlTimestamp>,
}
