<!--
  Strategic workforce planning (`/planning`): a plan is a draft world of
  aggregate hypothetical headcount — it never touches live worker records.
  Per department and date: planned demand, projected supply (stated attrition,
  no hires assumed), the headcount gap, and the competency gap against a role
  profile; plus whether the plan serves its strategic objectives. Gaps are
  aggregate and levers are suggestions, never decisions.
-->
<script lang="ts">
  import { page } from "$app/state";
  import {
    addPlanObjective,
    createWorkforcePlan,
    getWorkforcePlan,
    listRoleProfiles,
    listWorkforcePlans,
    money,
    planAlignment,
    planCost,
    planForecast,
    removeDemandLine,
    setDemandLine,
    setLineObjectives,
    setPlanStatus,
  } from "#lib/api/wpm.js";
  import { percentWithWorkings } from "#lib/format.js";
  import LilyGantt from "#lib/components/LilyGantt.svelte";
  import LilyKanban from "#lib/components/LilyKanban.svelte";
  import { t, tf, tv } from "#lib/i18n.svelte.js";

  type Plans = Awaited<ReturnType<typeof listWorkforcePlans>>;
  type Plan = Awaited<ReturnType<typeof getWorkforcePlan>>;
  type Forecast = Awaited<ReturnType<typeof planForecast>>;
  type Alignment = Awaited<ReturnType<typeof planAlignment>>;
  type Cost = Awaited<ReturnType<typeof planCost>>;

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
  let roles = $state<Awaited<ReturnType<typeof listRoleProfiles>>>([]);
  let totals = $state<Record<string, number>>({});
  let details = $state<Plan[]>([]);
  let error = $state<string | null>(null);

  let name = $state("");
  let org = $state("");
  let start = $state("");
  let end = $state("");
  let attritionPct = $state<number | null>(null);
  let budget = $state<number | null>(null);
  let budgetCurrency = $state("GBP");
  let onCostPct = $state<number | null>(null);
  let department = $state("");
  let lineRole = $state("");
  let targetOn = $state("");
  let headcount = $state(1);
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

  const message = (cause: unknown) => (cause instanceof Error ? cause.message : String(cause));
  const editable = (p: Plan) => p.status !== "archived";

  async function loadDetail() {
    plan = null;
    forecast = null;
    alignment = null;
    cost = null;
    if (!selected) return;
    try {
      [plan, forecast, alignment, cost] = await Promise.all([
        getWorkforcePlan(selected),
        planForecast(selected),
        planAlignment(selected),
        planCost(selected),
      ]);
    } catch (cause) {
      error = message(cause);
    }
  }

  async function loadAll() {
    try {
      plans = await listWorkforcePlans();
      if (!selected && plans.length > 0) selected = plans[0]?.pid ?? "";
      details = await Promise.all(plans.map((p) => getWorkforcePlan(p.pid)));
      totals = Object.fromEntries(
        details.map((d) => [d.pid, d.demand_lines.reduce((sum, l) => sum + l.target_headcount, 0)]),
      );
      await loadDetail();
    } catch (cause) {
      error = message(cause);
    }
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
</script>

<svelte:head><title>{tf("pages.planning.page_title", { planning: t("nav.planning") })}</title></svelte:head>

<h1>{t("nav.planning")}</h1>
{#if error}<p class="error" data-testid="error">{error}</p>{/if}
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
<table data-testid="plan-compare">
  <thead><tr><th>{t("pages.planning.plan")}</th><th>{t("pages.planning.status")}</th><th>{t("pages.planning.horizon")}</th><th>{t("pages.planning.lines")}</th><th>{t("pages.planning.planned_headcount")}</th></tr></thead>
  <tbody>
    {#each plans as p (p.pid)}
      <tr>
        <td><button type="button" onclick={() => { selected = p.pid; void loadDetail(); }}>{p.name}</button></td>
        <td>{tv(p.status)}</td>
        <td>{p.horizon_start} → {p.horizon_end}</td>
        <td>{p.demand_lines}</td>
        <td>{totals[p.pid] ?? "—"}</td>
      </tr>
    {:else}
      <tr><td colspan="5" class="muted">{t("pages.planning.no_plans_yet")}</td></tr>
    {/each}
  </tbody>
</table>

<form
  onsubmit={(event) => {
    event.preventDefault();
    void run(async () => {
      const created = await createWorkforcePlan({
        name,
        organization_ref: org,
        horizon_start: start,
        horizon_end: end,
        ...(attritionPct !== null ? { attrition_bp: Math.round(attritionPct * 100) } : {}),
        ...(budget !== null
          ? { budget_minor: Math.round(budget * 100), budget_currency: budgetCurrency }
          : {}),
        ...(onCostPct !== null ? { on_cost_bp: Math.round(onCostPct * 100) } : {}),
      });
      name = "";
      selected = created.pid;
    });
  }}
>
  <label>{t("pages.planning.name")} <input data-testid="plan-name" bind:value={name} required /></label>
  <label>
    {t("pages.planning.organization")}
    <select bind:value={org} required>
      {#each organizationRefs as ref (ref)}<option value={ref}>{ref}</option>{/each}
    </select>
  </label>
  <label>{t("pages.planning.from")} <input type="date" bind:value={start} required /></label>
  <label>{t("pages.planning.to")} <input type="date" bind:value={end} required /></label>
  <label>{t("pages.planning.annual_attrition_blank_observed")} <input type="number" min="0" max="100" step="0.1" bind:value={attritionPct} /></label>
  <label>{t("pages.planning.annual_hiring_budget")} <input type="number" min="0" step="any" bind:value={budget} /></label>
  <label>{t("pages.planning.currency")} <input maxlength="3" size="4" bind:value={budgetCurrency} /></label>
  <label>{t("pages.planning.employer_on_cost")} <input type="number" min="0" max="100" step="0.1" bind:value={onCostPct} /></label>
  <button type="submit" data-testid="plan-create">{t("pages.planning.create_plan")}</button>
</form>

{#if plan}
  <h2>{plan.name} <span class="chip">{tv(plan.status)}</span></h2>
  {#each NEXT[plan.status] ?? [] as next (next)}
    <button type="button" onclick={() => void run(() => setPlanStatus(plan!.pid, next))}>{tf("pages.planning.mark", { next: tv(next) })}</button>
  {/each}

  {#if forecast}
    <p class="muted">{forecast.derivation}</p>
    <p data-testid="plan-assumptions">
      {t("pages.planning.attrition_assumption")}
      {#if forecast.assumptions.attrition_bp !== null}
        {tf("pages.planning.a_year", { attrition_bp: (forecast.assumptions.attrition_bp / 100).toFixed(1), attrition_source: tv(forecast.assumptions.attrition_source) })}
      {:else}
        <strong>{t("pages.planning.insufficient_history")}</strong> {t("pages.planning.no_projection_until_a_plan")}
      {/if}
      {tf("pages.planning.hires_assumed", { hires_assumed: forecast.assumptions.hires_assumed })}
    </p>
    <table data-testid="plan-forecast">
      <thead>
        <tr><th>{t("pages.planning.department")}</th><th>{t("pages.planning.date")}</th><th>{t("pages.planning.today")}</th><th>{t("pages.planning.planned")}</th><th>{t("pages.planning.projected_supply")}</th><th>{t("pages.planning.gap")}</th><th>{t("pages.planning.suggested_levers")}</th></tr>
      </thead>
      <tbody>
        {#each forecast.departments as row (row.department + row.target_on)}
          <tr>
            <td>{row.department}</td><td>{row.target_on}</td>
            <td>{row.opening_headcount}</td><td>{row.planned_demand}</td>
            <td>{row.projected_supply ?? "—"}</td>
            <td class:warn={(row.headcount_gap ?? 0) > 0}>{row.headcount_gap ?? "—"}</td>
            <td>{row.levers?.join(", ") ?? "—"}</td>
          </tr>
          {#each row.competency_gaps as gap (gap.job_title + (gap.skill ?? ""))}
            <tr class="muted">
              <td colspan="2">{tf("pages.planning.needs", { job_title: gap.job_title, skill: gap.skill, importance: tv(gap.importance), min_proficiency: gap.min_proficiency })}</td>
              <td colspan="5">
                {tf("pages.planning.need_proficient_now_shortfall", { needed: gap.needed, proficient_now: gap.proficient_now, shortfall: gap.shortfall, reskill_pool: gap.reskill_pool })}
              </td>
            </tr>
          {/each}
        {:else}
          <tr><td colspan="7" class="muted">{t("pages.planning.no_demand_lines_yet")}</td></tr>
        {/each}
      </tbody>
    </table>
  {/if}

  {#if cost}
    <h3>{t("pages.planning.cost_of_closing_the_gaps_by_hiring")}</h3>
    <p class="muted">{cost.derivation}</p>
    {#if !cost.salary_visible}
      <p data-testid="plan-cost-hidden">{t("pages.planning.salary_figures_need_payroll_read")}</p>
    {/if}
    <table data-testid="plan-cost">
      <thead><tr><th>{t("pages.planning.department")}</th><th>{t("pages.planning.date")}</th><th>{t("pages.planning.hires_needed")}</th><th>{t("pages.planning.unit_cost")}</th><th>{t("pages.planning.source")}</th><th>{t("pages.planning.annual_cost")}</th></tr></thead>
      <tbody>
        {#each cost.groups as g (g.department + g.target_on)}
          <tr>
            <td>{g.department}</td><td>{g.target_on}</td><td>{g.hires_needed ?? "—"}</td>
            <td>{money(g.unit_cost_minor, cost.currency)}</td>
            <td>{tv(g.unit_cost_source) || tv(g.reason) || "—"}</td>
            <td>{money(g.annual_cost_minor, cost.currency)}</td>
          </tr>
        {/each}
      </tbody>
    </table>
    {#if cost.salary_visible}
      <p data-testid="plan-cost-total">
        {tf("pages.planning.total_annual_incl_on_cost", { on_cost_bp: (cost.assumptions.on_cost_bp / 100).toFixed(1), currency: money(cost.total_annual_cost_minor, cost.currency) })}
        {#if cost.uncosted_groups > 0}<span class="muted">{tf("pages.planning.line_group_s_could_not_be_costed", { uncosted_groups: cost.uncosted_groups })}</span>{/if}
        {#if cost.affordability}
          {tf("pages.planning.budget", { currency: money(cost.affordability.budget_minor, cost.currency) })}
          <span class:warn={!cost.affordability.within_budget}>
            {tf("pages.planning.remaining", { within_budget: cost.affordability.within_budget ? t("pages.planning.within_budget") : t("pages.planning.over_budget"), currency: money(cost.affordability.remaining_minor, cost.currency) })}
          </span>
        {/if}
      </p>
    {/if}
  {/if}

  {#if alignment}
    <h3>{t("pages.planning.alignment_with_strategy")}</h3>
    <p data-testid="plan-alignment">
      {tf("pages.planning.planned_headcount_tied_to_an", { aligned_share: percentWithWorkings(alignment.aligned_share), in_plan_departments: alignment.critical_roles_without_bench.in_plan_departments, all_departments: alignment.critical_roles_without_bench.all_departments })}
    </p>
    {#if alignment.unresourced_objectives.length > 0}
      <p>{tf("pages.planning.objectives_with_no_demand_behind", { unresourced_objectives: alignment.unresourced_objectives.join("; ") })}</p>
    {/if}
    {#if alignment.unaligned_demand_lines.length > 0}
      <p class="muted">{tf("pages.planning.demand_serving_no_objective", { target_headcount: alignment.unaligned_demand_lines.map((l) => `${l.department} ${l.target_on} (${l.target_headcount})`).join("; ") })}</p>
    {/if}
  {/if}

  <h3>{t("pages.planning.demand_lines")}</h3>
  <table>
    <thead>
      <tr><th>{t("pages.planning.department")}</th><th>{t("pages.planning.role")}</th><th>{t("pages.planning.date")}</th><th>{t("pages.planning.headcount")}</th>{#each plan.objectives as o (o.pid)}<th>{o.title}</th>{/each}<th></th></tr>
    </thead>
    <tbody>
      {#each plan.demand_lines as line (line.pid)}
        <tr>
          <td>{line.department}</td><td>{line.job_title ?? "any"}</td><td>{line.target_on}</td><td>{line.target_headcount}</td>
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
        void run(() =>
          setDemandLine(plan!.pid, {
            department,
            target_on: targetOn,
            target_headcount: headcount,
            ...(lineRole ? { role_profile_pid: lineRole } : {}),
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
      <label>{t("pages.planning.target_date")} <input type="date" min={plan.horizon_start} max={plan.horizon_end} bind:value={targetOn} required /></label>
      <label>{t("pages.planning.headcount")} <input type="number" min="0" bind:value={headcount} required /></label>
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

<style>
  td.warn { color: #b45309; font-weight: 600; }
</style>
