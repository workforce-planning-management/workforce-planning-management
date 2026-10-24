//! `SeaORM` Entity — `pay_assessment_records`. What a person recorded for a category of workers and a pay report: a
//! justification, a remedy, a joint assessment and its outcome (WPM-R146). No personal data.

#![allow(missing_docs)]

use sea_orm::entity::prelude::*;
use serde::{Deserialize, Serialize};

#[derive(Clone, Debug, PartialEq, DeriveEntityModel, Eq, Serialize, Deserialize)]
#[sea_orm(table_name = "pay_assessment_records")]
pub struct Model {
    pub created_at: DateTimeWithTimeZone,
    pub updated_at: DateTimeWithTimeZone,
    #[sea_orm(primary_key)]
    pub id: i32,
    #[sea_orm(unique)]
    pub pid: Uuid,
    pub organization_ref: String,
    pub category: String,
    pub reported_on: Date,
    pub justified: bool,
    pub justification: Option<String>,
    pub remedied_on: Option<Date>,
    pub assessed_on: Option<Date>,
    pub outcome: Option<String>,
    pub representatives: Option<String>,
    pub recorded_by: Option<String>,
}

#[derive(Copy, Clone, Debug, EnumIter, DeriveRelation)]
pub enum Relation {}

impl ActiveModelBehavior for ActiveModel {}
