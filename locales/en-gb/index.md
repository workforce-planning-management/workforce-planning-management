# Workforce Planning Management

**Everything about your people, from hire to retire, in one place.**

Welcome! Workforce Planning Management (WPM) is an open-source HR platform that brings hiring, time and leave, development, planning and pay together, so that nothing gets lost between inboxes and spreadsheets. It is a working demonstration you can run, read and learn from.

> ⚠️ **Demo software.** WPM is not a production HR or payroll system: it uses synthetic data only, and its statutory calculations are illustrative. Please keep real personal data out of it. See [the regulatory position](../../spec/regulatory.md).

[Take the guided tutorial](start/index.md) [See what it means for your role](for/index.md) [View the code on GitHub](https://github.com/workforce-planning-management/workforce-planning-management)

## Why WPM

Applicants tracked in inboxes, leave in spreadsheets, rotas on whiteboards: every handoff between those silos loses something. WPM is built around a different idea.

- **One system, not six.** The whole employee lifecycle lives together, so a new starter's onboarding, leave, training and pay never need retyping.
- **Privacy by construction.** What must not be stored gets no column, and what must not be disclosed gets no endpoint. There is no diagnosis column and no author on an anonymous pulse answer.
- **Numbers you can trust.** An unknown is never shown as a zero, roll-ups name no one, and every estimate says what it rests on.
- **Made for the world.** The app speaks 17 locales and these role briefs 21, right to left included, and rules that differ by country are data rather than code. The translations are AI-written and not yet reviewed by native speakers.
- **Open and traceable.** It is open source, and every behaviour traces to a numbered requirement, design decision and task.
- **Works on its own.** A Rust JSON API, a SvelteKit client and PostgreSQL. It builds and runs from a fresh clone, and links to sibling apps are opt-in.

## What it does

- 🤝 **Hiring and the record.** Requisitions, the applicant pipeline, onboarding, worker records and the org chart.
- 🗓️ **Everyday work.** Time and leave, shifts with working-time guardrails, the on-call rota with swaps and reminders, and the directory.
- 🌱 **Development.** Reviews, 360°s, learning, skills gap analysis, training time recommendations and mandatory learning.
- 📊 **Leadership.** Strategic workforce planning, metrics and insights, and a CEO dashboard that fits on one iPad screen.
- 👋 **Joiners and leavers.** Dated checklists, and a last-day handover with an audit trail.
- 📣 **Communication.** Announcements with audiences, links and read counts.
- 💚 **Rights and wellbeing.** Wellbeing prompts, the anonymous pulse, adjustments, subject access, erasure and retention.
- 💳 **Pay.** Payroll and compensation, with illustrative statutory calculations.

## How it compares

This is orientation, not a scoreboard: each kind of system below is the right answer for someone. [Compare WPM with each product](../../docs/compare/index.md).

| | WPM | All-in-one suites (Workday, SAP SuccessFactors, BambooHR) | Open-source HR (frappe/hrms, OrangeHRM, Horilla, Odoo HR) | Point tools (Greenhouse, Lever, Gusto, ADP) |
| --- | --- | --- | --- | --- |
| **Covers** | The whole lifecycle, hire to retire | Similar ground, on one platform | The same core | One pillar, done well |
| **Source** | Open source | Commercial | Open source | Commercial |
| **For real HR today?** | No: a demonstration | Yes | Yes | Yes |
| **Stands out for** | Privacy by construction, traceable design, skills gaps and training time | One integrated, supported platform | Far more integrations, localisations and deployments | Depth in a single job |
| **Choose it when** | You want to evaluate, learn from or build on an open design | You need a supported system now | You want open source in production now | One job needs depth and you can connect the rest |

## Who it is for

- **Chief officers.** One page for each leadership role says what WPM gives you and how to read it: [the role briefs](for/index.md).
- **HR and people teams.** See how the pieces fit, page by page, in [the guided tour](start/index.md).
- **Developers and architects.** Start with [the specification](../../spec/index.md) and [the install guide](../../INSTALL.md).
- **Assessors.** [The readiness assessment](../../spec/readiness-assessment.md) sets out what stands between WPM and production, plainly.

## Get started in three steps

1. **Read your brief.** Pick your role and see what WPM offers it.
2. **Take the tour.** Run the demo on your own machine, with fictitious data.
3. **Join in.** Tell us what to compare against or improve: [how to comment](../../RFC.md) and [how to contribute](../../CONTRIBUTING.md).

[Start the tutorial](start/index.md) [Browse the documentation](../../index.md)
