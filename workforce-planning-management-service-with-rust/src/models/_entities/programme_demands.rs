//! Row type — `programme_demands`. A programme's claim on a pool for one month, in hundredths of an FTE (WPM-R57).

#![allow(missing_docs)]

use chrono::{DateTime, FixedOffset, NaiveDate};
use serde::{Deserialize, Serialize};
use uuid::Uuid;

#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub struct Model {
    pub created_at: DateTime<FixedOffset>,
    pub updated_at: DateTime<FixedOffset>,
    pub id: i32,
    pub pid: Uuid,
    pub programme_ref: String,
    pub pool_pid: Uuid,
    pub month: NaiveDate,
    pub fte_centi: i64,
    pub status: String,
    pub source: String,
    pub peer_event_id: Option<String>,
    pub received_at: Option<DateTime<FixedOffset>>,
    pub peer_sent_at: Option<DateTime<FixedOffset>>,
    pub peer_withdrawn_at: Option<DateTime<FixedOffset>>,
    pub deleted_at: Option<DateTime<FixedOffset>>,
}
