import { defineEnvVars } from "@sveltejs/kit/env";

export const variables = defineEnvVars({
  WPM_API_URL: { schema: (input) => input || "http://localhost:1241" },
  AUTH_API_URL: { schema: (input) => input || "http://localhost:1241" },
  // Public address of this site, for the sitemap and robots.txt; blank = the
  // request origin (set it behind a proxy).
  // The demo (the guided tour at /demo): the base URL of a SEPARATE service instance that holds only
  // fictitious data and runs with authentication off. Blank = no demo: signing in as the example address
  // is then an ordinary sign-in. Never point this at a service that holds real data.
  WPM_DEMO_API_URL: { schema: (input) => input || "" },
  WPM_PUBLIC_URL: { schema: (input) => input || "" },
});
