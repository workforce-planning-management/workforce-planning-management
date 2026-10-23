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
  import type { JobLevelFramework } from "#lib/api/types.js";
  import DataTable from "#lib/components/DataTable.svelte";
  import type {
    DataGridColumn,
    DataGridRow,
  } from "@lilydesignsystem/svelte-data-grid";

  const columns = $derived<DataGridColumn[]>([
    { id: "code", header: t("jobLevels.level"), sortable: true, rowHeader: true },
    { id: "title", header: t("jobLevels.title"), sortable: true },
    { id: "summary", header: t("jobLevels.summary") },
    { id: "experience", header: t("jobLevels.experience"), sortable: true },
    { id: "management_equivalent", header: t("jobLevels.management"), sortable: true },
  ]);

  let framework = $state<JobLevelFramework | null>(null);
  let error = $state<string | null>(null);

  $effect(() => {
    let stale = false;
    void (async () => {
      try {
        const [first] = await listJobLevelFrameworks();
        const loaded = first ? await getJobLevelFramework(first.id) : null;
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

{#if framework}
  <p data-testid="levels-source">
    <strong>{framework.name}</strong>. {t("jobLevels.source")}: {framework.source}
  </p>
  <p class="muted">{t("jobLevels.noPay")}</p>

  <DataTable
    testid="levels-table"
    label={t("nav.jobLevels")}
    columns={columns}
    rows={framework.levels as unknown as DataGridRow[]}
    rowId={(row) => String(row.code)}
    rowAttrs={(row) => ({ "data-testid": `level-${row.code}` })}
  />
{:else if !error}
  <p>{t("common.loading")}</p>
{/if}
