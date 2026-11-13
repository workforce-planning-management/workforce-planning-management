//! Row type — `requisitions`. One funded job opening with its hiring pipeline (WPM-R1).

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
    pub organization_ref: String,
    pub department: String,
    pub job_title: String,
    pub headcount: i32,
    pub salary_min_minor: Option<i64>,
    pub salary_max_minor: Option<i64>,
    pub salary_currency: Option<String>,
    pub status: String,
    pub opened_on: Option<NaiveDate>,
    pub filled_on: Option<NaiveDate>,
    pub deleted_at: Option<DateTime<FixedOffset>>,
    /// The programme the post serves (`thing:` URN), if any.
    pub programme_ref: Option<String>,
    /// The engagement basis of the post.
    pub basis: String,
    /// How long a basis that ends is expected to last, in calendar months.
    pub expected_duration_months: Option<i32>,
}
