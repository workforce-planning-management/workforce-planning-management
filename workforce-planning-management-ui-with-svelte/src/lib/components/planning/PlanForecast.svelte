<!--
  What a plan projects (WPM-R137): headcount now against planned by department and date, the cost of closing the
  gaps by hiring, and how much of the plan serves a strategic objective. All aggregate; a figure the caller may not
  see is withheld, and an unknown is shown as unknown.
-->
<script lang="ts">
  import { arrow, formatDate, formatDecimal, formatNumber } from "#lib/locale-format.js";
  import { money, PLAN_BASES } from "#lib/api/wpm.js";
  import type { planAlignment, planCost, planForecast } from "#lib/api/wpm.js";
  import { percentWithWorkings } from "#lib/format.js";
  import { t, tf, tn, tv } from "#lib/i18n.svelte.js";
  import DataTable from "#lib/components/DataTable.svelte";
  import type { DataGridColumn, DataGridRow } from "@lilydesignsystem/svelte-data-grid";

  type Forecast = Awaited<ReturnType<typeof planForecast>>;
  type Alignment = Awaited<ReturnType<typeof planAlignment>>;
  type Cost = Awaited<ReturnType<typeof planCost>>;

  let {
    forecast,
    cost,
    alignment,
  }: { forecast: Forecast | null; cost: Cost | null; alignment: Alignment | null } = $props();

  const programmeColumns = $derived<DataGridColumn[]>([
    {
      id: "programme_ref",
      header: t("plan.programme"),
      sortable: true,
      accessor: (r) => r.programme_ref ?? t("plan.noProgramme"),
    },
    { id: "target_on", type: "date", header: t("pages.planning.date"), sortable: true },
    { id: "headcount", header: t("pages.planning.headcount"), sortable: true, align: "end" },
    { id: "fte", header: t("contingent.fte"), sortable: true, align: "end" },
  ]);

  const costColumns = (currency: string): DataGridColumn[] => [
    { id: "department", header: t("pages.planning.department"), sortable: true },
    { id: "target_on", type: "date", header: t("pages.planning.date"), sortable: true },
    { id: "hires_needed", header: t("pages.planning.hires_needed"), sortable: true, align: "end" },
    {
      id: "unit_cost_minor",
      header: t("pages.planning.unit_cost"),
      sortable: true,
      align: "end",
      format: (v) => money(v as number | null, currency),
    },
    {
      id: "unit_cost_source",
      header: t("pages.planning.source"),
      sortable: true,
      accessor: (r) => tv(String(r.unit_cost_source ?? "")) || tv(String(r.reason ?? "")) || null,
    },
    {
      id: "annual_cost_minor",
      header: t("pages.planning.annual_cost"),
      sortable: true,
      align: "end",
      format: (v) => money(v as number | null, currency),
    },
  ];
</script>

