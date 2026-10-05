// Unit tests: the money formatter's masked/absent honesty, the API
// path map (contract mirror), and the 13-locale i18n parity pin.

import { describe, expect, it, vi } from "vitest";

import { money } from "../../src/lib/api/wpm";
import {
  DEFAULT_LOCALE,
  LOCALES,
  STRING_KEYS,
  STRINGS_BY_LOCALE,
  i18n,
  tp,
  isRtl,
  translate,
} from "../../src/lib/i18n.svelte";

describe("money", () => {
  it("formats minor units as locale currency", () => {
    expect(money(360000, "GBP", "en")).toBe("£3,600.00");
    expect(money(0, "GBP", "en")).toBe("£0.00");
  });

  it("renders the masked/absent state as an em dash, never zero", () => {
    expect(money(null, "GBP", "en")).toBe("—");
    expect(money(undefined, "GBP", "en")).toBe("—");
    expect(money(360000, null, "en")).toBe("—");
  });
});

describe("i18n", () => {
  it("covers every key in every locale (parity)", () => {
    for (const locale of LOCALES) {
      const table = STRINGS_BY_LOCALE[locale];
      if (locale.endsWith("-001")) {
        for (const key of STRING_KEYS) {
          expect(table[key], `${locale} missing ${key}`).toBeTruthy();
        }
        expect(Object.keys(table).sort()).toEqual([...STRING_KEYS].sort());
      } else {
        // A regional locale (en-gb, es-es, …) holds only its overrides of
        // the language's -001 base; it must not invent keys.
        for (const key of Object.keys(table)) {
          expect(STRING_KEYS, `${locale} has stray ${key}`).toContain(key);
        }
      }
    }
  });

  it("translates with en fallback and flags RTL locales", () => {
    expect(translate("nav.workers", "de-001")).toBe("Arbeitskräfte");
    expect(translate("nav.workers", DEFAULT_LOCALE)).toBe("Workers");
    expect(isRtl("ar")).toBe(true);
    expect(isRtl("ur")).toBe(true);
    expect(isRtl("en")).toBe(false);
    expect(isRtl("es-MX")).toBe(false);
  });

  it("regional locales override their -001 base and fall back to it", () => {
    expect(translate("org.myOrganizations", "en-001")).toBe("My organizations");
    expect(translate("org.myOrganizations", "en-gb")).toBe("My organisations");
    expect(translate("org.myOrganizations", "en-us")).toBe("My organizations");
    expect(translate("nav.workers", "es-es")).toBe(
      translate("nav.workers", "es-001"),
    );
  });

  it("fills server-derived finding text from a code and its figures", () => {
    expect(
      tp(
        "insights.headcount_shrinking.observation",
        {
          pct: 15,
          opening: 100,
          closing: 85,
        },
        "en-001",
      ),
    ).toBe("Headcount fell 15% (100 to 85).");
    expect(
      tp("insights.turnover_high.observation", { pct: 25 }, "de-001"),
    ).toBe("Die Fluktuation lag im Zeitraum bei 25 %.");
    expect(tp("insights.unknown_code.observation", {}, "en-001")).toBeNull();
  });

  it("normalises en_US and en-US to en-us rather than collapsing to en-001", () => {
    i18n.set("en_US");
    expect(i18n.locale).toBe("en-us");
    i18n.set("en-US");
    expect(i18n.locale).toBe("en-us");
    i18n.set("en");
    expect(i18n.locale).toBe("en-001");
    i18n.set("es-MX");
    expect(i18n.locale).toBe("es-001");
  });
});

