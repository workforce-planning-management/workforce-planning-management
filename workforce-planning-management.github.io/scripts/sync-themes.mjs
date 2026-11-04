#!/usr/bin/env node
// Copy every Lily theme stylesheet (the picker bar offers them all) from the installed @lilydesignsystem/themes
// package into static/assets/themes/, where ThemePicker's `themesUrl` fetches them at run time. The same convention
// as the WPM UI's scripts/sync-themes.mjs: the copy is generated on every install (postinstall) and not committed.
import { cpSync, mkdirSync, readdirSync, rmSync } from "node:fs";
import { createRequire } from "node:module";
import { dirname, join } from "node:path";
import { fileURLToPath } from "node:url";

const require = createRequire(import.meta.url);
const site = dirname(dirname(fileURLToPath(import.meta.url)));
const dist = dirname(require.resolve("@lilydesignsystem/themes/light.css"));
const dest = join(site, "static", "assets", "themes");

rmSync(dest, { recursive: true, force: true });
mkdirSync(dest, { recursive: true });
const themes = readdirSync(dist).filter((f) => f.endsWith(".css"));
for (const t of themes) cpSync(join(dist, t), join(dest, t));
console.log(`sync-themes: copied ${themes.length} theme stylesheets to static/assets/themes/`);
