# Comparisons

Where Workforce Planning Management (WPM) sits relative to adjacent
kinds of systems. These are orientation notes, not benchmarks or
feature-war tables.

## All-in-one HRIS platforms (Workday, SAP SuccessFactors, BambooHR)

Commercial HRIS suites cover similar ground — ATS, core HR, time &
leave, performance, learning, succession, payroll — as one integrated
platform. WPM follows the same five-pillar shape but differs by being
**a consumer application over a federated identity family**: it does
not own identity itself (person/worker/organization/course are
separate services, referenced by `EntityRef` URN), it is open source,
implemented in Rust end to end, and spec-driven — every behaviour
traces to a numbered requirement, design decision, and task. See
[spec/index.md](spec/index.md).

## Point solutions (ATS-only, payroll-only, LMS-only tools)

Point solutions (Greenhouse/Lever for ATS; Gusto/ADP for payroll;
standalone LMS products) each do one pillar well but leave the
handoffs between them to integration work or spreadsheets — exactly
the scatter [spec/purpose.md](spec/purpose.md) names as the problem
WPM addresses. WPM trades per-pillar depth for one operational system
across the whole employee lifecycle, hire to retire.

## The wider identity/registry family

WPM is one of several **consumer applications** built on the same
federated identity registries (person, worker, organization, course
services), alongside siblings such as case-folder and
patient-flow. What distinguishes WPM within that family is its domain:
the employment relationship and its operational state, not clinical
or case-management state. See
[spec/integrations.md](spec/integrations.md).

## Context

If you know of a system this should be compared against, see
[RFC.md](RFC.md) — that feedback is explicitly wanted.
