<!--
  Strategic workforce planning (`/planning`): a plan is a draft world of
  aggregate hypothetical headcount — it never touches live worker records.
  Per department and date: planned demand, projected supply (stated attrition,
  no hires assumed), the headcount gap, and the competency gap against a role
  profile; plus whether the plan serves its strategic objectives. Gaps are
  aggregate and levers are suggestions, never decisions.
-->
<script lang="ts">
  import { arrow, formatDate, formatDateRange, numberToInput, parseScaled } from "#lib/locale-format.js";
  import { guardedLoad } from "#lib/loader.svelte.js";
  import PlanForecast from "#lib/components/planning/PlanForecast.svelte";
  import PlanTolerances from "#lib/components/planning/PlanTolerances.svelte";
  import { errorText } from "#lib/write.js";
  import ErrorNote from "#lib/components/ErrorNote.svelte";
  import { page } from "$app/state";
  import {
    addPlanObjective,
    createWorkforcePlan,
    getWorkforcePlan,
    listRoleProfiles,
    listWorkforcePlans,
    planAlignment,
    planCost,
    planForecast,
    planEscalations,
    planTolerances,
    removeDemandLine,
    setDemandLine,
    setLineObjectives,
    setPlanStatus,
    PLAN_BASES,
  } from "#lib/api/wpm.js";
  import LilyGantt from "#lib/components/LilyGantt.svelte";
  import LilyKanban from "#lib/components/LilyKanban.svelte";
  import { i18n, t, tf, tv } from "#lib/i18n.svelte.js";
  import { currencyCode, toMinor } from "#lib/money.js";
  import DataTable from "#lib/components/DataTable.svelte";
  import type { DataGridColumn, DataGridRow } from "@lilydesignsystem/svelte-data-grid";

  type Plans = Awaited<ReturnType<typeof listWorkforcePlans>>;
  type Plan = Awaited<ReturnType<typeof getWorkforcePlan>>;
  type Forecast = Awaited<ReturnType<typeof planForecast>>;
  type Alignment = Awaited<ReturnType<typeof planAlignment>>;
  type Cost = Awaited<ReturnType<typeof planCost>>;
  type Tolerances = Awaited<ReturnType<typeof planTolerances>>;
  type Escalations = Awaited<ReturnType<typeof planEscalations>>;

  const NEXT: Record<string, string[]> = {
    draft: ["active", "archived"],
    active: ["archived"],
    archived: [],
  };

  const organizationRefs = $derived((page.data.scope ?? []) as string[]);

  let plans = $state<Plans>([]);
  let selected = $state("");
  let plan = $state<Plan | null>(null);
  let forecast = $state<Forecast | null>(null);
  let alignment = $state<Alignment | null>(null);
  let cost = $state<Cost | null>(null);
  let tolerances = $state<Tolerances | null>(null);
  let escalations = $state<Escalations>([]);
  let roles = $state<Awaited<ReturnType<typeof listRoleProfiles>>>([]);
  let totals = $state<Record<string, number>>({});
  let details = $state<Plan[]>([]);
  let error = $state<string | null>(null);

  let name = $state("");
  let org = $state("");
  let start = $state("");
  let end = $state("");
  /** Percentages and the FTE are typed in the page's own number format ("2,5" in German); empty is unknown (WPM-T375). */
  let attritionPct = $state("");
  /** The budget as typed, in the page's own number format; empty is no budget. */
  let budget = $state("");
  /** No currency is assumed: a budget names its own (WPM-T376). */
  let budgetCurrency = $state("");
  let onCostPct = $state("");
  let department = $state("");
  let lineRole = $state("");
  let targetOn = $state("");
  let headcount = $state(1);
  let lineBasis = $state("permanent");
  let lineEnds = $state("");
  let lineProgramme = $state("");
  /** FTE across the headcount, in people (2.5); empty is unknown, never assumed. */
  let lineFte = $state("");
  let objectiveTitle = $state("");

  /** The plans' horizons as bars, each demand line a milestone under its plan. */
  const timeline = $derived.by(() => {
    if (details.length === 0) return null;
    const tasks = details.flatMap((p) => [
      { id: p.pid, label: p.name, start: p.horizon_start, end: p.horizon_end },
      ...p.demand_lines.map((l) => ({
        id: l.pid,
        label: `${l.department}${l.job_title ? ` (${l.job_title})` : ""} · ${l.target_headcount}`,
        start: l.target_on,
        end: l.target_on,
        parentId: p.pid,
      })),
    ]);
    const starts = tasks.map((x) => x.start).sort();
    const ends = tasks.map((x) => x.end).sort();
    return {
      tasks,
      range: { start: starts[0] ?? "", end: ends[ends.length - 1] ?? "" },
    };
  });
  const today = new Date().toISOString().slice(0, 10);

  const PLAN_COLUMNS = ["draft", "active", "archived"].map((id) => ({ id, title: id }));
  const planCards = $derived(
    plans.map((p) => ({ id: p.pid, columnId: p.status, title: `${p.name} · ${p.demand_lines} line(s)` })),
  );

  async function movePlan(pid: string, to: string) {
    if (plans.find((p) => p.pid === pid)?.status === to) return;
    selected = pid;
    await run(() => setPlanStatus(pid, to));
  }

  const message = (cause: unknown) => (errorText(cause));
  const editable = (p: Plan) => p.status !== "archived";

  const readDetail = guardedLoad(
    () =>
      Promise.all([
        getWorkforcePlan(selected),
        planForecast(selected),
        planAlignment(selected),
        planCost(selected),
        planTolerances(selected),
        planEscalations(selected),
      ]),
    (answer) => {
      [plan, forecast, alignment, cost, tolerances, escalations] = answer;
    },
    (cause) => {
      error = message(cause);
    },
  );
  async function loadDetail() {
    plan = null;
    forecast = null;
    alignment = null;
    cost = null;
    tolerances = null;
    escalations = [];
    if (!selected) return;
    await readDetail();
  }

  const readPlans = guardedLoad(
    async () => {
      const found = await listWorkforcePlans();
      const full = await Promise.all(found.map((p) => getWorkforcePlan(p.pid)));
      return { found, full };
    },
    ({ found, full }) => {
      plans = found;
      if (!selected && found.length > 0) selected = found[0]?.pid ?? "";
      details = full;
      totals = Object.fromEntries(
        full.map((d) => [d.pid, d.demand_lines.reduce((sum, l) => sum + l.target_headcount, 0)]),
      );
    },
    (cause) => {
      error = message(cause);
    },
  );
  async function loadAll() {
    await readPlans();
    await loadDetail();
  }

  $effect(() => {
    void (async () => {
      try {
        roles = await listRoleProfiles();
      } catch (cause) {
        error = message(cause);
      }
      await loadAll();
    })();
  });

  $effect(() => {
    if (!org && organizationRefs.length > 0) org = organizationRefs[0] ?? "";
  });

  async function run(action: () => Promise<unknown>) {
    error = null;
    try {
      await action();
      await loadAll();
    } catch (cause) {
      error = message(cause);
    }
  }

  async function toggleObjective(lineId: string, current: string[], objective: string) {
    const next = current.includes(objective)
      ? current.filter((o) => o !== objective)
      : [...current, objective];
    await run(() => setLineObjectives(selected, lineId, next));
  }

  const compareColumns = $derived<DataGridColumn[]>([
    { id: "name", header: t("pages.planning.plan"), sortable: true, rowHeader: true, cell: planCell },
    { id: "status", header: t("pages.planning.status"), sortable: true, format: (v) => tv(String(v)) },
    {
      id: "horizon_start",
      header: t("pages.planning.horizon"),
      sortable: true,
      format: (_v, r) => formatDateRange(r.horizon_start as string, r.horizon_end as string),
    },
    { id: "demand_lines", header: t("pages.planning.lines"), sortable: true, align: "end" },
    {
      id: "planned_headcount",
      header: t("pages.planning.planned_headcount"),
      sortable: true,
      align: "end",
      accessor: (r) => totals[String(r.pid)] ?? null,
    },
  ]);

