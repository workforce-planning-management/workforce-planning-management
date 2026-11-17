//! Row type — `oversight_pack_withdrawals`. A mark that a frozen pack was withdrawn, with the date and the reason; the pack itself is never changed (WPM-R69).

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
    pub pack_pid: Uuid,
    pub withdrawn_on: SqlDate,
    pub reason: String,
}
