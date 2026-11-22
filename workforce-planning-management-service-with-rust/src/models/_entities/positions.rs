//! Row type — `positions`. A budgeted seat (WPM-R152, WPM-D104): cost centre, budget in FTE percent, funding kind and
//! state, with the holder, the requisition and the plan demand line as links. No pay.

#![expect(
    missing_docs,
    reason = "a generated row type: the module header names the table"
)]

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
    pub cost_centre: String,
    pub title: String,
    pub budget_fte_percent: i32,
    pub funding_kind: String,
    pub funding_ends_on: Option<SqlDate>,
    pub programme_ref: Option<String>,
    pub state: String,
    pub holder_worker_pid: Option<Uuid>,
    pub requisition_pid: Option<Uuid>,
    pub demand_line_pid: Option<Uuid>,
    pub overtime_status: String,
    pub number: Option<String>,
    pub recorded_by: Option<String>,
}
