//! Row type — `rota_members`. One member of a rota, in rotation order.

#![allow(missing_docs)]

use serde::{Deserialize, Serialize};
use uuid::Uuid;

#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
pub struct Model {
    pub id: i32,
    pub rota_pid: Uuid,
    pub worker_pid: Uuid,
    pub position: i32,
}
