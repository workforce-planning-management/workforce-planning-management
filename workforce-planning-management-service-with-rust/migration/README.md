# workforce-planning-management-migration

The database migrations for the
[workforce planning management service](https://github.com/workforce-planning-management/workforce-planning-management)
(SeaORM migrations, written as explicit SQL, applied in dated order). The crate's library name is `migration`:

```rust
use migration::Migrator;
```

> Demo software: not a production HR or payroll system; synthetic data only.
