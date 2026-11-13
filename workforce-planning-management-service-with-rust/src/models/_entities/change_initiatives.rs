//! Row type — `change_initiatives`. An automation / AI / process change.

#![allow(missing_docs)]

use chrono::{DateTime, FixedOffset, NaiveDate};
use serde::{Deserialize, Serialize};
use uuid::Uuid;

#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
pub struct Model {
    pub created_at: DateTime<FixedOffset>,
    pub updated_at: DateTime<FixedOffset>,
    pub id: i32,
    pub pid: Uuid,
    pub name: String,
    pub description: Option<String>,
    pub kind: String,
    pub status: String,
    pub starts_on: Option<NaiveDate>,
    pub deleted_at: Option<DateTime<FixedOffset>>,
}
