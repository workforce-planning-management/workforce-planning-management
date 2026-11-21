//! Loading the ABAC policy (WPM-D52): a policy file that cannot be read, and policy JSON that is not a policy, each fall back
//! to the built-in default with a warning, so the service always starts protected; a reload swaps the live policy.

use serial_test::serial;
use workforce_planning_management::auth::{policy, policy_from_env, reload_policy};

const INLINE: &str = "WORKFORCE_PLANNING_MANAGEMENT_ABAC_POLICY";
const FILE: &str = "WORKFORCE_PLANNING_MANAGEMENT_ABAC_POLICY_FILE";
const REFERENCE: &str = concat!(
    env!("CARGO_MANIFEST_DIR"),
    "/config/abac-policy.reference.json"
);

fn set(name: &str, value: Option<&str>) {
    unsafe {
        match value {
            Some(v) => std::env::set_var(name, v),
            None => std::env::remove_var(name),
        }
    }
}

#[tokio::test]
#[serial]
async fn an_unreadable_or_invalid_policy_falls_back_to_the_default_and_a_reload_swaps_it() {
    let default = authentication_verifier::Policy::default_policy();

    // A file that is not there, and JSON that is not JSON or not a policy: the default, never no policy.
    set(INLINE, None);
    set(FILE, Some("/nonexistent/abac-policy.json"));
    assert_eq!(policy_from_env(), default, "an unreadable file");
    set(FILE, None);
    for bad in ["{ not json", r#"{"rules": "not a list"}"#] {
        set(INLINE, Some(bad));
        assert_eq!(policy_from_env(), default, "{bad}");
    }

    // The reference policy is not the default; a reload puts it live, and another reload, with nothing set, restores the default.
    set(INLINE, None);
    set(FILE, Some(REFERENCE));
    let reference = policy_from_env();
    assert_ne!(
        reference, default,
        "the reference policy differs from the default"
    );
    reload_policy();
    assert_eq!(*policy().current(), reference);
    set(FILE, None);
    reload_policy();
    assert_eq!(*policy().current(), default);
}
