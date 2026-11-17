//! Row type — `announcement_reads`. That a worker has read an announcement (the reader sees their own; editors see only a count).

#![allow(missing_docs)]

use crate::sql_time::SqlTimestamp;
use serde::{Deserialize, Serialize};
use uuid::Uuid;

#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
pub struct Model {
    pub id: i32,
    pub announcement_pid: Uuid,
    pub worker_pid: Uuid,
    pub read_at: SqlTimestamp,
}
