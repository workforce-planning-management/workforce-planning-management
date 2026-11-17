//! Row type — `succession_plans`. One critical role's succession plan (WPM-R12).

#![allow(missing_docs)]

use crate::sql_time::{SqlDate, SqlTimestamp};
use serde::{Deserialize, Serialize};
use uuid::Uuid;

#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
pub struct Model {
    pub created_at: SqlTimestamp,
    pub updated_at: SqlTimestamp,
    pub id: i32,
    pub pid: Uuid,
    pub role_title: String,
    pub department: String,
    pub criticality: i32,
    pub incumbent_pid: Option<Uuid>,
    /// How likely the incumbent is to leave: `low` | `medium` | `high`.
    pub risk_of_loss: Option<String>,
    /// When the role is expected to fall vacant, when that is known.
    pub vacancy_expected_on: Option<SqlDate>,
    pub deleted_at: Option<SqlTimestamp>,
}
