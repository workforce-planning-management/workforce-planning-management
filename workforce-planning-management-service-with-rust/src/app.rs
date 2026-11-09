//! Application wiring: [`App`] reads the environment, connects, registers routes, the blanket auth
//! guard and the API-version middleware, runs the command line, and holds the truncate lifecycle for
//! `workforce-planning-management`.

use axa::{
    AppContext, Environment, Error, Result,
    boot::{BootResult, TestApp},
    cli::{Cli, Command, DbCommand, parse_vars},
    config::{Config, Defaults, Logger, MB},
    db::truncate_table,
    routes::{AppRoutes, Routes},
    server,
    task::Tasks,
};
use axum::{
    Router as AxumRouter,
    extract::Request,
    middleware::Next,
    response::{IntoResponse, Response},
};
use migration::{Migrator, MigratorTrait};

use crate::{auth, controllers, models::_entities::prelude::*, rules, tasks};

/// Blanket auth-enforcement middleware: reads `WPM_REQUIRE_AUTH` per
/// request and delegates to the pure [`auth::enforce`] (public paths
/// and the disabled flag pass through; otherwise a valid bearer token —
/// PASETO or Keycloak JWT, per the `paseto`/`keycloak` Cargo feature —
/// is required (`401`) and its `attrs` must satisfy the ABAC policy
/// for the derived action (`403`)). On by default — see `auth.rs` and
/// `spec/auth.md`.
async fn require_auth_mw(req: Request, next: Next) -> Response {
    let path = req.uri().path().to_string();
    let method = req.method().clone();
    // The MCP endpoint (WPM-R149): the caller must be signed in, and each tool call is then decided by the API's own policy,
    // under the same token, so the endpoint adds no access (WPM-D101).
    if auth::require_auth() && method == axum::http::Method::POST && path == "/mcp" {
        let verifier = auth::verifier().current();
        return match auth::bearer_claims(req.headers(), &verifier) {
            Ok(claims) if controllers::mcp::token_is_for_this_resource(&claims.aud) => {
                next.run(req).await
            }
            Ok(_) => (
                axum::http::StatusCode::UNAUTHORIZED,
                "the token was not issued for this MCP resource".to_string(),
            )
                .into_response(),
            Err((status, msg)) => (status, msg).into_response(),
        };
    }
    let policy = auth::policy().current();
    let verifier = auth::verifier().current();
    // A self-service write (WPM-R130, WPM-D76) skips the policy's action check, which lets only HR
    // write; it still needs a valid token, and its handler resolves the person from that token.
    if auth::require_auth() && rules::self_service::is_write(method.as_str(), &path) {
        return match auth::bearer_claims(req.headers(), &verifier) {
            Ok(_) => next.run(req).await,
            Err((status, msg)) => (status, msg).into_response(),
        };
    }
    // An attribute-scoped write (an occupational-health role recording a status): the policy cannot
    // name that role, so the token's own attribute lets it past the action check. The handler
    // checks the attribute again; a caller without it is judged by the ordinary policy.
    if auth::require_auth()
        && let Some(attribute) = rules::self_service::attribute_for_write(method.as_str(), &path)
        && let Ok(claims) = auth::bearer_claims(req.headers(), &verifier)
        && claims
            .attrs
            .get(attribute)
            .is_some_and(|values| values.iter().any(|v| v == "true"))
    {
        return next.run(req).await;
    }
    match auth::enforce(
        auth::require_auth(),
        &method,
        &path,
        req.headers(),
        &verifier,
        &policy,
    ) {
        Ok(()) => next.run(req).await,
        Err((status, msg)) => (status, msg).into_response(),
    }
}

/// The standing warning while sign-in is not enforced (WPM-D52).
fn warn_auth_off() {
    tracing::warn!(
        "WPM_REQUIRE_AUTH is off: sign-in is NOT enforced. Development only; production refuses to start this way"
    );
}