describe("api path map", () => {
  it("calls the exact proxy paths the service mounts", async () => {
    const calls: string[] = [];
    vi.stubGlobal(
      "fetch",
      vi.fn(async (url: string | URL) => {
        calls.push(String(url));
        return new Response("[]", {
          status: 200,
          headers: { "content-type": "application/json" },
        });
      }),
    );
    const wpm = await import("../../src/lib/api/wpm");
    await wpm.listWorkers({ department: "engineering" });
    await wpm.getWorker("p1");
    await wpm.orgChart("organization:abc");
    await wpm.listRequisitions("open");
    await wpm.listRuns();
    await wpm.runPayslips("r1");
    await wpm.benchmarkComparison("organization:abc");
    await wpm.listSkills();
    await wpm.categorySuggestions();
    await wpm.skillUsage("s1");
    await wpm.skillsMatrix();
    await wpm.trainingAnalytics();
    await wpm.capabilityAnalysis();
    await wpm.capabilityAnalysis({ minProficiency: 4, minDepth: 1 });
    await wpm.listCpdRequirements();
    await wpm.listCpdEntries("w1");
    await wpm.cpdProgress("w1");
    await wpm.cpdOverview();
    await wpm.roleMatches("w1");
    await wpm.opportunities("w1");
    await wpm.listMobilityInterests("w1");
    await wpm.mobilityInterestSummary();
    await wpm.listChangeInitiatives();
    await wpm.getChangeInitiative("c1");
    await wpm.changeReadiness("c1");
    await wpm.listWorkforcePlans();
    await wpm.getWorkforcePlan("p1");
    await wpm.planForecast("p1");
    await wpm.planAlignment("p1");
    await wpm.planCost("p1");
    await wpm.planCost("p1", "EUR");
    await wpm.listRoleProfiles();
    await wpm.listRoleProfiles("uk-gdad-pcf");
    await wpm.listFrameworks();
    await wpm.listSelectableFrameworks();
    await wpm.listFrameworkRoles("w1");
    await wpm.roleHistory("w1", "esco");
    await wpm.skillHistory("w1");
    await wpm.skillsAsOf("w1", "2026-01-01");
    await wpm.listAspirations("w1");
    await wpm.upline("w1");
    await wpm.downline("w1");
    await wpm.downlineAspirations("w1");
    await wpm.workerGroups("w1");
    await wpm.listGroups();
    await wpm.listGroups("organization:abc");
    await wpm.groupSkills("g1");
    await wpm.groupMembers("g1");
    await wpm.dottedLine("w1");
    await wpm.transferWorker("w1", "organization:b");
    await wpm.frameworkRoleSkills("w1", "esco");
    await wpm.searchEscoOccupations("dev");
    await wpm.getEscoOccupation("http://x/1");
    await wpm.searchEscoSkills("tea");
    await wpm.roleProgression("rp1");
    await wpm.getRoleProfile("rp1");
    await wpm.roleGap("rp1");
    await wpm.workerRoleGap("w1", "rp1");
    await wpm.workforceMetrics();
    await wpm.workforceMetrics({ from: "2026-01-01", to: "2026-06-30" });
    await wpm.listEmergencyContacts("w1");
    await wpm.addEmergencyContact("w1", { name: "Sam", relationship: "Partner", phone: "12345" });
    await wpm.updateEmergencyContact("c1", { priority: 2 });
    await wpm.removeEmergencyContact("c1");
    await wpm.listBackups("w1");
    await wpm.addBackup("w1", { backup_pid: "w2" });
    await wpm.removeBackup("b1");
    await wpm.workerCover("w1");
    await wpm.workerCover("w1", "2026-10-06");
    await wpm.listRotas();
    await wpm.getRota("r1");
    await wpm.getRota("r1", { from: "2026-10-05", to: "2026-11-01" });
    await wpm.createRota({ organization_ref: "organization:x", name: "N", period_days: 7, starts_on: "2026-10-05", members: ["w1"] });
    await wpm.updateRota("r1", { members: ["w2", "w1"] });
    await wpm.retireRota("r1");
    await wpm.addRotaSwap("r1", { worker_pid: "w2", starts_on: "2026-10-05", ends_on: "2026-10-06" });
    await wpm.removeRotaSwap("s1");
    await wpm.workerOnCall("w1");
    await wpm.listSwapRequests("r1");
    await wpm.requestSwap("r1", { requester_pid: "w1", taker_pid: "w2", starts_on: "2026-10-05", ends_on: "2026-10-11" });
    await wpm.decideSwap("s1", "accept");
    await wpm.workerSwapRequests("w1");
    await wpm.employeeDirectory();
    await wpm.employeeDirectory({ q: "ann lee", department: "Finance", limit: 100 });
    await wpm.workforceInsights();
    await wpm.workforceInsights({ from: "2026-01-01", to: "2026-06-30" });
    await wpm.listPaths();
    await wpm.pathProgress("path1");
    await wpm.mentorshipOverview(30);
    await wpm.listWellbeingEntitlements();
    await wpm.workerWellbeingPrompts("p1");
    await wpm.wellbeingUptake();
    await wpm.workingTime("engineering");
    await wpm.listPulseSurveys();
    await wpm.pulseResults("s1");
    await wpm.listAppraisals("p1");
    await wpm.appraisalRequests("p1");
    await wpm.getAppraisal("a1");
    await wpm.appraisalReport("a1");
    await wpm.retentionReport();
    await wpm.listNotifications("p1");
    await wpm.listErgonomicAssessments("p1");
    await wpm.listAdjustmentRequests("p1");
    await wpm.ergonomicIssues();
    expect(calls).toEqual([
      "/api/proxy/workers?department=engineering",
      "/api/proxy/workers/p1",
      "/api/proxy/org-chart?organization=organization%3Aabc",
      "/api/proxy/requisitions?status=open",
      "/api/proxy/payroll-runs",
      "/api/proxy/payroll-runs/r1/payslips",
      "/api/proxy/benchmarks/comparison?organization=organization%3Aabc",
      "/api/proxy/skills",
      "/api/proxy/skills/category-suggestions",
      "/api/proxy/skills/s1/usage",
      "/api/proxy/learning/skills-matrix",
      "/api/proxy/learning/training-analytics",
      "/api/proxy/workforce-intelligence/capability-analysis",
      "/api/proxy/workforce-intelligence/capability-analysis?min_proficiency=4&min_depth=1",
      "/api/proxy/cpd-requirements",
      "/api/proxy/workers/w1/cpd-entries",
      "/api/proxy/workers/w1/cpd-progress",
      "/api/proxy/cpd/overview",
      "/api/proxy/workers/w1/role-matches",
      "/api/proxy/workers/w1/opportunities",
      "/api/proxy/workers/w1/mobility-interests",
      "/api/proxy/mobility/interest-summary",
      "/api/proxy/change-initiatives",
      "/api/proxy/change-initiatives/c1",
      "/api/proxy/change-initiatives/c1/readiness",
      "/api/proxy/workforce-plans",
      "/api/proxy/workforce-plans/p1",
      "/api/proxy/workforce-plans/p1/forecast",
      "/api/proxy/workforce-plans/p1/alignment",
      "/api/proxy/workforce-plans/p1/cost",
      "/api/proxy/workforce-plans/p1/cost?currency=EUR",
      "/api/proxy/role-profiles",
      "/api/proxy/role-profiles?framework=uk-gdad-pcf",
      "/api/proxy/capability-frameworks",
      "/api/proxy/frameworks/selectable",
      "/api/proxy/workers/w1/framework-roles",
      "/api/proxy/workers/w1/role-history?framework=esco",
      "/api/proxy/workers/w1/skill-history",
      "/api/proxy/workers/w1/skills-as-of?at=2026-01-01",
      "/api/proxy/workers/w1/aspirations",
      "/api/proxy/workers/w1/upline",
      "/api/proxy/workers/w1/downline",
      "/api/proxy/workers/w1/downline-aspirations",
      "/api/proxy/workers/w1/groups",
      "/api/proxy/groups",
      "/api/proxy/groups?organization_ref=organization%3Aabc",
      "/api/proxy/groups/g1/skills",
      "/api/proxy/groups/g1/members",
      "/api/proxy/workers/w1/dotted-line",
      "/api/proxy/workers/w1/transfer",
      "/api/proxy/workers/w1/framework-roles/esco/skills",
      "/api/proxy/esco/occupations?q=dev&limit=25",
      "/api/proxy/esco/occupation?uri=http%3A%2F%2Fx%2F1",
      "/api/proxy/esco/skills?q=tea&limit=8",
      "/api/proxy/role-profiles/rp1/progression",
      "/api/proxy/role-profiles/rp1",
      "/api/proxy/role-profiles/rp1/gap",
      "/api/proxy/workers/w1/role-gap?role_profile_pid=rp1",
      "/api/proxy/workforce-intelligence/metrics",
      "/api/proxy/workforce-intelligence/metrics?from=2026-01-01&to=2026-06-30",
      "/api/proxy/workers/w1/emergency-contacts",
      "/api/proxy/workers/w1/emergency-contacts",
      "/api/proxy/emergency-contacts/c1",
      "/api/proxy/emergency-contacts/c1",
      "/api/proxy/workers/w1/backups",
      "/api/proxy/workers/w1/backups",
      "/api/proxy/backups/b1",
      "/api/proxy/workers/w1/cover",
      "/api/proxy/workers/w1/cover?on=2026-10-06",
      "/api/proxy/rotas",
      "/api/proxy/rotas/r1",
      "/api/proxy/rotas/r1?from=2026-10-05&to=2026-11-01",
      "/api/proxy/rotas",
      "/api/proxy/rotas/r1",
      "/api/proxy/rotas/r1",
      "/api/proxy/rotas/r1/overrides",
      "/api/proxy/rota-overrides/s1",
      "/api/proxy/workers/w1/on-call",
      "/api/proxy/rotas/r1/swap-requests",
      "/api/proxy/rotas/r1/swap-requests",
      "/api/proxy/rota-swap-requests/s1/accept",
      "/api/proxy/workers/w1/swap-requests",
      "/api/proxy/directory",
      "/api/proxy/directory?q=ann+lee&department=Finance&limit=100",
      "/api/proxy/workforce-intelligence/insights",
      "/api/proxy/workforce-intelligence/insights?from=2026-01-01&to=2026-06-30",
      "/api/proxy/learning-paths",
      "/api/proxy/learning-paths/path1/progress",
      "/api/proxy/learning/mentorship-overview?days=30",
      "/api/proxy/wellbeing-entitlements",
      "/api/proxy/workers/p1/wellbeing-prompts",
      "/api/proxy/wellbeing/uptake",
      "/api/proxy/workforce/working-time?department=engineering",
      "/api/proxy/pulse-surveys",
      "/api/proxy/pulse-surveys/s1/results",
      "/api/proxy/workers/p1/appraisals",
      "/api/proxy/workers/p1/appraisal-requests",
      "/api/proxy/appraisals/a1",
      "/api/proxy/appraisals/a1/report",
      "/api/proxy/retention",
      "/api/proxy/workers/p1/notifications",
      "/api/proxy/workers/p1/ergonomic-assessments",
      "/api/proxy/workers/p1/adjustment-requests",
      "/api/proxy/ergonomics/issues",
    ]);
    vi.unstubAllGlobals();
  });
});
