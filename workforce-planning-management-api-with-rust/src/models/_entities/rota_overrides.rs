//! `SeaORM` Entity — `rota_overrides`. A swap: a worker on call for a date window regardless of the rotation.

#![allow(missing_docs)]

use sea_orm::entity::prelude::*;
use serde::{Deserialize, Serialize};

#[derive(Clone, Debug, PartialEq, DeriveEntityModel, Serialize, Deserialize)]
#[sea_orm(table_name = "rota_overrides")]
pub struct Model {
    pub created_at: DateTimeWithTimeZone,
    #[sea_orm(primary_key)]
    pub id: i32,
    #[sea_orm(unique)]
    pub pid: Uuid,
    pub rota_pid: Uuid,
    pub worker_pid: Uuid,
    pub starts_on: Date,
    pub ends_on: Date,
    pub note: Option<String>,
    pub created_by: Option<String>,
}

#[derive(Copy, Clone, Debug, EnumIter, DeriveRelation)]
pub enum Relation {}

impl ActiveModelBehavior for ActiveModel {}
