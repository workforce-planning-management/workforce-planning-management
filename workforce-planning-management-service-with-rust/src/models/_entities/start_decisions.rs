//! `SeaORM` Entity — `start_decisions`. A recorded decision to start or defer a programme, with its reason (WPM-R60, WPM-D50).

#![allow(missing_docs)]

use sea_orm::entity::prelude::*;
use serde::{Deserialize, Serialize};

#[derive(Clone, Debug, PartialEq, DeriveEntityModel, Eq, Serialize, Deserialize)]
#[sea_orm(table_name = "start_decisions")]
pub struct Model {
    pub created_at: DateTimeWithTimeZone,
    pub updated_at: DateTimeWithTimeZone,
    #[sea_orm(primary_key)]
    pub id: i32,
    #[sea_orm(unique)]
    pub pid: Uuid,
    pub organization_ref: String,
    pub programme_ref: String,
    pub decision: String,
    pub reason: String,
    pub fit: String,
    pub earliest_shift_months: Option<i32>,
    pub wip_exceeds: Option<bool>,
    pub decided_by: Option<String>,
}

#[derive(Copy, Clone, Debug, EnumIter, DeriveRelation)]
pub enum Relation {}

impl ActiveModelBehavior for ActiveModel {}
