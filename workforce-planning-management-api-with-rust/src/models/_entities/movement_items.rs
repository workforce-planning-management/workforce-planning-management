//! `SeaORM` Entity — `movement_items`. One dated item of a joiner or leaver checklist.

#![allow(missing_docs)]

use sea_orm::entity::prelude::*;
use serde::{Deserialize, Serialize};

#[derive(Clone, Debug, PartialEq, DeriveEntityModel, Serialize, Deserialize)]
#[sea_orm(table_name = "movement_items")]
pub struct Model {
    pub created_at: DateTimeWithTimeZone,
    pub updated_at: DateTimeWithTimeZone,
    #[sea_orm(primary_key)]
    pub id: i32,
    #[sea_orm(unique)]
    pub pid: Uuid,
    pub movement_pid: Uuid,
    pub position: i32,
    pub title: String,
    pub category: String,
    pub due_on: Date,
    pub assignee_pid: Option<Uuid>,
    pub done_on: Option<Date>,
    pub done_by: Option<String>,
    pub skipped_reason: Option<String>,
}

#[derive(Copy, Clone, Debug, EnumIter, DeriveRelation)]
pub enum Relation {}

impl ActiveModelBehavior for ActiveModel {}
