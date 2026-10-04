<script lang="ts">
  import { goto } from "$app/navigation";
  import { Grid, Willow as GridTheme } from "@svar-ui/svelte-grid";
  import {
    FilterBar,
    Willow as FilterTheme,
    createArrayFilter,
  } from "@svar-ui/svelte-filter";
  import { listWorkers, money } from "#lib/api/wpm.js";
  import { i18n, t, l } from "#lib/i18n.svelte.js";
  import type { Worker } from "#lib/api/types.js";

  let workers = $state<Worker[] | null>(null);
  let error = $state<string | null>(null);

  $effect(() => {
    void (async () => {
      try {
        workers = await listWorkers();
      } catch (cause) {
        error = cause instanceof Error ? cause.message : String(cause);
      }
    })();
  });

  const columns = $derived([
    { id: "worker_number", header: t("emp.number"), width: 120 },
    { id: "display_name", header: t("common.name"), flexgrow: 1 },
    { id: "job_title", header: t("common.jobTitle"), width: 170 },
    { id: "department", header: t("common.department"), width: 140 },
    { id: "status", header: t("common.status"), width: 110 },
    { id: "salary", header: t("emp.salary"), width: 130 },
  ]);

  // Flatten workers to grid rows; a masked salary renders the
  // translated "Hidden" token, never a fake zero.
  const rows = $derived(
    (workers ?? []).map((e) => ({
      id: e.pid,
      worker_number: e.worker_number,
      display_name: e.display_name,
      job_title: e.job_title,
      department: e.department,
      status: e.status,
      salary:
        e.salary_minor === null
          ? t("common.masked")
          : money(e.salary_minor, e.salary_currency, i18n.locale),
    })),
  );

  const filterFields = $derived([
    { id: "worker_number", label: t("emp.number"), type: "text" },
    { id: "display_name", label: t("common.name"), type: "text" },
    { id: "job_title", label: t("common.jobTitle"), type: "text" },
    { id: "department", label: t("common.department"), type: "text" },
    { id: "status", label: t("common.status"), type: "text" },
  ]);
  let filterRules = $state<unknown>(null);
  const filtered = $derived(
    filterRules
      ? createArrayFilter(filterRules as Parameters<typeof createArrayFilter>[0])(rows)
      : rows,
  );

  // Row selection opens the worker profile.
  function initGrid(api: {
    on(action: string, cb: (ev: { id: string | number }) => void): void;
  }) {
    api.on("select-row", (ev) => {
      void goto(l(`/workers/${ev.id}`));
    });
  }
</script>

<h1>{t("nav.workers")}</h1>

{#if error}
  <p class="error" data-testid="error">{t("common.error")}: {error}</p>
{:else if workers === null}
  <p>{t("common.loading")}</p>
{:else}
  <div data-testid="worker-table">
    <GridTheme>
      <FilterTheme>
        <div class="filter-wrap">
          <FilterBar
            fields={filterFields}
            onchange={({ value }: { value: unknown }) => (filterRules = value)}
          />
        </div>
        <div class="grid-wrap">
          <Grid data={filtered} {columns} select init={initGrid} />
        </div>
      </FilterTheme>
    </GridTheme>
  </div>
{/if}

<style>
  .filter-wrap {
    margin-bottom: 0.5rem;
  }
  .grid-wrap {
    height: 480px;
    overflow: hidden;
  }
</style>
