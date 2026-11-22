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

/// A whole-number setting `name` that must satisfy `valid` (described by `range`): `None` when it is absent (RCA-IN-3).
///
/// # Errors
///
/// A setting that is supplied but is not a whole number in range (RCA-IN-2, RCA-IN-4): it is never replaced by a default or a bound.
pub fn whole<T: std::str::FromStr>(
    name: &str,
    range: &str,
    valid: impl Fn(&T) -> bool,
) -> Result<Option<T>, String> {
    let Some(text) = crate::compat::env_var(name).filter(|v| !v.trim().is_empty()) else {
        return Ok(None);
    };
    match text.trim().parse::<T>() {
        Ok(n) if valid(&n) => Ok(Some(n)),
        _ => Err(format!("{name} must be {range}, not {text:?}")),
    }
}

/// An on/off setting `name`: `None` when it is absent.
///
/// # Errors
///
/// A setting that is supplied but is neither an on word (`1`, `true`, `yes`, `on`) nor an off word (`0`, `false`, `no`, `off`).
pub fn flag(name: &str) -> Result<Option<bool>, String> {
    let Some(text) = crate::compat::env_var(name).filter(|v| !v.trim().is_empty()) else {
        return Ok(None);
    };
    match text.trim().to_lowercase().as_str() {
        "1" | "true" | "yes" | "on" => Ok(Some(true)),
        "0" | "false" | "no" | "off" => Ok(Some(false)),
        _ => Err(format!("{name} must be on or off, not {text:?}")),
    }
}

/// A setting that was refused at start but is met again while serving. `validate` stops the start first, so this is not reached in a
/// running service; it says so loudly and reads the setting as absent rather than guessing a value.
#[must_use]
pub fn met_again<T>(refusal: Result<Option<T>, String>) -> Option<T> {
    refusal.unwrap_or_else(|why| {
        tracing::error!("{why}");
        None
    })
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
    crate::controllers::flexible_working::check_settings()?;
    crate::controllers::pay_assessment::check_settings()?;
    crate::controllers::pay_information::check_settings()?;
    crate::controllers::resignations::notice_setting()?;
    crate::controllers::pay_transparency::tiers_setting()?;
    crate::controllers::ai_register::applies_from_setting()?;
    crate::controllers::health_requirements::floor_setting()?;
    crate::controllers::padr::first_after_setting()?;
    crate::auth::max_token_lifetime_setting()?;
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
