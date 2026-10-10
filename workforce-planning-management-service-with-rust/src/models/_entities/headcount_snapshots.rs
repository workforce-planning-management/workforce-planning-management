//! `SeaORM` Entity — `headcount_snapshots`. Append-only aggregate
//! history of employed headcount per organization × department × date.

#![allow(missing_docs)]

use sea_orm::entity::prelude::*;
use serde::{Deserialize, Serialize};

#[derive(Clone, Debug, PartialEq, DeriveEntityModel, Serialize, Deserialize)]
#[sea_orm(table_name = "headcount_snapshots")]
pub struct Model {
    pub created_at: DateTimeWithTimeZone,
    pub updated_at: DateTimeWithTimeZone,
    #[sea_orm(primary_key)]
    pub id: i32,
    #[sea_orm(unique)]
    pub pid: Uuid,
    pub organization_ref: String,
    pub department: String,
    pub as_of: Date,
    pub headcount: i32,
    pub fte_percent_total: i64,
    pub starters: Option<i32>,
    pub leavers: Option<i32>,
}

#[derive(Copy, Clone, Debug, EnumIter, DeriveRelation)]
pub enum Relation {}

impl ActiveModelBehavior for ActiveModel {}
