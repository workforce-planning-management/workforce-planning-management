<script lang="ts">
  import { errorText } from "#lib/write.js";
  import ErrorNote from "#lib/components/ErrorNote.svelte";
  import { page } from "$app/state";
  import { benchmarkComparison, listBenchmarks, money } from "#lib/api/wpm.js";
  import { i18n, t, l } from "#lib/i18n.svelte.js";
  import type { Benchmark, ComparisonRow } from "#lib/api/types.js";
  import DataTable from "#lib/components/DataTable.svelte";
  import type {
    DataGridColumn,
    DataGridRow,
  } from "@lilydesignsystem/svelte-data-grid";

  // No switcher: every organization this person can read gets its own
  // comparison section below, fetched in parallel — not a single
  // guessed org. `scope` (not `organizations`) since it's expanded
  // through org confederation: a membership in a parent confederation
  // reads as every descendant too. The benchmark bands themselves are
  // a shared reference table, not organization-scoped, so they stay a
  // single list.
  const organizationRefs = $derived((page.data.scope ?? []) as string[]);

  let benchmarks = $state<Benchmark[] | null>(null);
  let rowsByOrganization = $state<Record<string, ComparisonRow[]> | null>(null);
  let error = $state<string | null>(null);

  $effect(() => {
    const refs = organizationRefs;
    void (async () => {
      try {
        const [bands, entries] = await Promise.all([
          listBenchmarks(),
          Promise.all(
            refs.map(async (ref) => {
              const comparison = await benchmarkComparison(ref);
              return [ref, comparison.rows] as const;
            }),
          ),
        ]);
        benchmarks = bands;
        rowsByOrganization = Object.fromEntries(entries);
      } catch (cause) {
        error = errorText(cause);
      }
    })();
  });

  const bandMoney = (field: "min_minor" | "median_minor" | "max_minor"): Pick<DataGridColumn, "accessor" | "format" | "exportValue"> => ({
    accessor: (r) => (r as unknown as Benchmark)[field],
    format: (_v, r) => money((r as unknown as Benchmark)[field], (r as unknown as Benchmark).currency, i18n.locale),
    // An export carries the amount with its currency, not a bare number of minor units.
    exportValue: (r) => money((r as unknown as Benchmark)[field], (r as unknown as Benchmark).currency, i18n.locale),
  });
  const bandColumns = $derived<DataGridColumn[]>([
    { id: "job_title", header: t("common.jobTitle"), sortable: true, rowHeader: true },
    { id: "min_minor", header: t("pages.benchmarks.min"), sortable: true, align: "end", ...bandMoney("min_minor") },
    { id: "median_minor", header: t("pages.benchmarks.median"), sortable: true, align: "end", ...bandMoney("median_minor") },
    { id: "max_minor", header: t("pages.benchmarks.max"), sortable: true, align: "end", ...bandMoney("max_minor") },
  ]);
  const comparisonColumns = $derived<DataGridColumn[]>([
    { id: "worker_pid", header: t("nav.workers"), sortable: true, cell: workerCell },
    { id: "job_title", header: t("common.jobTitle"), sortable: true },
    { id: "department", header: t("common.department"), sortable: true },
    { id: "flag", header: t("bench.flag"), sortable: true, cell: flagCell },
  ]);
</script>

{#snippet workerCell({ row }: { row: DataGridRow })}
  <a href={l(`/workers/${row.worker_pid}`)}>{String(row.worker_pid).slice(0, 8)}</a>
{/snippet}

{#snippet flagCell({ row }: { row: DataGridRow })}
  {#if row.flag}
    <span class={`chip flag-${row.flag}`}>{row.flag}</span>
  {:else}
    <span class="muted">—</span>
  {/if}
{/snippet}

<h1>{t("nav.benchmarks")}</h1>

{#if error}
  <ErrorNote testid="error" prefixed message={error} />
{:else if benchmarks === null || rowsByOrganization === null}
  <p>{t("common.loading")}</p>
{:else}
  <DataTable
    testid="bands"
    exportName="pay-bands"
    label={t("nav.benchmarks")}
    columns={bandColumns}
    rows={benchmarks as unknown as DataGridRow[]}
    rowId={(row) => String(row.pid)}
  />

  <h2>{t("bench.flag")}</h2>
  {#if organizationRefs.length === 0}
    <p class="muted">{t("org.noOrganizations")}</p>
  {:else}
    {#each organizationRefs as ref (ref)}
      <section class="benchmark-org-section">
        <h3><code>{ref}</code></h3>
        <DataTable
          testid="comparison"
          label={ref}
          columns={comparisonColumns}
          rows={(rowsByOrganization[ref] ?? []) as unknown as DataGridRow[]}
          rowId={(row) => String(row.worker_pid)}
          pageSize={25}
        />
      </section>
    {/each}
  {/if}
{/if}

<style>
  :global(.flag-below_min) {
    color: var(--red-day, #c0392b);
  }
  :global(.flag-above_max) {
    color: var(--state-reserved, #b57e10);
  }
  .benchmark-org-section + .benchmark-org-section {
    margin-top: 1rem;
  }
  .benchmark-org-section h3 {
    font-size: 0.9rem;
    margin: 0.75rem 0 0.5rem;
  }
  .benchmark-org-section h3 code {
    color: var(--muted);
    font-weight: 400;
  }
</style>
