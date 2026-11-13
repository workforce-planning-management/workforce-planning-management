//! Row type — `ergonomic_items`. One DSE checklist item (WPM-R32): `ok`/`issue` + an **equipment** note (WPM-D24 — no symptom field exists).

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
    pub assessment_pid: Uuid,
    pub name: String,
    /// The default item's stable key (WPM-T386); `None` for a custom item.
    pub item_key: Option<String>,
    pub ok: Option<bool>,
    pub note: Option<String>,
    pub deleted_at: Option<DateTime<FixedOffset>>,
}
