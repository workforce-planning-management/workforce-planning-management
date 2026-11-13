//! Row type — `appraisal_nominations`. One invited rater on one 360° appraisal (WPM-R29): who, and in which group (`self | manager | peer | report`).

#![allow(missing_docs)]

use chrono::{DateTime, FixedOffset};
use serde::{Deserialize, Serialize};
use uuid::Uuid;

#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub struct Model {
    pub created_at: DateTime<FixedOffset>,
    pub updated_at: DateTime<FixedOffset>,
    pub id: i32,
    pub pid: Uuid,
    pub appraisal_pid: Uuid,
    pub rater_pid: Uuid,
    pub rater_group: String,
}
