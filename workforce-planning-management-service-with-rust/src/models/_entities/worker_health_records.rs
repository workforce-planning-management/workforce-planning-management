//! `SeaORM` Entity — `worker_health_records`. A worker's status against one requirement: a status token and two dates, never a reason (WPM-R128, WPM-D75). Health data.

#![allow(missing_docs)]

use sea_orm::entity::prelude::*;
use serde::{Deserialize, Serialize};

#[derive(Clone, Debug, PartialEq, DeriveEntityModel, Eq, Serialize, Deserialize)]
#[sea_orm(table_name = "worker_health_records")]
pub struct Model {
    pub created_at: DateTimeWithTimeZone,
    pub updated_at: DateTimeWithTimeZone,
    #[sea_orm(primary_key)]
    pub id: i32,
    pub worker_pid: Uuid,
    pub requirement_pid: Uuid,
    pub status: String,
    pub recorded_on: Date,
    pub next_due: Option<Date>,
    pub recorded_by: Option<String>,
}

#[derive(Copy, Clone, Debug, EnumIter, DeriveRelation)]
pub enum Relation {}

impl ActiveModelBehavior for ActiveModel {}
