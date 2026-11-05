//! `SeaORM` Entity — `skill_courses`. A catalogue course that builds a skill: its hours, how many levels it typically adds, and a fee if one is recorded.

#![allow(missing_docs)]

use sea_orm::entity::prelude::*;
use serde::{Deserialize, Serialize};

#[derive(Clone, Debug, PartialEq, DeriveEntityModel, Serialize, Deserialize)]
#[sea_orm(table_name = "skill_courses")]
pub struct Model {
    pub created_at: DateTimeWithTimeZone,
    pub updated_at: DateTimeWithTimeZone,
    #[sea_orm(primary_key)]
    pub id: i32,
    #[sea_orm(unique)]
    pub pid: Uuid,
    pub skill_pid: Uuid,
    pub course_ref: String,
    pub title: String,
    pub hours: i32,
    pub levels: i32,
    pub deleted_at: Option<DateTimeWithTimeZone>,
    /// The fee in minor units; absent is unknown, never free (WPM-R227).
    pub fee_minor: Option<i64>,
    pub fee_currency: Option<String>,
}

#[derive(Copy, Clone, Debug, EnumIter, DeriveRelation)]
pub enum Relation {}

impl ActiveModelBehavior for ActiveModel {}
