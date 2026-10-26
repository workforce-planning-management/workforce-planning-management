//! `SeaORM` Entity — `consultations`. The consultation register (WPM-R105): a proposal as counts and dates, the body consulted, topics, a
//! document reference, dates and outcome. No attendee, representative or minute (WPM-D68).

#![allow(missing_docs)]

use sea_orm::entity::prelude::*;
use serde::{Deserialize, Serialize};

#[derive(Clone, Debug, PartialEq, DeriveEntityModel, Eq, Serialize, Deserialize)]
#[sea_orm(table_name = "consultations")]
pub struct Model {
    pub created_at: DateTimeWithTimeZone,
    pub updated_at: DateTimeWithTimeZone,
    #[sea_orm(primary_key)]
    pub id: i32,
    #[sea_orm(unique)]
    pub pid: Uuid,
    pub organization_ref: String,
    pub body_pid: Uuid,
    pub plan_pid: Option<Uuid>,
    pub establishment: String,
    pub profile: String,
    pub establishment_size: i32,
    pub dismissals: i32,
    pub earliest_termination_on: Date,
    pub topics: String,
    pub document_ref: Option<String>,
    pub started_on: Option<Date>,
    pub ended_on: Option<Date>,
    pub outcome: String,
    pub agreement_pid: Option<Uuid>,
    pub recorded_by: Option<String>,
}

#[derive(Copy, Clone, Debug, EnumIter, DeriveRelation)]
pub enum Relation {}

impl ActiveModelBehavior for ActiveModel {}
