//! Row type — `professional_registrations`. A registration with an expiry.

#![allow(missing_docs)]

use crate::sql_time::{SqlDate, SqlTimestamp};
use serde::{Deserialize, Serialize};
use uuid::Uuid;

#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
pub struct Model {
    pub created_at: SqlTimestamp,
    pub updated_at: SqlTimestamp,
    pub id: i32,
    pub pid: Uuid,
    pub worker_pid: Uuid,
    pub body: String,
    pub reference: Option<String>,
    pub registered_on: Option<SqlDate>,
    pub expires_on: Option<SqlDate>,
    pub deleted_at: Option<SqlTimestamp>,
}
