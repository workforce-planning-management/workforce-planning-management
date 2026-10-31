# Spec-driven delivery

[`spec/`](../spec/index.md) is the single source of truth. **No code lands
without the spec describing it.**

## The loop

1. **Requirement** (`WPM-R*`) — who can do what, with acceptance criteria.
2. **Design decision** (`WPM-D*`) — only when a choice needs explaining
   (usually a privacy or honesty stance). Recorded beside the requirement.
3. **Task** (`WPM-T*`) in [`spec/tasks.md`](../spec/tasks.md).
4. **Code and tests**, in the same change.
5. **Update the task entry** with what was actually verified.

Requirements and decisions live in `spec/requirements.md` / `spec/design.md`
**or** — for newer areas — in a topic file those link to
([people-directory-and-cover](../spec/people-directory-and-cover.md),
[skills-and-training](../spec/skills-and-training.md),
[joiners-and-leavers](../spec/joiners-and-leavers.md),
[communication-and-leadership](../spec/communication-and-leadership.md),
[locales](../spec/locales-for-global-sharing-with-svelte/index.md)). Next free
ids (as of 2026-10-31): **WPM-R218**, **WPM-D154**, **WPM-T409**. WPM-R56–R132, WPM-R144–R205, WPM-D45–D83,
WPM-D99–D133, WPM-T103–T224 and WPM-T246–T344 are taken by [`spec/plan.md`](../spec/plan.md) and
[`spec/production-readiness.md`](../spec/production-readiness.md) and
[`spec/operations-and-governance.md`](../spec/operations-and-governance.md), some built and
some still proposed; WPM-R71, WPM-D51 and WPM-T120–T122 were withdrawn by WPM-D55
and are not reused.

## Writing a task entry

`tasks.md` entries are dense and honest. Follow the existing shape:

```
- [x] WPM-T## (YYYY-MM-DD) **Title.** *(traces to WPM-R##, WPM-D##)* Builds on WPM-T##.
      What it does (endpoints, rules, UI), the rules it pins, what is
      *deliberately not done*, and the verification: suite counts, clippy,
      svelte-check, vitest, Playwright, build.
```

- Put new entries under the newest phase heading (currently **Phase 25**, cross-app
  bridges and the worldwide target), not at the end of an earlier part.
- **Every task has an entry in `tasks.md`**, even when its detail lives in a topic
  file (`worldwide.md`, `cross-app-bridges.md`): a one-line entry with the status and
  a link is enough. `scripts/status.py` counts `tasks.md` only, and it once
  reported "highest task done: WPM-T267" for weeks because it missed entries written
  as `- [x] (2026-10-30) WPM-T…` (fixed 2026-10-30, WPM-T391).
- A task done in part is `[~]` with "Part done (date):" and what remains; `[ ]` is
  not started.
- **Say what you did not verify.** "Not done:" and "Not tested:" are normal and
  expected; a claim you did not run is a defect (this repo has had to correct
  one: a handover kind that was implemented but untested).
- Mark done only when it is done. A deliberate deferral stays `[ ]` with its
  reason, and an open task says precisely what remains (the counts are in
  [`implementation-status.md`](../spec/implementation-status.md)).
- A task entry for a ranked/aggregate view states its **derivation** and what
  it excludes.

## Where else the spec must change

| If you… | Also update |
| --- | --- |
| add a table | [`domain-model.md`](../spec/domain-model.md) |
| add a role, a restricted read or a new audience | the sensitivity map in [`auth.md`](../spec/auth.md) |
| add an audited action | [`audit.md`](../spec/audit.md) |
| add personal data | [`regulatory.md`](../spec/regulatory.md) and the rights wiring ([privacy](privacy-and-data-rules.md)) |
| add a term | [`glossary.md`](../spec/glossary.md) |
| change test counts or strategy | [`testing.md`](../spec/testing.md) |
| add a topic file | the table in [`spec/index.md`](../spec/index.md), `requirements.md`/`design.md` pointers, [`llms.txt`](../llms.txt) |

Also update the owning subproject's `CHANGELOG.md` in the same change (a
repo-level summary goes in the root `CHANGELOG.md`; milestones in `NEWS.md`).

## Triage

New ideas go to [`spec/roadmap.md`](../spec/roadmap.md). A benchmark against
other projects is recorded in `.sota/` and summarised in
[`COMPARISONS.md`](../COMPARISONS.md): cite code or docs for what a comparator
ships, not its README.
