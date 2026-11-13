//! Row type — `worker_pay_positions`. A worker's band and step on a reference pay scale (WPM-R54).

#![allow(missing_docs)]

use chrono::{DateTime, FixedOffset, NaiveDate};
use serde::{Deserialize, Serialize};
use uuid::Uuid;

#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
pub struct Model {
    pub created_at: DateTime<FixedOffset>,
    pub updated_at: DateTime<FixedOffset>,
    pub id: i32,
    pub worker_pid: Uuid,
    pub scale_id: String,
    pub band: String,
    pub step: i32,
    pub step_since: NaiveDate,
    pub recorded_by: Option<String>,
    pub on_behalf: bool,
}
