//! `SeaORM` Entity — `cpd_entries`. One CPD activity a worker recorded.

#![allow(missing_docs)]

use sea_orm::entity::prelude::*;
use serde::{Deserialize, Serialize};

#[derive(Clone, Debug, PartialEq, DeriveEntityModel, Serialize, Deserialize)]
#[sea_orm(table_name = "cpd_entries")]
pub struct Model {
    pub created_at: DateTimeWithTimeZone,
    pub updated_at: DateTimeWithTimeZone,
    #[sea_orm(primary_key)]
    pub id: i32,
    #[sea_orm(unique)]
    pub pid: Uuid,
    pub worker_pid: Uuid,
    pub entry_date: Date,
    pub activity: String,
    pub category: String,
    pub unit: String,
    pub amount_hundredths: i64,
    pub evidence_note: Option<String>,
    pub evidence_url: Option<String>,
    pub source: String,
    pub external_ref: Option<String>,
    pub verified_on: Option<Date>,
    pub verified_by: Option<String>,
    pub deleted_at: Option<DateTimeWithTimeZone>,
}

#[derive(Copy, Clone, Debug, EnumIter, DeriveRelation)]
pub enum Relation {}

impl ActiveModelBehavior for ActiveModel {}
