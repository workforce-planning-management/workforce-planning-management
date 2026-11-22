//! Row type — `role_profiles`. What a role (keyed by job title) requires.

#![expect(
    missing_docs,
    reason = "a generated row type: the module header names the table"
)]

use crate::sql_time::SqlTimestamp;
use serde::{Deserialize, Serialize};
use uuid::Uuid;

#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
pub struct Model {
    pub created_at: SqlTimestamp,
    pub updated_at: SqlTimestamp,
    pub id: i32,
    pub pid: Uuid,
    pub job_title: String,
    pub description: Option<String>,
    pub source_ref: Option<String>,
    pub framework_slug: Option<String>,
    pub external_ref: Option<String>,
    pub profession: Option<String>,
    pub role_name: Option<String>,
    pub level_name: Option<String>,
    pub level_order: Option<i32>,
    pub management_track: bool,
    pub job_level_framework: Option<String>,
    pub job_level: Option<i32>,
    pub pay_scale_id: Option<String>,
    pub pay_band: Option<String>,
    pub deleted_at: Option<SqlTimestamp>,
}
