//! `SeaORM` Entity — `goal_measures`. A measure of a goal: a number with its start, target and current value, or a yes-or-no milestone. Every value is optional (WPM-R224).

#![allow(missing_docs)]

use sea_orm::entity::prelude::*;
use serde::{Deserialize, Serialize};

#[derive(Clone, Debug, PartialEq, DeriveEntityModel, Eq, Serialize, Deserialize)]
#[sea_orm(table_name = "goal_measures")]
pub struct Model {
    pub created_at: DateTimeWithTimeZone,
    pub updated_at: DateTimeWithTimeZone,
    #[sea_orm(primary_key)]
    pub id: i32,
    #[sea_orm(unique)]
    pub pid: Uuid,
    pub goal_pid: Uuid,
    pub worker_pid: Uuid,
    pub name: String,
    pub kind: String,
    pub unit: Option<String>,
    pub start_value: Option<i64>,
    pub target_value: Option<i64>,
    pub current_value: Option<i64>,
    pub done: Option<bool>,
}

#[derive(Copy, Clone, Debug, EnumIter, DeriveRelation)]
pub enum Relation {}
