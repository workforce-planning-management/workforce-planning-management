//! Row type — `worker_contractor_details`. A contractor's supplier, route and rate (WPM-R80). The rate is masked like salary.

#![allow(missing_docs)]

use crate::sql_time::SqlTimestamp;
use serde::{Deserialize, Serialize};
use uuid::Uuid;

#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub struct Model {
    pub created_at: SqlTimestamp,
    pub updated_at: SqlTimestamp,
    pub id: i32,
    pub worker_pid: Uuid,
    pub supplier_ref: Option<String>,
    pub supplier_contact_ref: Option<String>,
    pub route: Option<String>,
    pub rate_minor: Option<i64>,
    pub rate_currency: Option<String>,
    pub rate_basis: Option<String>,
    pub recorded_by: Option<String>,
}
