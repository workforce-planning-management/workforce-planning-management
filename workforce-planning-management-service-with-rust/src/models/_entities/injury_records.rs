//! `SeaORM` Entity — `injury_records`. An incident: the proof that reports were owed and a pointer to the statutory record, with no description and no medical detail (WPM-R221, WPM-D179).

#![allow(missing_docs)]

use sea_orm::entity::prelude::*;
use serde::{Deserialize, Serialize};

#[derive(Clone, Debug, PartialEq, DeriveEntityModel, Eq, Serialize, Deserialize)]
#[sea_orm(table_name = "injury_records")]
pub struct Model {
    pub created_at: DateTimeWithTimeZone,
    pub updated_at: DateTimeWithTimeZone,
    #[sea_orm(primary_key)]
    pub id: i32,
    #[sea_orm(unique)]
    pub pid: Uuid,
    pub organization_ref: String,
    pub jurisdiction: String,
    pub kind: String,
    pub worker_pid: Option<Uuid>,
    pub occurred_at: DateTimeWithTimeZone,
    pub learned_at: Option<DateTimeWithTimeZone>,
    pub class: String,
    pub absence_days: Option<i32>,
    pub people_affected: i32,
    pub statutory_record_ref: Option<String>,
    pub keep_until: Option<Date>,
    pub recorded_by: Option<String>,
}

#[derive(Copy, Clone, Debug, EnumIter, DeriveRelation)]
pub enum Relation {}
