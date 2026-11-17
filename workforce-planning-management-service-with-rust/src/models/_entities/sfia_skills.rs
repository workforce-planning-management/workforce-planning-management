//! Row type — `sfia_skills`. A skill from the deployer's licensed SFIA download.

#![allow(missing_docs)]

use crate::sql_time::SqlTimestamp;
use serde::{Deserialize, Serialize};

#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub struct Model {
    pub created_at: SqlTimestamp,
    pub updated_at: SqlTimestamp,
    pub id: i32,
    pub code: String,
    pub name: String,
    pub category: String,
    pub subcategory: String,
    pub description: String,
    pub retired_in: Option<String>,
}
