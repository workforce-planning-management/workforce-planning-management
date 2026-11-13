//! Row type — `oversight_packs`. A monthly snapshot of the oversight position, criteria, measures and capacity view, frozen as of a date and never changed (WPM-R69). Aggregates only; no person appears.

#![allow(missing_docs)]

use chrono::{DateTime, FixedOffset, NaiveDate};
use serde::{Deserialize, Serialize};
use serde_json::Value;
use uuid::Uuid;

#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub struct Model {
    pub created_at: DateTime<FixedOffset>,
    pub updated_at: DateTime<FixedOffset>,
    pub id: i32,
    pub pid: Uuid,
    pub organization_ref: String,
    pub as_of: NaiveDate,
    pub month: NaiveDate,
    pub payload: Value,
}
