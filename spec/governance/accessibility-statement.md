# Accessibility statement (WPM-R92)

> ⚠️ **This software has not been audited for accessibility.** This is a statement of what
> is known and not known, written so a deployer can publish an accurate one of their own.
> A deployer must publish their own statement for their own deployment, after testing it.

**Target standard**: Web Content Accessibility Guidelines 2.2, level AA.
**Status against the target**: **not verified.** No audit, no assistive-technology testing
and no automated accessibility test (such as axe) has been run on this software.

## What is built in

- Semantic structure from Svelte components, labelled form controls, and 35 ARIA
  attributes across the pages and components.
- **Text size**: a picker offers seven sizes, remembered per browser.
- **Themes**: a picker offers forty-five themes, including accessible (high-contrast and
  government design-system) variants. Contrast has not been measured across them.
- **Language and direction**: the page declares its language, and right-to-left languages
  (Arabic, Urdu) set the page direction.
- **Responsive layout**: checked by the end-to-end tests for overflow at phone and desktop
  sizes in light and dark themes; the CEO dashboard is built to fit one screen.
- **Focus**: visible focus styles exist in the theme stylesheets and on the dashboard
  controls; they have not been checked everywhere.

## What is known to be missing

- **No skip-to-content link.**
- **No automated accessibility check** in the test suite, so regressions would not be caught.
- **No screen-reader, keyboard-only or voice-control test** has been done on any page.
- **Contrast** has not been measured for all forty-five themes.
- **Translations are unreviewed.** Every page's text is in the catalogue in 13 languages, but all
  of it except the English was written by an AI assistant and no person has reviewed it
  ([official-language statement](official-language-statement.md)). Messages the server sends are
  in English.
- **Drag-and-drop**: the requisition board uses a board component; a keyboard alternative
  has not been verified (WCAG 2.2 criterion 2.5.7).
- **Target size** (WCAG 2.2 criterion 2.5.8) has not been measured.
- **The content manager** (`/admin`) is a third-party tool loaded from a public CDN, not
  covered by anything above.

## How to test before you publish a statement

1. Run an automated checker over every page in each theme and language (axe or similar) and
   fix what it finds; add it to your tests.
2. Walk every task with a keyboard only; check focus order, visible focus, and no traps.
3. Use at least one screen reader on the main tasks (sign in, find a person, request leave,
   read a payslip).
4. Check contrast for the themes you offer; remove the ones that fail.
5. Check zoom to 200% and 400%, and reflow at a 320-pixel-wide viewport.
6. Record what you tested, with which tools, on which date, and what failed.

## Feedback and enforcement

Deployer to complete: how a person reports an accessibility problem, the response time, and
the body that handles an unresolved complaint.
