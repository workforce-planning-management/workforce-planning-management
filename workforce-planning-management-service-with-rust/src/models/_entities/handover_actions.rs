//! `SeaORM` Entity — `handover_actions`. The audit trail of a leaver handover: what was held and where it went.

#![allow(missing_docs)]

use sea_orm::entity::prelude::*;
use serde::{Deserialize, Serialize};

#[derive(Clone, Debug, PartialEq, DeriveEntityModel, Serialize, Deserialize)]
#[sea_orm(table_name = "handover_actions")]
pub struct Model {
    #[sea_orm(primary_key)]
    pub id: i32,
    #[sea_orm(unique)]
    pub pid: Uuid,
    pub movement_pid: Uuid,
    pub kind: String,
    pub subject_pid: Uuid,
    pub subject_label: Option<String>,
    pub from_worker: Uuid,
    pub to_worker: Option<Uuid>,
    pub action: String,
    pub note: Option<String>,
    pub performed_by: Option<String>,
    pub performed_at: DateTimeWithTimeZone,
}

#[derive(Copy, Clone, Debug, EnumIter, DeriveRelation)]
pub enum Relation {}

impl ActiveModelBehavior for ActiveModel {}
