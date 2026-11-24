# License

Each subproject declares its own SPDX license expression in its manifest, and
that declaration is authoritative for that subproject:

- **workforce-planning-management-service-with-rust** (`Cargo.toml` `license`):
  `MIT OR Apache-2.0 OR BSD-3-Clause OR GPL-2.0-only OR GPL-3.0-only OR OGL-UK-3.0 OR CC-BY-SA-4.0`
- **workforce-planning-management-ui-with-svelte** (`package.json` `license`):
  `MIT OR Apache-2.0`
- **workforce-planning-management.github.io**, the documentation site
  (`package.json` `license`): `MIT OR Apache-2.0`

`OR` is the SPDX disjunction: you may use each subproject under **any one** of
its listed licenses, at your option.

The full text of every license is in the [LICENSES/](LICENSES) directory, one
`LICENSES/<lowercase-spdx-id>/index.md` per license: see
[Full license texts](#full-license-texts) below. The MIT and BSD-3-Clause texts
carry the copyright line below, with the year the project began (2026). The
Apache-2.0, GPL and OGL-UK-3.0 texts are the SPDX license-list texts, and the
CC-BY-SA-4.0 text is Creative Commons' own legal code, all unmodified. The SPDX identifiers are <https://spdx.org/licenses/>.

## Why several options

Offering a choice lets a downstream project take the terms that fit its own:
MIT, Apache-2.0 and BSD-3-Clause for permissive reuse (Apache-2.0 adds an
express patent grant), and GPL-2.0-only or GPL-3.0-only for projects that need
a copyleft-compatible license. Offering all seven on the Rust crates keeps them
compatible with the other crates in the same family. The UI offers the two most
common permissive options.

A check in CI keeps the `license` fields in `Cargo.toml` and `package.json` in
step with this list, and checks that each option has its
`LICENSES/<id>/index.md` and that no license text sits anywhere else
(WPM-T125).

## Copyright

Copyright © Joel Parker Henderson (<joel@joelparkerhenderson.com>).

Documentation and specification files at the repository root and under
[spec/](spec/) are offered under the same terms as the Rust crate expression
above. Custom license options are available on request: contact
<joel@joelparkerhenderson.com>.

## Full license texts

The full text of each license is in its own directory:

- [MIT](LICENSES/mit/index.md)
- [Apache-2.0](LICENSES/apache-2.0/index.md)
- [BSD-3-Clause](LICENSES/bsd-3-clause/index.md)
- [GPL-2.0-only](LICENSES/gpl-2.0-only/index.md)
- [GPL-3.0-only](LICENSES/gpl-3.0-only/index.md)
- [OGL-UK-3.0](LICENSES/ogl-uk-3.0/index.md)
- [CC-BY-SA-4.0](LICENSES/cc-by-sa-4.0/index.md)
