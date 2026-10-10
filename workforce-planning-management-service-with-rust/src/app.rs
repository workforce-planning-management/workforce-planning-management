//! loco.rs application wiring: the [`App`] `Hooks` implementation that
//! registers routes, the blanket auth guard, the API-version
//! middleware, and the truncate lifecycle for
//! `workforce-planning-management-service`.

use async_trait::async_trait;
use axum::{
    Router as AxumRouter,
    extract::Request,
    middleware::Next,
    response::{IntoResponse, Response},
};
use loco_rs::{
    Result,
    app::{AppContext, Hooks, Initializer},
    bgworker::Queue,
    boot::{BootResult, StartMode, create_app},
    config::Config,
    controller::AppRoutes,
    db::truncate_table,
    environment::Environment,
    task::Tasks,
};
use migration::Migrator;
use std::path::Path;

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

/// The loco.rs application hooks for `workforce-planning-management-service`.
pub struct App;
#[async_trait]
impl Hooks for App {
    fn app_name() -> &'static str {
        env!("CARGO_CRATE_NAME")
    }

    fn app_version() -> String {
        format!(
            "{} ({})",
            env!("CARGO_PKG_VERSION"),
            option_env!("BUILD_SHA")
                .or(option_env!("GITHUB_SHA"))
                .unwrap_or("dev")
        )
    }

    async fn boot(
        mode: StartMode,
        environment: &Environment,
        config: Config,
    ) -> Result<BootResult> {
        create_app::<Self, Migrator>(mode, environment, config).await
    }

    async fn initializers(_ctx: &AppContext) -> Result<Vec<Box<dyn Initializer>>> {
        Ok(vec![])
    }

    fn routes(_ctx: &AppContext) -> AppRoutes {
        AppRoutes::with_default_routes()
            .add_route(controllers::hr_core::routes())
            .add_route(controllers::acquisition::routes())
            .add_route(controllers::assessments::routes())
            .add_route(controllers::workforce::routes())
            .add_route(controllers::development::routes())
            .add_route(controllers::learning::routes())
            .add_route(controllers::roles::routes())
            .add_route(controllers::cpd::routes())
            .add_route(controllers::mobility::routes())
            .add_route(controllers::lms::routes())
            .add_route(controllers::change::routes())
            .add_route(controllers::planning::routes())
            .add_route(controllers::capacity::routes())
            .add_route(controllers::engagements::routes())
            .add_route(controllers::me::routes())
            .add_route(controllers::resignations::routes())
            .add_route(controllers::flexible_working::routes())
            .add_route(controllers::equality_monitoring::routes())
            .add_route(controllers::health_requirements::routes())
            .add_route(controllers::esco::routes())
            .add_route(controllers::framework_roles::routes())
            .add_route(controllers::career::routes())
            .add_route(controllers::reporting::routes())
            .add_route(controllers::groups::routes())
            .add_route(controllers::announcements::routes())
            .add_route(controllers::contacts::routes())
            .add_route(controllers::directory::routes())
            .add_route(controllers::pay_scales::routes())
            .add_route(controllers::job_levels::routes())
            .add_route(controllers::grades::routes())
            .add_route(controllers::pay_positions::routes())
            .add_route(controllers::expenses::routes())
            .add_route(controllers::rotas::routes())
            .add_route(controllers::handover::routes())
            .add_route(controllers::movements::routes())
            .add_route(controllers::skill_gaps::routes())
            .add_route(controllers::training_plan::routes())
            .add_route(controllers::rota_swaps::routes())
            .add_route(controllers::transfers::routes())
            .add_route(controllers::talent::routes())
            .add_route(controllers::intelligence::routes())
            .add_route(controllers::payroll::routes())
            .add_route(controllers::wellbeing::routes())
            .add_route(controllers::appraisals::routes())
            .add_route(controllers::privacy::routes())
            .add_route(controllers::notifications::routes())
            .add_route(controllers::ergonomics::routes())
            .add_route(controllers::adjustments::routes())
            .add_route(controllers::organizations::routes())
            .add_route(controllers::audits::routes())
            .add_route(controllers::docs::routes())
            .add_route(controllers::metrics::routes())
            .add_route(crate::security::routes())
    }

    async fn after_routes(router: AxumRouter, ctx: &AppContext) -> Result<AxumRouter> {
        // Seed the active backend's verifier (boot-time key/JWKS fetch;
        // env fallback — the service always boots), then keep keys +
        // policy fresh.
        let production = ctx.environment == Environment::Production;
        auth::startup_check(
            auth::require_auth(),
            production,
            auth::key_source_configured(),
        )
        .map_err(|message| loco_rs::Error::string(&message))?;
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
            Err(message) => return Err(loco_rs::Error::string(&message)),
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
            Err(message) => return Err(loco_rs::Error::string(&message)),
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
        auth::init().await;
        auth::spawn_key_refresh();
        auth::spawn_policy_watcher();
        // Later layers are outer: headers wrap everything (so a 401 or 429
        // carries them), the rate limit runs before the token is checked.
        // Need-to-know (WPM-R114) sits inside the sign-in guard: a token is verified first.
        Ok(router
            .layer(axum::middleware::from_fn_with_state(
                ctx.clone(),
                crate::need_to_know::need_to_know_mw,
            ))
            .layer(axum::middleware::from_fn(require_auth_mw))
            .layer(axum::middleware::from_fn(
                crate::version::require_version_mw,
            ))
            .layer(axum::middleware::from_fn(crate::security::rate_limit_mw))
            .layer(axum::middleware::from_fn(
                crate::security::security_headers_mw,
            )))
    }

    async fn connect_workers(_ctx: &AppContext, _queue: &Queue) -> Result<()> {
        Ok(())
    }

    fn register_tasks(tasks: &mut Tasks) {
        tasks.register(tasks::seed::Seed);
        tasks.register(tasks::rota_reminders::RotaReminders);
        tasks.register(tasks::pay_progression_reminders::PayProgressionReminders);
        tasks.register(tasks::snapshot::SnapshotHeadcount);
        tasks.register(tasks::replay_erasures::ReplayErasures);
        tasks.register(tasks::import_framework::ImportFramework);
        tasks.register(tasks::import_esco::ImportEsco);
        tasks.register(tasks::verify_audit_chain::VerifyAuditChain);
        // tasks-inject (do not remove)
    }

    async fn truncate(ctx: &AppContext) -> Result<()> {
        truncate_table(&ctx.db, EventOutbox).await?;
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

    async fn seed(_ctx: &AppContext, _base: &Path) -> Result<()> {
        Ok(())
    }
}
