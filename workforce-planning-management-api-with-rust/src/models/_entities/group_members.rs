//! `SeaORM` Entity — `group_members`. One person in one group over an interval.

#![allow(missing_docs)]

use sea_orm::entity::prelude::*;
use serde::{Deserialize, Serialize};

#[derive(Clone, Debug, PartialEq, DeriveEntityModel, Serialize, Deserialize)]
#[sea_orm(table_name = "group_members")]
pub struct Model {
    pub created_at: DateTimeWithTimeZone,
    pub updated_at: DateTimeWithTimeZone,
    #[sea_orm(primary_key)]
    pub id: i32,
    #[sea_orm(unique)]
    pub pid: Uuid,
    pub group_pid: Uuid,
    pub worker_pid: Uuid,
    pub role: String,
    pub joined_at: DateTimeWithTimeZone,
    pub left_at: Option<DateTimeWithTimeZone>,
    pub recorded_by: Option<String>,
    pub on_behalf: bool,
}

#[derive(Copy, Clone, Debug, EnumIter, DeriveRelation)]
pub enum Relation {}

impl ActiveModelBehavior for ActiveModel {}
