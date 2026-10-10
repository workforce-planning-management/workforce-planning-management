//! `SeaORM` Entity — `flexible_working_requests`. A worker's request for a different working arrangement and its decision (WPM-R126).

#![allow(missing_docs)]

use sea_orm::entity::prelude::*;
use serde::{Deserialize, Serialize};

#[derive(Clone, Debug, PartialEq, DeriveEntityModel, Eq, Serialize, Deserialize)]
#[sea_orm(table_name = "flexible_working_requests")]
pub struct Model {
    pub created_at: DateTimeWithTimeZone,
    pub updated_at: DateTimeWithTimeZone,
    #[sea_orm(primary_key)]
    pub id: i32,
    #[sea_orm(unique)]
    pub pid: Uuid,
    pub worker_pid: Uuid,
    pub kind: String,
    pub requested_on: Date,
    pub proposed_start: Date,
    pub proposed_fte_percent: Option<i32>,
    pub effect_on_team: Option<String>,
    pub how_to_manage: Option<String>,
    pub status: String,
    pub decide_by: Date,
    pub decided_on: Option<Date>,
    pub decided_by: Option<String>,
    pub decision_reason: Option<String>,
    pub decision_note: Option<String>,
    pub counter_note: Option<String>,
    pub counter_fte_percent: Option<i32>,
    pub trial_until: Option<Date>,
    pub appealed_on: Option<Date>,
    pub appeal_note: Option<String>,
    pub appeal_decided_by: Option<String>,
}

#[derive(Copy, Clone, Debug, EnumIter, DeriveRelation)]
pub enum Relation {}

impl ActiveModelBehavior for ActiveModel {}
