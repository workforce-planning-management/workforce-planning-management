//! `SeaORM` Entity — `asset_issues`. Who holds an asset, from when to when (WPM-R155). Closed by a return or a write-off.

#![allow(missing_docs)]

use sea_orm::entity::prelude::*;
use serde::{Deserialize, Serialize};

#[derive(Clone, Debug, PartialEq, DeriveEntityModel, Eq, Serialize, Deserialize)]
#[sea_orm(table_name = "asset_issues")]
pub struct Model {
    pub created_at: DateTimeWithTimeZone,
    pub updated_at: DateTimeWithTimeZone,
    #[sea_orm(primary_key)]
    pub id: i32,
    #[sea_orm(unique)]
    pub pid: Uuid,
    pub asset_pid: Uuid,
    pub worker_pid: Uuid,
    pub issued_on: Date,
    pub returned_on: Option<Date>,
    pub outcome: Option<String>,
    pub note: Option<String>,
    pub recorded_by: Option<String>,
}

#[derive(Copy, Clone, Debug, EnumIter, DeriveRelation)]
pub enum Relation {}

impl ActiveModelBehavior for ActiveModel {}
