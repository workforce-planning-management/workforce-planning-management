// The content manager's page (WPM-T249): the pinned CMS script and the policy that lets only it, and the inline
// language script, run.

import { createHash } from "node:crypto";

/** The CMS release, pinned; a new version needs a new hash. */
export const CMS_SCRIPT = {
  src: "https://unpkg.com/@sveltia/cms@0.230.0/dist/sveltia-cms.js",
  integrity:
    "sha384-AzagiFkRmjAMDuffoNGXass0+WVk9WahF+XJyeht4DDPxO1MHHYDp84YqPoN+XFv",
};

/** The script that sets the CMS's interface language before it starts. */
export function prefsScript(ui: string | undefined): string {
  return `
      try {
        const key = "sveltia-cms.prefs";
        const prefs = JSON.parse(localStorage.getItem(key) || "{}") || {};
        prefs.locale = ${JSON.stringify(ui)};
        localStorage.setItem(key, JSON.stringify(prefs));
      } catch {
        // A blocked store only costs the language choice.
      }
    `;
}

/** The policy for this page, allowing `inline` by its hash. */
export function adminCsp(inline: string): string {
  const hash = createHash("sha256").update(inline).digest("base64");
  return [
    "default-src 'self'",
    `script-src 'self' https://unpkg.com 'sha256-${hash}'`,
    "style-src 'self' 'unsafe-inline'",
    "font-src 'self' https://cdn.jsdelivr.net",
    "img-src 'self' blob: data: https://avatars.githubusercontent.com",
    "media-src blob:",
    "frame-src blob:",
    "connect-src 'self' blob: data: https://unpkg.com https://api.github.com https://www.githubstatus.com",
    "manifest-src blob:",
    "object-src 'none'",
    "base-uri 'self'",
    "form-action 'self'",
    "frame-ancestors 'none'",
  ].join("; ");
}
