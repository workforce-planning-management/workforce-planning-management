//! `SeaORM` Entity — `qualifications`. A qualification a person holds: declared, or verified by HR with an evidence reference (WPM-R171).

#![allow(missing_docs)]

use sea_orm::entity::prelude::*;
use serde::{Deserialize, Serialize};

#[derive(Clone, Debug, PartialEq, DeriveEntityModel, Eq, Serialize, Deserialize)]
#[sea_orm(table_name = "qualifications")]
pub struct Model {
    pub created_at: DateTimeWithTimeZone,
    pub updated_at: DateTimeWithTimeZone,
    #[sea_orm(primary_key)]
    pub id: i32,
    #[sea_orm(unique)]
    pub pid: Uuid,
    pub worker_pid: Uuid,
    pub kind: String,
    pub title: String,
    pub awarding_body: Option<String>,
    pub framework: Option<String>,
    pub level: Option<i32>,
    pub awarded_on: Option<Date>,
    pub expires_on: Option<Date>,
    pub verified_by: Option<String>,
    pub verified_on: Option<Date>,
    pub evidence_ref: Option<String>,
    pub recorded_by: Option<String>,
}

#[derive(Copy, Clone, Debug, EnumIter, DeriveRelation)]
pub enum Relation {}

impl ActiveModelBehavior for ActiveModel {}
