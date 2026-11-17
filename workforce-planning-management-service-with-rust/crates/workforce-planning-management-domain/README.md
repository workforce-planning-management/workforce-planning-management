# workforce-planning-management-domain

The domain logic and business rules of Workforce Planning Management (WPM): every lifecycle state machine, the leave, time
and scheduling arithmetic, payslip arithmetic, the planning and capacity rules, the floors that keep a roll-up from naming
anyone, and the rest of the pure core. It has no database, no web framework, no async runtime and no clock: dates and "today"
are always supplied, and each rule is tested exhaustively by its own unit tests.

The service, `workforce-planning-management-service-with-rust`, loads the data, calls a rule and writes the result; it
re-exports this crate as `rules`, so `crate::rules::leave` there is `workforce_planning_management_domain::leave` here.

> ⚠️ **Demo software, not yet ready for production use.** Synthetic data only.

    cargo test -p workforce-planning-management-domain     # from the service's directory

See `../../../spec/index.md` for the specification, and WPM-D203 in `../../../spec/rust-axum-stack.md` for why this is a crate.
