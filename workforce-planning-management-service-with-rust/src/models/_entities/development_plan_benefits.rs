//! `SeaORM` Entity — `development_plan_benefits`. A stated yearly benefit of a development plan with its named source: an estimate, never derived from a skill level or a score (WPM-R230, WPM-D173).

#![allow(missing_docs)]

use sea_orm::entity::prelude::*;
use serde::{Deserialize, Serialize};

#[derive(Clone, Debug, PartialEq, DeriveEntityModel, Eq, Serialize, Deserialize)]
#[sea_orm(table_name = "development_plan_benefits")]
pub struct Model {
    pub created_at: DateTimeWithTimeZone,
    pub updated_at: DateTimeWithTimeZone,
    #[sea_orm(primary_key)]
    pub id: i32,
    #[sea_orm(unique)]
    pub pid: Uuid,
    pub plan_pid: Uuid,
    pub worker_pid: Uuid,
    pub year: i32,
    pub amount_minor: i64,
    pub currency: String,
    pub source: String,
    pub stated_by: Option<String>,
    pub stated_on: Date,
    pub deleted_at: Option<DateTimeWithTimeZone>,
}

#[derive(Copy, Clone, Debug, EnumIter, DeriveRelation)]
pub enum Relation {}