/// The `workforce-planning-management` application.
pub struct App;
/// Every route group the service serves. Needs no database, so a test can read the list (`tests/route_coverage.rs`).
#[must_use]
pub fn route_groups() -> Vec<Routes> {
    vec![
        axa::health::routes(),
        controllers::hr_core::routes(),
        controllers::acquisition::routes(),
        controllers::assessments::routes(),
        controllers::workforce::routes(),
        controllers::development::routes(),
        controllers::learning::routes(),
        controllers::roles::routes(),
        controllers::cpd::routes(),
        controllers::mobility::routes(),
        controllers::lms::routes(),
        controllers::change::routes(),
        controllers::planning::routes(),
        controllers::capacity::routes(),
        controllers::conversion_plans::routes(),
        controllers::funding::routes(),
        controllers::governance::routes(),
        controllers::key_person::routes(),
        controllers::oversight::routes(),
        controllers::early_warning::routes(),
        controllers::oversight_packs::routes(),
        controllers::cutover::routes(),
        controllers::programmes::routes(),
        controllers::engagements::routes(),
        controllers::me::routes(),
        controllers::resignations::routes(),
        controllers::flexible_working::routes(),
        controllers::equality_monitoring::routes(),
        controllers::pay_transparency::routes(),
        controllers::pay_information::routes(),
        controllers::my_payslips::routes(),
        controllers::training_economics::routes(),
        controllers::individual_goals::routes(),
        controllers::padr::routes(),
        controllers::separation::routes(),
        controllers::injury_records::routes(),
        controllers::pay_assessment::routes(),
        controllers::human_capital::routes(),
        controllers::pay_range::routes(),
        controllers::bargaining::routes(),
        controllers::consultations::routes(),
        controllers::decision_gates::routes(),
        controllers::equipment::routes(),
        controllers::absence_categories::routes(),
        controllers::assignments::routes(),
        controllers::mandatory_learning::routes(),
        controllers::coverage_evidence::routes(),
        controllers::qualifications::routes(),
        controllers::bridges::routes(),
        controllers::category_vocabulary::routes(),
        controllers::cost_centres::routes(),
        controllers::holidays::routes(),
        controllers::policies::routes(),
        controllers::credentials::routes(),
        controllers::skill_mappings::routes(),
        controllers::ai_register::routes(),
        controllers::webhooks::routes(),
        controllers::positions::routes(),
        controllers::health_requirements::routes(),
        controllers::esco::routes(),
        controllers::sfia::routes(),
        controllers::framework_roles::routes(),
        controllers::career::routes(),
        controllers::reporting::routes(),
        controllers::groups::routes(),
        controllers::announcements::routes(),
        controllers::contacts::routes(),
        controllers::directory::routes(),
        controllers::duplicates::routes(),
        controllers::pay_scales::routes(),
        controllers::job_levels::routes(),
        controllers::grades::routes(),
        controllers::pay_positions::routes(),
        controllers::expenses::routes(),
        controllers::rotas::routes(),
        controllers::handover::routes(),
        controllers::movements::routes(),
        controllers::skill_gaps::routes(),
        controllers::training_plan::routes(),
        controllers::rota_swaps::routes(),
        controllers::transfers::routes(),
        controllers::talent::routes(),
        controllers::intelligence::routes(),
        controllers::payroll::routes(),
        controllers::wellbeing::routes(),
        controllers::appraisals::routes(),
        controllers::privacy::routes(),
        controllers::statutory_defaults::routes(),
        controllers::jurisdictions::routes(),
        controllers::notifications::routes(),
        controllers::ergonomics::routes(),
        controllers::adjustments::routes(),
        controllers::organizations::routes(),
        controllers::audits::routes(),
        controllers::docs::routes(),
        controllers::metrics::routes(),
        crate::security::routes(),
    ]
}

/// The routes, as `axa` serves them.
#[must_use]
pub fn app_routes() -> AppRoutes {
    route_groups()
        .into_iter()
        .fold(AppRoutes::empty(), AppRoutes::add_route)
}

