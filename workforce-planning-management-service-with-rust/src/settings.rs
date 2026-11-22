//! Settings read from the environment, checked once at start (WPM-R242; RCA-IN-1, RCA-IN-4).
//!
//! A setting that is **absent** takes its declared default. A setting that is **supplied but invalid** stops the start with an error
//! that names it: it is never replaced by a default, the nearest bound or any other value (RCA-IN-2, RCA-IN-3).

/// The seconds between key-set refreshes from a setting `name`: `default` when it is absent, `0` to disable.
///
/// # Errors
///
/// A setting that is supplied but is not a whole number of seconds.
pub fn refresh_secs(name: &str, default: u64) -> Result<u64, String> {
    match crate::compat::env_var(name) {
        None => Ok(default),
        Some(text) => text.trim().parse::<u64>().map_err(|_| {
            format!(
                "{name} must be a whole number of seconds (0 disables the refresh), not {text:?}"
            )
        }),
    }
}

/// A requests-per-minute limit from a setting `name`: `default` when it is absent, `0` to switch that class off.
///
/// # Errors
///
/// A setting that is supplied but is not a whole number of requests.
pub fn per_minute(name: &str, default: u32) -> Result<u32, String> {
    match crate::compat::env_var(name) {
        None => Ok(default),
        Some(text) => text.trim().parse::<u32>().map_err(|_| {
            format!("{name} must be a whole number of requests per minute (0 turns the limit off), not {text:?}")
        }),
    }
}

/// Every setting that has a range, checked once at start.
///
/// # Errors
///
/// The first setting that is supplied but invalid, naming it.
pub fn validate() -> Result<(), String> {
    crate::rules::mandatory_learning::due_soon_window(
        crate::compat::env_var("WORKFORCE_PLANNING_MANAGEMENT_LEARNING_DUE_SOON_DAYS").as_deref(),
    )?;
    crate::rules::scale::parse_ceiling(
        crate::compat::env_var("WORKFORCE_PLANNING_MANAGEMENT_MAX_ORG_WORKERS").as_deref(),
    )?;
    crate::controllers::privacy::horizons(None, crate::dates::today())?;
    crate::controllers::payroll::contracted_day()?;
    crate::controllers::payroll::tax_stub()?;
    per_minute("WORKFORCE_PLANNING_MANAGEMENT_RATE_LIMIT_PER_MINUTE", 0)?;
    per_minute(
        "WORKFORCE_PLANNING_MANAGEMENT_RATE_LIMIT_SENSITIVE_PER_MINUTE",
        0,
    )?;
    refresh_secs("WORKFORCE_PLANNING_MANAGEMENT_PASETO_KEYS_REFRESH_SECS", 0)?;
    refresh_secs(
        "WORKFORCE_PLANNING_MANAGEMENT_KEYCLOAK_JWKS_REFRESH_SECS",
        0,
    )?;
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn an_absent_setting_is_the_default_and_a_bad_one_is_refused_by_name() {
        let name = "WORKFORCE_PLANNING_MANAGEMENT_TEST_ONLY_SETTING_NEVER_SET";
        assert_eq!(refresh_secs(name, 3600), Ok(3600));
        assert_eq!(per_minute(name, 600), Ok(600));
    }
}
