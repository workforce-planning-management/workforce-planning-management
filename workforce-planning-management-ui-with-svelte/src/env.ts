import { defineEnvVars } from "@sveltejs/kit/env";

export const variables = defineEnvVars({
  WPM_API_URL: { schema: (input) => input || "http://localhost:5150" },
  AUTH_API_URL: { schema: (input) => input || "http://localhost:5150" },
});
