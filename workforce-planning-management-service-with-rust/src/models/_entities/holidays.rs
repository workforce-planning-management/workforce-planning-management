//! `SeaORM` Entity — `holidays`. A public holiday for an organization and region (WPM-R5): the date a rule gives, the day it is
//! taken, and whether it was generated or added by hand.

#![allow(missing_docs)]

use sea_orm::entity::prelude::*;
use serde::{Deserialize, Serialize};

#[derive(Clone, Debug, PartialEq, Eq, DeriveEntityModel, Serialize, Deserialize)]
#[sea_orm(table_name = "holidays")]
pub struct Model {
    pub created_at: DateTimeWithTimeZone,
    pub updated_at: DateTimeWithTimeZone,
    #[sea_orm(primary_key)]
    pub id: i32,
    #[sea_orm(unique)]
    pub pid: Uuid,
    pub organization_ref: String,
    pub region: String,
    pub year: i32,
    pub name: String,
    pub natural_on: Date,
    pub observed_on: Date,
    pub source: String,
    pub recorded_by: Option<String>,
}

#[derive(Copy, Clone, Debug, EnumIter, DeriveRelation)]
pub enum Relation {}

impl ActiveModelBehavior for ActiveModel {}
