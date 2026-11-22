//! Row type — `cutovers`. A programme's go-live at a site over a window of dates (WPM-R70).

#![expect(
    missing_docs,
    reason = "a generated row type: the module header names the table"
)]

use crate::sql_time::{SqlDate, SqlTimestamp};
use serde::{Deserialize, Serialize};
use serde_json::Value;
use uuid::Uuid;

#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub struct Model {
    pub created_at: SqlTimestamp,
    pub updated_at: SqlTimestamp,
    pub id: i32,
    pub pid: Uuid,
    pub organization_ref: String,
    pub programme_ref: String,
    pub name: String,
    pub site: String,
    pub window_start: SqlDate,
    pub window_end: SqlDate,
    pub rota_pid: Option<Uuid>,
    pub site_aliases: Value,
}
