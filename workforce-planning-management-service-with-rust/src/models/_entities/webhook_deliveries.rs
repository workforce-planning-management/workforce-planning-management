//! Row type — `webhook_deliveries`. One event for one subscription: its kind, the URN it is about and how the attempts
//! went (WPM-R151). No payload body. For a hire or an ending only, `data` keeps the handoff the event carried (a `person:` and an
//! `organization:` reference, a date, and the status change), so the payload can say who (proposed WPM-D145).

#![allow(missing_docs)]

use chrono::{DateTime, FixedOffset};
use serde::{Deserialize, Serialize};
use serde_json::Value;
use uuid::Uuid;

#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub struct Model {
    pub created_at: DateTime<FixedOffset>,
    pub updated_at: DateTime<FixedOffset>,
    pub id: i32,
    pub pid: Uuid,
    pub subscription_pid: Uuid,
    pub event_id: Uuid,
    pub kind: String,
    pub subject: String,
    pub data: Option<Value>,
    pub occurred_at: DateTime<FixedOffset>,
    pub status: String,
    pub attempts: i32,
    pub last_status: Option<i32>,
    pub last_error: Option<String>,
    pub next_attempt_at: Option<DateTime<FixedOffset>>,
    pub delivered_at: Option<DateTime<FixedOffset>>,
}
