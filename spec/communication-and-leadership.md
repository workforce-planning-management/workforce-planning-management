# Announcements, insights and the CEO dashboard (WPM-R47–R49)

Telling people things, saying what the numbers mean, and one screen for the
person at the top. Delivered as WPM-T69, T74, T82–T86.

## WPM-R47 — Announcement feed

*As a person I read my organization's news; as an editor I post it.*

- Plain text only (title ≤ 200, body ≤ 5000): the API stores a body verbatim
  and the UI shows it escaped with line breaks, so a post can never carry
  markup. Pinned first, then newest published; optional **publish day** and
  **expiry day** (live from the publish day through the expiry day inclusive).
- Everyone who can read an organization reads its **live** posts. **Posting,
  editing, retiring and seeing scheduled or expired posts belong to the
  organization's `hr_admin` / `org_admin`** when auth is on.
- **Department audience:** a post may be aimed at one department; only that
  department (case-insensitive) and editors see it — enforced from the
  caller's own worker records when auth is on; `?department=` previews it.
- **Links:** up to three `{label, url}`, **https only** (no `http:`,
  `javascript:`, `data:`, protocol-relative or spaced addresses); the UI
  re-checks and opens them `rel="noopener noreferrer"`. (There is no file
  storage, so no uploads.)
- **Read receipts** (WPM-D35): a person marks a post read and sees which they
  have read; **editors see only a count, never who**. A post for another
  department is a 404 to someone outside it.
- `POST|GET /api/announcements` (`?organization=&department=&include=all`),
  `GET|PUT|DELETE /api/announcements/{pid}`,
  `POST …/read`, `GET /api/workers/{pid}/announcement-reads`. UI:
  `/announcements`, a latest-three panel on the signed-in home page.

## WPM-R48 — CEO dashboard

*As the CEO I see the state of the workforce on one screen, with no
scrolling, on an iPad.*

- `/ceo`: **six tiles** — headcount (with change over the period), turnover,
  open vacancies (+ median time-to-fill), succession gaps, a six-month
  headcount line, and the top insights. A **period** control (30 calendar days · 90
  calendar days · 12 months · year to date; default 12 months) applies to turnover,
  leavers, time-to-fill, the headcount change and the insights; it is in the
  URL (`?range=`). Every tile links to the page behind it.
- **Fits one screen** (WPM-D36): sized for an **iPad (9th generation), 2160 ×
  1620 device pixels = 1080 × 810 CSS pixels at 2×**. Sizes are in `em` off
  the viewport's short side with a grid that shares the height, so the same
  layout fits that screen and a literal 2160 × 1620 one.
- No new numbers: the figures are the existing views. A figure that cannot be
  loaded shows "—", never 0. The trend is month-end headcount computed from
  hire and termination dates, so it needs no snapshot job.
- Charts follow the data-viz method: one 2 px series with a direct end label
  and no legend box, hairline grid, hover/touch crosshair, a visually hidden
  data table, the reference palette's light and dark tokens (the series colour
  validated in both modes), status shown as icon + label, never colour alone.

## WPM-R49 — Workforce insights

*As HR I see which of the numbers deserve attention, and what to look at
next.*

- `GET /api/workforce-intelligence/insights?from=&to=` derives findings from
  the shared metrics ([`/metrics`](glossary.md#metrics), WPM-T44): high
  turnover (≥ 20%), headcount shrinking or growing (≥ 10%), spans of control
  too wide (> 12) or narrow (< 3), slow time-to-fill (> 60 calendar days). Thresholds
  are published heuristics (`thresholds` in the payload), not benchmarks. A
  metric that is unknown yields no finding.
- Each finding has a stable `code`, the English text, and **`params`** (the
  figures), so the UI renders it in the viewer's language from
  `insights.<code>.observation|suggestion` strings and falls back to the
  server's English for a code it does not know.

## Design decisions

- **WPM-D35 — Read receipts count; they do not watch.** A reader sees their
  own reads; editors get a number. Reads are the person's data: exported and
  erased with them.
- **WPM-D36 — A dashboard is a view of existing derivations, and it fits.**
  It adds no figures of its own; it is tested at its target viewport(s) for
  page scroll **and** for any tile clipping its own content (tiles hide
  overflow, so a page-level check alone would miss it).
