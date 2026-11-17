//! Row type — `sfia_skill_levels`. A level a SFIA skill is defined at.

#![allow(missing_docs)]

use crate::sql_time::SqlTimestamp;
use serde::{Deserialize, Serialize};

#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub struct Model {
    pub created_at: SqlTimestamp,
    pub updated_at: SqlTimestamp,
    pub id: i32,
    pub code: String,
    pub level: i32,
    pub description: String,
    pub attributes: Option<String>,
    pub retired_in: Option<String>,
}
