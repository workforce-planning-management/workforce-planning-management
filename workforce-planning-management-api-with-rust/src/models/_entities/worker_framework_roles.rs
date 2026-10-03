//! `SeaORM` Entity — `worker_framework_roles`. The role a person says they
//! hold now in an external framework (a PCF role level or an ESCO occupation).

#![allow(missing_docs)]

use sea_orm::entity::prelude::*;
use serde::{Deserialize, Serialize};

#[derive(Clone, Debug, PartialEq, DeriveEntityModel, Serialize, Deserialize)]
#[sea_orm(table_name = "worker_framework_roles")]
pub struct Model {
    pub created_at: DateTimeWithTimeZone,
    pub updated_at: DateTimeWithTimeZone,
    #[sea_orm(primary_key)]
    pub id: i32,
    #[sea_orm(unique)]
    pub pid: Uuid,
    pub worker_pid: Uuid,
    pub framework_slug: String,
    pub role_profile_pid: Option<Uuid>,
    pub esco_occupation_uri: Option<String>,
    pub role_label: String,
    pub selected_on: Date,
}

#[derive(Copy, Clone, Debug, EnumIter, DeriveRelation)]
pub enum Relation {}

impl ActiveModelBehavior for ActiveModel {}
