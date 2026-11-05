//! `SeaORM` Entity — `worker_goals`. One person's goal for a period: a record of intent and evidence, never a score (WPM-R224, WPM-D163).

#![allow(missing_docs)]

use sea_orm::entity::prelude::*;
use serde::{Deserialize, Serialize};

#[derive(Clone, Debug, PartialEq, DeriveEntityModel, Eq, Serialize, Deserialize)]
#[sea_orm(table_name = "worker_goals")]
pub struct Model {
    pub created_at: DateTimeWithTimeZone,
    pub updated_at: DateTimeWithTimeZone,
    #[sea_orm(primary_key)]
    pub id: i32,
    #[sea_orm(unique)]
    pub pid: Uuid,
    pub worker_pid: Uuid,
    pub organization_ref: String,
    pub title: String,
    pub description: Option<String>,
    pub period_start: Date,
    pub period_end: Date,
    pub status: String,
    pub proposed_by: String,
    pub agreed_on: Option<Date>,
    pub skill_pid: Option<Uuid>,
    pub skill_target_level: Option<i32>,
    pub plan_item_pid: Option<Uuid>,
    pub aligns_to: Option<String>,
    pub deleted_at: Option<DateTimeWithTimeZone>,
}

#[derive(Copy, Clone, Debug, EnumIter, DeriveRelation)]
pub enum Relation {}
