//! `SeaORM` Entity — `ai_register_entries`. What a deployer recorded for one scoring feature under Article 26 of the AI
//! Act (WPM-R148): the category, the oversight role, when people were informed and the log retention. No personal data.

#![allow(missing_docs)]

use sea_orm::entity::prelude::*;
use serde::{Deserialize, Serialize};

#[derive(Clone, Debug, PartialEq, DeriveEntityModel, Eq, Serialize, Deserialize)]
#[sea_orm(table_name = "ai_register_entries")]
pub struct Model {
    pub created_at: DateTimeWithTimeZone,
    pub updated_at: DateTimeWithTimeZone,
    #[sea_orm(primary_key)]
    pub id: i32,
    #[sea_orm(unique)]
    pub pid: Uuid,
    #[sea_orm(unique)]
    pub feature_id: String,
    pub annex_category: Option<String>,
    pub overseer_role: Option<String>,
    pub representatives_informed_on: Option<Date>,
    pub workers_informed_on: Option<Date>,
    pub log_retention_months: Option<i32>,
    pub retention_note: Option<String>,
    pub recorded_by: Option<String>,
}

#[derive(Copy, Clone, Debug, EnumIter, DeriveRelation)]
pub enum Relation {}

impl ActiveModelBehavior for ActiveModel {}
