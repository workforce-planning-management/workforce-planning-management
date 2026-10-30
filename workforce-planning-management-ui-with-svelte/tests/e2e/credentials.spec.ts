// Credentials on /me and the skills crosswalk on /skills. The API is stubbed per test; an unstubbed /api/proxy call 404s loudly.

import { expect, test } from "@playwright/test";
import { WORKER, stubSignedIn } from "./support";

test.describe("Credentials", () => {
  test.beforeEach(async ({ page }) => {
    await stubSignedIn(page);
  });

  test("a worker issues a credential for a verified entry, sees it once, then revokes it", async ({
    page,
  }) => {
    const entry = (pid: string, verified: string | null) => ({
      pid,
      entry_date: "2026-09-01",
      activity: `Course ${pid}`,
      category: "course",
      unit: "hours",
      amount: 3,
      amount_hundredths: 300,
      evidence_note: null,
      evidence_url: null,
      source: "self",
      lms_url: null,
      verified_on: verified,
      verified_by: verified ? "hr" : null,
      skill_pid: null,
    });
    let issued: Record<string, unknown> | null = null;
    await page.route(
      `**/api/proxy/workers/${WORKER.pid}/cpd-entries`,
      (route) =>
        route.fulfill({ json: [entry("e1", "2026-09-02"), entry("e2", null)] }),
    );
    await page.route("**/api/proxy/me/credentials", (route) => {
      if (route.request().method() === "POST") {
        issued = {
          pid: "c1",
          source_pid: "e1",
          kind: "cpd_entry",
          issued_at: "2026-10-25T10:00:00Z",
          revoked_at: null,
          status_index: 0,
        };
        return route.fulfill({
          json: {
            pid: "c1",
            status_index: 0,
            format: "vc+jwt",
            credential: "aaa.bbb.ccc",
          },
        });
      }
      return route.fulfill({ json: { credentials: issued ? [issued] : [] } });
    });
    await page.route("**/api/proxy/me/credentials/c1/revoke", (route) => {
      issued = { ...(issued as object), revoked_at: "2026-10-25T11:00:00Z" };
      return route.fulfill({ json: { pid: "c1", revoked: true } });
    });

    await page.goto("/me");
    const panel = page.getByTestId("my-credentials");
    await expect(panel.getByTestId("credential-row")).toHaveCount(1); // only the verified entry
    await panel.getByTestId("credential-issue").click();
    await expect(panel.getByTestId("credential-token")).toHaveValue(
      "aaa.bbb.ccc",
    );
    await expect(panel.getByTestId("credential-state")).toContainText(
      "Issued 25 Oct 2026",
    );
    await panel.getByTestId("credential-revoke").click();
    await expect(panel.getByTestId("credential-token-box")).toHaveCount(0);
    await expect(panel.getByTestId("credential-state")).toContainText(
      "Revoked 25 Oct 2026",
    );
    await expect(panel.getByTestId("credential-issue")).toBeVisible();
  });

  test("the crosswalk lists mappings, says which count, and adds one", async ({
    page,
  }) => {
    const mappings: Array<Record<string, unknown>> = [
      {
        pid: "m1",
        from_skill_pid: "s1",
        to_skill_pid: "s2",
        from_skill: "Python",
        to_skill: "Scripting",
        confidence: "broader",
        basis: "judgement",
        source: "HR note",
        counts: false,
      },
    ];
    let posted: unknown = null;
    await page.route("**/api/proxy/skills", (route) =>
      route.fulfill({
        json: [
          {
            pid: "s1",
            name: "Python",
            category: "technical",
            external_refs: [],
          },
          {
            pid: "s2",
            name: "Scripting",
            category: "technical",
            external_refs: [],
          },
        ],
      }),
    );
    await page.route("**/api/proxy/skill-mappings", (route) => {
      if (route.request().method() === "POST") {
        posted = route.request().postDataJSON();
        mappings.push({
          pid: "m2",
          from_skill: "Scripting",
          to_skill: "Python",
          confidence: "exact",
          basis: "authority",
          source: "ESCO",
          counts: true,
          ...(posted as object),
        });
        return route.fulfill({ json: { pid: "m2" } });
      }
      return route.fulfill({ json: { note: "n", mappings } });
    });
    await page.goto("/skills");
    const panel = page.getByTestId("crosswalk");
    await expect(panel.getByTestId("crosswalk-row")).toContainText(
      "Related only",
    );
    await panel.getByTestId("crosswalk-from").selectOption("s2");
    await panel.getByTestId("crosswalk-to").selectOption("s1");
    await panel.getByTestId("crosswalk-source").fill("ESCO");
    await panel.getByTestId("crosswalk-add").click();
    await expect(panel.getByTestId("crosswalk-row")).toHaveCount(2);
    expect(posted).toEqual({
      from_skill_pid: "s2",
      to_skill_pid: "s1",
      confidence: "exact",
      basis: "authority",
      source: "ESCO",
    });
  });
});
