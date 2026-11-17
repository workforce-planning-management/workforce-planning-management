//! Row type — `esco_skills`. A pinned ESCO skill concept.

#![allow(missing_docs)]

use crate::sql_time::SqlTimestamp;
use serde::{Deserialize, Serialize};

#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
pub struct Model {
    pub created_at: SqlTimestamp,
    pub updated_at: SqlTimestamp,
    pub id: i32,
    pub uri: String,
    pub label: String,
    pub skill_type: Option<String>,
    pub reuse_level: Option<String>,
}
