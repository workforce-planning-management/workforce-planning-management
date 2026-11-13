//! Row type — `cutovers`. A programme's go-live at a site over a window of dates (WPM-R70).

#![allow(missing_docs)]

use chrono::{DateTime, FixedOffset, NaiveDate};
use serde::{Deserialize, Serialize};
use serde_json::Value;
use uuid::Uuid;

#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub struct Model {
    pub created_at: DateTime<FixedOffset>,
    pub updated_at: DateTime<FixedOffset>,
    pub id: i32,
    pub pid: Uuid,
    pub organization_ref: String,
    pub programme_ref: String,
    pub name: String,
    pub site: String,
    pub window_start: NaiveDate,
    pub window_end: NaiveDate,
    pub rota_pid: Option<Uuid>,
    pub site_aliases: Value,
}
