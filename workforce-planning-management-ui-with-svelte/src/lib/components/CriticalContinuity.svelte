<!--
  Critical posts: continuity over 36 months (WPM-R290). Tenures, days held and days empty of the posts HR has marked
  critical, by cost centre or by band, with every critical post together. A senior group is small: a group with fewer posts
  than the floor is withheld for every reader, and so is the next smallest, so it cannot be had from the total. A withheld
  figure says "withheld", never a number. No holder is named. A failed load hides the panel.
-->
<script lang="ts">
  import { formatInteger } from "#lib/locale-format.js";
  import { guardedLoad } from "#lib/loader.svelte.js";
  import { criticalContinuity, type CriticalGroup } from "#lib/api/wpm.js";
  import { t, i18n } from "#lib/i18n.svelte.js";
  import DataTable from "#lib/components/DataTable.svelte";
  import type { DataGridColumn, DataGridRow } from "@lilydesignsystem/svelte-data-grid";

  let { organization, revision = 0 }: { organization: string; revision?: number } = $props();

  type Report = Awaited<ReturnType<typeof criticalContinuity>>;
  let report = $state<Report | null>(null);
  let groupBy = $state<"cost_centre" | "band">("cost_centre");

  const load = guardedLoad(
    () => criticalContinuity(organization),
    (found) => (report = found),
    () => (report = null),
  );
  $effect(() => {
    void organization;
    void revision;
    void load();
  });

  const table = $derived(report ? (groupBy === "band" ? report.bands : report.cost_centres) : null);
  const rows = $derived<CriticalGroup[]>(table && table.groups.length > 0 ? [...table.groups, table.total] : []);
  const label = (g: string) =>
    g === "all" ? t("pages.positions.all_posts") : g === "unknown" ? t("pages.positions.unknown") : g;
  // A withheld group says so; a figure the service could not add up is unknown, never zero.
  const figure = (r: DataGridRow, key: keyof CriticalGroup) =>
    r.withheld ? t("pages.positions.withheld") : r[key] === null ? t("pages.positions.unknown") : formatInteger(r[key] as number, i18n.locale);

  const columns = $derived<DataGridColumn[]>([
    { id: "group", header: t("pages.positions.group"), rowHeader: true, accessor: (r) => label(String(r.group)) },
    { id: "posts", header: t("pages.positions.positions"), align: "end", accessor: (r) => figure(r, "posts") },
    { id: "tenures", header: t("pages.positions.tenures"), align: "end", accessor: (r) => figure(r, "tenures") },
    { id: "days_held", header: t("pages.positions.days_held"), align: "end", accessor: (r) => figure(r, "days_held") },
    { id: "days_empty", header: t("pages.positions.days_empty"), align: "end", accessor: (r) => figure(r, "days_empty") },
    { id: "held_now", header: t("pages.positions.held_now"), align: "end", accessor: (r) => figure(r, "held_now") },
    { id: "counted_partly", header: t("pages.positions.counted_partly"), align: "end", accessor: (r) => figure(r, "counted_partly") },
  ]);
</script>

{#if report}
  <section data-testid="critical-continuity">
    <h2>{t("pages.positions.continuity")}</h2>
    <p class="muted">{t("pages.positions.continuity_note")}</p>
    {#if rows.length === 0}
      <p class="muted" data-testid="critical-continuity-empty">{t("pages.positions.no_critical")}</p>
    {:else}
      <p>
        <label>
          {t("pages.positions.group_by")}
          <select bind:value={groupBy} data-testid="continuity-group-by">
            <option value="cost_centre">{t("pages.positions.cost_centre")}</option>
            <option value="band">{t("pages.positions.by_band")}</option>
          </select>
        </label>
      </p>
      <DataTable
        testid="critical-continuity-table"
        label={t("pages.positions.continuity")}
        {columns}
        rows={rows as unknown as DataGridRow[]}
        rowId={(row) => String(row.group)}
        pageSize={25}
      />
    {/if}
  </section>
{/if}
