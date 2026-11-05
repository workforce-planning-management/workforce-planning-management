//! `SeaORM` Entity — `padr_versions`. One version of a personal development review: its content, its two signatures and, once signed, the objectives as they stood (WPM-R172, WPM-D165).

#![allow(missing_docs)]

use sea_orm::entity::prelude::*;
use serde::{Deserialize, Serialize};

#[derive(Clone, Debug, PartialEq, DeriveEntityModel, Eq, Serialize, Deserialize)]
#[sea_orm(table_name = "padr_versions")]
pub struct Model {
    pub created_at: DateTimeWithTimeZone,
    pub updated_at: DateTimeWithTimeZone,
    #[sea_orm(primary_key)]
    pub id: i32,
    #[sea_orm(unique)]
    pub pid: Uuid,
    pub review_pid: Uuid,
    pub worker_pid: Uuid,
    pub version: i32,
    pub period_start: Date,
    pub period_end: Date,
    pub values_summary: Option<String>,
    pub wellbeing_offered: Option<bool>,
    pub development_needs: Json,
    pub reviewer_signed_on: Option<Date>,
    pub reviewee_signed_on: Option<Date>,
    pub reviewee_comment: Option<String>,
    pub objectives: Option<Json>,
}

#[derive(Copy, Clone, Debug, EnumIter, DeriveRelation)]
pub enum Relation {}
