//! Row type — `rota_members`. One member of a rota, in rotation order.

#![expect(
    missing_docs,
    reason = "a generated row type: the module header names the table"
)]

use serde::{Deserialize, Serialize};
use uuid::Uuid;

#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
pub struct Model {
    pub id: i32,
    pub rota_pid: Uuid,
    pub worker_pid: Uuid,
    pub position: i32,
}
