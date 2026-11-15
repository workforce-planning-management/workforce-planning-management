# Themes and contrast

WPM-R247, WPM-D196; task WPM-T497. Applies to the UI (`workforce-planning-management-ui-with-svelte/`) and the website
(`workforce-planning-management.github.io/`). The theme picker itself is
[lily-design-system-svelte-with-picker-bar](lily-design-system-svelte-with-picker-bar/index.md).

## WPM-R247 — Light and dark pass; any other theme is the reader's choice

**As** a reader, **I want** the light and the dark theme to meet WCAG 2.2 AA everywhere, and every other Lily theme
still on offer, **so that** the two themes I get by default are readable, and I can still choose a theme that suits me
even if it does not meet the standard.

Acceptance criteria:

1. **Light and dark must pass.** Every page of the UI and of the website, in the Lily `light` theme and in the Lily
   `dark` theme, has no violation that axe's automated WCAG A and AA rules find. The UI is checked with axe's
   `wcag2a` and `wcag2aa` tags on every page route (`tests/e2e/a11y-routes.spec.ts`); the website with its WCAG 2.2 AA
   tags, failing on a serious or critical violation (WPM-R217). A violation in either theme fails the run.
2. **Light and dark are the defaults.** A reader who has not chosen a theme gets light or dark: the website
   matches the browser's colour scheme; the UI starts in light whatever the browser asks for (checked 2026-11-15;
   following the browser there is not yet built).
3. **Every other Lily theme stays on offer and is not held to (1).** A reader may choose any theme the picker
   lists. A violation in a theme other than light or dark does not fail a run and is not a defect; the website's
   tests report it on the test as an annotation, so it can be read. No theme is removed because it fails.
4. **Every theme still loads cleanly.** Each theme on the website loads with no content security policy error, whatever
   its contrast.
5. **The fix goes in WPM's own styles, not in a theme.** WPM does not edit or override Lily's theme files to make
   light or dark pass; it chooses which theme tokens its own styles use (WPM-D196).

## WPM-D196 — WPM vouches for two themes, and the reader owns the rest

- **Two themes are vouched for.** Light and dark are what a reader sees without choosing, so they are the themes
  WPM is accountable for. Holding all 57 to WCAG would mean either removing most of them or overriding their colours
  until they stopped being themselves.
- **A chosen theme is the reader's decision.** A reader who picks a theme with low contrast, or a decorative one,
  has chosen how the page looks to them. The picker shows every Lily theme
  ([lily-design-system-svelte-with-picker-bar](lily-design-system-svelte-with-picker-bar/index.md)); this does not
  change that.
- **Reported, not failed.** The website still runs axe over every theme so a regression is visible, but only light
  and dark fail the run. The UI runs axe in light and dark only; another theme there is unchecked.
- **Text colours come from tokens that hold 4.5:1.** A theme's `--color-primary` and fixed colours like the
  closed-bed red are fill colours, not text colours: Lily dark's primary is 3.39:1 as link text on its panel, and
  4.12:1 under its own content colour. So, in the UI:
  - link text is `--link`, the theme's primary mixed 65 per cent with the reading colour;
  - a filled control (`button.primary`, `a.btn.primary`) is the reading and panel colours inverted, as on the website
    (WPM-D175);
  - error text is Lily's `--lily-error-text`, which each theme already mixes toward its reading colour.
- **Rejected:** overriding Lily's `dark.css` (it is copied from the package, and the next update would undo it);
  testing every theme in the UI and allowing failures (69 pages by 57 themes is 3,933 runs to report what no one
  is held to); dropping the themes that fail (the owner wants them kept, on 2026-11-15).
