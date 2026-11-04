# Workforce Planning Management

**An open-source HR platform covering the employee lifecycle from hire to retire.**

Workforce Planning Management (WPM) combines recruitment, time and leave, employee development, workforce planning and payroll in one application. It is demonstration software, intended for evaluation and as a reference design.

> **Demo software.** WPM is not a production HR or payroll system. It uses synthetic data only, and its statutory calculations are illustrative. Do not enter real personal data. See [the regulatory position](../../spec/regulatory.md).

[Run the demo](start/index.md) [Role briefs](for/index.md) [Source code on GitHub](https://github.com/workforce-planning-management/workforce-planning-management)

## Benefits

HR data is often held across separate tools, such as applicant inboxes, leave spreadsheets and rota boards. Each transfer between them adds manual work and a risk of error. WPM is designed to reduce those transfers.

- **Single system.** Onboarding, leave, training and pay records are held in one application, which reduces re-keying.
- **Data minimization.** Data that should not be held has no field, and data that should not be disclosed has no API route. For example, there is no diagnosis field, and pulse survey responses have no author.
- **Reporting rules.** Unknown values are not reported as zero, aggregates do not identify individuals, and estimates state their basis.
- **Localization.** The application is available in 17 locales and the role briefs in 21, including right-to-left languages. Country-specific rules are held as data. Translations were produced by AI and have not been reviewed by native speakers.
- **Traceability.** The source code is open, and each behaviour is traced to a numbered requirement, design decision and task.
- **Independent deployment.** The stack is a Rust JSON API, a SvelteKit client and PostgreSQL. It builds from a fresh clone, and integrations with sibling applications are optional.

## Features

- **Recruitment and records.** Requisitions, applicant pipeline, onboarding, worker records and org chart.
- **Time and scheduling.** Time and leave, shifts with working-time checks, the on-call rota with swaps and reminders, and a staff directory.
- **Development.** Reviews, 360° feedback, learning, skills gap analysis, training time recommendations and mandatory learning.
- **Planning and reporting.** Strategic workforce planning, metrics and insights, and a CEO dashboard.
- **Joiners and leavers.** Dated checklists and a last-day handover with an audit trail.
- **Communication.** Announcements with audiences, links and read counts.
- **Employee rights and wellbeing.** Wellbeing prompts, an anonymous pulse survey, adjustments, subject access, erasure and retention.
- **Pay.** Payroll and compensation, with illustrative statutory calculations.

## Comparison

The table compares WPM with three categories of HR system. It is a general orientation, not a benchmark. See also the [comparison by product](../../docs/compare/index.md).

| | WPM | All-in-one suites (Workday, SAP SuccessFactors, BambooHR) | Open-source HR (frappe/hrms, OrangeHRM, Horilla, Odoo HR) | Point tools (Greenhouse, Lever, Gusto, ADP) |
| --- | --- | --- | --- | --- |
| **Scope** | Full lifecycle, hire to retire | Similar scope on one platform | Same core functions | One function |
| **Licensing** | Open source | Commercial | Open source | Commercial |
| **Suitable for production use** | No | Yes | Yes | Yes |
| **Main strength** | Data minimization, traceable design, skills gap and training time analysis | One integrated, supported platform | More integrations, localizations and deployments | Depth in one function |
| **Typical fit** | Evaluation, reference design or further development | Organizations that need a supported system | Organizations that want open source in production | Organizations that need depth in one function and can integrate the rest |

## Intended audience

- **Executives.** The [role briefs](for/index.md) set out what WPM provides for each leadership role and how to read its output.
- **HR teams.** The [demo tutorial](start/index.md) covers each page using fictitious data.
- **Developers and architects.** See the [specification](../../spec/index.md) and the [install guide](../../INSTALL.md).
- **Assessors.** The [readiness assessment](../../spec/readiness-assessment.md) lists the gaps between WPM and production use.

## Next steps

1. **Read the brief for your role.** Each brief describes what WPM provides and what remains open.
2. **Run the demo.** The tour runs on a local machine with fictitious data.
3. **Send feedback.** See [how to comment](../../RFC.md) and [how to contribute](../../CONTRIBUTING.md).

[Run the demo](start/index.md) [Documentation](../../index.md)