/// What the service falls back to when the environment does not say.
fn defaults() -> Defaults {
    Defaults {
        database: "workforce_planning_management",
        port: 5150,
        production_port: Some(5150),
        body_limit: 2 * MB,
        max_connections: 20,
        ..Defaults::default()
    }
}

/// The modules logged. `tower_http` stays at warnings: its request spans hold the whole URI with its query string, which WPM never
/// logs (WPM-R122); `src/request_log.rs` writes the request line instead.
fn log_filter(level: axa::logger::LogLevel) -> String {
    format!(
        "{}={level},axa={level},sea_orm_migration={level},tower_http=warn",
        env!("CARGO_CRATE_NAME")
    )
}

/// A request that runs longer than this is answered `408` (WPM-R116), so a slow caller cannot hold a worker and a connection. Whole
/// seconds, rounded up.
fn request_timeout_seconds(milliseconds: Option<&str>) -> Result<u64> {
    let milliseconds = milliseconds.unwrap_or("30000");
    milliseconds
        .parse::<u64>()
        .map(|ms| ms.div_ceil(1000))
        .map_err(|_| Error::string("WPM_REQUEST_TIMEOUT_MS must be a whole number of milliseconds"))
}

/// The one browser origin that may call the API directly. Production requires it and never takes `*`, so a forgotten setting stops
/// the service and does not open it to every origin.
fn cors_origins(origin: Option<&str>, required: bool) -> Result<Vec<String>> {
    match origin.map(str::trim).filter(|o| !o.is_empty()) {
        Some("*") => Err(Error::string("WPM_CORS_ORIGIN is one origin, never `*`")),
        Some(origin) => Ok(vec![origin.to_string()]),
        None if required => Err(Error::string(
            "environment variable `WPM_CORS_ORIGIN` not found",
        )),
        None => Ok(Vec::new()),
    }
}

/// The application.
impl App {
    /// The service's version, with the build it came from.
    #[must_use]
    pub fn version() -> String {
        format!(
            "{} ({})",
            env!("CARGO_PKG_VERSION"),
            option_env!("BUILD_SHA")
                .or(option_env!("GITHUB_SHA"))
                .unwrap_or("dev")
        )
    }

    /// The configuration for `environment`: `axa`'s from the environment, with what WPM decides for itself.
    ///
    /// # Errors
    /// A setting is missing or does not parse.
    pub fn config(environment: &Environment) -> Result<Config> {
        let mut config = Config::from_env(environment, &defaults())?;
        if *environment == Environment::Production {
            config.http.timeout_seconds = request_timeout_seconds(
                crate::compat::env_var("WPM_REQUEST_TIMEOUT_MS").as_deref(),
            )?;
            config.http.compression = true;
            config.http.cors_origins =
                cors_origins(crate::compat::env_var("WPM_CORS_ORIGIN").as_deref(), true)?;
        } else {
            // Only production lets a browser call the API directly: the UI calls it through its own server.
            config.http.cors_origins = Vec::new();
        }
        if environment.to_string() == "demo" {
            // `axa` reads any other name as production-like. The container demo runs in the clear, on a database it migrates itself.
            config.http.timeout_seconds = 0;
            config.http.compression = false;
            config.http.hsts = false;
            config.database.auto_migrate = crate::compat::env_var("DB_AUTO_MIGRATE")
                .is_none_or(|value| crate::auth::parse_bool(&value));
        }
        if config.logger.override_filter.is_none() {
            config.logger.override_filter = Some(log_filter(config.logger.level));
        }
        Ok(config)
    }

    /// How the service logs, for a process that serves nothing.
    ///
    /// # Panics
    /// Never in practice: the development defaults need no settings.
    #[must_use]
    pub fn logging() -> Logger {
        let environment = Environment::from_env();
        Self::config(&environment).map_or_else(
            |_| {
                let mut logger = Config::from_env(&Environment::Development, &defaults())
                    .map(|config| config.logger)
                    .expect("the development defaults need no settings");
                logger.override_filter = Some(log_filter(logger.level));
                logger
            },
            |config| config.logger,
        )
    }

