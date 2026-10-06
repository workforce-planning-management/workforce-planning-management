// Server-side (BFF) configuration. Never imported by browser code.

import {
  WPM_API_URL as WPM_API_URL_ENV,
  AUTH_API_URL as AUTH_API_URL_ENV,
  WPM_PUBLIC_URL as WPM_PUBLIC_URL_ENV,
} from "$app/env/private";

/** Base URL of the workforce-planning-management service the BFF proxies to. */
export const WPM_API_URL: string = WPM_API_URL_ENV;

/** Authentication service base URL — for the session→PASETO exchange
 *  and the magic-link login flow (WPM-T18). */
export const AUTH_API_URL: string = AUTH_API_URL_ENV;

/** Public address of this site (no trailing slash), for the sitemap and
 *  robots.txt; empty = use the request origin. */
export const WPM_PUBLIC_URL: string = WPM_PUBLIC_URL_ENV;
