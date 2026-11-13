//! Row type — `worker_aspirations`. A future role or skill target: an aspiration, learning goal, or growth idea.

#![allow(missing_docs)]

use chrono::{DateTime, FixedOffset};
use serde::{Deserialize, Serialize};
use uuid::Uuid;

#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
pub struct Model {
    pub created_at: DateTime<FixedOffset>,
    pub updated_at: DateTime<FixedOffset>,
    pub id: i32,
    pub pid: Uuid,
    pub worker_pid: Uuid,
    pub kind: String,
    pub framework_slug: Option<String>,
    pub role_profile_pid: Option<Uuid>,
    pub esco_occupation_uri: Option<String>,
    pub role_label: Option<String>,
    pub skill_pid: Option<Uuid>,
    pub target_level: Option<i32>,
    pub horizon: String,
    pub status: String,
    pub note: Option<String>,
    pub visibility: String,
    pub recorded_by: Option<String>,
    pub on_behalf: bool,
    pub deleted_at: Option<DateTime<FixedOffset>>,
}
