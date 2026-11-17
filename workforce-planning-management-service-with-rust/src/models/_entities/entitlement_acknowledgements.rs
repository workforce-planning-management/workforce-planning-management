//! Row type — `entitlement_acknowledgements`. One worker's response to a wellbeing prompt (WPM-R25): an HR workflow fact (`booked | done | declined | dismissed`), never a vaccination status (WPM-D17).

#![allow(missing_docs)]

use crate::sql_time::{SqlDate, SqlTimestamp};
use serde::{Deserialize, Serialize};
use uuid::Uuid;

#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub struct Model {
    pub created_at: SqlTimestamp,
    pub updated_at: SqlTimestamp,
    pub id: i32,
    pub pid: Uuid,
    pub entitlement_pid: Uuid,
    pub worker_pid: Uuid,
    pub response: String,
    pub responded_on: SqlDate,
    pub reminded_on: Option<SqlDate>,
}
