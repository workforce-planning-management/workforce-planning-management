//! Multi-organization membership round trip: grant (both the
//! employment-linked and staff-only shapes) → admin listing by
//! `person_ref` → duplicate-grant `422` → revoke → the row drops out
//! of the listing. `/me/organizations` (token-derived) is only
//! exercised for its documented signed-out-caller behaviour here —
//! this crate's DB-gated request-test harness never mints a bearer
//! token (no test anywhere sets an `Authorization` header), so the
//! "sees my own memberships" path is covered by code review of
//! `memberships_for` (shared with the `?person_ref=` listing this
//! suite does exercise), not by a request test.

use authentication_verifier::Claims;
use loco_rs::testing::prelude::*;
use serde_json::{Value, json};
use serial_test::serial;
use workforce_planning_management_service::app::App;
use workforce_planning_management_service::models::memberships;

use super::{a_person, an_org, seed_worker};

/// A `Claims` value for `person_ref`, for calling
/// `memberships::caller_scope_refs` directly (the crate's request-test
/// harness never mints a bearer token, so the confederation
/// scope-expansion test below calls the model function in-process
/// rather than through `/api/me/organizations/scope`).
fn claims_for(person_ref: &str) -> Claims {
    let sub = person_ref
        .strip_prefix("person:")
        .expect("person_ref")
        .to_string();
    Claims {
        sub,
        email: "test@example.com".into(),
        name: "Test Person".into(),
        iss: "authentication-service".into(),
        aud: "main-x-service".into(),
        exp: 2_000_000_000,
        iat: 1_900_000_000,
        nbf: None,
        sid: "test-sid".into(),
        scope: Vec::new(),
        roles: Vec::new(),
        attrs: std::collections::BTreeMap::new(),
    }
}

#[tokio::test]
#[serial]
#[ignore = "requires PostgreSQL (config/test.yaml); run with `cargo test -- --ignored`"]
async fn organization_membership_round_trip() {
    request::<App, _, _>(|request, _ctx| async move {
        let org = an_org();
        let worker_pid = seed_worker!(&request, &org, "OM-1", None).await;
        let worker: Value = request
            .get(&format!("/api/workers/{worker_pid}"))
            .await
            .json();
        let person_ref = worker["person_ref"].as_str().unwrap().to_string();
        assert_eq!(worker["organization_ref"].as_str().unwrap(), org);

        // `/me/organizations` with no bearer token: signed out sees
        // nothing, never a 401 — the documented convention.
        let mine = request.get("/api/me/organizations").await;
        mine.assert_status_ok();
        let mine: Value = mine.json();
        assert!(mine.as_array().unwrap().is_empty());

        // Malformed inputs, ahead of anything valid existing.
        assert_eq!(
            request
                .post("/api/organization-memberships")
                .json(&json!({
                    "person_ref": "not-a-urn", "organization_ref": org,
                    "role": "member", "starts_on": "2026-01-01",
                }))
                .await
                .status_code(),
            422,
            "malformed person_ref refused"
        );
        assert_eq!(
            request
                .post("/api/organization-memberships")
                .json(&json!({
                    "person_ref": person_ref, "organization_ref": org,
                    "role": "superadmin", "starts_on": "2026-01-01",
                }))
                .await
                .status_code(),
            422,
            "role outside the closed vocabulary refused"
        );

        // A worker_pid ties a grant to that worker's own organization
        // (there's no DB foreign key anywhere in this schema, so this
        // app-level check is the only referential integrity there is)
        // — a mismatched organization_ref alongside a real worker_pid
        // is refused, even though person_ref matches.
        let mismatched_org = an_org();
        assert_eq!(
            request
                .post("/api/organization-memberships")
                .json(&json!({
                    "person_ref": person_ref, "organization_ref": mismatched_org,
                    "worker_pid": worker_pid, "role": "member",
                    "starts_on": "2026-01-01",
                }))
                .await
                .status_code(),
            422,
            "worker_pid's real organization_ref must match the payload's"
        );

        // Case (a): the employment-linked grant — worker_pid set,
        // organization_ref matches that worker's real org.
        let membership: Value = request
            .post("/api/organization-memberships")
            .json(&json!({
                "person_ref": person_ref, "organization_ref": org,
                "worker_pid": worker_pid, "role": "member",
                "starts_on": "2026-01-01",
            }))
            .await
            .json();
        let membership_pid = membership["pid"].as_str().expect("membership pid");

        // Case (b): a staff/admin-only grant in a different org — no
        // worker_pid at all, per the unified table's second shape
        // (this is the "belongs to an org without being employed
        // there" case; no organization_ref agreement to check since
        // there's no worker_pid to check it against).
        let staff_org = an_org();
        let staff_grant: Value = request
            .post("/api/organization-memberships")
            .json(&json!({
                "person_ref": person_ref, "organization_ref": staff_org,
                "role": "hr_admin", "starts_on": "2026-01-01",
            }))
            .await
            .json();
        let staff_grant_pid = staff_grant["pid"].as_str().expect("staff grant pid");

        // Duplicate (person, org, role) is refused.
        assert_eq!(
            request
                .post("/api/organization-memberships")
                .json(&json!({
                    "person_ref": person_ref, "organization_ref": org,
                    "worker_pid": worker_pid, "role": "member",
                    "starts_on": "2026-02-01",
                }))
                .await
                .status_code(),
            422,
            "duplicate (person, org, role) refused"
        );

        // The admin listing shows both live grants, across both orgs,
        // in one unified read — no switcher.
        let listed: Value = request
            .get(&format!(
                "/api/organization-memberships?person_ref={person_ref}"
            ))
            .await
            .json();
        let rows = listed.as_array().unwrap();
        assert_eq!(rows.len(), 2, "both live grants listed");
        let employment_row = rows
            .iter()
            .find(|r| r["pid"] == membership_pid)
            .expect("employment-linked row present");
        assert_eq!(employment_row["employed"], true);
        assert_eq!(employment_row["organization_ref"], org.as_str());
        let staff_row = rows
            .iter()
            .find(|r| r["pid"] == staff_grant_pid)
            .expect("staff-only row present");
        assert_eq!(staff_row["employed"], false);
        assert_eq!(staff_row["role"], "hr_admin");

        // Revoke the employment-linked grant; it drops out, the
        // staff-only grant remains.
        request
            .delete(&format!("/api/organization-memberships/{membership_pid}"))
            .await
            .assert_status_ok();
        let after: Value = request
            .get(&format!(
                "/api/organization-memberships?person_ref={person_ref}"
            ))
            .await
            .json();
        let after_rows = after.as_array().unwrap();
        assert_eq!(after_rows.len(), 1, "revoked row no longer listed");
        assert_eq!(after_rows[0]["pid"], staff_grant_pid);

        // The now-revoked (person, org, role) can be granted again.
        assert_eq!(
            request
                .post("/api/organization-memberships")
                .json(&json!({
                    "person_ref": person_ref, "organization_ref": org,
                    "worker_pid": worker_pid, "role": "member",
                    "starts_on": "2026-03-01",
                }))
                .await
                .status_code(),
            200,
            "re-grant after revoke succeeds"
        );

        // `person_ref` is required on the admin listing.
        assert_eq!(
            request
                .get("/api/organization-memberships")
                .await
                .status_code(),
            422,
            "missing person_ref refused"
        );
    })
    .await;
}

