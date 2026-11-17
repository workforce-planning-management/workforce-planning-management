//! Row type — `benefit_plans`. One benefit plan with minor-unit costs (WPM-R9).

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
    pub name: String,
    pub kind: String,
    pub provider: String,
    pub worker_cost_minor: i64,
    pub employer_cost_minor: i64,
    pub currency: String,
    pub deleted_at: Option<SqlTimestamp>,
}
