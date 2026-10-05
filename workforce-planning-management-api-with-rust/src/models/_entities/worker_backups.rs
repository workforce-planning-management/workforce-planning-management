//! `SeaORM` Entity — `worker_backups`. A colleague who covers for a worker when they are out, ranked, optionally for a dated window.

#![allow(missing_docs)]

use sea_orm::entity::prelude::*;
use serde::{Deserialize, Serialize};

#[derive(Clone, Debug, PartialEq, DeriveEntityModel, Serialize, Deserialize)]
#[sea_orm(table_name = "worker_backups")]
pub struct Model {
    pub created_at: DateTimeWithTimeZone,
    pub updated_at: DateTimeWithTimeZone,
    #[sea_orm(primary_key)]
    pub id: i32,
    #[sea_orm(unique)]
    pub pid: Uuid,
    pub worker_pid: Uuid,
    pub backup_pid: Uuid,
    pub priority: i32,
    pub starts_on: Option<Date>,
    pub ends_on: Option<Date>,
    pub note: Option<String>,
    pub recorded_by: Option<String>,
    pub on_behalf: bool,
    pub deleted_at: Option<DateTimeWithTimeZone>,
}

#[derive(Copy, Clone, Debug, EnumIter, DeriveRelation)]
pub enum Relation {}

impl ActiveModelBehavior for ActiveModel {}
