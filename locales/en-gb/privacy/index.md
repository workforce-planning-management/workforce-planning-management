# Privacy notice

This notice says what the Workforce Planning Management (WPM) website and the WPM application keep in your
browser, why, for how long, and how to remove it. The website is
<https://workforce-planning-management.github.io/>; the application is the WPM demonstration software, wherever
it is run. (Updated 2026-11-17, WPM-R253.)

Neither one has analytics, advertising, or third-party scripts, fonts, or images. Nothing either one keeps in
your browser is used to follow you from site to site.

## The website

The website is a set of static pages. It has no sign-in and no forms. What it keeps is in your own browser:

| What | Where it is kept | Name | What is kept | Why |
| --- | --- | --- | --- | --- |
| Your theme | Your browser's local storage | `mxi.wpm.theme` | The theme's name, for example `dark` | So the next page and your next visit look the way you chose |
| Your theme | A cookie for this website only | `wpm-theme` | The theme's name, for example `dark` | So the theme comes back even if your browser keeps cookies but clears local storage, or the other way round |
| Your text size | Your browser's local storage | `mxi.wpm.text-size` | The size's name, for example `large` | So the next page and your next visit use the text size you chose |
| Your language | Nothing is kept | | | The language is part of each page's address (for example `/cy-001/`), so there is nothing to remember |
| Where you were on a page | Your browser's session storage, for this tab only | `sveltekit:history-info`, `sveltekit:navigation-snapshot`, `sveltekit:snapshot` | Scroll positions, and the order of the pages you opened in this tab | So Back and Forward return you to where you were. The framework the website is built with (SvelteKit) keeps these |

No choice is kept until you make it. Until you choose, the website follows your browser's light or dark setting
and its normal text size, and keeps no theme or text size. A choice is kept when you make it in the picker bar,
and choosing again replaces it. The session storage is written as you move between pages.

Other parts of the website keep nothing:

- **Search** runs in your browser, on an index the website publishes, and keeps nothing. Your words are put in
  the page's address (`?q=`) so a result page can be shared; when that address is loaded from the web host (on a
  reload, or from a shared link), the host receives it with the request, as it does any address.
- **The readiness sliders** on the path to production keep nothing, and a reload restores the author's
  estimates.
- **Share** opens the site you pick (email, LinkedIn, Reddit, Bluesky, or Mastodon) only when you pick it. That
  site's own privacy notice then applies.

## The WPM application

The application keeps the same kinds of choices, and the cookie that keeps you signed in:

| What | Where it is kept | Name | What is kept | Why |
| --- | --- | --- | --- | --- |
| Your theme | Your browser's local storage | `mxi.wpm.theme` | The theme's name | So every page looks the way you chose |
| Your text size | Your browser's local storage | `mxi.wpm.text-size` | The size's name | So every page uses the text size you chose |
| Your language | Your browser's local storage | `mxi.wpm.locale` | The language code of the last page you opened, for example `cy-001` | So an address without a language opens in yours |
| Your language | A cookie for the application only | `wpm-locale` | The same language code | So the server can send an address without a language to yours |
| Your sign-in | A cookie for the application only, which the page's scripts cannot read | `__Host-mxi_session` (`mxi_session` on a developer's own computer) | A random number that stands for your signed-in session | So you stay signed in from page to page; without it you would sign in on every page |
| The content editor's language | Your browser's local storage, only if you open the content editor (`/admin`) | `sveltia-cms.prefs` | The editor's settings, among them its language | So the editor opens in the application's language |
| Where you were on a page | Your browser's session storage, for this tab only | `sveltekit:history-info`, `sveltekit:navigation-snapshot`, `sveltekit:snapshot` | Scroll positions, and the order of the pages you opened in this tab | So Back and Forward return you to where you were |

Unlike the website, the application keeps the theme and the text size as soon as it shows them, before you
choose, and keeps the language of each page you open. Keys from before the project's rename (`mxi.hcm.theme`, `mxi.hcm.locale`) are moved to the
names above and deleted.

The sign-in cookie is the only item that stands for you. The random number in it means nothing by itself: the
signed-in session it points to is kept by the application's server, and signing out ends it. What the application
holds about the people in its records is covered in the last section but one, below.

## Who receives it

Local storage and session storage never leave your browser.

A cookie is sent with each request your browser makes to the site that set it, and to no other site
(`SameSite=Lax`, and `Secure` over HTTPS):

- The website's `wpm-theme` cookie goes to GitHub Pages, which hosts the website. No program of the project's
  reads it.
- The application's `wpm-locale` and sign-in cookies go to the server the application runs on, which reads them
  to choose your language and to know you are signed in.

GitHub, which hosts the website, receives the requests your browser makes, as any web host does.
[GitHub's privacy statement](https://docs.github.com/en/site-policy/privacy-policies/github-general-privacy-statement)
says what it records. The organisation that runs the application decides what its server records.

## How long it is kept

- The website's `wpm-theme` cookie: 400 calendar days after you last chose a theme, the longest a browser keeps a
  cookie.
- The application's `wpm-locale` cookie: 365 calendar days after you last opened a page.
- The sign-in cookie: until you sign out or close the browser.
- Local storage: until you clear it.
- Session storage: until you close the tab.

## How to remove it

Clear the site's data in your browser's settings (cookies and site data), for the website, the application, or
both. The website then behaves as on a first visit; the application asks you to sign in again. Signing out of the
application removes the sign-in cookie.

## Why there is no consent banner

Each item is kept to give you the service you asked for: the website or the application the way you set it, in
your language, and, in the application, signed in. That is the kind of storage the UK and EU rules on cookies (the
Privacy and Electronic Communications Regulations, and the ePrivacy Directive) treat as strictly necessary, which
needs no consent. This is the project's reading of those rules, not legal advice.

## The records the application holds

This notice covers what is kept in your browser. The application also holds records about people (employees,
applicants, and their contacts) on its server. Whoever runs the application is responsible for those records and
writes the privacy notice for the people in them
([the assessment checklist](../../../spec/governance/assessment-checklist.md), question 3.10). The demonstration
software holds synthetic data only.

## Questions

Ask in the project's repository, <https://github.com/workforce-planning-management/workforce-planning-management>,
or write to the maintainer named there.
