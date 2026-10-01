//! `SeaORM` Entity — `organization_confederations`. One direct
//! parent→child organization edge; arbitrary-depth nesting is chained
//! edges, walked in Rust (`crate::rules::org_access::descendants_of`),
//! not recursive SQL.

#![allow(missing_docs)]

use sea_orm::entity::prelude::*;
use serde::{Deserialize, Serialize};

#[derive(Clone, Debug, PartialEq, DeriveEntityModel, Serialize, Deserialize)]
#[sea_orm(table_name = "organization_confederations")]
pub struct Model {
    pub created_at: DateTimeWithTimeZone,
    pub updated_at: DateTimeWithTimeZone,
    #[sea_orm(primary_key)]
    pub id: i32,
    #[sea_orm(unique)]
    pub pid: Uuid,
    pub parent_organization_ref: String,
    pub child_organization_ref: String,
    pub starts_on: Date,
    pub ends_on: Option<Date>,
    pub deleted_at: Option<DateTimeWithTimeZone>,
}

#[derive(Copy, Clone, Debug, EnumIter, DeriveRelation)]
pub enum Relation {}
