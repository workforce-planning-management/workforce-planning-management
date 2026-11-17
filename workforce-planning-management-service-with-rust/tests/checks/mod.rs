//! Checks that need no running service or need only their own database: the source and schema guards, the route table, the
//! schedule, the `health` command and the adoption of a database `SeaORM` migrated. Each was its own test binary until WPM-T534.

mod health_command;
mod migration_adoption;
mod no_union_membership;
mod personal_data_inventory;
mod route_coverage;
mod scheduler_config;
