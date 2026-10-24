// Groups (`/groups`): communities of practice and interest. A group lists its members and what it knows in
// aggregate, with a skill withheld when too few people declare it. The API is stubbed per test; an unstubbed
// /api/proxy call 404s loudly.

import { expect, test } from "@playwright/test";
import { WORKER, stubSignedIn } from "./support";

const GROUP = {
  pid: "aaaaaaaa-aaaa-4aaa-8aaa-aaaaaaaaaaaa",
  organization_ref: WORKER.organization_ref,
  scope: "organization",
  name: "Rust guild",
  kind: "practice",
  description: "Share what we learn",
  members: 2,
};

test.describe("Groups", () => {
  test.beforeEach(async ({ page }) => {
    await stubSignedIn(page);
  });

  test("lists the groups, opens one to show members and what it knows, and starts a new one", async ({
    page,
  }) => {
    const groups = [GROUP];
    const created: unknown[] = [];
    await page.route("**/api/proxy/groups?organization_ref=*", (route) =>
      route.fulfill({ json: groups }),
    );
    await page.route("**/api/proxy/groups", (route) => {
      created.push(route.request().postDataJSON());
      groups.push({ ...GROUP, pid: "bbbbbbbb-bbbb-4bbb-8bbb-bbbbbbbbbbbb", name: "Data circle", members: 0 });
      return route.fulfill({ json: { pid: "bbbbbbbb-bbbb-4bbb-8bbb-bbbbbbbbbbbb" } });
    });
    await page.route(`**/api/proxy/groups/${GROUP.pid}/members`, (route) =>
      route.fulfill({
        json: {
          group: GROUP,
          members: [
            {
              worker_pid: WORKER.pid,
              display_name: "Test Worker 001",
              job_title: "Engineer",
              role: "lead",
              joined_at: "2026-03-01T09:00:00Z",
              on_behalf: false,
            },
          ],
        },
      }),
    );
    await page.route(`**/api/proxy/groups/${GROUP.pid}/skills`, (route) =>
      route.fulfill({
        json: {
          group: GROUP,
          floor: 3,
          skills: [
            {
              skill_pid: "cccccccc-cccc-4ccc-8ccc-cccccccccccc",
              skill: "Rust",
              declared: 4,
              coverage: 0.5,
              levels: { "1": 0, "2": 1, "3": 2, "4": 1, "5": 0 },
            },
          ],
          withheld_below_floor: 2,
        },
      }),
    );

    await page.goto("/groups");
    await expect(page.getByTestId("group-list")).toContainText("Rust guild");
    await page.getByRole("button", { name: "Rust guild" }).click();
    const detail = page.getByTestId("group-detail");
    await expect(detail).toContainText("Test Worker 001");
    await expect(detail.getByTestId("group-skills")).toContainText("Rust");
    await expect(detail.getByTestId("group-skills")).toContainText("50%");
    // Two skills had fewer people than the floor: counted, never named.
    await expect(detail).toContainText("2");

    await page.getByRole("textbox", { name: /Start a group/ }).fill("Data circle");
    await page.getByRole("button", { name: "Start", exact: true }).click();
    await expect(page.getByTestId("group-list")).toContainText("Data circle");
    expect(created).toEqual([
      {
        organization_ref: WORKER.organization_ref,
        scope: "organization",
        name: "Data circle",
        kind: "practice",
      },
    ]);
  });

  test("with no groups yet it says so", async ({ page }) => {
    await page.route("**/api/proxy/groups?organization_ref=*", (route) => route.fulfill({ json: [] }));
    await page.goto("/groups");
    await expect(page.getByTestId("group-list")).toContainText("No groups yet");
  });

  test("a failed list shows the error, in words", async ({ page }) => {
    await page.route("**/api/proxy/groups?organization_ref=*", (route) =>
      route.fulfill({ status: 500, json: { description: "relation does not exist" } }),
    );
    await page.goto("/groups");
    await expect(page.getByTestId("error")).toContainText("Something went wrong on our side");
    await expect(page.getByTestId("error")).not.toContainText("relation");
  });
});
