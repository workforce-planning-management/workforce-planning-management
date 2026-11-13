//! Row type — `candidates`. One retained applicant profile, consent-bounded (WPM-R2, WPM-D8).

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
    pub person_ref: Option<String>,
    pub display_name: String,
    pub email: String,
    pub source: String,
    pub consent_until: Option<NaiveDate>,
    pub deleted_at: Option<DateTime<FixedOffset>>,
}