    /// Connect, and build the context handlers receive.
    ///
    /// # Errors
    /// The database cannot be reached.
    pub async fn context(environment: Environment, config: Config) -> Result<AppContext> {
        // Before anything opens a connection that may be `rediss://`.
        crate::shared_cache::install_tls_provider();
        let db = axa::db::connect(&config.database).await?;
        Ok(AppContext::new(environment, db, config))
    }

    /// The full router: `axa`'s standard middleware inside WPM's own layers.
    ///
    /// Errors become problems here as well as in [`Self::after_routes`], because `axa` compresses and a compressed body cannot be
    /// read after it: this pass has what a handler refused, that one what `axa` and WPM's own layers refused (WPM-D186).
    ///
    /// # Errors
    /// A boot check refuses to start.
    pub async fn router(ctx: &AppContext) -> Result<AxumRouter> {
        let router = app_routes()
            .into_router(ctx.clone())
            .layer(axum::middleware::from_fn(crate::problem::problem_mw));
        let router = axa::middleware::apply(router, &ctx.config.http)?;
        Self::after_routes(router, ctx).await
    }

    async fn db(ctx: &AppContext, command: DbCommand) -> Result<()> {
        match command {
            DbCommand::Migrate => Migrator::up(&ctx.db, None).await?,
            DbCommand::Down { steps } => Migrator::down(&ctx.db, Some(steps)).await?,
            DbCommand::Reset => Migrator::fresh(&ctx.db).await?,
            DbCommand::Status => Migrator::status(&ctx.db).await?,
            DbCommand::Truncate => Self::truncate(ctx).await?,
        }
        Ok(())
    }

    /// Run the command line.
    ///
    /// # Errors
    /// The command failed.
    pub async fn run(cli: Cli) -> Result<()> {
        let environment = cli
            .environment
            .as_deref()
            .map_or_else(Environment::from_env, Environment::parse);
        let mut config = Self::config(&environment)?;
        if matches!(cli.command, Command::Check) {
            // WPM's own config, not `axa::cli::check`, which re-reads the environment and so reports `axa`'s timeout and CORS, not these.
            for (name, value) in config.settings() {
                println!("{name}={value}");
            }
            return Ok(());
        }
        config.apply_cli(&cli);
        let ctx = Self::context(environment, config).await?;
        axa::logger::init(env!("CARGO_CRATE_NAME"), &ctx.config.logger);
        if ctx.config.database.auto_migrate && !matches!(cli.command, Command::Db { .. }) {
            Migrator::up(&ctx.db, None).await?;
        }
        match cli.command {
            Command::Start { .. } => {
                let router = Self::router(&ctx).await?;
                server::serve(router, &ctx.config.server).await
            }
            Command::Db { command } => Self::db(&ctx, command).await,
            Command::Task { name, params } => {
                let mut tasks = Tasks::default();
                Self::register_tasks(&mut tasks);
                let Some(name) = name else {
                    for task in tasks.list() {
                        println!("{}\t{}", task.name, task.detail);
                    }
                    return Ok(());
                };
                tasks.run(&name, &ctx, &parse_vars(&params)?).await
            }
            Command::Check => Ok(()), // answered above
        }
    }

