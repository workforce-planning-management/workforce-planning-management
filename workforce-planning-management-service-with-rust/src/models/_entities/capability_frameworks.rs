//! Row type — `capability_frameworks`. A framework role profiles were
//! imported from, with its attribution and proficiency scale.

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
    pub slug: String,
    pub name: String,
    pub source_url: Option<String>,
    pub licence: Option<String>,
    pub attribution: Option<String>,
    pub scale_max: i32,
    pub scale_labels: Option<String>,
    pub imported_on: NaiveDate,
    pub note: Option<String>,
    pub version: Option<String>,
    pub redistribution: String,
    pub licence_holder: Option<String>,
    pub licence_reference: Option<String>,
    pub licence_checked_on: Option<NaiveDate>,
}
