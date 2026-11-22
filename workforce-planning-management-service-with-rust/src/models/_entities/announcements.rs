//! Row type — `announcements`. A feed item for an organization: title, body, optionally pinned, scheduled or expiring.

#![expect(
    missing_docs,
    reason = "a generated row type: the module header names the table"
)]

use crate::sql_time::{SqlDate, SqlTimestamp};
use serde::{Deserialize, Serialize};
use serde_json::Value;
use uuid::Uuid;

#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
pub struct Model {
    pub created_at: SqlTimestamp,
    pub updated_at: SqlTimestamp,
    pub id: i32,
    pub pid: Uuid,
    pub organization_ref: String,
    pub title: String,
    pub body: String,
    pub pinned: bool,
    pub publish_on: SqlDate,
    pub expires_on: Option<SqlDate>,
    pub department: Option<String>,
    pub links: Value,
    pub author: Option<String>,
    pub deleted_at: Option<SqlTimestamp>,
}
