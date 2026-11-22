//! Row type — `audit_logs`. One row per mutation or sensitive
//! read (who / what / when + a snapshot), across every record kind.

// A row type: the field-level shape is documented by the
// migration and the `audit_logs` table, not by per-field rustdoc.
#![expect(
    missing_docs,
    reason = "a generated row type: the module header names the table"
)]

use crate::sql_time::SqlTimestamp;
use serde::{Deserialize, Serialize};
use serde_json::Value;
use uuid::Uuid;

#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub struct Model {
    pub created_at: SqlTimestamp,
    pub updated_at: SqlTimestamp,
    pub id: i64,
    pub entity: String,
    pub entity_pid: Uuid,
    pub action: String,
    pub actor: Option<String>,
    pub snapshot: Option<Value>,
    /// Place in the hash chain (WPM-R115); assigned by the database.
    pub chain_seq: i64,
    /// The previous entry's hash (64 zeros for the first).
    pub prev_hash: String,
    /// The hash of this entry's content and `prev_hash`.
    pub entry_hash: String,
}
