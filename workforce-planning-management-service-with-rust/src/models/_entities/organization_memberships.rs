//! Row type — `organization_memberships`. One person's access
//! to one organization, with a role — covers both employment-linked
//! membership (`worker_pid` set) and staff/admin-only privilege grants
//! (`worker_pid` null) in one table.

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
    pub person_ref: String,
    pub organization_ref: String,
    pub worker_pid: Option<Uuid>,
    pub role: String,
    pub starts_on: NaiveDate,
    pub ends_on: Option<NaiveDate>,
    pub deleted_at: Option<DateTime<FixedOffset>>,
}
