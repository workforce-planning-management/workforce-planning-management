//! Row type — `bargaining_units`. The part of the workforce a recognised body covers, as rules over facts WPM holds (WPM-R102).
//! Who is covered is computed and counted with a floor; it is never stored (WPM-D68).

#![allow(missing_docs)]

use crate::sql_time::SqlTimestamp;
use serde::{Deserialize, Serialize};
use uuid::Uuid;

#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub struct Model {
    pub created_at: SqlTimestamp,
    pub updated_at: SqlTimestamp,
    pub id: i32,
    pub pid: Uuid,
    pub body_pid: Uuid,
    pub name: String,
    pub department: Option<String>,
    pub location: Option<String>,
    pub employment_type: Option<String>,
    pub job_title: Option<String>,
    pub recorded_by: Option<String>,
}
