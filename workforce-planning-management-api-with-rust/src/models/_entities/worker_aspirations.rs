//! `SeaORM` Entity — `worker_aspirations`. A future role or skill target: an aspiration, learning goal, or growth idea.

#![allow(missing_docs)]

use sea_orm::entity::prelude::*;
use serde::{Deserialize, Serialize};

#[derive(Clone, Debug, PartialEq, DeriveEntityModel, Serialize, Deserialize)]
#[sea_orm(table_name = "worker_aspirations")]
pub struct Model {
    pub created_at: DateTimeWithTimeZone,
    pub updated_at: DateTimeWithTimeZone,
    #[sea_orm(primary_key)]
    pub id: i32,
    #[sea_orm(unique)]
    pub pid: Uuid,
    pub worker_pid: Uuid,
    pub kind: String,
    pub framework_slug: Option<String>,
    pub role_profile_pid: Option<Uuid>,
    pub esco_occupation_uri: Option<String>,
    pub role_label: Option<String>,
    pub skill_pid: Option<Uuid>,
    pub target_level: Option<i32>,
    pub horizon: String,
    pub status: String,
    pub note: Option<String>,
    pub shared: bool,
    pub recorded_by: Option<String>,
    pub on_behalf: bool,
    pub deleted_at: Option<DateTimeWithTimeZone>,
}

#[derive(Copy, Clone, Debug, EnumIter, DeriveRelation)]
pub enum Relation {}

impl ActiveModelBehavior for ActiveModel {}
