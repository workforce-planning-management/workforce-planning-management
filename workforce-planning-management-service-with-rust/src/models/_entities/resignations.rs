//! `SeaORM` Entity — `resignations`. A worker's logged intent to resign and its outcome (WPM-R127).

#![allow(missing_docs)]

use sea_orm::entity::prelude::*;
use serde::{Deserialize, Serialize};

#[derive(Clone, Debug, PartialEq, DeriveEntityModel, Eq, Serialize, Deserialize)]
#[sea_orm(table_name = "resignations")]
pub struct Model {
    pub created_at: DateTimeWithTimeZone,
    pub updated_at: DateTimeWithTimeZone,
    #[sea_orm(primary_key)]
    pub id: i32,
    #[sea_orm(unique)]
    pub pid: Uuid,
    pub worker_pid: Uuid,
    pub logged_on: Date,
    pub proposed_last_day: Date,
    pub reason: Option<String>,
    pub status: String,
    pub agreed_last_day: Option<Date>,
    pub movement_pid: Option<Uuid>,
    pub decided_by: Option<String>,
    pub decided_at: Option<DateTimeWithTimeZone>,
}

#[derive(Copy, Clone, Debug, EnumIter, DeriveRelation)]
pub enum Relation {}

impl ActiveModelBehavior for ActiveModel {}
