//! `SeaORM` Entity — `policy_versions`. One version of a policy: title, a reference to the document (no text), effective date,
//! audience and days to acknowledge (WPM-R156).

#![allow(missing_docs)]

use sea_orm::entity::prelude::*;
use serde::{Deserialize, Serialize};

#[derive(Clone, Debug, PartialEq, DeriveEntityModel, Eq, Serialize, Deserialize)]
#[sea_orm(table_name = "policy_versions")]
pub struct Model {
    pub created_at: DateTimeWithTimeZone,
    pub updated_at: DateTimeWithTimeZone,
    #[sea_orm(primary_key)]
    pub id: i32,
    #[sea_orm(unique)]
    pub pid: Uuid,
    pub policy_pid: Uuid,
    pub version: i32,
    pub title: String,
    pub document_ref: String,
    pub effective_on: Date,
    pub audience_kind: String,
    pub audience_value: Option<String>,
    pub acknowledge_within_days: i32,
    pub recorded_by: Option<String>,
}

#[derive(Copy, Clone, Debug, EnumIter, DeriveRelation)]
pub enum Relation {}

impl ActiveModelBehavior for ActiveModel {}
