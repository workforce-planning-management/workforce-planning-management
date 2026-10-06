import { defineEnvVars } from "@sveltejs/kit/env";

export const variables = defineEnvVars({
  WPM_API_URL: { schema: (input) => input || "http://localhost:5150" },
  AUTH_API_URL: { schema: (input) => input || "http://localhost:5150" },
  // Public address of this site, for the sitemap and robots.txt; blank = the
  // request origin (set it behind a proxy).
  WPM_PUBLIC_URL: { schema: (input) => input || "" },
});
