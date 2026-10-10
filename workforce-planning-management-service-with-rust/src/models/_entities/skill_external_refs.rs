//! `SeaORM` Entity — `skill_external_refs`. How a catalogue skill is known in
//! an external framework (PCF name, ESCO concept URI).

#![allow(missing_docs)]

use sea_orm::entity::prelude::*;
use serde::{Deserialize, Serialize};

#[derive(Clone, Debug, PartialEq, DeriveEntityModel, Serialize, Deserialize)]
#[sea_orm(table_name = "skill_external_refs")]
pub struct Model {
    pub created_at: DateTimeWithTimeZone,
    pub updated_at: DateTimeWithTimeZone,
    #[sea_orm(primary_key)]
    pub id: i32,
    #[sea_orm(unique)]
    pub pid: Uuid,
    pub skill_pid: Uuid,
    pub framework_slug: String,
    #[sea_orm(column_name = "ref")]
    pub reference: String,
    pub label: Option<String>,
    pub version: Option<String>,
}

#[derive(Copy, Clone, Debug, EnumIter, DeriveRelation)]
pub enum Relation {}

impl ActiveModelBehavior for ActiveModel {}