    #[allow(clippy::too_many_lines)] // boot checks first, then the layers
    /// The boot checks, then WPM's layers around the router.
    ///
    /// # Errors
    /// A boot check refuses to start.
    pub async fn after_routes(router: AxumRouter, ctx: &AppContext) -> Result<AxumRouter> {
        // Seed the active backend's verifier (boot-time key/JWKS fetch;
        // env fallback — the service always boots), then keep keys +
        // policy fresh.
        let production = ctx.environment == Environment::Production;
        // The shared cache: Redis in production, non-personal lookups only (WPM-D78).
        let cache =
            crate::shared_cache::Cache::from_env().map_err(|cause| Error::string(&cause))?;
        crate::shared_cache::install(std::sync::Arc::new(cache));
        let redis = crate::compat::env_var("REDIS_URL").filter(|uri| !uri.is_empty());
        if production && redis.is_none() {
            tracing::warn!(
                "REDIS_URL is not set, so each instance keeps its own in-memory cache; set it to share one"
            );
        }
        if let Some(uri) = &redis {
            let allow = crate::compat::env_var("WPM_ALLOW_PLAINTEXT_REDIS")
                .is_some_and(|v| auth::parse_bool(&v));
            match crate::hardening::redis_check(production, uri, allow) {
                Ok(Some(warning)) => tracing::warn!("{warning}"),
                Ok(None) => {}
                Err(message) => return Err(Error::string(&message)),
            }
        }
        auth::startup_check(
            auth::require_auth(),
            production,
            auth::key_source_configured(),
        )
        .map_err(|message| Error::string(&message))?;
        if auth::require_auth() {
            if !auth::key_source_configured() {
                tracing::warn!(
                    "sign-in is enforced but no token key source is configured: every request will be refused"
                );
            }
        } else {
            warn_auth_off();
            tokio::spawn(async {
                loop {
                    tokio::time::sleep(std::time::Duration::from_secs(600)).await;
                    warn_auth_off();
                }
            });
        }
        // Production refuses a plaintext connection to the database (WPM-R116).
        let allow_plaintext = crate::compat::env_var("WPM_ALLOW_PLAINTEXT_DATABASE")
            .is_some_and(|v| auth::parse_bool(&v));
        match crate::hardening::database_check(
            production,
            &ctx.config.database.uri,
            allow_plaintext,
        ) {
            Ok(Some(warning)) => tracing::warn!("{warning}"),
            Ok(None) => {}
            Err(message) => return Err(Error::string(&message)),
        }
        // Equality monitoring is special-category data and off unless a lawful basis is recorded
        // (WPM-R125, WPM-D74). A half-configured deployment does not start.
        match controllers::equality_monitoring::gate_from_env() {
            Ok(crate::rules::equality_monitoring::Gate::On {
                basis,
                config,
                floor,
            }) => {
                tracing::warn!(
                    categories = config.offered().len(),
                    floor,
                    "equality monitoring is ENABLED on the lawful basis recorded as: {basis}"
                );
            }
            Ok(crate::rules::equality_monitoring::Gate::Off) => {}
            Err(message) => return Err(Error::string(&message)),
        }
        // Workplace health requirements hold health data and are off unless the deployer records a
        // lawful basis (WPM-R128, WPM-D75).
        if let Some(basis) = rules::health_requirements::gate(
            crate::compat::env_var("WPM_HEALTH_REQUIREMENTS_BASIS").as_deref(),
        ) {
            tracing::warn!(
                "workplace health requirements are ENABLED on the lawful basis recorded as: {basis}"
            );
        }
        // The key that signs list cursors (WPM-D189). A value that is set but unusable stops the start; none is a key for
        // this process, which is enough for one instance and refuses another instance's cursors with a `422`.
        let cursor_key =
            controllers::cursor_key_from(crate::compat::env_var("WPM_CURSOR_KEY").as_deref())
                .map_err(|message| Error::string(&message))?;
        if cursor_key.is_none() && ctx.environment.is_production_like() {
            tracing::warn!(
                "WPM_CURSOR_KEY is not set, so each instance signs list cursors with its own key; set it when more than one runs"
            );
        }
        controllers::install_cursor_key(cursor_key);
        // The failed sign-in lockout (WPM-R242, WPM-D190). A count that is set but unreadable stops the start.
        let lockout = crate::security::lockout_from(
            crate::compat::env_var("LOCKOUT_CLIENT_ADDRESS_AFTER_COUNT").as_deref(),
        )
        .map_err(|message| Error::string(&message))?;
        if lockout.is_some() {
            tracing::info!(
                "a client address is locked out for up to 15 minutes after LOCKOUT_CLIENT_ADDRESS_AFTER_COUNT 401s"
            );
        }
        // The variable no longer does anything (WPM-D187). Refusing to start would take a running service down over it.
        if let Some(warning) = crate::security::trust_forwarded_warning(
            crate::compat::env_var("WPM_TRUST_FORWARDED").as_deref(),
        ) {
            tracing::warn!("{warning}");
        }
        auth::init().await;
        auth::spawn_key_refresh();
        auth::spawn_policy_watcher();
        // Later layers are outer: headers wrap everything (so a 401 or 429
        // carries them), the rate limit runs before the token is checked.
        // Need-to-know (WPM-R114) sits inside the sign-in guard: a token is verified first.
        // The idempotency check (WPM-R132) is innermost: by then the caller is signed in and allowed
        // past need-to-know, so a replay is only ever of the caller's own earlier success.
        // The MCP endpoint calls the API through a copy of it that has the sign-in and need-to-know layers, so a tool is
        // refused exactly as the route is.
        let api = router
            .clone()
            .layer(axum::middleware::from_fn_with_state(
                ctx.clone(),
                crate::need_to_know::need_to_know_mw,
            ))
            .layer(axum::middleware::from_fn(require_auth_mw))
            .layer(axum::middleware::from_fn(crate::problem::problem_mw));
        let router = controllers::mcp::mount(router, api, ctx.clone());
        let router = router
            .layer(axum::middleware::from_fn_with_state(
                ctx.clone(),
                crate::idempotency::idempotency_mw,
            ))
            .layer(axum::middleware::from_fn_with_state(
                ctx.clone(),
                crate::need_to_know::need_to_know_mw,
            ))
            .layer(axum::middleware::from_fn(require_auth_mw))
            // A 401 says it wants a bearer token (RFC 6750) and, on /mcp, where the OAuth metadata is (WPM-T304).
            .layer(axum::middleware::from_fn(controllers::mcp::challenge_mw))
            .layer(axum::middleware::from_fn(
                crate::version::require_version_mw,
            ));
        // Outside sign-in, so it sees every 401, and inside the rate limit, which puts the address on the request
        // (WPM-R242, WPM-D190). Off unless a count is set.
        let router = match lockout {
            Some(limiter) => router.layer(axum::middleware::from_fn_with_state(
                limiter,
                axa::ratelimit::lock_out_failures,
            )),
            None => router,
        };
        Ok(router
            // Outside `axa`'s layers, so it resolves the address itself, from the same list (WPM-D187).
            .layer(axum::middleware::from_fn_with_state(
                std::sync::Arc::new(crate::security::RateLimits::from_env(
                    ctx.config.http.trusted_proxies.clone(),
                )),
                crate::security::rate_limit_mw,
            ))
            // Outside everything that refuses a request, so a 401, 406, 408, 429 or panic is a problem too (WPM-R238).
            .layer(axum::middleware::from_fn(crate::problem::problem_mw))
            .layer(axum::middleware::from_fn(
                crate::security::security_headers_mw,
            ))
            // Outermost: every response, a 401 or 429 included, carries the request id, and every
            // request leaves one line with no query string or identifier (WPM-R122).
            .layer(axum::middleware::from_fn(
                crate::request_log::request_log_mw,
            )))
    }

