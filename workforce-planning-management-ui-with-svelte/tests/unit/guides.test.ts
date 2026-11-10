// The walkthroughs (WPM-R244) stay true to the app: every task has its words in English and starts from a page the app has.

import { readFileSync } from "node:fs";
import { join } from "node:path";
import { describe, expect, it } from "vitest";
import { GUIDES, GUIDE_ROLES, isGuideRole } from "../../src/lib/guides";
import { ROUTES } from "../e2e/axe-routes";

const en = JSON.parse(
  readFileSync(join(__dirname, "../../content/locales/en-001/ui.json"), "utf8"),
) as Record<string, unknown>;

function lookup(key: string): unknown {
  return key
    .split(".")
    .reduce<unknown>(
      (node, part) => (node as Record<string, unknown> | undefined)?.[part],
      en,
    );
}

describe("walkthroughs", () => {
  it("offer an employee, a manager and HR", () => {
    expect([...GUIDE_ROLES]).toEqual(["employee", "manager", "hr"]);
    expect(isGuideRole("hr")).toBe(true);
    expect(isGuideRole("ceo")).toBe(false);
  });

  for (const role of GUIDE_ROLES) {
    it(`${role}: every task has a title and a body, and starts from a real page`, () => {
      expect(typeof lookup(`guides.${role}.title`)).toBe("string");
      expect(typeof lookup(`guides.${role}.intro`)).toBe("string");
      expect(GUIDES[role].length).toBeGreaterThanOrEqual(4);
      for (const task of GUIDES[role]) {
        expect(typeof lookup(`guides.${role}.${task.id}.title`), task.id).toBe(
          "string",
        );
        expect(typeof lookup(`guides.${role}.${task.id}.body`), task.id).toBe(
          "string",
        );
        expect(Object.keys(ROUTES), task.path).toContain(task.path);
      }
    });
  }
});
