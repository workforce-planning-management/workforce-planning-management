// Expense claims. The API is stubbed per test; an unstubbed /api/proxy call 404s loudly.

import AxeBuilder from "@axe-core/playwright";
import { expect, test, type Page } from "@playwright/test";
import { WORKER, stubSignedIn } from "./support";

test.describe("Expense claims", () => {
  test.beforeEach(async ({ page }) => {
    await stubSignedIn(page);
  });

  /** A tiny in-memory expense service: one claim, with the `can` flags the real one sends. */
  async function stubExpenses(
    page: Page,
    role: "claimant" | "decider",
    seed?: { status: string },
  ) {
    const cid = "cccccccc-cccc-4ccc-8ccc-cccccccccccc";
    type Item = {
      pid: string;
      incurred_on: string;
      category: string;
      amount_minor: number;
      description: string | null;
      receipt_ref: string | null;
    };
    const state = {
      claim: seed
        ? {
            pid: cid,
            status: seed.status,
            title: "Leeds visit",
            currency: "GBP",
            note: null as string | null,
            paid: null as string | null,
          }
        : (null as null | {
            pid: string;
            status: string;
            title: string;
            currency: string;
            note: string | null;
            paid: string | null;
          }),
      items: (seed
        ? [
            {
              pid: "i1",
              incurred_on: "2026-10-01",
              category: "travel",
              amount_minor: 4250,
              description: "train",
              receipt_ref: null,
            },
          ]
        : []) as Item[],
    };
    const sent: Array<{ verb: string; body: unknown }> = [];
    const summary = () => ({
      pid: cid,
      worker_pid: WORKER.pid,
      title: state.claim!.title,
      description: null,
      currency: state.claim!.currency,
      status: state.claim!.status,
      total_minor: state.items.reduce((n, i) => n + i.amount_minor, 0),
      item_count: state.items.length,
      submitted_at: null,
      decided_at: null,
      decision_note: state.claim!.note,
      reimbursed_on: state.claim!.paid,
      on_behalf: false,
      worker_name: WORKER.display_name,
    });
    const detail = () => {
      const st = state.claim!.status;
      const seen = new Map<string, number>();
      for (const i of state.items)
        seen.set(
          `${i.incurred_on}|${i.category}|${i.amount_minor}`,
          (seen.get(`${i.incurred_on}|${i.category}|${i.amount_minor}`) ?? 0) +
            1,
        );
      return {
        ...summary(),
        items: state.items.map((i) => ({
          ...i,
          possible_duplicate:
            (seen.get(`${i.incurred_on}|${i.category}|${i.amount_minor}`) ??
              0) > 1,
        })),
        can: {
          edit: role === "claimant" && st === "draft",
          submit:
            role === "claimant" && st === "draft" && state.items.length > 0,
          withdraw: role === "claimant" && st === "submitted",
          cancel: role === "claimant" && (st === "draft" || st === "submitted"),
          decide: role === "decider" && st === "submitted",
          reimburse: role === "decider" && st === "approved",
        },
      };
    };
    await page.route(
      `**/api/proxy/workers/${WORKER.pid}/expense-claims`,
      async (route) => {
        if (route.request().method() === "POST") {
          const body = route.request().postDataJSON();
          state.claim = {
            pid: cid,
            status: "draft",
            title: body.title,
            currency: body.currency,
            note: null,
            paid: null,
          };
          return route.fulfill({ json: summary() });
        }
        return route.fulfill({ json: state.claim ? [summary()] : [] });
      },
    );
    await page.route(
      (url) => url.pathname === "/api/proxy/expense-claims",
      (route) => {
        const wanted = new URL(route.request().url()).searchParams.get(
          "status",
        );
        return route.fulfill({
          json: state.claim && state.claim.status === wanted ? [summary()] : [],
        });
      },
    );
    await page.route(`**/api/proxy/expense-claims/${cid}`, (route) =>
      route.fulfill({ json: detail() }),
    );
    // Generic verb handler first: Playwright prefers the LAST registered match, so the
    // more specific `/items` route below must come after it.
    await page.route(`**/api/proxy/expense-claims/${cid}/*`, (route) => {
      const verb = route.request().url().split("/").pop()!;
      const body = route.request().postDataJSON();
      sent.push({ verb, body });
      const to: Record<string, string> = {
        submit: "submitted",
        withdraw: "draft",
        cancel: "cancelled",
        approve: "approved",
        reject: "rejected",
        reimburse: "reimbursed",
      };
      state.claim!.status = to[verb]!;
      if (verb === "reject") state.claim!.note = body?.note ?? null;
      if (verb === "reimburse")
        state.claim!.paid = body?.reimbursed_on ?? "2026-10-07";
      return route.fulfill({ json: summary() });
    });
    await page.route(`**/api/proxy/expense-claims/${cid}/items`, (route) => {
      const b = route.request().postDataJSON();
      state.items.push({
        pid: `i${state.items.length + 1}`,
        description: null,
        receipt_ref: null,
        ...b,
      });
      return route.fulfill({ json: { pid: `i${state.items.length}` } });
    });
    await page.route("**/api/proxy/expense-items/*", (route) => {
      const pid = route.request().url().split("/").pop();
      state.items = state.items.filter((i) => i.pid !== pid);
      return route.fulfill({ status: 204, body: "" });
    });
    return sent;
  }

  test("a person builds an expense claim, sees a duplicate flagged, submits it, and cannot approve it", async ({
    page,
  }) => {
    await stubExpenses(page, "claimant");
    await page.goto("/me");
    const panel = page.getByTestId("expense-claims");
    await expect(panel.getByTestId("expense-none")).toBeVisible();
    await panel.getByTestId("expense-title").fill("Leeds visit");
    await panel.getByTestId("expense-currency").fill("GBP");
    await panel.getByTestId("expense-create").click();
    // The new draft opens: add two identical lunches and a fare.
    const add = async (category: string, amount: string) => {
      await panel.getByTestId("expense-date").fill("2026-10-01");
      await panel.getByTestId("expense-category").selectOption(category);
      await panel.getByTestId("expense-amount").fill(amount);
      await panel.getByTestId("expense-add").click();
    };
    await add("travel", "42.50");
    await add("meals", "12.50");
    await expect(panel.getByTestId("expense-total")).toHaveText("£55.00");
    await expect(panel.getByTestId("expense-duplicate")).toHaveCount(0);
    await add("meals", "12.50");
    await expect(panel.getByTestId("expense-total")).toHaveText("£67.50");
    await expect(panel.getByTestId("expense-duplicate")).toHaveCount(2);
    // Flagged, not refused: remove one, and the flag goes.
    await panel.getByTestId("expense-remove").last().click();
    await expect(panel.getByTestId("expense-duplicate")).toHaveCount(0);
    await panel.getByTestId("expense-submit").click();
    await expect(panel.getByTestId("expense-status")).toContainText(
      "Submitted",
    );
    // Nothing editable, and — the point — no way for the claimant to decide.
    await expect(panel.getByTestId("expense-item-form")).toHaveCount(0);
    await expect(panel.getByTestId("expense-approve")).toHaveCount(0);
    await expect(panel.getByTestId("expense-reject")).toHaveCount(0);
    await panel.getByTestId("expense-withdraw").click();
    await expect(panel.getByTestId("expense-status")).toContainText("Draft");
  });

  test("no currency is assumed: a claim cannot start until the person names one (WPM-T376)", async ({
    page,
  }) => {
    await stubExpenses(page, "claimant");
    const posted: unknown[] = [];
    page.on("request", (request) => {
      if (
        request.method() === "POST" &&
        request.url().endsWith(`/workers/${WORKER.pid}/expense-claims`)
      )
        posted.push(request.postDataJSON());
    });
    await page.goto("/en-001/me");
    const panel = page.getByTestId("expense-claims");
    await expect(panel.getByTestId("expense-currency")).toHaveValue("");
    await panel.getByTestId("expense-title").fill("Osaka visit");
    await panel.getByTestId("expense-create").click();
    await expect(panel.getByTestId("error")).toContainText("three-letter code");
    expect(posted).toHaveLength(0);
    // A yen claim: the amount is whole yen, not hundredths (WPM-T373).
    await panel.getByTestId("expense-currency").fill("jpy");
    await panel.getByTestId("expense-create").click();
    await expect
      .poll(() => posted.at(-1))
      .toEqual({
        title: "Osaka visit",
        currency: "JPY",
      });
    await panel.getByTestId("expense-date").fill("2026-10-01");
    await panel.getByTestId("expense-amount").fill("1500");
    await panel.getByTestId("expense-add").click();
    await expect(panel.getByTestId("expense-total")).toHaveText("JP¥1,500");
    await expect(panel.getByTestId("expense-items")).toContainText("JP¥1,500");
  });

  test("an amount is read in the page's own number format: a decimal comma in German (WPM-T376)", async ({
    page,
  }) => {
    await stubExpenses(page, "claimant");
    const items: Array<{ amount_minor: number }> = [];
    page.on("request", (request) => {
      if (request.method() === "POST" && request.url().endsWith("/items"))
        items.push(request.postDataJSON());
    });
    await page.goto("/de-001/me");
    const panel = page.getByTestId("expense-claims");
    await panel.getByTestId("expense-title").fill("Reise nach Köln");
    await panel.getByTestId("expense-currency").fill("EUR");
    await panel.getByTestId("expense-create").click();
    await panel.getByTestId("expense-date").fill("2026-10-01");
    await panel.getByTestId("expense-amount").fill("1.234,50");
    await panel.getByTestId("expense-add").click();
    await expect.poll(() => items.at(-1)?.amount_minor).toBe(123_450);
    await expect(panel.getByTestId("expense-total")).toHaveText(
      /^1\.234,50\s€$/u,
    );
    await panel.getByTestId("expense-date").fill("2026-10-02");
    await panel.getByTestId("expense-amount").fill("12,5");
    await panel.getByTestId("expense-add").click();
    await expect.poll(() => items.at(-1)?.amount_minor).toBe(1250);
    await expect(panel.getByTestId("expense-total")).toHaveText(
      /^1\.247,00\s€$/u,
    );
    // Text that is not a number is refused before anything is sent.
    await panel.getByTestId("expense-amount").fill("zwölf");
    await panel.getByTestId("expense-add").click();
    await expect(panel.getByTestId("expense-error")).toBeVisible();
    expect(items).toHaveLength(2);
    const result = await new AxeBuilder({ page })
      .include('[data-testid="expense-claims"]')
      .withTags(["wcag2a", "wcag2aa"])
      .analyze();
    expect(
      result.violations.map(
        (v) => `${v.id}: ${v.nodes.map((n) => n.html).join(" | ")}`,
      ),
    ).toEqual([]);
  });

  test("a manager decides claims from the queue: a rejection needs a note, then approve and reimburse", async ({
    page,
  }) => {
    const sent = await stubExpenses(page, "decider", { status: "submitted" });
    await page.goto("/expenses");
    const queue = page.getByTestId("expense-queue");
    await expect(queue).toContainText(WORKER.display_name);
    await expect(queue).toContainText("£42.50");
    await page.getByTestId(/expense-open-/).click();
    // Rejecting without a reason is refused before anything is sent.
    await page.getByTestId("expense-reject").click();
    await expect(page.getByTestId("expense-error")).toBeVisible();
    expect(sent).toHaveLength(0);
    // A decider cannot edit the claim.
    await expect(page.getByTestId("expense-item-form")).toHaveCount(0);
    await page.getByTestId("expense-note").fill("fine");
    await page.getByTestId("expense-approve").click();
    await expect
      .poll(() => sent.at(-1))
      .toEqual({ verb: "approve", body: { note: "fine" } });
    // It leaves the submitted queue; reimburse it from the approved one.
    await expect(page.getByTestId("expense-queue-empty")).toBeVisible();
    await page.getByTestId("expense-queue-status").selectOption("approved");
    await page.getByTestId(/expense-open-/).click();
    await page.getByTestId("expense-paid-on").fill("2026-10-07");
    await page.getByTestId("expense-reimburse").click();
    await expect
      .poll(() => sent.at(-1))
      .toEqual({
        verb: "reimburse",
        body: { reimbursed_on: "2026-10-07" },
      });
    await expect(page.getByTestId("expense-queue-empty")).toBeVisible();
  });
});