</script>

{#snippet planCell({ row }: { row: DataGridRow })}
  <button type="button" onclick={() => { selected = String(row.pid); void loadDetail(); }}>{row.name}</button>
{/snippet}

<svelte:head><title>{tf("pages.planning.page_title", { planning: t("nav.planning") })}</title></svelte:head>

<h1>{t("nav.planning")}</h1>
{#if error}<ErrorNote testid="error" message={error} />{/if}
<p class="muted">
  {t("pages.planning.a_plan_is_a_draft_world_of")}
</p>

<h2>{t("pages.planning.plans")}</h2>
<LilyKanban
  label={t("pages.planning.workforce_plans_by_status")}
  columns={PLAN_COLUMNS}
  cards={planCards}
  onMove={(pid, to) => void movePlan(pid, to)}
/>
{#if timeline}
  <LilyGantt
    label={t("pages.planning.plan_timeline")}
    caption="Plan horizons and demand-line target dates"
    range={timeline.range}
    tasks={timeline.tasks}
    {today}
    timeUnit="month"
  />
{/if}
{#if plans.length === 0}
  <p class="muted" data-testid="plan-compare">{t("pages.planning.no_plans_yet")}</p>
{:else}
  <DataTable
    testid="plan-compare"
    label={t("pages.planning.plan")}
    columns={compareColumns}
    rows={plans as unknown as DataGridRow[]}
    rowId={(row) => String(row.pid)}
  />
{/if}

<form
  onsubmit={(event) => {
    event.preventDefault();
    error = null;
    let budgetPart = {};
    if (budget.trim() !== "") {
      const code = currencyCode(budgetCurrency);
      if (code === null) {
        error = t("pages.planning.budget_needs_currency");
        return;
      }
      const minor = toMinor(budget, code, i18n.locale);
      if (minor === null || minor < 0) {
        error = t("pages.planning.bad_budget");
        return;
      }
      budgetPart = { budget_minor: minor, budget_currency: code };
    }
    // A percentage to basis points: "2,5" is 250.
    const attritionBp = attritionPct.trim() === "" ? undefined : parseScaled(attritionPct, 2);
    const onCostBp = onCostPct.trim() === "" ? undefined : parseScaled(onCostPct, 2);
    for (const [bp, label] of [
      [attritionBp, t("pages.planning.annual_attrition_blank_observed")],
      [onCostBp, t("pages.planning.employer_on_cost")],
    ] as const) {
      if (bp === null || (bp !== undefined && (bp < 0 || bp > 10000))) {
        error = tf("common.badNumber", { field: label, min: 0, max: 100, example: numberToInput(2.5) });
        return;
      }
    }
    void run(async () => {
      const created = await createWorkforcePlan({
        name,
        organization_ref: org,
        horizon_start: start,
        horizon_end: end,
        ...(typeof attritionBp === "number" ? { attrition_bp: attritionBp } : {}),
        ...budgetPart,
        ...(typeof onCostBp === "number" ? { on_cost_bp: onCostBp } : {}),
      });
      name = "";
      selected = created.pid;
    });
  }}
>
  <label>{t("pages.planning.name")} <input data-testid="plan-name" bind:value={name} required /></label>
  <label>
    {t("pages.planning.organization")}
    {#if organizationRefs.length > 0}
      <select bind:value={org} required>
        {#each organizationRefs as ref (ref)}<option value={ref}>{ref}</option>{/each}
      </select>
    {:else}
      <!-- No membership to choose from (a demo service with enforcement off): the URN is typed. -->
      <input data-testid="plan-org" bind:value={org} required />
    {/if}
  </label>
  <label>{t("pages.planning.from")} <input type="date" data-testid="plan-start" bind:value={start} required /></label>
  <label>{t("pages.planning.to")} <input type="date" data-testid="plan-end" bind:value={end} required /></label>
  <label>{t("pages.planning.annual_attrition_blank_observed")} <input inputmode="decimal" data-testid="plan-attrition" bind:value={attritionPct} /></label>
  <label>{t("pages.planning.annual_hiring_budget")} <input inputmode="decimal" data-testid="plan-budget" bind:value={budget} /></label>
  <label>{t("pages.planning.currency")} <input maxlength="3" size="4" data-testid="plan-currency" bind:value={budgetCurrency} /></label>
  <label>{t("pages.planning.employer_on_cost")} <input inputmode="decimal" data-testid="plan-on-cost" bind:value={onCostPct} /></label>
  <button type="submit" data-testid="plan-create">{t("pages.planning.create_plan")}</button>
</form>

{#if plan}
  <h2>{plan.name} <span class="chip">{tv(plan.status)}</span></h2>
  {#each NEXT[plan.status] ?? [] as next (next)}
    <button type="button" onclick={() => void run(() => setPlanStatus(plan!.pid, next))}>{tf("pages.planning.mark", { next: tv(next) })}</button>
  {/each}

  <PlanForecast {forecast} {cost} {alignment} />

  {#if tolerances}
    <PlanTolerances
      planPid={plan.pid}
      {tolerances}
      {escalations}
      editable={editable(plan)}
      {run}
    />
  {/if}

  <h3>{t("pages.planning.demand_lines")}</h3>
  <!-- Plain table: one checkbox column per strategic objective (dynamic columns) whose inputs keep focus across re-renders. -->
  <table>
    <thead>
      <tr><th>{t("pages.planning.department")}</th><th>{t("pages.planning.role")}</th><th>{t("pages.planning.date")}</th><th>{t("pages.planning.headcount")}</th><th>{t("contingent.fte")}</th><th>{t("engagement.basis")}</th><th>{t("plan.programme")}</th>{#each plan.objectives as o (o.pid)}<th>{o.title}</th>{/each}<th></th></tr>
    </thead>
    <tbody>
      {#each plan.demand_lines as line (line.pid)}
        <tr>
          <td>{line.department}</td><td>{line.job_title ?? "any"}</td><td>{formatDate(line.target_on)}</td><td>{line.target_headcount}</td>
          <td>{line.target_fte ?? "—"}</td>
          <td>{tv(line.basis)}{#if line.expected_ends_on} <span class="muted">{arrow()} {formatDate(line.expected_ends_on)}</span>{/if}</td>
          <td>{line.programme_ref ?? "—"}</td>
          {#each plan.objectives as o (o.pid)}
            <td>
              <input
                type="checkbox"
                aria-label={`${line.department} serves ${o.title}`}
                checked={line.objective_pids.includes(o.pid)}
                disabled={!editable(plan)}
                onchange={() => void toggleObjective(line.pid, line.objective_pids, o.pid)}
              />
            </td>
          {/each}
          <td>
            {#if editable(plan)}<button type="button" onclick={() => void run(() => removeDemandLine(plan!.pid, line.pid))}>{t("pages.planning.remove")}</button>{/if}
          </td>
        </tr>
      {/each}
    </tbody>
  </table>

  {#if editable(plan)}
    <h3>{t("pages.planning.add_demand")}</h3>
    <form
      onsubmit={(event) => {
        event.preventDefault();
        // FTE across the headcount to centi-FTE: "2,5" is 250.
        const fteCenti = lineFte.trim() === "" ? undefined : parseScaled(lineFte, 2);
        if (fteCenti === null || (fteCenti !== undefined && fteCenti < 0)) {
          error = tf("common.badNumberAtLeast", { field: t("plan.fteOptional"), min: 0, example: numberToInput(2.5) });
          return;
        }
        error = null;
        void run(() =>
          setDemandLine(plan!.pid, {
            department,
            target_on: targetOn,
            target_headcount: headcount,
            basis: lineBasis,
            ...(lineRole ? { role_profile_pid: lineRole } : {}),
            ...(fteCenti === undefined ? {} : { target_fte_percent: fteCenti }),
            ...(lineBasis !== "permanent" && lineEnds ? { expected_ends_on: lineEnds } : {}),
            ...(lineProgramme.trim() ? { programme_ref: lineProgramme.trim() } : {}),
          }),
        );
      }}
    >
      <label>{t("pages.planning.department")} <input data-testid="line-department" bind:value={department} required /></label>
      <label>
        {t("pages.planning.role_optional")}
        <select bind:value={lineRole}>
          <option value="">{t("pages.planning.any")}</option>
          {#each roles as r (r.pid)}<option value={r.pid}>{r.job_title}</option>{/each}
        </select>
      </label>
      <label>{t("pages.planning.target_date")} <input data-testid="line-target" type="date" min={plan.horizon_start} max={plan.horizon_end} bind:value={targetOn} required /></label>
      <label>{t("pages.planning.headcount")} <input type="number" min="0" bind:value={headcount} required /></label>
      <label>{t("plan.fteOptional")} <input data-testid="line-fte" inputmode="decimal" bind:value={lineFte} /></label>
      <label>
        {t("engagement.basis")}
        <select data-testid="line-basis" bind:value={lineBasis}>
          {#each PLAN_BASES as b (b)}<option value={b}>{tv(b)}</option>{/each}
        </select>
      </label>
      {#if lineBasis !== "permanent"}
        <label>{t("plan.expectedEnd")} <input data-testid="line-ends" type="date" min={targetOn || plan.horizon_start} bind:value={lineEnds} required={lineBasis !== "intern"} /></label>
      {/if}
      <label>{t("plan.programmeOptional")} <input data-testid="line-programme" bind:value={lineProgramme} /></label>
      <button type="submit" data-testid="line-add">{t("pages.planning.add")}</button>
    </form>
    <h3>{t("pages.planning.add_an_objective")}</h3>
    <form
      onsubmit={(event) => {
        event.preventDefault();
        void run(async () => {
          await addPlanObjective(plan!.pid, { title: objectiveTitle });
          objectiveTitle = "";
        });
      }}
    >
      <label>{t("pages.planning.objective")} <input data-testid="objective-title" bind:value={objectiveTitle} required /></label>
      <button type="submit">{t("pages.planning.add")}</button>
    </form>
  {/if}
{/if}
