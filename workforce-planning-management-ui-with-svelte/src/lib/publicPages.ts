// Which pages are reachable with no session, and which of those are worth
// indexing. Shared by the sign-in gate (`routes/+layout.server.ts`) and the
// sitemap, so the sitemap can never list a page the gate would redirect.

/** Route prefixes (without the locale) reachable with no session. */
export const PUBLIC_PATHS = ["/signin", "/verify", "/tour"] as const;

/** Whether `rest` (a path with its locale prefix removed) needs no session. `/`
 *  is exact-matched: every path starts with "/", so a prefix match would make the
 *  whole app public. */
export function isPublicPath(rest: string): boolean {
  return rest === "/" || PUBLIC_PATHS.some((path) => rest.startsWith(path));
}

/**
 * The public pages worth indexing: the splash, the tour and the sign-in page.
 * `/verify` is left out — it is a one-time-token landing page, not content.
 * Every signed-in page is absent by construction (they redirect to sign-in).
 */
export const INDEXABLE_PAGES = ["/", "/tour", "/signin"] as const;
