//! `SeaORM` Entity — `role_profiles`. What a role (keyed by job title) requires.

#![allow(missing_docs)]

use sea_orm::entity::prelude::*;
use serde::{Deserialize, Serialize};

#[derive(Clone, Debug, PartialEq, DeriveEntityModel, Serialize, Deserialize)]
#[sea_orm(table_name = "role_profiles")]
pub struct Model {
    pub created_at: DateTimeWithTimeZone,
    pub updated_at: DateTimeWithTimeZone,
    #[sea_orm(primary_key)]
    pub id: i32,
    #[sea_orm(unique)]
    pub pid: Uuid,
    pub job_title: String,
    pub description: Option<String>,
    pub source_ref: Option<String>,
    pub framework_slug: Option<String>,
    pub external_ref: Option<String>,
    pub profession: Option<String>,
    pub role_name: Option<String>,
    pub level_name: Option<String>,
    pub level_order: Option<i32>,
    pub management_track: bool,
    pub deleted_at: Option<DateTimeWithTimeZone>,
}

#[derive(Copy, Clone, Debug, EnumIter, DeriveRelation)]
pub enum Relation {}

impl ActiveModelBehavior for ActiveModel {}
