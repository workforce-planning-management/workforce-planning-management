<!--
  Employee directory (`/directory`): who works where. Employed workers in
  the organizations the caller can read, searchable by name, title,
  department, location or manager. Server-derived and deliberately narrow:
  no pay, dates or person reference ever reach this page.
-->
<script lang="ts">
  import { employeeDirectory } from "#lib/api/wpm.js";
  import { t, l } from "#lib/i18n.svelte.js";
  import type { DirectoryEntry } from "#lib/api/types.js";

  /** Most rows asked for at once; the search narrows beyond that. */
  const LIMIT = 100;

  let query = $state("");
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
          error = cause instanceof Error ? cause.message : String(cause);
        }
      })();
    }, 200);
    return () => clearTimeout(timer);
  });
</script>

<svelte:head><title>{t("nav.directory")} — WPM</title></svelte:head>

<h1>{t("nav.directory")}</h1>
{#if error}<p class="error" data-testid="error">{t("common.error")}: {error}</p>{/if}

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
  <table data-testid="directory-table">
    <thead>
      <tr>
        <th>{t("common.name")}</th>
        <th>{t("common.jobTitle")}</th>
        <th>{t("common.department")}</th>
        <th>{t("directory.location")}</th>
        <th>{t("directory.manager")}</th>
      </tr>
    </thead>
    <tbody>
      {#each entries as e (e.pid)}
        <tr>
          <td>
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
          </td>
          <td>{e.job_title}</td>
          <td>{e.department}</td>
          <td>{e.location ?? t("org.noLocation")}</td>
          <td>{e.manager_name ?? "—"}</td>
        </tr>
      {/each}
    </tbody>
  </table>
{/if}
