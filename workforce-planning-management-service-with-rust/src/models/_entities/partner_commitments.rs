//! Row type — `partner_commitments`. A partner organization's aggregate capacity for a pool in a month (WPM-R58).

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
    pub partner_ref: String,
    pub pool_pid: Uuid,
    pub month: SqlDate,
    pub fte_centi: i64,
    pub status: String,
    pub deleted_at: Option<SqlTimestamp>,
}
