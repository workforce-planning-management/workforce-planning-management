#!/usr/bin/env node
// Copy the two Lily theme stylesheets the site offers (light and dark) from the installed @lilydesignsystem/themes
// package into static/assets/themes/, where ThemePicker's `themesUrl` fetches them at run time. The same convention
// as the WPM UI's scripts/sync-themes.mjs: the copy is generated on every install (postinstall) and not committed.
import { cpSync, mkdirSync, rmSync } from "node:fs";
import { createRequire } from "node:module";
import { dirname, join } from "node:path";
import { fileURLToPath } from "node:url";

const require = createRequire(import.meta.url);
const site = dirname(dirname(fileURLToPath(import.meta.url)));
const dist = dirname(require.resolve("@lilydesignsystem/themes/light.css"));
const dest = join(site, "static", "assets", "themes");

// The themes the site offers and audits; ThemePicker's `themes` in src/routes/+layout.svelte lists the same two.
const THEMES = ["light", "dark"];

rmSync(dest, { recursive: true, force: true });
mkdirSync(dest, { recursive: true });
for (const t of THEMES) cpSync(join(dist, `${t}.css`), join(dest, `${t}.css`));
console.log(`sync-themes: copied ${THEMES.length} theme stylesheets to static/assets/themes/`);
