//! `workforce-planning-management` binary entrypoint: the service's command line, plus `scheduler`.
//! See the crate library docs for the service overview.

#![warn(clippy::pedantic)]
#![forbid(unsafe_code)]
#![deny(missing_docs)]
#![deny(
    clippy::unwrap_used,
    clippy::expect_used,
    clippy::panic,
    clippy::unreachable,
    clippy::todo,
    clippy::unimplemented,
    clippy::indexing_slicing,
    clippy::string_slice,
    clippy::arithmetic_side_effects,
    // Each exception is `#[expect]`ed, with a reason (RCA-EXC-6).
    clippy::allow_attributes,
    clippy::allow_attributes_without_reason,
    // Every literal names its type, so no figure silently becomes an `i32` or `f64` (RCA-INT).
    clippy::default_numeric_fallback
)]

#[cfg(target_env = "musl")]
#[global_allocator]
static GLOBAL: mimalloc::MiMalloc = mimalloc::MiMalloc;

use workforce_planning_management::{app::App, scheduler};

#[tokio::main]
async fn main() -> axa::Result<()> {
    let arguments: Vec<String> = std::env::args().skip(1).collect();
    if let Some((first, rest)) = arguments.split_first()
        && first == "scheduler"
    {
        axa::logger::init(env!("CARGO_CRATE_NAME"), &App::logging());
        return scheduler::run(rest).await;
    }
    App::run(axa::cli::parse(
        env!("CARGO_PKG_NAME"),
        env!("CARGO_PKG_VERSION"),
        env!("CARGO_PKG_DESCRIPTION"),
    ))
    .await
}
