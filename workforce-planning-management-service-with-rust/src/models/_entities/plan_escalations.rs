//! `SeaORM` Entity — `plan_escalations`. A breach of a plan's tolerance taken to someone who can decide.

#![allow(missing_docs)]

use sea_orm::entity::prelude::*;
use serde::{Deserialize, Serialize};

#[derive(Clone, Debug, PartialEq, DeriveEntityModel, Serialize, Deserialize)]
#[sea_orm(table_name = "plan_escalations")]
pub struct Model {
    pub created_at: DateTimeWithTimeZone,
    pub updated_at: DateTimeWithTimeZone,
    #[sea_orm(primary_key)]
    pub id: i32,
    #[sea_orm(unique)]
    pub pid: Uuid,
    pub plan_pid: Uuid,
    pub dimension: String,
    pub level: i32,
    pub status: String,
    pub raised_by: Option<String>,
    pub raised_on: Date,
    pub level_raised_on: Option<Date>,
    pub note: Option<String>,
    pub rag_at_raise: String,
    pub deviation_at_raise: Option<i64>,
    pub limit_at_raise: i64,
    pub decision: Option<String>,
    pub decided_by: Option<String>,
    pub decided_on: Option<Date>,
    pub decision_note: Option<String>,
    pub rag_at_decision: Option<String>,
    pub deviation_at_decision: Option<i64>,
}

#[derive(Copy, Clone, Debug, EnumIter, DeriveRelation)]
pub enum Relation {}

impl ActiveModelBehavior for ActiveModel {}
