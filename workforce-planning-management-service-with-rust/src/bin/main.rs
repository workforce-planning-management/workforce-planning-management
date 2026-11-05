//! `workforce-planning-management` binary entrypoint: the service's command line, plus `scheduler`.
//! See the crate library docs for the service overview.

#![warn(clippy::pedantic)]
#![forbid(unsafe_code)]
#![deny(missing_docs)]

#[cfg(target_env = "musl")]
#[global_allocator]
static GLOBAL: mimalloc::MiMalloc = mimalloc::MiMalloc;

use axa::cli::Cli;
use clap::Parser;
use workforce_planning_management::{app::App, scheduler};

#[tokio::main]
async fn main() -> axa::Result<()> {
    let arguments: Vec<String> = std::env::args().skip(1).collect();
    if arguments.first().is_some_and(|first| first == "scheduler") {
        axa::logger::init(env!("CARGO_CRATE_NAME"), &App::logging());
        return scheduler::run(&arguments[1..]).await;
    }
    App::run(Cli::parse()).await
}
