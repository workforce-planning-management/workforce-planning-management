//! `SeaORM` Entity — `collective_agreements`. A dated, sourced, versioned agreement tied to one bargaining unit, with the terms WPM can
//! use (WPM-R103). Who it applies to is derived from the unit's rules and never stored (WPM-D68).

#![allow(missing_docs)]

use sea_orm::entity::prelude::*;
use serde::{Deserialize, Serialize};

#[derive(Clone, Debug, PartialEq, DeriveEntityModel, Eq, Serialize, Deserialize)]
#[sea_orm(table_name = "collective_agreements")]
pub struct Model {
    pub created_at: DateTimeWithTimeZone,
    pub updated_at: DateTimeWithTimeZone,
    #[sea_orm(primary_key)]
    pub id: i32,
    #[sea_orm(unique)]
    pub pid: Uuid,
    pub unit_pid: Uuid,
    pub title: String,
    pub version: i32,
    pub effective_from: Date,
    pub effective_to: Option<Date>,
    pub source: String,
    pub pay_scale_id: Option<String>,
    pub max_weekly_hours: Option<i32>,
    pub notice_weeks: Option<i32>,
    pub consultation_days: Option<i32>,
    pub recorded_by: Option<String>,
}

#[derive(Copy, Clone, Debug, EnumIter, DeriveRelation)]
pub enum Relation {}

impl ActiveModelBehavior for ActiveModel {}
