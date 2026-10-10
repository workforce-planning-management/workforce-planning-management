//! `SeaORM` Entity — `workforce_plans`. A workforce plan (scenario): hypothetical aggregate headcount.

#![allow(missing_docs)]

use sea_orm::entity::prelude::*;
use serde::{Deserialize, Serialize};

#[derive(Clone, Debug, PartialEq, DeriveEntityModel, Serialize, Deserialize)]
#[sea_orm(table_name = "workforce_plans")]
pub struct Model {
    pub created_at: DateTimeWithTimeZone,
    pub updated_at: DateTimeWithTimeZone,
    #[sea_orm(primary_key)]
    pub id: i32,
    #[sea_orm(unique)]
    pub pid: Uuid,
    pub name: String,
    pub organization_ref: String,
    pub horizon_start: Date,
    pub horizon_end: Date,
    pub rationale: Option<String>,
    pub attrition_bp: Option<i32>,
    pub budget_minor: Option<i64>,
    pub budget_currency: Option<String>,
    pub on_cost_bp: Option<i32>,
    pub status: String,
    pub deleted_at: Option<DateTimeWithTimeZone>,
}

#[derive(Copy, Clone, Debug, EnumIter, DeriveRelation)]
pub enum Relation {}

impl ActiveModelBehavior for ActiveModel {}
