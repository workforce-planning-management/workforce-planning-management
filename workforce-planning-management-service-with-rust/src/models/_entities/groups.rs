//! Row type — `groups`. A community of practice, community of interest, or other group.

#![expect(
    missing_docs,
    reason = "a generated row type: the module header names the table"
)]

use crate::sql_time::SqlTimestamp;
use serde::{Deserialize, Serialize};
use uuid::Uuid;

#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
pub struct Model {
    pub created_at: SqlTimestamp,
    pub updated_at: SqlTimestamp,
    pub id: i32,
    pub pid: Uuid,
    pub organization_ref: String,
    pub scope: String,
    pub name: String,
    pub kind: String,
    pub description: Option<String>,
    pub deleted_at: Option<SqlTimestamp>,
}
