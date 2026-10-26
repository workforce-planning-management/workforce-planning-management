//! `SeaORM` Entity — `bargaining_units`. The part of the workforce a recognised body covers, as rules over facts WPM holds (WPM-R102).
//! Who is covered is computed and counted with a floor; it is never stored (WPM-D68).

#![allow(missing_docs)]

use sea_orm::entity::prelude::*;
use serde::{Deserialize, Serialize};

#[derive(Clone, Debug, PartialEq, DeriveEntityModel, Eq, Serialize, Deserialize)]
#[sea_orm(table_name = "bargaining_units")]
pub struct Model {
    pub created_at: DateTimeWithTimeZone,
    pub updated_at: DateTimeWithTimeZone,
    #[sea_orm(primary_key)]
    pub id: i32,
    #[sea_orm(unique)]
    pub pid: Uuid,
    pub body_pid: Uuid,
    pub name: String,
    pub department: Option<String>,
    pub location: Option<String>,
    pub employment_type: Option<String>,
    pub job_title: Option<String>,
    pub recorded_by: Option<String>,
}

#[derive(Copy, Clone, Debug, EnumIter, DeriveRelation)]
pub enum Relation {}

impl ActiveModelBehavior for ActiveModel {}
