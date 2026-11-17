//! Row type — `idempotency_keys`. A caller's key, the fingerprint of the request that claimed it and the response to replay; kept for one calendar day (WPM-R132).

#![allow(missing_docs)]

use crate::sql_time::SqlTimestamp;
use serde::{Deserialize, Serialize};

#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub struct Model {
    pub created_at: SqlTimestamp,
    pub updated_at: SqlTimestamp,
    pub id: i32,
    pub actor: String,
    pub idem_key: String,
    pub fingerprint: String,
    pub status_code: Option<i32>,
    pub content_type: Option<String>,
    #[serde(skip)]
    pub response_body: Option<Vec<u8>>,
}
