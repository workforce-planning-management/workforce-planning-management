// Before-and-after record for splitting the largest pages (WPM-T267): the worker page, planning and the role
// profiles, in light, dark and a phone width. Does nothing unless PAGE_SHOTS=<dir> is set. It writes a PNG and a
// plain-text dump of each page, so two runs can be compared with `diff -r`, and the PNGs looked at.
//
//   PAGE_SHOTS=/tmp/before pnpm exec playwright test tests/e2e/page-shots.spec.ts

import { mkdirSync, writeFileSync } from "node:fs";
import { join } from "node:path";
import { test, type Page } from "@playwright/test";
import { WORKER } from "./support";
import { base, stubPlanning, stubRoles, stubWorkerPage } from "./fixtures";

const dir = process.env.PAGE_SHOTS;

const VIEWS = [
  { name: "light", scheme: "light", width: 1280, height: 900 },
  { name: "dark", scheme: "dark", width: 1280, height: 900 },
  { name: "phone", scheme: "light", width: 390, height: 844 },
] as const;

const PAGES: Array<{ name: string; path: string; stub: (p: Page) => Promise<void>; ready: string }> = [
  { name: "worker", path: `/workers/${WORKER.pid}`, stub: stubWorkerPage, ready: "adjustments" },
  { name: "planning", path: "/planning", stub: stubPlanning, ready: "" },
  { name: "roles", path: "/roles", stub: stubRoles, ready: "" },
];

test.describe("page shots (WPM-T267)", () => {
  test.skip(!dir, "set PAGE_SHOTS=<dir> to write screenshots and text dumps");
  for (const view of VIEWS) {
    for (const spec of PAGES) {
      test(`${spec.name} — ${view.name}`, async ({ page }) => {
        mkdirSync(dir!, { recursive: true });
        await page.setViewportSize({ width: view.width, height: view.height });
        await page.emulateMedia({ colorScheme: view.scheme });
        await base(page);
        await spec.stub(page);
        if (process.env.PAGE_SHOTS_DEBUG) {
          page.on("console", (m) => m.type() === "error" && console.log("CONSOLE", m.text().slice(0, 400)));
          page.on("pageerror", (e) => console.log("PAGEERROR", e.message.slice(0, 300)));
          page.on("response", (r) => r.status() === 404 && r.url().includes("/api/proxy") && console.log("404", r.url()));
        }
        await page.goto(spec.path);
        if (spec.ready) await page.getByTestId(spec.ready).waitFor();
        else await page.getByRole("heading", { level: 1 }).waitFor();
        await page.waitForLoadState("networkidle");
        await page.waitForTimeout(500);
        const stem = `${spec.name}-${view.name}`;
        await page.screenshot({ path: join(dir!, `${stem}.png`), fullPage: true });
        const text = await page.locator("main, body").first().innerText();
        writeFileSync(join(dir!, `${stem}.txt`), text + "\n");
      });
    }
  }
});
