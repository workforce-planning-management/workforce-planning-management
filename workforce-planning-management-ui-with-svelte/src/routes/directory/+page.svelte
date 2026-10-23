<!--
  Employee directory (`/directory`): who works where. Employed workers in
  the organizations the caller can read, searchable by name, title,
  department, location or manager. Server-derived and deliberately narrow:
  no pay, dates or person reference ever reach this page.
-->
<script lang="ts">
  import { errorText } from "#lib/write.js";
  import ErrorNote from "#lib/components/ErrorNote.svelte";
  import { page } from "$app/state";
  import { employeeDirectory } from "#lib/api/wpm.js";
  import { t, l, tf } from "#lib/i18n.svelte.js";
  import type { DirectoryEntry } from "#lib/api/types.js";
  import DataTable from "#lib/components/DataTable.svelte";
  import type {
    DataGridColumn,
    DataGridRow,
  } from "@lilydesignsystem/svelte-data-grid";

  /** Most rows asked for at once; the search narrows beyond that. */
  const LIMIT = 100;

  // The header's search picker sends people here with `?q=`; typing in the box below takes over.
  let query = $state(page.url.searchParams.get("q") ?? "");
  $effect(() => {
    const asked = page.url.searchParams.get("q");
    if (asked !== null) query = asked;
  });
  let department = $state("");
  let entries = $state<DirectoryEntry[] | null>(null);
  // Departments seen so far, so choosing one doesn't empty the picker.
  let departments = $state<string[]>([]);
  let error = $state<string | null>(null);

  $effect(() => {
    const q = query.trim();
    const dept = department;
    // Debounce typing: one request per pause, the latest wins.
    const timer = setTimeout(() => {
      void (async () => {
        try {
          const found = await employeeDirectory({
            q,
            department: dept,
            limit: LIMIT,
          });
          entries = found;
          error = null;
          departments = [
            ...new Set([...departments, ...found.map((e) => e.department)]),
          ].sort((a, b) => a.localeCompare(b));
        } catch (cause) {
          error = errorText(cause);
        }
      })();
    }, 200);
    return () => clearTimeout(timer);
  });

  const columns = $derived<DataGridColumn[]>([
    { id: "display_name", header: t("common.name"), sortable: true, cell: nameCell },
    { id: "job_title", header: t("common.jobTitle"), sortable: true },
    { id: "department", header: t("common.department"), sortable: true },
    {
      id: "location",
      header: t("directory.location"),
      sortable: true,
      format: (v) => (v == null ? t("org.noLocation") : String(v)),
    },
    { id: "manager_name", header: t("directory.manager"), sortable: true },
  ]);
</script>

{#snippet nameCell({ row }: { row: DataGridRow })}
  {@const e = row as unknown as DirectoryEntry}
  <a href={l(`/workers/${e.pid}`)}>{e.display_name}</a>
  {#each e.on_call as rota (rota)}
    <span class="chip" data-testid="on-call" title={rota}>{t("rota.who")}: {rota}</span>
  {/each}
  {#if e.away_today}
    <span class="chip" data-testid="away">{t("directory.away")}</span>
    <span class="muted" data-testid="covered-by">
      {#if e.covered_by}{t("directory.coveredBy")} {e.covered_by}{:else}{t("directory.noCover")}{/if}
    </span>
  {/if}
{/snippet}

<svelte:head><title>{tf("pages.directory.page_title", { directory: t("nav.directory") })}</title></svelte:head>

<h1>{t("nav.directory")}</h1>
{#if error}<ErrorNote testid="error" prefixed message={error} />{/if}

<p>
  <input
    type="search"
    data-testid="directory-search"
    placeholder={t("directory.search")}
    aria-label={t("directory.search")}
    bind:value={query}
  />
  <select
    data-testid="directory-department"
    aria-label={t("common.department")}
    bind:value={department}
  >
    <option value="">{t("directory.allDepartments")}</option>
    {#each departments as name (name)}
      <option value={name}>{name}</option>
    {/each}
  </select>
</p>

{#if entries === null}
  {#if !error}<p>{t("common.loading")}</p>{/if}
{:else if entries.length === 0}
  <p class="muted" data-testid="directory-none">{t("directory.noMatches")}</p>
{:else}
  <DataTable
    testid="directory-table"
    label={t("nav.directory")}
    columns={columns}
    rows={entries as unknown as DataGridRow[]}
    rowId={(row) => String(row.pid)}
  />
{/if}
