//! Row type — `pulse_surveys`. One anonymous pulse survey (WPM-R28): a name, one question, an active window.

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
    pub name: String,
    pub question: String,
    pub active_from: Option<NaiveDate>,
    pub active_until: Option<NaiveDate>,
    pub deleted_at: Option<DateTime<FixedOffset>>,
}