    /// Every task, for `task <name>` and the schedule.
    pub fn register_tasks(tasks: &mut Tasks) {
        tasks.register(tasks::seed::Seed);
        tasks.register(tasks::scim_push::ScimPush);
        tasks.register(tasks::webhook_deliver::WebhookDeliver);
        tasks.register(tasks::rota_reminders::RotaReminders);
        tasks.register(tasks::pay_progression_reminders::PayProgressionReminders);
        tasks.register(tasks::policy_reminders::PolicyReminders);
        tasks.register(tasks::snapshot::SnapshotHeadcount);
        tasks.register(tasks::replay_erasures::ReplayErasures);
        tasks.register(tasks::import_framework::ImportFramework);
        tasks.register(tasks::engagement_end_reminders::EngagementEndReminders);
        tasks.register(tasks::qualification_expiry_reminders::QualificationExpiryReminders);
        tasks.register(tasks::coverage_expiry_reminders::CoverageExpiryReminders);
        tasks.register(tasks::early_warning_sweep::EarlyWarningSweep);
        tasks.register(tasks::oversight_pack_freeze::OversightPackFreeze);
        tasks.register(tasks::import_esco::ImportEsco);
        tasks.register(tasks::import_sfia::ImportSfia);
        tasks.register(tasks::import_sfiaplus::ImportSfiaplus);
        tasks.register(tasks::verify_audit_chain::VerifyAuditChain);
        // tasks-inject (do not remove)
    }

