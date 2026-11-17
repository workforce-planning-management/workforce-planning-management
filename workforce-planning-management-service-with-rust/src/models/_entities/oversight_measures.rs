//! Row type — `oversight_measures`. A measure behind a criterion: threshold, direction and sustain rule (WPM-R66).

#![allow(missing_docs)]

use crate::sql_time::SqlTimestamp;
use serde::{Deserialize, Serialize};
use uuid::Uuid;

#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub struct Model {
    pub created_at: SqlTimestamp,
    pub updated_at: SqlTimestamp,
    pub id: i32,
    pub pid: Uuid,
    pub criterion_pid: Uuid,
    pub name: String,
    pub origin: String,
    pub derivation: Option<String>,
    pub unit: String,
    pub threshold: i64,
    pub direction: String,
    pub sustain: i32,
}
