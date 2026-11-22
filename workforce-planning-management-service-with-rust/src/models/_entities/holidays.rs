//! Row type — `holidays`. A public holiday for an organization and region (WPM-R5): the date a rule gives, the day it is
//! taken, and whether it was generated or added by hand.

#![expect(
    missing_docs,
    reason = "a generated row type: the module header names the table"
)]

use crate::sql_time::{SqlDate, SqlTimestamp};
use serde::{Deserialize, Serialize};
use uuid::Uuid;

#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub struct Model {
    pub created_at: SqlTimestamp,
    pub updated_at: SqlTimestamp,
    pub id: i32,
    pub pid: Uuid,
    pub organization_ref: String,
    pub region: String,
    pub year: i32,
    pub name: String,
    pub natural_on: SqlDate,
    pub observed_on: SqlDate,
    pub source: String,
    pub recorded_by: Option<String>,
}