    /// Empty the tables tests depend on.
    ///
    /// # Errors
    /// Propagates a database failure.
    pub async fn truncate(ctx: &AppContext) -> Result<()> {
        truncate_table(&ctx.db, IdempotencyKeys).await?;
        truncate_table(&ctx.db, EventOutbox).await?;
        truncate_table(&ctx.db, PostFundings).await?;
        truncate_table(&ctx.db, ConversionPlans).await?;
        truncate_table(&ctx.db, EngagementDecisions).await?;
        truncate_table(&ctx.db, EngagementReminders).await?;
        truncate_table(&ctx.db, WorkerHealthRecords).await?;
        truncate_table(&ctx.db, HealthRequirements).await?;
        truncate_table(&ctx.db, EqualityDeclarations).await?;
        truncate_table(&ctx.db, FlexibleWorkingRequests).await?;
        truncate_table(&ctx.db, Resignations).await?;
        truncate_table(&ctx.db, WorkerContactDetails).await?;
        truncate_table(&ctx.db, EngagementStatusAssessments).await?;
        truncate_table(&ctx.db, WorkerContractorDetails).await?;
        truncate_table(&ctx.db, EngagementExtensions).await?;
        truncate_table(&ctx.db, StartDecisions).await?;
        truncate_table(&ctx.db, CapacitySettings).await?;
        truncate_table(&ctx.db, PartnerCommitments).await?;
        truncate_table(&ctx.db, ProgrammeDemands).await?;
        truncate_table(&ctx.db, SkillPoolMembers).await?;
        truncate_table(&ctx.db, SkillPools).await?;
        truncate_table(&ctx.db, AuditLogs).await?;
        truncate_table(&ctx.db, EntitlementAcknowledgements).await?;
        truncate_table(&ctx.db, WellbeingEntitlements).await?;
        truncate_table(&ctx.db, PulseResponses).await?;
        truncate_table(&ctx.db, PulseSurveys).await?;
        truncate_table(&ctx.db, AppraisalResponses).await?;
        truncate_table(&ctx.db, AppraisalNominations).await?;
        truncate_table(&ctx.db, Appraisals).await?;
        truncate_table(&ctx.db, Notifications).await?;
        truncate_table(&ctx.db, AdjustmentRequests).await?;
        truncate_table(&ctx.db, ErgonomicItems).await?;
        truncate_table(&ctx.db, ErgonomicAssessments).await?;
        truncate_table(&ctx.db, ProgramPlacements).await?;
        truncate_table(&ctx.db, EarlyCareerPrograms).await?;
        truncate_table(&ctx.db, PipelineMembers).await?;
        truncate_table(&ctx.db, TalentPipelines).await?;
        truncate_table(&ctx.db, DevelopmentPlanItems).await?;
        truncate_table(&ctx.db, DevelopmentPlans).await?;
        truncate_table(&ctx.db, AssessmentResults).await?;
        truncate_table(&ctx.db, Assessments).await?;
        truncate_table(&ctx.db, AssessmentInstruments).await?;
        truncate_table(&ctx.db, Benchmarks).await?;
        truncate_table(&ctx.db, Payslips).await?;
        truncate_table(&ctx.db, PayrollRuns).await?;
        truncate_table(&ctx.db, SuccessionCandidates).await?;
        truncate_table(&ctx.db, SuccessionPlans).await?;
        truncate_table(&ctx.db, TrainingEnrollments).await?;
        truncate_table(&ctx.db, FeedbackEntries).await?;
        truncate_table(&ctx.db, Goals).await?;
        truncate_table(&ctx.db, Reviews).await?;
        truncate_table(&ctx.db, ReviewCycles).await?;
        truncate_table(&ctx.db, BenefitEnrollments).await?;
        truncate_table(&ctx.db, BenefitPlans).await?;
        truncate_table(&ctx.db, ShiftAssignments).await?;
        truncate_table(&ctx.db, Shifts).await?;
        truncate_table(&ctx.db, LeaveRequests).await?;
        truncate_table(&ctx.db, LeaveEntitlements).await?;
        truncate_table(&ctx.db, TimeEntries).await?;
        truncate_table(&ctx.db, OnboardingItems).await?;
        truncate_table(&ctx.db, Interviews).await?;
        truncate_table(&ctx.db, Applications).await?;
        truncate_table(&ctx.db, Candidates).await?;
        truncate_table(&ctx.db, Requisitions).await?;
        truncate_table(&ctx.db, Workers).await?;
        Ok(())
    }
}

