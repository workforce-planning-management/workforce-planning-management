//! `SeaORM` Entity — `sfiaplus_resources`. A `SFIAplus` resource for a SFIA skill at a level (a pointer).

#![allow(missing_docs)]

use sea_orm::entity::prelude::*;
use serde::{Deserialize, Serialize};

#[derive(Clone, Debug, PartialEq, Eq, DeriveEntityModel, Serialize, Deserialize)]
#[sea_orm(table_name = "sfiaplus_resources")]
pub struct Model {
    pub created_at: DateTimeWithTimeZone,
    pub updated_at: DateTimeWithTimeZone,
    #[sea_orm(primary_key)]
    pub id: i32,
    pub code: String,
    pub level: i32,
    pub kind: String,
    pub title: String,
    pub provider: Option<String>,
    pub url: Option<String>,
    pub retired_in: Option<String>,
}

#[derive(Copy, Clone, Debug, EnumIter, DeriveRelation)]
pub enum Relation {}

impl ActiveModelBehavior for ActiveModel {}
