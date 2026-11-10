# Help, learning to use WPM, and what is required of me

WPM-R243–R245, WPM-D191–D193; tasks WPM-T484–WPM-T487. The maintainer asked on 2026-11-10, in these words: "emphasize how to
get help for bugs, confusion, workflows" (in the UI's help page and header link, the README and the documentation site, and in-context
hints: "for any error, show the user how to get help or suggestions"), "make it easy to upload appraisals", "create much better
e-learning" (chosen: learning to use WPM), and "make it simpler to have an employee find mandatory training and mandatory
certifications".

## WPM-R243 — Getting help is obvious everywhere

**As** anyone using WPM, **I want** to see at once how to report a bug, how to ask when something is confusing and where a task
is explained, **so that** a failure or a puzzle does not leave me stuck.

Acceptance criteria:

1. Every page's header has a **Help** link, signed in or not, to `/<locale>/help`.
2. `/help` is public (no session) and listed in the sitemap. It says how to report a bug (the issue tracker, with what to put in
   the report and never real personal data), that a security problem goes by email under the security policy and not in a public
   issue, how to ask a question, where each task is explained (the tour, the walkthroughs of WPM-R244 and the guides on the
   website), and what to try when an error appears.
3. **Every error says what to try next and links to help.** The shared error note (every page's failures go through it, WPM-T261)
   adds a line chosen by the kind of failure: a refusal or a missing record points to the person's manager or HR, a rejected entry
   to checking it and reloading, anything else to trying again and reporting it if it persists. The service's own message is still
   never shown.
4. An address that does not exist, or a page that fails to load, shows a page in the reader's language with a link home and a
   link to help, never the framework's bare error.
5. The tour ends with a pointer to help. The root `README.md`, the documentation index (`index.md`, the site's `/docs/`) and the
   site's English landing page each have a **Getting help** section.

## WPM-D191 — Help points to the project's own channels, and says so

- **Nothing is invented.** The channels are the ones `CONTRIBUTING.md` and `SECURITY.md` name: the repository's issue tracker for
  bugs and questions, email to the maintainer for a vulnerability (never a public issue). The UI links the issue tracker and the
  security policy; it does not print an email address of its own.
- **A deployer would change them.** The help page says these links go to the WPM project, and that an organization running WPM for
  its staff would point the page at its own service desk. There is no setting for that yet: WPM is demonstration software, and a
  setting nobody uses would be a promise nobody tests.
- **What to try is chosen from the catalogue's own failure sentences**, not from the service's message, so it is in the reader's
  language and says nothing the page does not know.

## Uploading an appraisal: an open question (WPM-T485)

"Make it easy to upload appraisals" has more than one reading, and none of them is settled by the spec. What WPM has today:

- **360° appraisals** (WPM-R29, WPM-D21): an appraisal, its raters and their scores per competency, entered in the app; the report is
  a group aggregate with a floor. Before WPM-T485 the only way in was the subject's worker page, which nothing on `/me` linked to,
  and the "New 360" button fixed three competencies.
- **Performance reviews** with a 1–5 rating, and the **PADR** (WPM-R172, WPM-D176), written and signed in the app.
- **No file store at all.** WPM keeps no documents: "No certificate is stored: the evidence is a reference to where it is held"
  (WPM-R171), and review content is high sensitivity (`auth.md`).

The readings, and what each would need:

1. **Attach a completed appraisal document** (a PDF of a paper appraisal). Needs a document store, which WPM has deliberately never
   had; review content is high sensitivity, so it would also need erasure, subject access, retention and audit for files. It runs
   against the WPM-R171 stance. Not built.
2. **Record an appraisal done elsewhere**, as mandatory learning records a completion done elsewhere (WPM-R170): the date, the kind
   (performance review or PADR), who held it, and a **reference to where the document is held**, no file. This is the smallest
   fit with the privacy rules, and it would let a paper PADR count toward the pay-step standard (WPM-R173). It needs a table and a
   migration, which this task's limits did not allow. Not built.
3. **Import appraisal records in bulk** (a CSV of 360° responses or review ratings from another system). Responses carry the
   rater's identity procedurally and the floor is the protection (WPM-D21); a bulk import would bypass "once per rater" and
   needs a decision on who may do it. Not built.

**Built (WPM-T485):** only what makes the existing path easy to find and use: `/me` links to the person's own record at its
appraisals, the panel says how a 360 runs, and the person running a 360 names its competencies (up to twelve, as the service
allows) instead of the three the button fixed. The maintainer is asked which reading is meant; reading 2 is the recommendation.
