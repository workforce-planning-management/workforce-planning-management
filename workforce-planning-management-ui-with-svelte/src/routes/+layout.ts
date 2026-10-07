// SPA mode (family front-end convention, drift accepted): no SSR, no
// prerender — every route loads client-side through the same-origin
// BFF proxy. This also lets the Playwright suite stub the API with
// `page.route` and run without the Rust service.
export const ssr = false;
export const prerender = false;

import { browser } from "$app/env";
import { redirect } from "@sveltejs/kit";
import { listMyOrganizations, listMyOrganizationScope } from "#lib/api/wpm.js";
import { localeFromNavigator, localePath } from "#lib/locales.js";
import type { LayoutLoad } from "./$types";

// Multi-organization membership (no switcher): the caller's org set is
// fetched once here, in the root layout's own *universal* load — not
// `+layout.server.ts`, whose `fetch` would make a real server-to-server
// call invisible to `page.route` (see that file's own note) — so every
// page inherits it via `page.data.organizations`/`page.data.scope`
// without re-fetching. Runs in the browser under this app's `ssr=false`
// SPA mode, exactly like every other data fetch in this app. Degrades
// to `[]` on any proxy/backend failure, matching this app's "always
// boots" pattern elsewhere (e.g. the theme migration guard in
// +layout.svelte).
//
// `organizations` (literal membership rows: pid/role/employed) and
// `scope` (a flat list of every organization ref the caller can read,
// expanded through org confederation — a membership in a parent
// reads as every descendant too) are DIFFERENT sets once confederation
// is in play: `scope` is a superset. The "My organizations" list on
// the dashboard uses `organizations` (it shows literal grants); pages
// that render one section per readable organization (`/org-chart`,
// `/benchmarks`) use `scope`.
// A bare `/` (no locale prefix, and no remembered locale — the server hook
// redirects those itself) is sent to the locale route for the browser's own
// language: `navigator.languages`, so `cy_GB` goes to `/cy-001/` (or `/cy-gb/` if
// the app had one). A browser language the app does not serve falls back to what
// the server negotiated from `Accept-Language`. Here, in the load, so the page
// never renders unprefixed.
export const load: LayoutLoad = async ({ data, fetch, url }) => {
  if (browser && url.pathname === "/") {
    const target = localeFromNavigator(navigator) ?? data.fallbackLocale;
    redirect(307, `${localePath(target, "/")}${url.search}`);
  }
  if (!data.signedIn) return { ...data, organizations: [], scope: [] };
  try {
    const [organizations, scope] = await Promise.all([
      listMyOrganizations({ fetch }),
      listMyOrganizationScope({ fetch }),
    ]);
    return { ...data, organizations, scope };
  } catch {
    return { ...data, organizations: [], scope: [] };
  }
};
