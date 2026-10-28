//! `SeaORM` Entity — `category_vocabulary`. One entry of an organization's assignment category vocabulary (WPM-R162): a code, a label,
//! the national standard it comes from where it does, and, for a job role, the staff group it belongs to.

#![allow(missing_docs)]

use sea_orm::entity::prelude::*;
use serde::{Deserialize, Serialize};

#[derive(Clone, Debug, PartialEq, Eq, DeriveEntityModel, Serialize, Deserialize)]
#[sea_orm(table_name = "category_vocabulary")]
pub struct Model {
    pub created_at: DateTimeWithTimeZone,
    pub updated_at: DateTimeWithTimeZone,
    #[sea_orm(primary_key)]
    pub id: i32,
    #[sea_orm(unique)]
    pub pid: Uuid,
    pub organization_ref: String,
    pub kind: String,
    pub code: String,
    pub label: String,
    pub standard: Option<String>,
    pub standard_code: Option<String>,
    pub staff_group: Option<String>,
    pub recorded_by: Option<String>,
}

#[derive(Copy, Clone, Debug, EnumIter, DeriveRelation)]
pub enum Relation {}

impl ActiveModelBehavior for ActiveModel {}