#[tokio::test]
#[serial]
#[ignore = "requires PostgreSQL (config/test.yaml); run with `cargo test -- --ignored`"]
async fn organization_confederation_round_trip() {
    request::<App, _, _>(|request, ctx| async move {
        let parent = an_org();
        let child = an_org();
        let grandchild = an_org();

        // Self-loop refused.
        assert_eq!(
            request
                .post("/api/organization-confederations")
                .json(&json!({
                    "parent_organization_ref": parent, "child_organization_ref": parent,
                    "starts_on": "2026-01-01",
                }))
                .await
                .status_code(),
            422,
            "self-loop refused"
        );

        // parent -> child.
        let edge: Value = request
            .post("/api/organization-confederations")
            .json(&json!({
                "parent_organization_ref": parent, "child_organization_ref": child,
                "starts_on": "2026-01-01",
            }))
            .await
            .json();
        let edge_pid = edge["pid"].as_str().expect("edge pid").to_string();

        // Duplicate edge refused.
        assert_eq!(
            request
                .post("/api/organization-confederations")
                .json(&json!({
                    "parent_organization_ref": parent, "child_organization_ref": child,
                    "starts_on": "2026-02-01",
                }))
                .await
                .status_code(),
            422,
            "duplicate edge refused"
        );

        // child -> grandchild (arbitrary-depth nesting).
        request
            .post("/api/organization-confederations")
            .json(&json!({
                "parent_organization_ref": child, "child_organization_ref": grandchild,
                "starts_on": "2026-01-01",
            }))
            .await
            .assert_status_ok();

        // grandchild -> parent would close a cycle (parent -> child ->
        // grandchild -> parent); refused.
        assert_eq!(
            request
                .post("/api/organization-confederations")
                .json(&json!({
                    "parent_organization_ref": grandchild, "child_organization_ref": parent,
                    "starts_on": "2026-01-01",
                }))
                .await
                .status_code(),
            422,
            "an edge that would close a cycle is refused"
        );

        // Direct-edge listing shows only the immediate child, not the
        // transitive grandchild.
        let listed: Value = request
            .get(&format!(
                "/api/organization-confederations?parent_organization_ref={parent}"
            ))
            .await
            .json();
        let rows = listed.as_array().unwrap();
        assert_eq!(rows.len(), 1, "one direct edge from parent");
        assert_eq!(rows[0]["child_organization_ref"], child.as_str());

        // At least one filter is required.
        assert_eq!(
            request
                .get("/api/organization-confederations")
                .await
                .status_code(),
            422,
            "missing filter refused"
        );

        // A membership in the PARENT only expands (through
        // confederation) to see the child and grandchild too — this is
        // the model-level scope-expansion this feature exists for;
        // exercised in-process since the harness never mints a bearer
        // token for `/api/me/organizations/scope`.
        let person_ref = a_person();
        request
            .post("/api/organization-memberships")
            .json(&json!({
                "person_ref": person_ref, "organization_ref": parent,
                "role": "member", "starts_on": "2026-01-01",
            }))
            .await
            .assert_status_ok();
        let claims = claims_for(&person_ref);
        let mut scope = memberships::caller_scope_refs(&ctx.db, Some(&claims))
            .await
            .expect("scope expands");
        scope.sort();
        let mut expected = vec![parent.clone(), child.clone(), grandchild.clone()];
        expected.sort();
        assert_eq!(scope, expected, "parent membership expands to every descendant");

        // Revoking the parent -> child edge collapses the scope back
        // to just the parent (the grandchild, only reachable through
        // the now-revoked edge, drops out too).
        request
            .delete(&format!("/api/organization-confederations/{edge_pid}"))
            .await
            .assert_status_ok();
        let scope_after = memberships::caller_scope_refs(&ctx.db, Some(&claims))
            .await
            .expect("scope shrinks");
        assert_eq!(scope_after, vec![parent.clone()], "revoked edge drops descendants");
    })
    .await;
}
