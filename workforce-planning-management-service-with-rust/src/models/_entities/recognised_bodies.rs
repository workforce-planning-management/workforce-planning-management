//! Row type — `recognised_bodies`. A union, works council or staff forum the organization recognises (WPM-R102): a name, a kind, a
//! basis, a date and a contact role. No member, representative or contact person is held (WPM-D68).

#![expect(
    missing_docs,
    reason = "a generated row type: the module header names the table"
)]

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
    pub name: String,
    pub kind: String,
    pub basis: String,
    pub recognised_on: SqlDate,
    pub contact_role: String,
    pub recorded_by: Option<String>,
}
