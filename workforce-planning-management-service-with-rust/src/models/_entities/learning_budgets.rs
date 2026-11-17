//! Row type — `learning_budgets`. A learning budget for one department, year and currency (WPM-R227, WPM-T451). Names nobody.

#![allow(missing_docs)]

use crate::sql_time::SqlTimestamp;
use serde::{Deserialize, Serialize};
use uuid::Uuid;

#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub struct Model {
    pub created_at: SqlTimestamp,
    pub updated_at: SqlTimestamp,
    pub id: i32,
    pub pid: Uuid,
    pub organization_ref: String,
    pub department: String,
    pub year: i32,
    pub amount_minor: i64,
    pub currency: String,
    pub recorded_by: Option<String>,
}
