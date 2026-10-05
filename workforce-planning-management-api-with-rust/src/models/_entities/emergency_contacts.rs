//! `SeaORM` Entity — `emergency_contacts`. A person a worker names to be reached in an emergency (third-party personal data).

#![allow(missing_docs)]

use sea_orm::entity::prelude::*;
use serde::{Deserialize, Serialize};

#[derive(Clone, Debug, PartialEq, DeriveEntityModel, Serialize, Deserialize)]
#[sea_orm(table_name = "emergency_contacts")]
pub struct Model {
    pub created_at: DateTimeWithTimeZone,
    pub updated_at: DateTimeWithTimeZone,
    #[sea_orm(primary_key)]
    pub id: i32,
    #[sea_orm(unique)]
    pub pid: Uuid,
    pub worker_pid: Uuid,
    pub name: String,
    pub relationship: String,
    pub phone: String,
    pub alt_phone: Option<String>,
    pub email: Option<String>,
    pub priority: i32,
    pub note: Option<String>,
    pub recorded_by: Option<String>,
    pub on_behalf: bool,
    pub deleted_at: Option<DateTimeWithTimeZone>,
}

#[derive(Copy, Clone, Debug, EnumIter, DeriveRelation)]
pub enum Relation {}

impl ActiveModelBehavior for ActiveModel {}
