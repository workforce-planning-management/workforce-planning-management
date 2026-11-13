//! Row type — `equality_declarations`. A worker's own voluntary answer to one monitored category (WPM-R125). Special-category data: readable by the worker only.

#![allow(missing_docs)]

use chrono::{DateTime, FixedOffset};
use serde::{Deserialize, Serialize};
use uuid::Uuid;

#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub struct Model {
    pub created_at: DateTime<FixedOffset>,
    pub updated_at: DateTime<FixedOffset>,
    pub id: i32,
    pub worker_pid: Uuid,
    pub category: String,
    pub value: String,
}
