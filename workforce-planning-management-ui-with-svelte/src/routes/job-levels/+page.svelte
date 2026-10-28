<!--
  Job levels (`/job-levels`): a published ladder of levels — Google's technical
  levels L3–L11 — with what each level is, typical experience and its management
  equivalent. Reference data with no pay; a field the source does not state shows
  "—", never a guess.
-->
<script lang="ts">
  import { errorText } from "#lib/write.js";
  import ErrorNote from "#lib/components/ErrorNote.svelte";
  import { getJobLevelFramework, listJobLevelFrameworks } from "#lib/api/wpm.js";
  import { t, tf } from "#lib/i18n.svelte.js";
  import type { JobLevelFramework, JobLevelFrameworkSummary } from "#lib/api/types.js";
  import DataTable from "#lib/components/DataTable.svelte";
  import type {
    DataGridColumn,
    DataGridRow,
  } from "@lilydesignsystem/svelte-data-grid";

  let frameworks = $state<JobLevelFrameworkSummary[]>([]);
  let chosen = $state("");
  let framework = $state<JobLevelFramework | null>(null);
  let error = $state<string | null>(null);

  // An imported ladder (SFIA's levels of responsibility) carries named attributes; a coded one has none.
  const hasAttributes = $derived(framework?.levels.some((l) => (l.attributes ?? []).length > 0) ?? false);
  const rows = $derived(
    (framework?.levels ?? []).map((l) => ({
      ...l,
      attributes: (l.attributes ?? []).map((a) => `${a.name.replaceAll("_", " ")}: ${a.text}`).join(" · "),
    })),
  );

  const columns = $derived<DataGridColumn[]>([
    { id: "code", header: t("jobLevels.level"), sortable: true, rowHeader: true },
    { id: "title", header: t("jobLevels.title"), sortable: true },
    { id: "summary", header: t("jobLevels.summary") },
    { id: "experience", header: t("jobLevels.experience"), sortable: true },
    { id: "management_equivalent", header: t("jobLevels.management"), sortable: true },
    ...(hasAttributes
      ? [{ id: "attributes", header: t("jobLevels.attributes") } satisfies DataGridColumn]
      : []),
  ]);

  $effect(() => {
    let stale = false;
    void (async () => {
      try {
        const list = await listJobLevelFrameworks();
        if (!stale) {
          frameworks = list;
          chosen = chosen || list[0]?.id || "";
        }
      } catch (cause) {
        if (!stale) error = errorText(cause);
      }
    })();
    return () => {
      stale = true;
    };
  });

  $effect(() => {
    const id = chosen;
    if (!id) return;
    let stale = false;
    void (async () => {
      try {
        const loaded = await getJobLevelFramework(id);
        if (!stale) framework = loaded;
      } catch (cause) {
        if (!stale) error = errorText(cause);
      }
    })();
    return () => {
      stale = true;
    };
  });
</script>

<svelte:head><title>{tf("pages.jobLevels.page_title", { jobLevels: t("nav.jobLevels") })}</title></svelte:head>

<h1>{t("nav.jobLevels")}</h1>
{#if error}<ErrorNote testid="error" prefixed message={error} />{/if}

{#if frameworks.length > 1}
  <label>
    {t("jobLevels.framework")}
    <select data-testid="levels-framework" bind:value={chosen}>
      {#each frameworks as f (f.id)}<option value={f.id}>{f.name}</option>{/each}
    </select>
  </label>
{/if}

{#if framework}
  <p data-testid="levels-source">
    <strong>{framework.name}</strong>. {t("jobLevels.source")}: {framework.source}
  </p>
  <p class="muted">{t("jobLevels.noPay")}</p>

  <DataTable
    testid="levels-table"
    label={t("nav.jobLevels")}
    columns={columns}
    rows={rows as unknown as DataGridRow[]}
    rowId={(row) => String(row.code)}
    rowAttrs={(row) => ({ "data-testid": `level-${row.code}` })}
  />
{:else if !error}
  <p>{t("common.loading")}</p>
{/if}
