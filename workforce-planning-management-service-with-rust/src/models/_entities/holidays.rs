//! Row type — `holidays`. A public holiday for an organization and region (WPM-R5): the date a rule gives, the day it is
//! taken, and whether it was generated or added by hand.

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
    pub region: String,
    pub year: i32,
    pub name: String,
    pub natural_on: NaiveDate,
    pub observed_on: NaiveDate,
    pub source: String,
    pub recorded_by: Option<String>,
}
