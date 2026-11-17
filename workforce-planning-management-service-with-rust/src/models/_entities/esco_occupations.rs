//! Row type — `esco_occupations`. A pinned ESCO occupation concept.

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
    pub isco_code: Option<String>,
    pub description: Option<String>,
}
