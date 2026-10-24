// Job levels, pay bands and pay scales. The API is stubbed per test; an unstubbed /api/proxy call 404s loudly.

import { expect, test } from "@playwright/test";
import { WORKER, stubSignedIn } from "./support";

test.describe("Job levels, pay bands and pay scales", () => {
  test.beforeEach(async ({ page }) => {
    await stubSignedIn(page);
  });

  test("a person records their job level; it is cleared on request, and hidden when not theirs", async ({
    page,
  }) => {
    const ladder = {
      id: "google-levels",
      name: "Google technical levels (L3–L11)",
      organization: "Google",
      track: "technical",
      source: "A published summary. Not an official Google publication.",
      levels: [5, 6].map((n) => ({
        number: n,
        code: `L${n}`,
        title: n === 5 ? "Senior Software Engineer" : "Staff Software Engineer",
        summary: "",
        experience: null,
        management_equivalent: null,
      })),
    };
    let held: Record<string, unknown> | null = null;
    const sent: Array<Record<string, unknown>> = [];
    await page.route(
      (url) => url.pathname === "/api/proxy/job-levels",
      (route) => route.fulfill({ json: [{ ...ladder, levels: ["L5", "L6"] }] }),
    );
    await page.route(
      (url) => url.pathname === "/api/proxy/job-levels/google-levels",
      (route) => route.fulfill({ json: ladder }),
    );
    await page.route(
      `**/api/proxy/workers/${WORKER.pid}/job-level`,
      async (route) => {
        const method = route.request().method();
        if (method === "PUT") {
          const body = route.request().postDataJSON();
          sent.push(body);
          const level = ladder.levels.find((l) => l.code === body.level)!;
          held = {
            framework: ladder.id,
            framework_name: ladder.name,
            level,
            effective_on: body.effective_on ?? "2026-10-06",
            on_behalf: false,
          };
          return route.fulfill({ json: { job_level: held } });
        }
        if (method === "DELETE") {
          held = null;
          return route.fulfill({ status: 204, body: "" });
        }
        return route.fulfill({ json: { job_level: held } });
      },
    );
    await page.goto("/me");
    const panel = page.getByTestId("job-level");
    await expect(panel.getByTestId("job-level-none")).toBeVisible();
    await panel.getByTestId("job-level-choice").selectOption("L5");
    await panel.getByTestId("job-level-since").fill("2024-03-01");
    await panel.getByTestId("job-level-save").click();
    await expect(panel.getByTestId("job-level-held")).toContainText("L5");
    await expect(panel.getByTestId("job-level-held")).toContainText(
      "Senior Software Engineer",
    );
    await expect(panel.getByTestId("job-level-held")).toContainText(
      "2024-03-01",
    );
    expect(sent.at(-1)).toMatchObject({
      framework: "google-levels",
      level: "L5",
    });
    await panel.getByTestId("job-level-clear").click();
    await expect(panel.getByTestId("job-level-none")).toBeVisible();
  });

  test("a person records their pay band and step, sees when they are eligible to move up", async ({
    page,
  }) => {
    const step = (pounds: number, years: number | null) => ({
      annual_minor: pounds * 100,
      years_to_next: years,
    });
    const scale = {
      id: "national-2026-27",
      name: "Public-sector pay scale 2026/27",
      framework: "banded-pay",
      nation: "national",
      currency: "GBP",
      effective_from: "2026-04-01",
      uplift_tenths_percent: 33,
      source: "Government pay circular for 2026/27, Annex 1",
      minutes_per_week: 2250,
      bands: [
        {
          code: "6",
          closed: false,
          steps: [step(40559, 2), step(42805, 3), step(48841, null)],
        },
      ],
      allowances: [],
    };
    let held: Record<string, unknown> | null = null;
    const sent: Array<Record<string, unknown>> = [];
    await page.route(
      (url) => url.pathname === "/api/proxy/pay-scales",
      (route) => route.fulfill({ json: [{ ...scale, bands: ["6"] }] }),
    );
    await page.route(
      (url) => url.pathname === "/api/proxy/pay-scales/national-2026-27",
      (route) => route.fulfill({ json: scale }),
    );
    await page.route(
      `**/api/proxy/workers/${WORKER.pid}/pay-position`,
      async (route) => {
        const method = route.request().method();
        if (method === "PUT") {
          const body = route.request().postDataJSON();
          sent.push(body);
          held = {
            scale: scale.id,
            scale_name: scale.name,
            band: body.band,
            step: body.step,
            currency: "GBP",
            annual_minor: 4055900,
            step_since: body.step_since ?? "2026-10-06",
            on_behalf: false,
            progression: {
              kind: "not_yet",
              eligible_on: "2026-12-01",
              days_remaining: 56,
              next_annual_minor: 4280500,
            },
          };
          return route.fulfill({ json: { pay_position: held } });
        }
        if (method === "DELETE") {
          held = null;
          return route.fulfill({ status: 204, body: "" });
        }
        return route.fulfill({ json: { pay_position: held } });
      },
    );
    await page.goto("/me");
    const panel = page.getByTestId("pay-position");
    await expect(panel.getByTestId("pay-position-none")).toBeVisible();
    await panel.getByTestId("pay-position-band").selectOption("6");
    await panel.getByTestId("pay-position-step").selectOption("1");
    await panel.getByTestId("pay-position-since").fill("2024-12-01");
    await panel.getByTestId("pay-position-save").click();
    await expect(panel.getByTestId("pay-position-held")).toContainText(
      "Band 6, step 1",
    );
    await expect(panel.getByTestId("pay-position-held")).toContainText(
      "£40,559",
    );
    await expect(panel.getByTestId("pay-position-progression")).toContainText(
      "Eligible to move up on 2026-12-01 (in 56 calendar days), to £42,805",
    );
    expect(sent.at(-1)).toEqual({
      scale: "national-2026-27",
      band: "6",
      step: 1,
      step_since: "2024-12-01",
    });
    await panel.getByTestId("pay-position-clear").click();
    await expect(panel.getByTestId("pay-position-none")).toBeVisible();
  });

  test("the job-level and pay-position panels disappear for anyone who may not see them", async ({
    page,
  }) => {
    for (const path of ["job-level", "pay-position"]) {
      await page.route(`**/api/proxy/workers/${WORKER.pid}/${path}`, (route) =>
        route.fulfill({ status: 403, json: { error: "forbidden" } }),
      );
    }
    await page.goto("/me");
    // Wait until the page has settled on something that is not these panels.
    await expect(page.getByTestId("emergency-contacts")).toBeVisible();
    await expect(page.getByTestId("job-level")).toHaveCount(0);
    await expect(page.getByTestId("pay-position")).toHaveCount(0);
  });

  test("a role profile links a job level to a pay band, set by the editor", async ({
    page,
  }) => {
    const pid = "11111111-1111-4111-8111-111111111111";
    const ladder = {
      id: "google-levels",
      name: "Google technical levels",
      organization: "Google",
      track: "technical",
      source: "",
      levels: [
        {
          number: 5,
          code: "L5",
          title: "Senior Software Engineer",
          summary: "",
          experience: null,
          management_equivalent: null,
        },
      ],
    };
    const step = (pounds: number, years: number | null) => ({
      annual_minor: pounds * 100,
      years_to_next: years,
    });
    const scale = {
      id: "national-2026-27",
      name: "Public-sector pay scale 2026/27",
      framework: "banded-pay",
      nation: "national",
      currency: "GBP",
      effective_from: "2026-04-01",
      uplift_tenths_percent: 33,
      source: "Government pay circular for 2026/27, Annex 1",
      minutes_per_week: 2250,
      bands: [
        {
          code: "7",
          closed: false,
          steps: [step(50129, 2), step(52712, 3), step(57365, null)],
        },
      ],
      allowances: [],
    };
    let grade: Record<string, unknown> = {
      role_profile: pid,
      job_level: null,
      pay_band: null,
    };
    const sent: Array<Record<string, unknown>> = [];
    const stub = (path: string, json: unknown) =>
      page.route(
        (url) => url.pathname === `/api/proxy${path}`,
        (route) => route.fulfill({ json }),
      );
    await stub("/skills", []);
    await stub("/role-profiles", [
      {
        pid,
        job_title: "Platform Engineer",
        profession: null,
        role_name: null,
        level_order: null,
      },
    ]);
    await stub("/capability-frameworks", []);
    await stub("/mobility/interest-summary", { derivation: "", targets: [] });
    await stub(`/role-profiles/${pid}`, {
      pid,
      job_title: "Platform Engineer",
      description: null,
      source_ref: null,
      framework: null,
      requirements: [],
    });
    await stub(`/role-profiles/${pid}/gap`, {
      derivation: "",
      role: { pid, job_title: "Platform Engineer" },
      headcount: 0,
      requirements: [],
    });
    await stub(`/role-profiles/${pid}/progression`, {
      profile: "none",
      next: [],
    });
    await stub("/job-levels", [{ ...ladder, levels: ["L5"] }]);
    await stub("/job-levels/google-levels", ladder);
    await stub("/pay-scales", [{ ...scale, bands: ["7"] }]);
    await stub("/pay-scales/national-2026-27", scale);
    await page.route(
      `**/api/proxy/role-profiles/${pid}/grade`,
      async (route) => {
        const method = route.request().method();
        if (method === "PUT") {
          const body = route.request().postDataJSON();
          sent.push(body);
          grade = {
            role_profile: pid,
            job_level: body.job_level
              ? {
                  framework: ladder.id,
                  framework_name: ladder.name,
                  level: ladder.levels[0],
                }
              : null,
            pay_band: body.pay_band
              ? {
                  scale: scale.id,
                  scale_name: scale.name,
                  band: "7",
                  currency: "GBP",
                  entry_minor: 5012900,
                  top_minor: 5736500,
                }
              : null,
          };
          return route.fulfill({ json: grade });
        }
        if (method === "DELETE") {
          grade = { role_profile: pid, job_level: null, pay_band: null };
          return route.fulfill({ status: 204, body: "" });
        }
        return route.fulfill({ json: grade });
      },
    );
    await page.goto("/roles");
    const panel = page.getByTestId("role-grade");
    await expect(panel.getByTestId("role-grade-none")).toBeVisible();
    // Nothing chosen: refused client-side, nothing sent.
    await panel.getByTestId("role-grade-save").click();
    await expect(panel.getByTestId("error")).toBeVisible();
    expect(sent).toHaveLength(0);
    await panel.getByTestId("role-level-choice").selectOption("L5");
    await panel.getByTestId("role-band-choice").selectOption("7");
    await panel.getByTestId("role-grade-save").click();
    const held = panel.getByTestId("role-grade-held");
    await expect(held).toContainText("L5");
    await expect(held).toContainText("Band 7");
    await expect(held).toContainText("£50,129");
    await expect(held).toContainText("£57,365");
    expect(sent.at(-1)).toEqual({
      job_level: { framework: "google-levels", level: "L5" },
      pay_band: { scale: "national-2026-27", band: "7" },
    });
    await panel.getByTestId("role-grade-clear").click();
    await expect(panel.getByTestId("role-grade-none")).toBeVisible();
  });

  test("job levels show Google's ladder, with unstated fields as a dash and no pay", async ({
    page,
  }) => {
    const level = (
      number: number,
      title: string,
      experience: string | null,
      management_equivalent: string | null,
    ) => ({
      number,
      code: `L${number}`,
      title,
      summary: `${title} summary`,
      experience,
      management_equivalent,
    });
    const framework = {
      id: "google-levels",
      name: "Google technical levels (L3–L11)",
      organization: "Google",
      track: "technical",
      source: "A published summary. Not an official Google publication.",
      levels: [
        level(3, "Software Engineer I", "0–1 year", null),
        level(
          7,
          "Senior Staff / Principal Software Engineer",
          null,
          "Engineering Manager",
        ),
        level(8, "Distinguished / Principal Engineer", "15+ years", "Director"),
      ],
    };
    await page.route(
      (url) => url.pathname === "/api/proxy/job-levels",
      (route) =>
        route.fulfill({ json: [{ ...framework, levels: ["L3", "L7", "L8"] }] }),
    );
    await page.route(
      (url) => url.pathname === "/api/proxy/job-levels/google-levels",
      (route) => route.fulfill({ json: framework }),
    );
    await page.goto("/job-levels");
    await expect(page.getByTestId("levels-source")).toContainText(
      "Not an official Google publication",
    );
    await expect(page.getByTestId("level-L3")).toContainText("0–1 year");
    // L7 states no experience: a dash, not an invented figure.
    await expect(page.getByTestId("level-L7")).toContainText("—");
    await expect(page.getByTestId("level-L7")).toContainText(
      "Engineering Manager",
    );
    await expect(page.getByTestId("level-L8")).toContainText("Director");
    await expect(page.getByTestId("levels-table")).not.toContainText("£");
    await expect(page.getByTestId("levels-table")).not.toContainText("$");
  });

  test("pay scales show the circular and place a salary on a band", async ({
    page,
  }) => {
    const step = (pounds: number, years: number | null) => ({
      annual_minor: pounds * 100,
      years_to_next: years,
    });
    const scale = {
      id: "national-2026-27",
      name: "Public-sector pay scale 2026/27",
      framework: "banded-pay",
      nation: "national",
      currency: "GBP",
      effective_from: "2026-04-01",
      uplift_tenths_percent: 33,
      source: "Government pay circular for 2026/27, Annex 1",
      minutes_per_week: 2250,
      bands: [
        { code: "1", closed: true, steps: [step(26300, null)] },
        {
          code: "3",
          closed: false,
          steps: [step(26300, 2), step(27890, null)],
        },
        {
          code: "5",
          closed: false,
          steps: [step(32557, 2), step(35114, 2), step(39631, null)],
        },
      ],
      allowances: [
        {
          code: "on_call_weekday_weekend",
          name: "On-call",
          amount_minor: 2605,
        },
      ],
    };
    const asked: string[] = [];
    await page.route(
      (url) => url.pathname === "/api/proxy/pay-scales",
      (route) =>
        route.fulfill({ json: [{ ...scale, bands: ["1", "3", "5"] }] }),
    );
    await page.route(
      (url) => url.pathname === "/api/proxy/pay-scales/national-2026-27",
      (route) => route.fulfill({ json: scale }),
    );
    await page.route(
      (url) => url.pathname.endsWith("/position"),
      (route) => {
        const q = new URL(route.request().url()).searchParams;
        asked.push(q.toString());
        return route.fulfill({
          json: {
            scale: scale.id,
            band: q.get("band"),
            closed_to_new_entrants: false,
            currency: "GBP",
            basis: "annual, full-time",
            position: { kind: "on_step", step: 2 },
            progression: {
              kind: "not_yet",
              months_remaining: 6,
              next_annual_minor: 3963100,
            },
          },
        });
      },
    );
    await page.goto("/pay-scales");
    await expect(page.getByTestId("pay-scale-source")).toContainText(
      "pay circular for 2026/27",
    );
    await expect(page.getByTestId("pay-scale-source")).toContainText("+3.3%");
    // Band 5 has three steps with years; band 3 has two (no intermediate); band 1 is closed.
    const band5 = page.getByTestId("band-5");
    await expect(band5).toContainText("£32,557");
    await expect(band5).toContainText("£35,114");
    await expect(band5).toContainText("£39,631");
    await expect(page.getByTestId("band-3")).toContainText("—");
    await expect(page.getByTestId("band-1")).toContainText("Closed");
    await expect(page.getByTestId("pay-allowances")).toContainText("£26.05");

    await page.getByTestId("pay-band").selectOption("5");
    await page.getByTestId("pay-salary").fill("35114");
    await page.getByTestId("pay-step").fill("2");
    await page.getByTestId("pay-months").fill("18");
    await page.getByTestId("pay-ask").click();
    await expect(page.getByTestId("pay-position")).toContainText("On step 2");
    await expect(page.getByTestId("pay-progression")).toContainText(
      "6 months to go",
    );
    await expect(page.getByTestId("pay-progression")).toContainText("£39,631");
    // The salary is sent in pence; nothing is persisted client-side.
    expect(asked.at(-1)).toContain("salary_minor=3511400");
    expect(asked.at(-1)).toContain("months_on_step=18");
  });
});
