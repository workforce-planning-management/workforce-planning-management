# Cutover readiness: the people side of a go-live (WPM-R70, WPM-T117)

For a programme's go-live at a site over a window of dates, the service reports whether the people side is
ready. **Whether to go live is a human decision outside WPM**; the report is evidence for it and says so.

A **cutover** has a programme (a `thing:` URN), a name, a site, a window of calendar days (at most 92, the
longest the on-call schedule shows), and optionally the on-call rota that gives hypercare cover. It lists the
**roles** that must be staffed (a role profile, the course that trains it, a headcount) and the **skill pools**
it depends on.

## The three checks

Each is `ready`, `at_risk` or `unknown`, with reasons in words (and the same reasons as data, `details`: a code and its figures, so a client can say them in its own language). The overall result is the worst: a known risk
outweighs an unknown, which outweighs ready. **An unknown is never ready.**

| Check | Ready | At risk | Unknown |
| --- | --- | --- | --- |
| **Trained staff** | Every listed role has at least its headcount trained at the site | A role is short, and people of the role with no site recorded could not close the gap | No role is listed; or a role is short but enough people of the role with no site recorded have completed the course that they might close it |
| **Hypercare cover** | Every day of the window has someone on call | One or more days have nobody on call (the reason gives the count and the first day) | No rota is linked |
| **Key-person risk** | Every critical skill in the listed pools has a shared, or covered, holder at the site | A critical skill has no holder, or one holder with no backup or ready successor | No pool is listed, or none of the pools' skills is marked critical on a role profile |

**Trained** means: a person of the role (their chosen framework role, or their job title), employed on the first
day of the window, recorded at the site (its name or any alias, compared without case, punctuation or spacing), who completed the named course on or before that day, and whose
certificate (if it expires) lasts through the window.

The key-person check reads the people recorded **at the site**: only they count as holders, and only they as backups. A
holder elsewhere in the organization does not cover. Holders with no site on file might be there, so a site risk they could
close is `unknown`, not `at_risk`.

A role names a course by its reference. The reference must be known to the learning records (a catalogue entry or any
enrollment) when the role is added; if it later stops being known (renamed), the trained-staff check is `unknown` and says so.

Planning a cutover (create, roles, pools, removal) is for an administrator of the organization. Reading the readiness is open
to the organization's scope. A cutover whose window has not ended and whose people side is `at_risk` opens an early warning
(`cutover_at_risk`) and tells the HR administrators; it closes when the risk passes.

## What the report holds

Counts, dates, and the names of roles and skills. It names no one, so it can be read by whoever reads the
cutover. It carries no clinical or operational go/no-go.

## Not verified

- A site spelt a way neither the name nor an alias covers is a different site; nothing suggests aliases.
- A role cannot be removed or re-pointed singly; remove the cutover and add it again.
- No user interface.
