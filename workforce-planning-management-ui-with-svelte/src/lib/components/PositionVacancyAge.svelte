<!--
  Vacancy age (WPM-R288): approved, open or frozen posts nobody holds, by how long they have been vacant, grouped by cost
  centre, by band or for the critical posts. A post vacant before the history began is "start unknown", never zero, and in no
  age band; abolished posts stand beside the ages, because abolishing a post ends its vacancy. A group under the floor is
  withheld for a reader who may not see holders, and says so. No holder is named. A failed load hides the panel.
-->
<script lang="ts">
  import { formatInteger } from "#lib/locale-format.js";
  import { guardedLoad } from "#lib/loader.svelte.js";
  import { positionVacancyAge, type VacancyGroup, type WithheldCount } from "#lib/api/wpm.js";
  import { t, i18n } from "#lib/i18n.svelte.js";
  import DataTable from "#lib/components/DataTable.svelte";
  import type { DataGridColumn, DataGridRow } from "@lilydesignsystem/svelte-data-grid";

  let { organization, revision = 0 }: { organization: string; revision?: number } = $props();

  type Report = Awaited<ReturnType<typeof positionVacancyAge>>;
  let report = $state<Report | null>(null);
  let groupBy = $state<"cost_centre" | "band" | "critical">("cost_centre");

  const load = guardedLoad(
    () => positionVacancyAge(organization),
    (found) => (report = found),
    () => (report = null),
  );
  $effect(() => {
    void organization;
    void revision;
    void load();
  });

  const count = (n: WithheldCount | undefined) =>
    n === undefined ? "—" : n === "withheld" ? t("pages.positions.withheld") : formatInteger(n, i18n.locale);
  const AGES = [
    ["0-30", "pages.positions.days_0_30"],
    ["31-90", "pages.positions.days_31_90"],
    ["91-180", "pages.positions.days_91_180"],
    ["181+", "pages.positions.days_181"],
  ] as const;

  const rows = $derived<VacancyGroup[]>(
    !report
      ? []
      : groupBy === "band"
        ? report.bands
        : groupBy === "critical"
          ? report.critical
            ? [report.critical]
            : []
          : report.cost_centres,
  );
  const label = (g: string) =>
    g === "unknown" ? t("pages.positions.unknown") : g === "critical" ? t("pages.positions.by_critical") : g;

  const columns = $derived<DataGridColumn[]>([
    { id: "group", header: t("pages.positions.group"), rowHeader: true, accessor: (r) => label(String(r.group)) },
    { id: "vacancies", header: t("pages.positions.vacancies"), align: "end", accessor: (r) => count(r.vacancies as WithheldCount) },
    ...AGES.map(([band, key]) => ({
      id: `age_${band}`,
      header: t(key),
      align: "end" as const,
      accessor: (r: DataGridRow) => count((r.by_age as Record<string, WithheldCount>)[band]),
    })),
    { id: "start_unknown", header: t("pages.positions.start_unknown"), align: "end", accessor: (r) => count(r.start_unknown as WithheldCount) },
    { id: "frozen", header: t("pages.positions.frozen"), align: "end", accessor: (r) => count(r.frozen as WithheldCount) },
    { id: "no_review", header: t("pages.positions.no_review_date"), align: "end", accessor: (r) => count(r.frozen_without_review_date as WithheldCount) },
    { id: "overdue", header: t("pages.positions.review_overdue"), align: "end", accessor: (r) => count(r.review_overdue as WithheldCount) },
    { id: "abolished", header: t("pages.positions.abolished"), align: "end", accessor: (r) => count(r.abolished as number) },
  ]);
</script>

{#if report}
  <section data-testid="vacancy-age">
    <h2>{t("pages.positions.vacancy_age")}</h2>
    <p class="muted">{t("pages.positions.vacancy_age_note")}</p>
    <p>
      <label>
        {t("pages.positions.group_by")}
        <select bind:value={groupBy} data-testid="vacancy-group-by">
          <option value="cost_centre">{t("pages.positions.cost_centre")}</option>
          <option value="band">{t("pages.positions.by_band")}</option>
          <option value="critical">{t("pages.positions.by_critical")}</option>
        </select>
      </label>
    </p>
    {#if rows.length === 0}
      <p class="muted" data-testid="vacancy-age-empty">
        {groupBy === "critical" ? t("pages.positions.no_critical") : t("pages.positions.no_positions")}
      </p>
    {:else}
      <DataTable
        testid="vacancy-age-table"
        label={t("pages.positions.vacancy_age")}
        {columns}
        rows={rows as unknown as DataGridRow[]}
        rowId={(row) => String(row.group)}
        pageSize={25}
      />
    {/if}
  </section>
{/if}
