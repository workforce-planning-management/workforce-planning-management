# Privacy notice for this website

This notice covers the Workforce Planning Management documentation website,
<https://workforce-planning-management.github.io/>. It says what the website keeps in your browser, why, and how
to remove it. (Updated 2026-11-16, WPM-R253.)

The website is a set of static pages. It has no sign-in, no forms, no analytics, no advertising, and no
third-party scripts, fonts, or images. What it keeps is in your own browser: the choices you make in the picker
bar, and, while a tab is open, where you were on each page.

## What is kept, and why

| Your choice | Where it is kept | Name | What is kept | Why |
| --- | --- | --- | --- | --- |
| Theme | Your browser's local storage | `mxi.wpm.theme` | The theme's name, for example `dark` | So the next page and your next visit look the way you chose |
| Theme | A cookie for this website only | `wpm-theme` | The theme's name, for example `dark` | So the theme comes back even if your browser keeps cookies but clears local storage, or the other way round |
| Text size | Your browser's local storage | `mxi.wpm.text-size` | The size's name, for example `large` | So the next page and your next visit use the text size you chose |
| Language and locale | Nothing is kept | | | The language is part of each page's address (for example `/cy-001/`), so there is nothing to remember |
| Where you were on a page | Your browser's session storage, for this tab only | `sveltekit:history-info`, `sveltekit:navigation-snapshot`, `sveltekit:snapshot` | Scroll positions and the order of the pages you opened in this tab | So Back and Forward return you to where you were; the framework the website is built with (SvelteKit) keeps these, and the browser deletes them when you close the tab |

None of these identifies you. A choice holds only the name of a theme or a text size, the same for everyone who
makes the same choice; the session storage holds scroll positions and is gone when the tab is. Nothing is kept
with your name, an account, or an identifier, and nothing keeps a record of the pages you read once the tab is
closed.

## When it is kept

No choice is kept until you make it. Until you choose, the website follows your browser's light or dark setting
and its normal text size, and writes no theme or text size. A choice is kept only when you make it in the picker
bar, and choosing again replaces it. The session storage is written as you move between pages.

Other parts of the website keep nothing:

- **Search** runs in your browser, on an index the website publishes, and keeps nothing. Your words are put in
  the page's address (`?q=`) so a result page can be shared; when that address is loaded from the web host (on a
  reload, or from a shared link), the host receives it with the request, as it does any address.
- **The readiness sliders** on the path to production keep nothing, and a reload restores the author's
  estimates.
- **Share** opens the site you pick (email, LinkedIn, Reddit, Bluesky, or Mastodon) only when you pick it. That
  site's own privacy notice then applies.

## Who receives it

Local storage and session storage never leave your browser. The cookie is sent with each request your browser makes to this
website, as every cookie is; the website is static pages hosted by GitHub Pages, so no program of ours reads
it. The cookie is first-party (`SameSite=Lax`, and `Secure` over HTTPS), so it is not sent to any other site.

GitHub, which hosts the website, receives the requests your browser makes, as any web host does.
[GitHub's privacy statement](https://docs.github.com/en/site-policy/privacy-policies/github-general-privacy-statement)
says what it records.

## How long it is kept

The cookie expires 400 calendar days after you last chose a theme, the longest a browser keeps a cookie.
Local storage is kept until you clear it. Session storage is deleted when you close the tab.

## How to remove it

Clear this website's data in your browser's settings (cookies and site data). The website then behaves as on a
first visit. Choosing a theme or a text size keeps it again.

## Why there is no consent banner

These are kept only at your request, to show the website the way you asked, and hold nothing about you. That is
the kind of storage the UK and EU rules on cookies (the Privacy and Electronic Communications Regulations, and
the ePrivacy Directive) treat as strictly necessary for a service you asked for, which needs no consent. This
is the project's reading of those rules, not legal advice.

## The WPM application

This notice is for the website only. The WPM application keeps the same kinds of choices in the browser of each
person who uses it (theme, text size, and language). Whoever deploys the application writes the privacy notice
for the people whose records it holds ([the assessment checklist](../spec/governance/assessment-checklist.md),
question 3.10).

## Questions

Ask in the project's repository, <https://github.com/workforce-planning-management/workforce-planning-management>,
or write to the maintainer named there.
