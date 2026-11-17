//! Row type — `rotas`. A named on-call rotation in one organization.

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
    pub organization_ref: String,
    pub name: String,
    pub description: Option<String>,
    pub period_days: i32,
    pub starts_on: SqlDate,
    pub deleted_at: Option<SqlTimestamp>,
}
