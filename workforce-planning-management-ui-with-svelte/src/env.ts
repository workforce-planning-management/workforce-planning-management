import { defineEnvVars } from "@sveltejs/kit/env";

/** The settings prefix (WPM-R278) and the earlier ones, newest first, still read with a warning (WPM-D221). */
const PREFIX = "WORKFORCE_PLANNING_MANAGEMENT_";
const LEGACY_PREFIXES = ["WPM_", "HCM_"];
const warned = new Set<string>();

/**
 * The value of `name`, or, when it is blank, of its legacy spelling (`WPM_…`, then `HCM_…`), warning once per variable.
 * The rename is transitional: the fallback goes once no deployment sets the old names.
 */
export function withLegacy(
  name: string,
  input: string | undefined,
  env: Record<string, string | undefined> = globalThis.process?.env ?? {},
): string {
  if (input?.trim()) return input;
  const rest = name.startsWith(PREFIX) ? name.slice(PREFIX.length) : null;
  if (rest === null) return "";
  for (const prefix of LEGACY_PREFIXES) {
    const old = `${prefix}${rest}`;
    const value = env[old];
    if (value?.trim()) {
      if (!warned.has(old)) {
        warned.add(old);
        console.warn(
          `${old} is deprecated; rename it to ${name}. The old name will stop being read.`,
        );
      }
      return value;
    }
  }
  return "";
}

export const variables = defineEnvVars({
  WORKFORCE_PLANNING_MANAGEMENT_API_URL: {
    schema: (input) =>
      withLegacy("WORKFORCE_PLANNING_MANAGEMENT_API_URL", input) ||
      "http://localhost:1241",
  },
  AUTH_API_URL: { schema: (input) => input || "http://localhost:1241" },
  // Public address of this site, for the sitemap and robots.txt; blank = the
  // request origin (set it behind a proxy).
  // The demo (the guided tour at /demo): the base URL of a SEPARATE service instance that holds only
  // fictitious data and runs with authentication off. Blank = no demo: signing in as the example address
  // is then an ordinary sign-in. Never point this at a service that holds real data.
  WORKFORCE_PLANNING_MANAGEMENT_DEMO_API_URL: {
    schema: (input) =>
      withLegacy("WORKFORCE_PLANNING_MANAGEMENT_DEMO_API_URL", input),
  },
  WORKFORCE_PLANNING_MANAGEMENT_PUBLIC_URL: {
    schema: (input) =>
      withLegacy("WORKFORCE_PLANNING_MANAGEMENT_PUBLIC_URL", input),
  },
});
