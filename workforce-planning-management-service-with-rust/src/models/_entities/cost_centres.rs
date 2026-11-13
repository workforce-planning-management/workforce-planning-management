//! Row type — `cost_centres`. An organization's register of cost centres (WPM-R163): a code, a name, an optional parent and the
//! days it is valid. Closed, never deleted.

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
    pub organization_ref: String,
    pub code: String,
    pub name: String,
    pub parent_code: Option<String>,
    pub valid_from: NaiveDate,
    pub valid_to: Option<NaiveDate>,
    pub recorded_by: Option<String>,
}
