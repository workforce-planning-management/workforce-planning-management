//! `SeaORM` Entity — `capability_frameworks`. A framework role profiles were
//! imported from, with its attribution and proficiency scale.

#![allow(missing_docs)]

use sea_orm::entity::prelude::*;
use serde::{Deserialize, Serialize};

#[derive(Clone, Debug, PartialEq, DeriveEntityModel, Serialize, Deserialize)]
#[sea_orm(table_name = "capability_frameworks")]
pub struct Model {
    pub created_at: DateTimeWithTimeZone,
    pub updated_at: DateTimeWithTimeZone,
    #[sea_orm(primary_key)]
    pub id: i32,
    #[sea_orm(unique)]
    pub pid: Uuid,
    #[sea_orm(unique)]
    pub slug: String,
    pub name: String,
    pub source_url: Option<String>,
    pub licence: Option<String>,
    pub attribution: Option<String>,
    pub scale_max: i32,
    pub scale_labels: Option<String>,
    pub imported_on: Date,
    pub note: Option<String>,
}

#[derive(Copy, Clone, Debug, EnumIter, DeriveRelation)]
pub enum Relation {}

impl ActiveModelBehavior for ActiveModel {}
