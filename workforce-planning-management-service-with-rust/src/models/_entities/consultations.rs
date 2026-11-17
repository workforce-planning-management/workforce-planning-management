//! Row type — `consultations`. The consultation register (WPM-R105): a proposal as counts and dates, the body consulted, topics, a
//! document reference, dates and outcome. No attendee, representative or minute (WPM-D68).

#![allow(missing_docs)]

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
    pub body_pid: Uuid,
    pub plan_pid: Option<Uuid>,
    pub establishment: String,
    pub profile: String,
    pub establishment_size: i32,
    pub dismissals: i32,
    pub earliest_termination_on: SqlDate,
    pub topics: String,
    pub document_ref: Option<String>,
    pub started_on: Option<SqlDate>,
    pub ended_on: Option<SqlDate>,
    pub outcome: String,
    pub agreement_pid: Option<Uuid>,
    pub recorded_by: Option<String>,
}
