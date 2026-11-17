//! Row type — `cost_centres`. An organization's register of cost centres (WPM-R163): a code, a name, an optional parent and the
//! days it is valid. Closed, never deleted.

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
    pub organization_ref: String,
    pub code: String,
    pub name: String,
    pub parent_code: Option<String>,
    pub valid_from: SqlDate,
    pub valid_to: Option<SqlDate>,
    pub recorded_by: Option<String>,
}
