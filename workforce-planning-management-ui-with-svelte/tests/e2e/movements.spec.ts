// Joiners and leavers. The API is stubbed per test; an unstubbed /api/proxy call 404s loudly.

import { expect, test } from "@playwright/test";
import { MASKED_WORKER, WORKER, stubSignedIn } from "./support";

test.describe("Joiners and leavers", () => {
  test.beforeEach(async ({ page }) => {
    await stubSignedIn(page);
  });

  test("joiners and leavers: checklist, last-day handover and audit trail", async ({
    page,
  }) => {
    const mv = "bbbbbbbb-0000-4000-8000-00000000b001";
    const movement = {
      pid: mv,
      kind: "leaver",
      worker_pid: WORKER.pid,
      worker_name: WORKER.display_name,
      department: "engineering",
      job_title: "Engineer",
      organization_ref: WORKER.organization_ref,
      effective_on: "2026-11-30",
      reason: "resignation",
      status: "open",
      notes: null,
      completed_at: null,
      progress: { closed: 1, total: 3, overdue: 1 },
    };
    const items = [
      {
        pid: "i1",
        position: 0,
        title: "Notice acknowledged and last day agreed",
        category: "admin",
        due_on: "2026-11-02",
        assignee_pid: null,
        assignee_name: null,
        done_on: null,
        done_by: null,
        skipped_reason: null,
        state: "overdue",
      },
      {
        pid: "i2",
        position: 1,
        title: "Equipment returned",
        category: "equipment",
        due_on: "2026-11-30",
        assignee_pid: null,
        assignee_name: null,
        done_on: "2026-11-30",
        done_by: null,
        skipped_reason: null,
        state: "done",
      },
      {
        pid: "i3",
        position: 2,
        title: "Exit interview offered",
        category: "people",
        due_on: "2026-11-27",
        assignee_pid: null,
        assignee_name: null,
        done_on: null,
        done_by: null,
        skipped_reason: "declined",
        state: "skipped",
      },
    ];
    let held = [
      {
        kind: "direct_report",
        subject_pid: "r1",
        label: "Ada Reports",
        can_reassign: true,
        needs_new_holder: true,
      },
      {
        kind: "access",
        subject_pid: "a1",
        label: "organization:x — hr_admin",
        can_reassign: false,
        needs_new_holder: false,
      },
    ];
    const trail: Array<Record<string, unknown>> = [];
    await page.route("**/api/proxy/movements?limit=500", (route) =>
      route.fulfill({ json: [movement] }),
    );
    await page.route(`**/api/proxy/movements/${mv}`, (route) =>
      route.fulfill({ json: { ...movement, items } }),
    );
    await page.route(`**/api/proxy/movements/${mv}/handover`, (route) =>
      route.fulfill({
        json: {
          last_day: "2026-11-30",
          remaining: held.length,
          counts: {},
          items: held,
        },
      }),
    );
    await page.route(`**/api/proxy/movements/${mv}/handover/all`, (route) => {
      trail.push(
        {
          kind: "direct_report",
          subject_pid: "r1",
          label: "Ada Reports",
          action: "reassigned",
          to_worker_pid: MASKED_WORKER.pid,
          to_worker_name: MASKED_WORKER.display_name,
          note: null,
          performed_by: null,
          performed_at: "2026-11-30T09:00:00Z",
        },
        {
          kind: "access",
          subject_pid: "a1",
          label: "organization:x — hr_admin",
          action: "revoked",
          to_worker_pid: null,
          to_worker_name: null,
          note: null,
          performed_by: null,
          performed_at: "2026-11-30T09:00:01Z",
        },
      );
      held = [];
      return route.fulfill({
        json: { handed_over: 1, access_revoked: 1, failed: [], remaining: 0 },
      });
    });
    await page.route(`**/api/proxy/movements/${mv}/handover/actions`, (route) =>
      route.fulfill({ json: trail }),
    );
    await page.route(`**/api/proxy/movements/${mv}/complete`, (route) =>
      route.fulfill({
        status: 422,
        json: { description: "1 checklist item(s) still open" },
      }),
    );

    // The list: leavers with progress, and the late count.
    await page.goto("/movements");
    const list = page.getByTestId("leavers-table");
    await expect(list).toContainText(WORKER.display_name);
    await expect(list).toContainText("1/3");
    await expect(page.getByTestId("overdue-chip")).toContainText("1 overdue");

    // The record: dated checklist states (icon + label, not colour alone).
    await page.goto(`/movements/${mv}`);
    const checklist = page.getByTestId("movement-items");
    await expect(checklist.locator('tr[data-state="overdue"]')).toContainText(
      "Overdue",
    );
    await expect(checklist.locator('tr[data-state="done"]')).toContainText(
      "Done",
    );
    await expect(checklist.locator('tr[data-state="skipped"]')).toContainText(
      "declined",
    );

    // The handover: what they hold; access is revoked, never handed over.
    const heldTable = page.getByTestId("held-items");
    await expect(heldTable).toContainText("Ada Reports");
    await expect(heldTable).toContainText("Revoke");
    // Completion is refused while anything is open, and the service says what.
    await page.getByTestId("movement-complete").click();
    await expect(page.getByTestId("error")).toContainText(
      "1 checklist item(s) still open",
    );

    // Hand everything to one person: the table empties and the trail shows each action.
    await page.getByTestId("hand-all-to").selectOption(MASKED_WORKER.pid);
    await page.getByTestId("hand-all").click();
    await expect(page.getByTestId("held-none")).toBeVisible();
    const log = page.getByTestId("handover-trail");
    await expect(log).toContainText("reassigned");
    await expect(log).toContainText(MASKED_WORKER.display_name);
    await expect(log).toContainText("revoked");
  });
});
