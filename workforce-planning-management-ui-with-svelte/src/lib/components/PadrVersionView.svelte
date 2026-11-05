<!--
  One version of a PADR, read only (WPM-R172). A signed version shows the objectives it froze, so a later change to a goal never
  reaches it (WPM-D165). A wellbeing answer not given shows a dash, never a no.
-->
<script lang="ts">
  import GoalCard from "#lib/components/GoalCard.svelte";
  import type { GoalView, PadrVersion } from "#lib/api/wpm.js";
  import { formatDateRange } from "#lib/locale-format.js";
  import { t, tv } from "#lib/i18n.svelte.js";

  let {
    version,
    stepLabels,
    liveObjectives,
    onchange,
  }: {
    version: PadrVersion;
    stepLabels: Map<string, string>;
    liveObjectives: GoalView[];
    onchange: () => void | Promise<void>;
  } = $props();

  const row = $derived(version.version);
  const shown = (value: number | null) => value ?? "—";
</script>

<section class="panel" data-testid="padr-read">
  <p class="muted">{formatDateRange(row.period_start, row.period_end)}</p>
  <h2>{t("padr.values")}</h2>
  <p data-testid="padr-values-read">{row.values_summary ?? "—"}</p>
  <h2>{t("padr.wellbeing")}</h2>
  <p data-testid="padr-wellbeing-read">
    {row.wellbeing_offered === null ? "—" : row.wellbeing_offered ? t("padr.yes") : t("padr.no")}
  </p>
  <h2>{t("padr.needs")}</h2>
  <ul>
    {#each row.development_needs as need, index (index)}
      <li data-testid="padr-need-read">
        {need.need}
        {#if need.plan_item_pid && stepLabels.has(need.plan_item_pid)}
          <span class="muted">· {t("goals.planStep")}: {stepLabels.get(need.plan_item_pid)}</span>
        {/if}
      </li>
    {:else}
      <li class="muted">—</li>
    {/each}
  </ul>
  <h2>{t("padr.objectives")}</h2>
  {#if row.objectives}
    <p class="muted" data-testid="padr-frozen">{t("padr.frozen")}</p>
    <ul>
      {#each row.objectives as objective (objective.pid)}
        <li data-testid="padr-frozen-objective">
          <strong>{objective.title}</strong>
          <span class="muted">· {tv(objective.status)} · {formatDateRange(objective.period_start, objective.period_end)}</span>
          {#if objective.measures.length > 0}
            <ul>
              {#each objective.measures as measure (measure.pid)}
                <li>
                  {measure.name}:
                  {#if measure.kind === "milestone"}
                    {measure.done === null ? "—" : measure.done ? t("goals.done") : t("goals.notDone")}
                  {:else}
                    {t("goals.current")}
                    {shown(measure.current_value)} · {t("goals.target")}
                    {shown(measure.target_value)}
                    {#if measure.unit}<span class="muted">{measure.unit}</span>{/if}
                  {/if}
                </li>
              {/each}
            </ul>
          {/if}
        </li>
      {:else}
        <li class="muted">{t("padr.noObjectives")}</li>
      {/each}
    </ul>
  {:else}
    {#each liveObjectives as objective (objective.goal.pid)}
      <GoalCard view={objective} party="manager" {onchange} />
    {:else}
      <p class="muted">{t("padr.noObjectives")}</p>
    {/each}
  {/if}
</section>
