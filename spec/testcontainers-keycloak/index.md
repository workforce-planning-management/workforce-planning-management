# Automated integration testing with Testcontainers and Keycloak

For backend services (like Spring Boot or .NET), spin up a real Keycloak
instance inside a container during your test lifecycle. This provides full
realism without depending on a permanent external server.

- Add the `testcontainers-keycloak` dependency to your project build file.
- Configure your test runner (like JUnit 5) to launch a dynamic Keycloak
  container before running tests.
- Import a pre-configured realm JSON file containing your test users and
  clients so you don't have to build them programmatically.
- Inject the dynamic Keycloak issuer URI into your application configuration
  properties before hitting secured endpoints with a requested JWT token.

## Container runtime in this workspace

This workspace uses Podman, not Docker (see `rust-loco-stack.md`). Testcontainers
can drive Podman through its Docker-compatible socket; fully qualify the image
name (for example `quay.io/keycloak/keycloak`) because Podman refuses short
names.

## Implementation in this workspace

The Rust API's `keycloak` backend (`src/auth/keycloak.rs`) is tested against a
real Keycloak, in `workforce-planning-management-api-with-rust/tests/keycloak.rs`:

- Dependency: `testcontainers` (dev-dependency); the container is a
  `GenericImage` for `quay.io/keycloak/keycloak:26.0` run as
  `start-dev --import-realm`, so no extra Keycloak module crate is needed.
- Realm: `tests/keycloak/realm-wpm.json` — realm `wpm`, public client
  `wpm-api` (password grant enabled for tests only) with the audience,
  realm-role, group, and `organization_ref` mappers from `spec/auth.md`;
  users `hr-user` (`wpm-hr`, `wpm-payroll`, group `/org/engineering`) and
  `plain-user` (no roles).
- Wiring: the test fetches real access tokens, sets `WPM_KEYCLOAK_ISSUER`,
  `WPM_KEYCLOAK_AUDIENCE`, and `WPM_KEYCLOAK_JWKS_URL` from the container's
  mapped port, boots the app, and calls secured routes. It is its own test
  binary because the auth `OnceLock`s are process-wide.
- Run (Podman; rootless Podman cannot run the Ryuk reaper):

```sh
export DOCKER_HOST="unix://$(podman machine inspect \
    --format '{{.ConnectionInfo.PodmanSocket.Path}}')"
export TESTCONTAINERS_RYUK_DISABLED=true
cargo test --no-default-features --features keycloak --test keycloak -- --ignored
```
