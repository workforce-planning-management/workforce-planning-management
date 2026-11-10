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