{#if forecast}
  <p class="muted">{forecast.derivation}</p>
  <p data-testid="plan-assumptions">
    {t("pages.planning.attrition_assumption")}
    {#if forecast.assumptions.attrition_bp !== null}
      {tf("pages.planning.a_year", { attrition_bp: formatDecimal(forecast.assumptions.attrition_bp / 100, 1), attrition_source: tv(forecast.assumptions.attrition_source) })}
    {:else}
      <strong>{t("pages.planning.insufficient_history")}</strong> {t("pages.planning.no_projection_until_a_plan")}
    {/if}
    {tf("pages.planning.hires_assumed", { hires_assumed: forecast.assumptions.hires_assumed })}
  </p>
  <!-- Plain table: each row is followed by colspan note rows (basis split, assumptions, competency gaps), which the grid cannot hold. -->
  <table data-testid="plan-forecast">
    <thead>
      <tr><th>{t("pages.planning.department")}</th><th>{t("pages.planning.date")}</th><th>{t("pages.planning.today")}</th><th>{t("pages.planning.planned")}</th><th>{t("pages.planning.projected_supply")}</th><th>{t("pages.planning.gap")}</th><th>{t("pages.planning.suggested_levers")}</th></tr>
    </thead>
    <tbody>
      {#each forecast.departments as row (row.department + row.target_on)}
        <tr>
          <td>{row.department}</td><td>{formatDate(row.target_on)}</td>
          <td>{row.opening_headcount}</td><td>{row.planned_demand}</td>
          <td>{row.projected_supply ?? "—"}</td>
          <td class:warn={(row.headcount_gap ?? 0) > 0}>{row.headcount_gap ?? "—"}</td>
          <td>{row.levers?.map((lever) => tv(lever)).join(", ") ?? "—"}</td>
        </tr>
        <tr class="muted" data-testid="forecast-basis">
          <td colspan="2">{t("plan.byBasis")}</td>
          <td colspan="5">
            {#each PLAN_BASES as b (b)}
              {@const supply = row.supply_by_basis[b]}
              {@const demand = row.demand_by_basis.find((d) => d.basis === b)}
              {#if supply?.headcount || demand}
                <span class="chip">
                  {tv(b)}: {demand?.headcount ?? 0} {arrow()} {supply?.headcount ?? "—"}
                  {#if demand?.fte != null || supply?.fte != null}
                    ({demand?.fte ?? "—"} {arrow()} {supply?.fte ?? "—"} {t("contingent.fte")})
                  {/if}
                </span>
              {/if}
            {/each}
            {#if row.fte_gap !== null}
              <span data-testid="forecast-fte-gap">· {t("plan.fteGap")}: {row.fte_gap}</span>
            {/if}
          </td>
        </tr>
        {#if row.conversions}
          <tr class="muted" data-testid="forecast-conversions">
            <td colspan="2">{t("plan.conversions")}</td>
            <td colspan="5">
              <span class="chip" data-testid="conversions-agreed">
                {t("plan.conversionsAgreed")}: {row.conversions.agreed.headcount}
                {#if row.conversions.agreed.fte !== null}({row.conversions.agreed.fte} {t("contingent.fte")}){/if}
              </span>
              <span class="chip" data-testid="conversions-hoped">
                {t("plan.conversionsHoped")}: {row.conversions.hoped.headcount}
                {#if row.conversions.hoped.fte !== null}({row.conversions.hoped.fte} {t("contingent.fte")}){/if}
              </span>
              {t("plan.conversionsNote")}
            </td>
          </tr>
        {/if}
        {#each row.supply_assumptions.slice(1) as note (note)}
          <tr class="muted"><td colspan="7">{note}</td></tr>
        {/each}
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
  {#if forecast.programmes.some((p) => p.programme_ref)}
    <h3>{t("plan.demandByProgramme")}</h3>
    <DataTable
      testid="plan-forecast-programmes"
      label={t("plan.demandByProgramme")}
      columns={programmeColumns}
      rows={forecast.programmes as unknown as DataGridRow[]}
      rowId={(row) => String(row.target_on) + String(row.programme_ref ?? "")}
    />
  {/if}
{/if}

{#if cost}
  <h3>{t("pages.planning.cost_of_closing_the_gaps_by_hiring")}</h3>
  <p class="muted">{cost.derivation}</p>
  {#if !cost.salary_visible}
    <p data-testid="plan-cost-hidden">{t("pages.planning.salary_figures_need_payroll_read")}</p>
  {/if}
  <DataTable
    testid="plan-cost"
    label={t("pages.planning.cost_of_closing_the_gaps_by_hiring")}
    columns={costColumns(cost.currency)}
    rows={cost.groups as unknown as DataGridRow[]}
    rowId={(row) => String(row.department) + String(row.target_on)}
  />
  {#if cost.salary_visible}
    <p data-testid="plan-cost-total">
      {tf("pages.planning.total_annual_incl_on_cost", { on_cost_bp: formatDecimal(cost.assumptions.on_cost_bp / 100, 1), currency: money(cost.total_annual_cost_minor, cost.currency) })}
      {#if cost.uncosted_groups > 0}<span class="muted">{tn("pages.planning.line_group_s_could_not_be_costed", cost.uncosted_groups, { uncosted_groups: cost.uncosted_groups })}</span>{/if}
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
    <p class="muted">{tf("pages.planning.demand_serving_no_objective", { target_headcount: alignment.unaligned_demand_lines.map((l) => `${l.department} ${formatDate(l.target_on)} (${formatNumber(l.target_headcount)})`).join("; ") })}</p>
  {/if}
{/if}

<style>
  td.warn {
    color: #b45309;
    font-weight: 600;
  }
</style>
