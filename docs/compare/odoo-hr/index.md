# WPM compared with Odoo HR

This page compares [Odoo HR](https://github.com/odoo/odoo) with Workforce Planning Management (WPM). It is a general orientation, not a benchmark.

> **WPM is demo software.** It is not a production HR or payroll system and uses synthetic data only.

## Summary

| | WPM | Odoo HR |
| --- | --- | --- |
| **Type** | All-in-one HR platform, demonstration software | Open-source HR platform |
| **Scope** | Full employee lifecycle, hire to retire | The `hr_*` addons of a wider business suite, including expenses (`hr_expense`) |
| **Licensing** | Open source | Open source |
| **Suitable for production use** | No | Yes |

## When Odoo HR is the better fit

- HR is needed alongside other business applications.
- The organization wants open-source HR software in production. Odoo HR has more integrations, localizations and deployments than WPM.
- A payroll hand-off of approved expense claims is required. WPM does not have this yet.

## When WPM is the better fit

- The aim is to evaluate, study or extend an open, specification-driven design.
- The aim is to review how privacy rules can be built into an HR data model.
- Synthetic data is sufficient, and no real employees are involved.

## Design differences in WPM

- **Data minimization.** Sensitive data has no field or is restricted. There is no diagnosis field, pulse survey responses have no author, and emergency contacts are visible only to the person and HR.
- **Reporting rules.** Aggregates do not identify individuals, and unknown values are not reported as zero.
- **Federated identity.** WPM has no user registry of its own. People, workers, organizations and courses are held in separate services and referenced by URN.
- **Traceability.** Each behaviour is traced to a numbered requirement, design decision and task.
- **Skills analysis.** WPM includes skills gap and training time analysis based on role requirements. The scan did not compare this capability.

## Basis for this comparison

A scan dated 2026-10-05 reviewed the code and documentation of this project. The written source is [COMPARISONS.md](../../../COMPARISONS.md). Details may have changed since it was written; corrections are welcome through [the RFC process](../../../RFC.md).

Odoo HR is a name of its owner. WPM is not affiliated with or endorsed by its owner.

[Run the demo](../../../locales/en-001/start/index.md) [All comparisons](../index.md)
