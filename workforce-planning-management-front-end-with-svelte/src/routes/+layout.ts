// SPA mode (family front-end convention, drift accepted): no SSR, no
// prerender — every route loads client-side through the same-origin
// BFF proxy. This also lets the Playwright suite stub the API with
// `page.route` and run without the Rust service.
export const ssr = false;
export const prerender = false;

import { listMyOrganizations, listMyOrganizationScope } from "$lib/api/wpm";
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
export const load: LayoutLoad = async ({ data, fetch }) => {
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
