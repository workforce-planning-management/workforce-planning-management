# Backend (Rust, Loco)

`workforce-planning-management-service-with-rust` — Axum + SeaORM + PostgreSQL,
`#![forbid(unsafe_code)]`, `#![warn(clippy::pedantic)]`. Stack-level agreements:
[the service's AGENTS.md](../workforce-planning-management-service-with-rust/AGENTS.md).

## Order of work for a new capability

1. **Pure rules** in `src/rules/<area>.rs` (register in `src/rules/mod.rs`):
   DB-free, clock-free (dates are arguments), `serde` only for output types, unit
   tests alongside. Test the edges: ties, empty, "nobody", "unknown", windows,
   completion gating. Make orderings total (tie-break by id) so pages are stable.
2. **Migration** `migration/src/m<YYYYMMDD>_<seq>_<name>.rs` — explicit SQL
   (`CREATE TABLE IF NOT EXISTS …`, constraints and `CHECK`s in the schema), add to
   `migration/src/lib.rs` in order. Plural table names. `pid UUID` public id.
3. **Entity** `src/models/_entities/<table>.rs` + `mod.rs` + `prelude.rs` (hand
   written, same shape as the others).
4. **Controller** `src/controllers/<area>.rs` + `pub mod` in
   `controllers/mod.rs` + `.add_route(…::routes())` in `src/app.rs`.
5. **OpenAPI** — add every route to `src/openapi.rs` with a one-line summary.
6. **Rights wiring** if it holds personal data
   ([privacy](privacy-and-data-rules.md)).
7. **Request test** in `tests/requests/<area>.rs` (+ `mod …;` in
   `tests/requests/mod.rs`), `#[ignore]`d, `#[serial]`.
8. **Spec and task entry** ([spec-driven-delivery](spec-driven-delivery.md)).

## Controller patterns

```rust
// Record-level authorization: Write = "the person themself, or HR".
auth::authorize_record(&caller, Action::Write, &auth::worker_resource_attrs(&worker))
    .map_err(record_rejection)?;
// Organization scope: None = auth off = unrestricted. Out of scope is a 404.
if let Some(refs) = memberships::scope_organization_refs(&ctx.db, caller.claims()).await? { … }
// Editor roles for an organization (announcements): hr_admin / org_admin.
memberships::has_role_in(&ctx.db, caller.claims(), org, rules::EDITOR_ROLES).await?
// Audit every mutation; the actor is caller.actor().
Audit::record(&txn, "worker", worker.pid, "action_name", caller.actor(), Some(json)).await?;
// Notify (closed list of kinds in rules::notify::KINDS):
Notification::push(&txn, worker_pid, "kind", "reference-only text", json!({…})).await?;
```

- Validation errors: `unprocessable("message")` (422). Unknown pid / out of
  scope: `Error::NotFound` (404). `records::find_worker(db, pid)` etc. return 404.
- Multi-step writes in one transaction (`ctx.db.begin()`); put the audit row and
  any notification in it.
- Lists: `Page { limit, offset }`, `check_offset()`, `with_page_headers(...)`
  (`x-total-count`).
- Never put a sensitive detail in an audit snapshot or a notification body.

## Gotchas learned the hard way

- **Do not read through `ctx.db` while a transaction is open.** The test pool is
  small; the read waits for the connection the transaction holds and surfaces as
  a 500. Read first, then `begin()`.
- loco `create_table` pluralizes names — this repo uses explicit SQL.
- `ModelError::EntityNotFound` is not a 404; return `Error::NotFound`.
- `active → terminated` routes via `offboarding`.
- Enforcement/Keycloak tests need their own binaries (process-wide `OnceLock`s).
- `jsonwebtoken` refuses a validation list mixing key families (RS256 + ES256):
  pick the header's algorithm if it is on the allow-list (`auth/keycloak.rs`).
- `json!` does not accept method-call syntax inside a literal array; build the
  `Vec<Value>` first.
- A leftover `\\` in a Rust string literal inside a Python-generated file is a
  literal backslash, not a line continuation — check generated SQL.
- Clippy pedantic: `implicit_hasher` (take `&HashSet<T, S>` with
  `S: BuildHasher`), `similar_names`, `too_many_arguments`/`too_many_lines`
  (an `#[allow(...)]` with a reason is acceptable for a one-arm-per-kind match),
  `assert!(x.is_empty())` in tests (compare against an empty `Vec`).
- Soft-deleted rows: filter `DeletedAt.is_null()` on every read.

## Money and time

Money is minor units (`i64`) + ISO-4217; no floats. "Employed on a date" is
`rules::metrics::is_employed_on` — use it, never re-derive it.
