// Unit tests: the money formatter's masked/absent honesty, the API
// path map (contract mirror), and the 13-locale i18n parity pin.

import { describe, expect, it, vi } from "vitest";

import { money } from "../../src/lib/api/wpm";
import {
  DEFAULT_LOCALE,
  LOCALES,
  STRING_KEYS,
  STRINGS_BY_LOCALE,
  PLURAL_KEYS,
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
      // A plural group's forms differ by language (WPM-T374): they are checked below, not here.
      const plain = (keys: readonly string[]) =>
        keys.filter((k) => !PLURAL_KEYS.some((b) => k.startsWith(`${b}.`)));
      if (locale.endsWith("-001")) {
        for (const key of plain(STRING_KEYS)) {
          expect(table[key], `${locale} missing ${key}`).toBeTruthy();
        }
        expect(plain(Object.keys(table)).sort()).toEqual(
          plain(STRING_KEYS).sort(),
        );
      } else {
        // A regional locale (en-gb, es-es, …) holds only its overrides of
        // the language's -001 base; it must not invent keys.
        for (const key of plain(Object.keys(table))) {
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
    await wpm.listRequisitionsPage({ status: "open", limit: 500, offset: 500 });
    await wpm.listRuns();
    await wpm.listRunsPage(500, 500);
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
    await wpm.planTolerances("p1");
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
    await wpm.addEmergencyContact("w1", {
      name: "Sam",
      relationship: "Partner",
      phone: "12345",
    });
    await wpm.updateEmergencyContact("c1", { priority: 2 });
    await wpm.removeEmergencyContact("c1");
    await wpm.listBackups("w1");
    await wpm.addBackup("w1", { backup_pid: "w2" });
    await wpm.removeBackup("b1");
    await wpm.workerCover("w1");
    await wpm.workerCover("w1", "2026-10-06");
    await wpm.listAnnouncements();
    await wpm.listAnnouncements({
      limit: 3,
      includeAll: true,
      organization: "organization:x",
    });
    await wpm.postAnnouncement({
      organization_ref: "organization:x",
      title: "T",
      body: "B",
    });
    await wpm.retireAnnouncement("a1");
    await wpm.markAnnouncementRead("a1", "w1");
    await wpm.myAnnouncementReads("w1");
    await wpm.listAnnouncements({ department: "Finance" });
    await wpm.listRotas();
    await wpm.getRota("r1");
    await wpm.getRota("r1", { from: "2026-10-05", to: "2026-11-01" });
    await wpm.createRota({
      organization_ref: "organization:x",
      name: "N",
      period_days: 7,
      starts_on: "2026-10-05",
      members: ["w1"],
    });
    await wpm.updateRota("r1", { members: ["w2", "w1"] });
    await wpm.retireRota("r1");
    await wpm.addRotaSwap("r1", {
      worker_pid: "w2",
      starts_on: "2026-10-05",
      ends_on: "2026-10-06",
    });
    await wpm.removeRotaSwap("s1");
    await wpm.workerOnCall("w1");
    await wpm.workerSkillGaps("w1");
    await wpm.workforceSkillGaps();
    await wpm.workforceSkillGaps({ department: "Finance", limit: 10 });
    await wpm.workerTrainingPlan("w1");
    await wpm.workerTrainingPlan("w1", { weeklyHours: 6, start: "2026-10-05" });
    await wpm.trainingDemand();
    await wpm.trainingDemand({ department: "Finance" });
    await wpm.skillCourses("k1");
    await wpm.addSkillCourse("k1", {
      course_ref: "course:x",
      title: "T",
      hours: 10,
    });
    await wpm.removeSkillCourse("c1");
    await wpm.setSkillTrainingHours("k1", 20);
    await wpm.listMovements();
    await wpm.listMovements({ kind: "leaver", status: "completed" });
    await wpm.listMovementsPage({ kind: "leaver" }, 500, 500);
    await wpm.getMovement("m1");
    await wpm.openMovement("w1", {
      kind: "leaver",
      effective_on: "2026-11-30",
      reason: "resignation",
    });
    await wpm.workerMovements("w1");
    await wpm.movementItemAction("i1", "done");
    await wpm.skipMovementItem("i1", "declined");
    await wpm.addMovementItem("m1", { title: "T", due_on: "2026-11-01" });
    await wpm.closeMovement("m1", "complete");
    await wpm.leaverHandover("m1");
    await wpm.reassignHeld("m1", {
      kind: "backup",
      subject_pid: "b1",
      to_worker_pid: "w2",
    });
    await wpm.handOverAll("m1", "w2");
    await wpm.handoverTrail("m1");
    await wpm.listSwapRequests("r1");
    await wpm.requestSwap("r1", {
      requester_pid: "w1",
      taker_pid: "w2",
      starts_on: "2026-10-05",
      ends_on: "2026-10-11",
    });
    await wpm.decideSwap("s1", "accept");
    await wpm.workerSwapRequests("w1");
    await wpm.listJobLevelFrameworks();
    await wpm.getJobLevelFramework("google-levels");
    await wpm.getWorkerJobLevel("w1");
    await wpm.setWorkerJobLevel("w1", {
      framework: "google-levels",
      level: "L5",
    });
    await wpm.clearWorkerJobLevel("w1");
    await wpm.listExpenseClaims("w1");
    await wpm.createExpenseClaim("w1", { title: "Trip", currency: "GBP" });
    await wpm.expenseQueue();
    await wpm.expenseQueue("approved");
    await wpm.expenseQueuePage("approved", 500, 500);
    await wpm.getExpenseClaim("c1");
    await wpm.addExpenseItem("c1", {
      incurred_on: "2026-10-01",
      category: "travel",
      amount_minor: 100,
    });
    await wpm.removeExpenseItem("i1");
    await wpm.moveExpenseClaim("c1", "submit");
    await wpm.moveExpenseClaim("c1", "reject", { note: "no" });
    await wpm.getWorkerPayPosition("w1");
    await wpm.setWorkerPayPosition("w1", {
      scale: "national-2026-27",
      band: "6",
      step: 1,
    });
    await wpm.clearWorkerPayPosition("w1");
    await wpm.getRoleGrade("p1");
    await wpm.setRoleGrade("p1", {
      job_level: { framework: "google-levels", level: "L5" },
    });
    await wpm.clearRoleGrade("p1");
    await wpm.listPayScales();
    await wpm.getPayScale("national-2026-27");
    await wpm.payPosition("national-2026-27", {
      band: "5",
      salary_minor: 3511400,
    });
    await wpm.payPosition("national-2026-27", {
      band: "6",
      step: 1,
      months_on_step: 24,
    });
    await wpm.employeeDirectory();
    await wpm.employeeDirectory({
      q: "ann lee",
      department: "Finance",
      limit: 100,
    });
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
    await wpm.workerEngagement("w1");
    await wpm.workerContractorDetails("w1");
    await wpm.workerFunding("w1");
    await wpm.workerConversionPlans("w1");
    await wpm.proposeConversionPlan("w1", {
      intent: "convert",
      target_on: "2027-01-01",
    });
    await wpm.moveConversionPlan("c1", "approve");
    await wpm.openConversionPlans();
    await wpm.openConversionPlans("organization:abc");
    await wpm.openConversionPlansPage("organization:abc", 500, 500);
    await wpm.contingentWorkforce();
    await wpm.contingentList("ending_31_to_60");
    await wpm.programmeRollup();
    await wpm.capacityView("organization:abc");
    await wpm.keyPersonRisk();
    await wpm.keyPersonRisk("organization:abc");
    await wpm.governanceLoad("organization:abc");
    await wpm.oversightStatus("organization:abc");
    await wpm.listOversightPacks("organization:abc");
    await wpm.getOversightPack("pack1");
    await wpm.freezeOversightPack("organization:abc");
    await wpm.withdrawOversightPack("pack1", "wrong month");
    await wpm.listCutovers("organization:abc");
    await wpm.cutoverReadiness("cut1");
    await wpm.createCutover({
      organization_ref: "organization:abc",
      programme_ref: "thing:p",
      name: "n",
      site: "s",
      window_start: "2026-11-02",
      window_end: "2026-11-15",
    });
    await wpm.addCutoverRole("cut1", {
      role_profile_pid: "r1",
      course_ref: "C",
      required_headcount: 2,
    });
    await wpm.addCutoverPool("cut1", "pool1");
    await wpm.removeCutover("cut1");
    await wpm.listSkillPools("organization:abc");
    await wpm.createSkillPool({
      organization_ref: "organization:abc",
      name: "p",
    });
    await wpm.addSkillPoolMember("pool1", { role_profile_pid: "r1" });
    await wpm.setProgrammeDemand({
      programme_ref: "thing:p",
      pool_pid: "pool1",
      month: "2026-11-01",
      fte_centi: 150,
    });
    await wpm.createOversightLevel({
      organization_ref: "organization:abc",
      ordinal: 1,
      name: "L",
    });
    await wpm.recordOversightPosition({
      organization_ref: "organization:abc",
      level_pid: "l1",
      effective_on: "2026-10-01",
      deciding_body: "b",
      reason: "r",
    });
    await wpm.createOversightCriterion({ level_pid: "l1", name: "c" });
    await wpm.createOversightMeasure({
      criterion_pid: "c1",
      name: "m",
      origin: "reported",
      unit: "u",
      threshold: 1,
      direction: "at_most",
      sustain: 3,
    });
    await wpm.putOversightReading("m1", {
      period_on: "2026-09-01",
      value: null,
    });
    await wpm.listOversightDerivations();
    await wpm.listEarlyWarnings("organization:abc");
    await wpm.sweepEarlyWarnings("organization:abc");
    await wpm.acknowledgeEarlyWarning("w1");
    await wpm.listDevelopmentPlans("w1");
    await wpm.setDevelopmentItemStatus("i1", "achieved");
    await wpm.listRecognisedBodies("organization:abc");
    await wpm.createRecognisedBody({
      organization_ref: "organization:abc",
      name: "n",
      kind: "union",
      basis: "other",
      recognised_on: "2026-01-01",
      contact_role: "r",
    });
    await wpm.listBargainingUnits("b1");
    await wpm.addBargainingUnit("b1", { name: "u", department: "Care" });
    await wpm.listConsultations("organization:abc");
    await wpm.openConsultation({
      body_pid: "b1",
      establishment: "e",
      profile: "great_britain",
      establishment_size: 100,
      dismissals: 20,
      earliest_termination_on: "2027-01-01",
      topics: ["pay"],
    });
    await wpm.beginConsultation("c1", { started_on: "2026-10-01" });
    await wpm.closeConsultation("c1", {
      outcome: "agreed",
      ended_on: "2026-10-20",
    });
    await wpm.consultationTimeline("c1");
    await wpm.readInformationPack("u1");
    await wpm.listFacilityTime("b1");
    await wpm.addFacilityTime("b1", {
      period_start: "2026-01-01",
      period_end: "2026-03-31",
      minutes: 600,
    });
    await wpm.listDecisionGates("organization:abc");
    await wpm.createDecisionGate({
      organization_ref: "organization:abc",
      change_kind: "other",
      body_pid: "b1",
    });
    await wpm.checkDecisionGate("organization:abc", "other");
    await wpm.getStatutoryDefaults();
    expect(calls).toEqual([
      "/api/proxy/workers?department=engineering",
      "/api/proxy/workers/p1",
      "/api/proxy/org-chart?organization=organization%3Aabc",
      "/api/proxy/requisitions?status=open",
      "/api/proxy/requisitions?status=open&limit=500&offset=500",
      "/api/proxy/payroll-runs",
      "/api/proxy/payroll-runs?limit=500&offset=500",
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
      "/api/proxy/workforce-plans/p1/tolerances",
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
      "/api/proxy/announcements",
      "/api/proxy/announcements?organization=organization%3Ax&include=all&limit=3",
      "/api/proxy/announcements",
      "/api/proxy/announcements/a1",
      "/api/proxy/announcements/a1/read",
      "/api/proxy/workers/w1/announcement-reads",
      "/api/proxy/announcements?department=Finance",
      "/api/proxy/rotas",
      "/api/proxy/rotas/r1",
      "/api/proxy/rotas/r1?from=2026-10-05&to=2026-11-01",
      "/api/proxy/rotas",
      "/api/proxy/rotas/r1",
      "/api/proxy/rotas/r1",
      "/api/proxy/rotas/r1/overrides",
      "/api/proxy/rota-overrides/s1",
      "/api/proxy/workers/w1/on-call",
      "/api/proxy/workers/w1/skill-gaps",
      "/api/proxy/workforce-intelligence/skill-gaps",
      "/api/proxy/workforce-intelligence/skill-gaps?department=Finance&limit=10",
      "/api/proxy/workers/w1/training-plan",
      "/api/proxy/workers/w1/training-plan?weekly_hours=6&start=2026-10-05",
      "/api/proxy/workforce-intelligence/training-demand",
      "/api/proxy/workforce-intelligence/training-demand?department=Finance",
      "/api/proxy/skills/k1/courses",
      "/api/proxy/skills/k1/courses",
      "/api/proxy/skill-courses/c1",
      "/api/proxy/skills/k1/training-hours",
      "/api/proxy/movements",
      "/api/proxy/movements?kind=leaver&status=completed",
      "/api/proxy/movements?kind=leaver&limit=500&offset=500",
      "/api/proxy/movements/m1",
      "/api/proxy/workers/w1/movements",
      "/api/proxy/workers/w1/movements",
      "/api/proxy/movement-items/i1/done",
      "/api/proxy/movement-items/i1/skip",
      "/api/proxy/movements/m1/items",
      "/api/proxy/movements/m1/complete",
      "/api/proxy/movements/m1/handover",
      "/api/proxy/movements/m1/handover",
      "/api/proxy/movements/m1/handover/all",
      "/api/proxy/movements/m1/handover/actions",
      "/api/proxy/rotas/r1/swap-requests",
      "/api/proxy/rotas/r1/swap-requests",
      "/api/proxy/rota-swap-requests/s1/accept",
      "/api/proxy/workers/w1/swap-requests",
      "/api/proxy/job-levels",
      "/api/proxy/job-levels/google-levels",
      "/api/proxy/workers/w1/job-level",
      "/api/proxy/workers/w1/job-level",
      "/api/proxy/workers/w1/job-level",
      "/api/proxy/workers/w1/expense-claims",
      "/api/proxy/workers/w1/expense-claims",
      "/api/proxy/expense-claims",
      "/api/proxy/expense-claims?status=approved",
      "/api/proxy/expense-claims?status=approved&limit=500&offset=500",
      "/api/proxy/expense-claims/c1",
      "/api/proxy/expense-claims/c1/items",
      "/api/proxy/expense-items/i1",
      "/api/proxy/expense-claims/c1/submit",
      "/api/proxy/expense-claims/c1/reject",
      "/api/proxy/workers/w1/pay-position",
      "/api/proxy/workers/w1/pay-position",
      "/api/proxy/workers/w1/pay-position",
      "/api/proxy/role-profiles/p1/grade",
      "/api/proxy/role-profiles/p1/grade",
      "/api/proxy/role-profiles/p1/grade",
      "/api/proxy/pay-scales",
      "/api/proxy/pay-scales/national-2026-27",
      "/api/proxy/pay-scales/national-2026-27/position?band=5&salary_minor=3511400",
      "/api/proxy/pay-scales/national-2026-27/position?band=6&step=1&months_on_step=24",
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
      "/api/proxy/workers/w1/engagement",
      "/api/proxy/workers/w1/contractor-details",
      "/api/proxy/workers/w1/funding",
      "/api/proxy/workers/w1/conversion-plans",
      "/api/proxy/workers/w1/conversion-plans",
      "/api/proxy/conversion-plans/c1/approve",
      "/api/proxy/conversion-plans",
      "/api/proxy/conversion-plans?organization=organization%3Aabc",
      "/api/proxy/conversion-plans?organization=organization%3Aabc&limit=500&offset=500",
      "/api/proxy/workforce-intelligence/contingent",
      "/api/proxy/engagements/contingent-list?list=ending_31_to_60",
      "/api/proxy/workforce-intelligence/programmes",
      "/api/proxy/capacity?organization=organization%3Aabc",
      "/api/proxy/workforce-intelligence/key-person",
      "/api/proxy/workforce-intelligence/key-person?organization=organization%3Aabc",
      "/api/proxy/governance-commitments/load?organization=organization%3Aabc",
      "/api/proxy/oversight/status?organization=organization%3Aabc",
      "/api/proxy/oversight/packs?organization=organization%3Aabc",
      "/api/proxy/oversight/packs/pack1",
      "/api/proxy/oversight/packs",
      "/api/proxy/oversight/packs/pack1/withdraw",
      "/api/proxy/cutovers?organization=organization%3Aabc",
      "/api/proxy/cutovers/cut1/readiness",
      "/api/proxy/cutovers",
      "/api/proxy/cutovers/cut1/roles",
      "/api/proxy/cutovers/cut1/pools",
      "/api/proxy/cutovers/cut1",
      "/api/proxy/skill-pools?organization=organization%3Aabc",
      "/api/proxy/skill-pools",
      "/api/proxy/skill-pools/pool1/members",
      "/api/proxy/programme-demands",
      "/api/proxy/oversight/levels",
      "/api/proxy/oversight/positions",
      "/api/proxy/oversight/criteria",
      "/api/proxy/oversight/measures",
      "/api/proxy/oversight/measures/m1/readings",
      "/api/proxy/oversight/derivations",
      "/api/proxy/early-warnings?organization=organization%3Aabc",
      "/api/proxy/early-warnings/sweep?organization=organization%3Aabc",
      "/api/proxy/early-warnings/w1/acknowledge",
      "/api/proxy/workers/w1/development-plans",
      "/api/proxy/development-plan-items/i1",
      "/api/proxy/recognised-bodies?organization=organization%3Aabc",
      "/api/proxy/recognised-bodies",
      "/api/proxy/recognised-bodies/b1/units",
      "/api/proxy/recognised-bodies/b1/units",
      "/api/proxy/consultations?organization=organization%3Aabc",
      "/api/proxy/consultations",
      "/api/proxy/consultations/c1/begin",
      "/api/proxy/consultations/c1/close",
      "/api/proxy/consultations/c1/timeline",
      "/api/proxy/bargaining-units/u1/information-pack",
      "/api/proxy/recognised-bodies/b1/facility-time",
      "/api/proxy/recognised-bodies/b1/facility-time",
      "/api/proxy/decision-gates?organization=organization%3Aabc",
      "/api/proxy/decision-gates",
      "/api/proxy/decision-gates/check?organization=organization%3Aabc&change=other",
      "/api/proxy/statutory-defaults",
    ]);
    vi.unstubAllGlobals();
  });
});
