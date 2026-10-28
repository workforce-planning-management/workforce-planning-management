// An accessibility (axe) check covers every page route. `tests/e2e/axe-routes.ts` lists the URL visited for each route and
// `tests/e2e/a11y-routes.spec.ts` runs axe on all of them; this test keeps that list whole.

import { readdirSync, statSync } from "node:fs";
import { join, relative } from "node:path";
import { describe, expect, it } from "vitest";

import { EXCLUDED, ROUTES } from "../e2e/axe-routes";

const ROUTES_DIR = join(__dirname, "../../src/routes");

/** Every directory under `src/routes` that holds a `+page.svelte`, as a route (`[param]` kept; the root is `/`). */
function pageRoutes(dir = ROUTES_DIR): string[] {
  return readdirSync(dir).flatMap((name) => {
    const path = join(dir, name);
    if (statSync(path).isDirectory()) return pageRoutes(path);
    if (name !== "+page.svelte") return [];
    const rel = relative(ROUTES_DIR, dir).split("\\").join("/");
    return [rel === "" ? "/" : `/${rel}`];
  });
}

describe("axe covers every page route", () => {
  const found = pageRoutes().sort();
  const listed = [...Object.keys(ROUTES), ...Object.keys(EXCLUDED)].sort();

  it("finds the pages", () => {
    expect(found.length).toBeGreaterThan(40);
  });

  it("every page route is checked or excluded with a reason", () => {
    const missing = found.filter((r) => !listed.includes(r));
    expect(missing, `add these to tests/e2e/axe-routes.ts: ${missing}`).toEqual(
      [],
    );
  });

  it("names no route that is gone", () => {
    const gone = listed.filter((r) => !found.includes(r));
    expect(gone, `remove these from tests/e2e/axe-routes.ts: ${gone}`).toEqual(
      [],
    );
  });

  it("no route is in both tables, and every exclusion has a reason", () => {
    expect(Object.keys(ROUTES).filter((r) => r in EXCLUDED)).toEqual([]);
    for (const [route, reason] of Object.entries(EXCLUDED)) {
      expect(reason.trim().length, route).toBeGreaterThan(10);
    }
  });

  it("every URL belongs to its route", () => {
    for (const [route, url] of Object.entries(ROUTES)) {
      const pattern = new RegExp(
        "^" +
          route
            .replace(/[.*+?^${}()|\\]/g, "\\$&")
            .replace(/\[[^\]]+\]/g, "[^/]+") +
          "$",
      );
      expect(url, route).toMatch(pattern);
    }
  });
});
