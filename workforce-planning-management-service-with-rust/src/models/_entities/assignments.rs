//! `SeaORM` Entity — `assignments`. A contract a worker holds (WPM-R161): job title, department, location, optional categories
//! (`None` is unknown), the FTE the contract says, and the days it runs. One primary is open at a time.

#![allow(missing_docs)]

use sea_orm::entity::prelude::*;
use serde::{Deserialize, Serialize};

#[derive(Clone, Debug, PartialEq, Eq, DeriveEntityModel, Serialize, Deserialize)]
#[sea_orm(table_name = "assignments")]
pub struct Model {
    pub created_at: DateTimeWithTimeZone,
    pub updated_at: DateTimeWithTimeZone,
    #[sea_orm(primary_key)]
    pub id: i32,
    #[sea_orm(unique)]
    pub pid: Uuid,
    pub worker_pid: Uuid,
    pub job_title: String,
    pub department: String,
    pub location: Option<String>,
    pub site: Option<String>,
    pub area_of_work: Option<String>,
    pub job_role: Option<String>,
    pub staff_group: Option<String>,
    pub occupation_code: Option<String>,
    pub fte_percent: i32,
    pub is_primary: bool,
    pub starts_on: Date,
    pub ends_on: Option<Date>,
    pub recorded_by: Option<String>,
}

#[derive(Copy, Clone, Debug, EnumIter, DeriveRelation)]
pub enum Relation {}

impl ActiveModelBehavior for ActiveModel {}
