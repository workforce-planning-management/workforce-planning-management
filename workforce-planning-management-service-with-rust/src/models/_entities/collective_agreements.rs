//! Row type — `collective_agreements`. A dated, sourced, versioned agreement tied to one bargaining unit, with the terms WPM can
//! use (WPM-R103). Who it applies to is derived from the unit's rules and never stored (WPM-D68).

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
    pub unit_pid: Uuid,
    pub title: String,
    pub version: i32,
    pub effective_from: NaiveDate,
    pub effective_to: Option<NaiveDate>,
    pub source: String,
    pub pay_scale_id: Option<String>,
    pub max_weekly_hours: Option<i32>,
    pub notice_weeks: Option<i32>,
    pub consultation_days: Option<i32>,
    pub recorded_by: Option<String>,
}
