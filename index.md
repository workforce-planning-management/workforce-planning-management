# Workforce Planning Management — documentation index

Workforce Planning Management (WPM) is an all-in-one HR platform for the
employee lifecycle, hire to retire, written as demonstration software with
synthetic data only. This page is the way in to its documents; it is also the
home page of the documentation site
([`workforce-planning-management.github.io/`](workforce-planning-management.github.io/README.md)).

> ⚠️ **Demo software.** Not a production HR or payroll system. See
> [spec/regulatory.md](spec/regulatory.md).

## Start here

- **[README.md](README.md)**: what WPM is, its status, the subprojects and how
  to run them.
- **[INSTALL.md](INSTALL.md)**: install, build and test, step by step.
- **[The family bridges tutorial](docs/tutorials/family-bridges.md)**: seven
  steps with `curl` through the bridges to the sibling project-portfolio-management
  and contact-relationship-management apps.

## The specification

- **[spec/index.md](spec/index.md)**: the single source of truth, its topic
  files and the requirements, design decisions and tasks.
- **[spec/worldwide.md](spec/worldwide.md)**: the target is worldwide, not the
  US or the UK; jurisdiction is data, and every locale is equal.
- **[spec/readiness-assessment.md](spec/readiness-assessment.md)**: the
  verdict for a high-assurance organization (not ready for production use).

## Role briefs

A page for each chief role and for governance, risk and compliance, in 21
locales: start with the [English source](locales/en-gb-oxendict/for/index.md)
or the [World English](locales/en-001/for/index.md) list, and see
[locales-x-for-roles](spec/locales-x-for-roles/index.md) for how they are
written. Every translation was written by an AI assistant and has not been
reviewed by a native speaker.

## The subprojects

- [The service](workforce-planning-management-service-with-rust/index.md): the
  Rust JSON API.
- [The UI](workforce-planning-management-ui-with-svelte/index.md): the
  SvelteKit client.
- [The matcher](workforce-planning-management-matcher-with-rust/README.md): the
  worker-record matcher crate.

## For contributors and agents

[CONTRIBUTING.md](CONTRIBUTING.md), [AGENTS.md](AGENTS.md),
[CHANGELOG.md](CHANGELOG.md), [NEWS.md](NEWS.md) and [llms.txt](llms.txt).
