# License

This repository holds two subprojects. Each declares its own SPDX
license expression in its manifest, and that declaration is
authoritative for that subproject:

- **workforce-planning-management-service-with-rust** (`Cargo.toml`
  `license`):
  `MIT OR Apache-2.0 OR BSD-3-Clause OR GPL-2.0-only OR GPL-3.0-only`
- **workforce-planning-management-ui-with-svelte**
  (`package.json` `license`): `MIT OR Apache-2.0`

`OR` is the SPDX disjunction: you may use each subproject under **any
one** of its listed licenses, at your option.

## License texts

The full text of each license is in this directory:

- MIT: [LICENSE-MIT](LICENSE-MIT)
- Apache-2.0: [LICENSE-APACHE](LICENSE-APACHE)
- BSD-3-Clause: [LICENSE-BSD-3-CLAUSE](LICENSE-BSD-3-CLAUSE)
- GPL-2.0-only: [LICENSE-GPL-2.0](LICENSE-GPL-2.0)
- GPL-3.0-only: [LICENSE-GPL-3.0](LICENSE-GPL-3.0)

The MIT and BSD-3-Clause files carry the copyright line below, with the year
the project began (2026). The Apache-2.0 and GPL texts are the SPDX
license-list texts, unmodified. The SPDX identifiers are
<https://spdx.org/licenses/>.

## Why several options

Offering a choice lets a downstream project take the terms that fit its own:
MIT, Apache-2.0 and BSD-3-Clause for permissive reuse (Apache-2.0 adds an
express patent grant), and GPL-2.0-only or GPL-3.0-only for projects that need
a copyleft-compatible license. Offering all five on the Rust crates keeps them
compatible with the other crates in the same family. The UI offers the two most
common permissive options.

A check in CI keeps the `license` fields in `Cargo.toml` and `package.json` in
step with this list (WPM-T125).

## Copyright

Copyright © Joel Parker Henderson (<joel@joelparkerhenderson.com>).

## Notes

- Documentation and specification files at the repository root and
  under [spec/](../spec/) are offered under the same terms as the Rust
  crate expression above.
- Custom license options are available on request — contact
  <joel@joelparkerhenderson.com>.