impl TestApp for App {
    /// Boot under `test`: a fresh schema unless `WPM_TEST_FRESH_SCHEMA=false`, then emptied unless `WPM_TEST_TRUNCATE=false`. With
    /// `false` nothing touches the schema, because the role drill runs as a role that may not (`scripts/verify-db-roles.sh`).
    async fn boot_test() -> Result<BootResult> {
        let flag = |name: &str| crate::compat::env_var(name).is_none_or(|v| v != "false");
        let environment = Environment::Test;
        let config = Self::config(&environment)?;
        let app_context = Self::context(environment, config).await?;
        if flag("WPM_TEST_FRESH_SCHEMA") {
            Migrator::fresh(&app_context.db).await?;
        }
        if flag("WPM_TEST_TRUNCATE") {
            Self::truncate(&app_context).await?;
        }
        let router = Self::router(&app_context).await?;
        Ok(BootResult {
            app_context,
            router: Some(router),
        })
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    /// `axa::cli::parse` refuses `db down`, `db reset` and `db truncate` (WPM-R237, WPM-D185). `Cli::parse()` would not.
    #[test]
    fn the_binary_reads_its_arguments_through_the_guard() {
        let main = include_str!("bin/main.rs");
        assert!(main.contains("axa::cli::parse("));
        assert!(!main.contains("Cli::parse("));
    }

    #[test]
    fn a_request_timeout_rounds_up_to_whole_seconds() {
        assert_eq!(request_timeout_seconds(None).unwrap(), 30);
        assert_eq!(request_timeout_seconds(Some("1500")).unwrap(), 2);
        assert!(request_timeout_seconds(Some("soon")).is_err());
    }

    #[test]
    fn production_needs_one_origin_and_never_every_origin() {
        assert!(cors_origins(None, true).is_err());
        assert!(cors_origins(Some(" "), true).is_err());
        assert!(cors_origins(Some("*"), true).is_err());
        assert_eq!(
            cors_origins(Some("https://hr.example"), true).unwrap(),
            ["https://hr.example"]
        );
    }

    #[test]
    fn the_request_tracing_never_logs_a_uri() {
        assert!(log_filter(axa::logger::LogLevel::Debug).ends_with("tower_http=warn"));
    }
}
