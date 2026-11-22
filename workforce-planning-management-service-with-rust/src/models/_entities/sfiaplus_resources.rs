//! Row type — `sfiaplus_resources`. A `SFIAplus` resource for a SFIA skill at a level (a pointer).

#![expect(
    missing_docs,
    reason = "a generated row type: the module header names the table"
)]

use crate::sql_time::SqlTimestamp;
use serde::{Deserialize, Serialize};

#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub struct Model {
    pub created_at: SqlTimestamp,
    pub updated_at: SqlTimestamp,
    pub id: i32,
    pub code: String,
    pub level: i32,
    pub kind: String,
    pub title: String,
    pub provider: Option<String>,
    pub url: Option<String>,
    pub retired_in: Option<String>,
}
