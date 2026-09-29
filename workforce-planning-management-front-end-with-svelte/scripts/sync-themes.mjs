// Copies the Lily reference theme stylesheets from the installed
// @lilydesignsystem/themes package into static/assets/themes/, where
// ThemePicker's `themesUrl="/assets/themes/"` expects to fetch them at
// runtime. Themes are no longer hand-vendored/committed here — this
// script is the single source of truth, re-run on every install so the
// static copy always matches whatever version is pinned in
// package.json. See @lilydesignsystem/themes' own README ("Copy into
// your own static assets") for the upstream-documented convention this
// follows.
import { cpSync, mkdirSync, readdirSync, rmSync } from "node:fs";
import { createRequire } from "node:module";
import { dirname, join } from "node:path";
import { fileURLToPath } from "node:url";

const require = createRequire(import.meta.url);
const projectRoot = dirname(dirname(fileURLToPath(import.meta.url)));

const themesPackageDist = dirname(
  require.resolve("@lilydesignsystem/themes/light.css"),
);
const destDir = join(projectRoot, "static", "assets", "themes");

rmSync(destDir, { recursive: true, force: true });
mkdirSync(destDir, { recursive: true });

let count = 0;
for (const entry of readdirSync(themesPackageDist)) {
  if (!entry.endsWith(".css")) continue;
  cpSync(join(themesPackageDist, entry), join(destDir, entry));
  count += 1;
}

console.log(`sync-themes: copied ${count} theme stylesheets to static/assets/themes/`);
