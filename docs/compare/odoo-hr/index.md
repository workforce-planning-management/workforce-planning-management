# WPM compared with Odoo HR

[Odoo HR](https://github.com/odoo/odoo) and Workforce Planning Management (WPM) solve overlapping problems in very different positions. This is orientation, not a benchmark: here is an honest way to choose.

> ⚠️ **WPM is demo software**, not a production HR or payroll system. It uses synthetic data only.

## At a glance

| | WPM | Odoo HR |
| --- | --- | --- |
| **What it is** | All-in-one HR platform, written as a demonstration | Open-source HR platform |
| **Covers** | The whole employee lifecycle, hire to retire | The `hr_*` addons of a much wider business suite, including expenses (`hr_expense`) |
| **Source** | Open source | Open source |
| **For real HR today?** | No | Yes |

## Choose Odoo HR when

- You want HR alongside the rest of a business suite.
- You want open-source HR in production now, with far more integrations, localizations and deployments than a demo.
- You need a payroll hand-off of approved expense claims, which WPM has not yet matched.

## Choose WPM when

- You want to evaluate, learn from or build on an open, specification-driven design.
- You want to see how an HR system behaves when privacy rules are built into its data model.
- You can work with synthetic data, and do not need it for real employees.

## Where WPM differs

- **Privacy by construction.** Sensitive data is unrepresentable or restricted: no diagnosis column, no author on a pulse answer, and emergency contacts visible only to the person and HR.
- **Honest numbers.** Every roll-up names no one, and an unknown is never shown as zero.
- **Federated identity.** WPM does not run its own user registry: people, workers, organizations and courses live in separate services and are referenced by URN.
- **Traceable.** Every behaviour traces to a numbered requirement, design decision and task.
- **Skills and training time.** WPM has a skills-gap and training-time capability built on role requirements, which the scan did not compare.

## How this page was written

A scan on 2026-10-05 checked what this project ships, from its code and docs rather than its README. The written basis is [COMPARISONS.md](../../../COMPARISONS.md); a detail may have changed since. If it has, [please tell us](../../../RFC.md).

Odoo HR is a name of its owner, and WPM is not affiliated with or endorsed by them.

[Take the guided tutorial](../../../locales/en-001/start/index.md) [See all comparisons](../index.md)
