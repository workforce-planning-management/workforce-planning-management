# WPM compared with BambooHR

This page compares BambooHR with Workforce Planning Management (WPM). It is a general orientation, not a benchmark.

> **WPM is demo software.** It is not a production HR or payroll system and uses synthetic data only.

## Summary

| | WPM | BambooHR |
| --- | --- | --- |
| **Type** | All-in-one HR platform, demonstration software | All-in-one commercial HR suite |
| **Scope** | Full employee lifecycle, hire to retire | ATS, core HR, time and leave, performance, learning, succession and payroll on one platform |
| **Licensing** | Open source | Commercial |
| **Suitable for production use** | No | Yes |

## When BambooHR is the better fit

- The organization needs a supported production system.
- The organization wants one vendor for the whole platform.

## When WPM is the better fit

- The aim is to evaluate, study or extend an open, specification-driven design.
- The aim is to review how privacy rules can be built into an HR data model.
- Synthetic data is sufficient, and no real employees are involved.

## Design differences in WPM

- **Licensing.** The source code is open.
- **Data minimization.** Sensitive data has no field or is restricted. There is no diagnosis field, pulse survey responses have no author, and emergency contacts are visible only to the person and HR.
- **Reporting rules.** Aggregates do not identify individuals, and unknown values are not reported as zero.
- **Federated identity.** WPM has no user registry of its own. People, workers, organizations and courses are held in separate services and referenced by URN.
- **Traceability.** Each behaviour is traced to a numbered requirement, design decision and task.

## Basis for this comparison

This product was not code-scanned. The page compares the type of system, as described in [COMPARISONS.md](../../../COMPARISONS.md), and makes no statement about its current features, pricing or roadmap. The written source is [COMPARISONS.md](../../../COMPARISONS.md). Details may have changed since it was written; corrections are welcome through [the RFC process](../../../RFC.md).

BambooHR is a name of its owner. WPM is not affiliated with or endorsed by its owner.

[Run the demo](../../../locales/en-001/start/index.md) [All comparisons](../index.md)
