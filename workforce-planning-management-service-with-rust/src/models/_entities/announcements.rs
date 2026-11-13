//! Row type — `announcements`. A feed item for an organization: title, body, optionally pinned, scheduled or expiring.

#![allow(missing_docs)]

use chrono::{DateTime, FixedOffset, NaiveDate};
use serde::{Deserialize, Serialize};
use serde_json::Value;
use uuid::Uuid;

#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
pub struct Model {
    pub created_at: DateTime<FixedOffset>,
    pub updated_at: DateTime<FixedOffset>,
    pub id: i32,
    pub pid: Uuid,
    pub organization_ref: String,
    pub title: String,
    pub body: String,
    pub pinned: bool,
    pub publish_on: NaiveDate,
    pub expires_on: Option<NaiveDate>,
    pub department: Option<String>,
    pub links: Value,
    pub author: Option<String>,
    pub deleted_at: Option<DateTime<FixedOffset>>,
}
