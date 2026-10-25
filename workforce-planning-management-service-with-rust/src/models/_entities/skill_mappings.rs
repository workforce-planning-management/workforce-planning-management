//! `SeaORM` Entity — `skill_mappings`. A skill declared under one record counts as another, with its confidence and basis (WPM-R158).

#![allow(missing_docs)]

use sea_orm::entity::prelude::*;
use serde::{Deserialize, Serialize};

#[derive(Clone, Debug, PartialEq, DeriveEntityModel, Eq, Serialize, Deserialize)]
#[sea_orm(table_name = "skill_mappings")]
pub struct Model {
    pub created_at: DateTimeWithTimeZone,
    pub updated_at: DateTimeWithTimeZone,
    #[sea_orm(primary_key)]
    pub id: i32,
    #[sea_orm(unique)]
    pub pid: Uuid,
    pub from_skill_pid: Uuid,
    pub to_skill_pid: Uuid,
    pub confidence: String,
    pub basis: String,
    pub source: String,
    pub recorded_by: Option<String>,
    pub recorded_on: Date,
}

#[derive(Copy, Clone, Debug, EnumIter, DeriveRelation)]
pub enum Relation {}

impl ActiveModelBehavior for ActiveModel {}
