//! `SeaORM` Entity — `coverage_evidence`. An organization's workers' compensation or employer's liability cover in one jurisdiction
//! (WPM-R220, WPM-D154): a record of the fact and its dates, with a reference to the certificate, never a copy.

#![allow(missing_docs)]

use sea_orm::entity::prelude::*;
use serde::{Deserialize, Serialize};

#[derive(Clone, Debug, PartialEq, DeriveEntityModel, Eq, Serialize, Deserialize)]
#[sea_orm(table_name = "coverage_evidence")]
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
    pub insurer: Option<String>,
    pub policy_reference: Option<String>,
    pub minimum_cover_minor: Option<i64>,
    pub currency: Option<String>,
    pub valid_from: Option<Date>,
    pub valid_to: Option<Date>,
    pub certificate_ref: Option<String>,
    pub recorded_by: Option<String>,
    pub retired_on: Option<Date>,
}

#[derive(Copy, Clone, Debug, EnumIter, DeriveRelation)]
pub enum Relation {}

impl ActiveModelBehavior for ActiveModel {}
