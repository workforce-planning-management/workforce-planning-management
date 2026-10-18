// The SSO entry point (`/signin/sso`, WPM-T38) sends the browser to the authentication service's SSO
// start with this app's own `/verify/sso` as the return address. It once threw a 500: SvelteKit's
// `redirect()` refuses an address on another origin. Not signed in: the route is public.

import { expect, test } from "@playwright/test";

test("redirects to the authentication service's SSO start, under a locale prefix too", async ({
  request,
}) => {
  for (const path of [
    "/signin/sso",
    "/en-us/signin/sso",
    "/cy-001/signin/sso",
  ]) {
    const response = await request.get(path, { maxRedirects: 0 });
    expect(response.status(), path).toBe(303);
    const location = new URL(response.headers()["location"]!);
    expect(location.pathname).toBe("/api/auth/sso/start");
    expect(location.searchParams.get("return_url")).toMatch(/\/verify\/sso$/);
  }
});
