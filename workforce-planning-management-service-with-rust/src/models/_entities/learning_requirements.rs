//! `SeaORM` Entity — `learning_requirements`. What must be learned, on what basis and for whom (WPM-R170, WPM-D117).

#![allow(missing_docs)]

use sea_orm::entity::prelude::*;
use serde::{Deserialize, Serialize};

#[derive(Clone, Debug, PartialEq, DeriveEntityModel, Eq, Serialize, Deserialize)]
#[sea_orm(table_name = "learning_requirements")]
pub struct Model {
    pub created_at: DateTimeWithTimeZone,
    pub updated_at: DateTimeWithTimeZone,
    #[sea_orm(primary_key)]
    pub id: i32,
    #[sea_orm(unique)]
    pub pid: Uuid,
    pub organization_ref: String,
    pub subject: String,
    pub basis: String,
    pub source: Option<String>,
    pub refresher_months: Option<i32>,
    pub courses: Json,
    pub audience: Json,
    pub accepts_transferred: bool,
    pub retired_on: Option<Date>,
    pub due_soon_days: Option<i32>,
    pub recorded_by: Option<String>,
}

#[derive(Copy, Clone, Debug, EnumIter, DeriveRelation)]
pub enum Relation {}

impl ActiveModelBehavior for ActiveModel {}
