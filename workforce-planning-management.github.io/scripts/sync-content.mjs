#!/usr/bin/env node
// Copy the repository documents this site shows into src/content/docs.json, so the site builds on its own once
// `git subtree` has exported this directory (the exported repository has no ../README.md to read).
//
//   node scripts/sync-content.mjs           rewrite src/content/docs.json from the monorepo
//   node scripts/sync-content.mjs --check   exit 1 if src/content/docs.json differs from the monorepo files
//
// Run from the monorepo only (it reads the parent directory). The list of documents is here and in
// src/lib/site.ts (DOC_PAGES); the build fails if a page names a document that is not in the copy.
import { readFileSync, writeFileSync, readdirSync, existsSync, statSync } from "node:fs";
import { dirname, join } from "node:path";
import { fileURLToPath } from "node:url";

const SITE = join(dirname(fileURLToPath(import.meta.url)), "..");
const ROOT = join(SITE, "..");
const OUT = join(SITE, "src", "content", "docs.json");

// The documents, by repository path. The role briefs are every locales/<locale>/for/<role>/index.md.
const DOCS = [
  "index.md",
  "README.md",
  "INSTALL.md",
  "docs/tutorials/family-bridges.md",
  "spec/index.md",
  "spec/worldwide.md",
];

/** @param {string} dir */
function peer(dir) {
  const p = join(ROOT, dir, ".locale-peer-id");
  return existsSync(p) ? readFileSync(p, "utf8").trim() : null;
}

// Each locale's list (for/index.md) and briefs, with the `.locale-peer-id` that names the same page in every locale:
// a brief's directory name is translated (`chief-risk-officer`, `prif-swyddog-risg`, `رئيس-المخاطر`), its peer id is not.
function briefs() {
  const lists = [];
  const pages = [];
  for (const locale of readdirSync(join(ROOT, "locales")).sort()) {
    const forDir = `locales/${locale}/for`;
    if (!existsSync(join(ROOT, forDir, "index.md"))) continue;
    lists.push({ locale, path: `${forDir}/index.md`, peer: peer(forDir) });
    for (const slug of readdirSync(join(ROOT, forDir)).sort()) {
      const dir = `${forDir}/${slug}`;
      if (!statSync(join(ROOT, dir)).isDirectory() || !existsSync(join(ROOT, dir, "index.md"))) continue;
      pages.push({ locale, slug, path: `${dir}/index.md`, peer: peer(dir) });
    }
  }
  return { lists, pages };
}

function render() {
  const { lists, pages } = briefs();
  /** @type {Record<string, string>} */
  const files = {};
  for (const p of [...DOCS, ...lists.map((l) => l.path), ...pages.map((b) => b.path)])
    files[p] = readFileSync(join(ROOT, p), "utf8");
  return (
    JSON.stringify(
      { generated: "by scripts/sync-content.mjs from the monorepo; do not edit by hand", lists, briefs: pages, files },
      null,
      2,
    ) + "\n"
  );
}

if (!existsSync(join(ROOT, "locales")) || !existsSync(join(ROOT, "spec", "index.md"))) {
  console.error("sync-content: run this inside the monorepo (the parent directory has no locales/ or spec/)");
  process.exit(2);
}
const text = render();
if (process.argv.includes("--check")) {
  const current = existsSync(OUT) ? readFileSync(OUT, "utf8") : "";
  if (current !== text) {
    console.error("src/content/docs.json is stale: run `pnpm run sync` (node scripts/sync-content.mjs)");
    process.exit(1);
  }
  console.log("src/content/docs.json is current");
} else {
  writeFileSync(OUT, text);
  const data = JSON.parse(text);
  console.log(
    `wrote src/content/docs.json (${Object.keys(data.files).length} documents: ${data.lists.length} locales, ${data.briefs.length} briefs)`,
  );
}
