//! Row type — `webhook_subscriptions`. An endpoint, the event kinds it wants and the name of the environment variable
//! holding its signing secret (WPM-R151). The secret itself is never stored.

#![expect(
    missing_docs,
    reason = "a generated row type: the module header names the table"
)]

use crate::sql_time::SqlTimestamp;
use serde::{Deserialize, Serialize};
use uuid::Uuid;

#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub struct Model {
    pub created_at: SqlTimestamp,
    pub updated_at: SqlTimestamp,
    pub id: i32,
    pub pid: Uuid,
    pub name: String,
    pub url: String,
    pub kinds: Vec<String>,
    pub secret_ref: String,
    pub active: bool,
    pub created_by: Option<String>,
}
