//! Row type — `worker_framework_roles`. The role a person says they
//! hold now in an external framework (a PCF role level or an ESCO occupation).

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
    pub worker_pid: Uuid,
    pub framework_slug: String,
    pub role_profile_pid: Option<Uuid>,
    pub esco_occupation_uri: Option<String>,
    pub role_label: String,
    pub selected_on: NaiveDate,
    pub started_at: DateTime<FixedOffset>,
    pub ended_at: Option<DateTime<FixedOffset>>,
    pub recorded_by: Option<String>,
    pub on_behalf: bool,
}
