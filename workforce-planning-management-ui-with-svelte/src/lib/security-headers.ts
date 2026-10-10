// Security response headers for every page and BFF response (WPM-R73).
//
// The content security policy itself comes from `kit.csp` in
// `vite.config.ts` (SvelteKit adds the nonces and hashes for its own inline
// scripts); this module adds the rest, as a pure function so a unit test can
// pin it. `frame-ancestors` is in the policy; `X-Frame-Options` is the same
// rule for older browsers.

/** The headers to add to a response for `url`. `forwardedProto` is the
 *  `x-forwarded-proto` request header a TLS-terminating proxy sets. */
export function securityHeaders(
  url: URL,
  forwardedProto: string | null,
): Record<string, string> {
  const headers: Record<string, string> = {
    "x-content-type-options": "nosniff",
    "x-frame-options": "DENY",
    "referrer-policy": "strict-origin-when-cross-origin",
    "permissions-policy":
      "camera=(), microphone=(), geolocation=(), payment=(), usb=()",
    "cross-origin-opener-policy": "same-origin",
    "cross-origin-resource-policy": "same-origin",
  };
  const https =
    url.protocol === "https:" || forwardedProto?.toLowerCase() === "https";
  if (https) {
    headers["strict-transport-security"] =
      "max-age=63072000; includeSubDomains";
  }
  // The content manager signs in through a popup, and `same-origin` severs the
  // popup's link back to the opener; `same-origin-allow-popups` keeps that
  // working while still isolating everything else.
  if (/(^|\/)admin(\/|$)/.test(url.pathname)) {
    headers["cross-origin-opener-policy"] = "same-origin-allow-popups";
  }
  // The BFF proxy carries personal data; never let a cache keep it.
  if (url.pathname.startsWith("/api/")) {
    headers["cache-control"] = "no-store";
  }
  return headers;
}
