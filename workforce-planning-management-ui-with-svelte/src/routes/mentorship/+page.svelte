<!--
  Mentorship area (`/mentorship`): the coaching overview — active
  pairings, mentor load (active mentees per mentor), unmatched active
  workers, and stale mentorships (no session within the window).
  All server-derived.
-->
<script lang="ts">
  import { mentorshipOverview } from "#lib/api/wpm.js";
  import { t, tf } from "#lib/i18n.svelte.js";
  import DataTable from "#lib/components/DataTable.svelte";
  import type {
    DataGridColumn,
    DataGridRow,
  } from "@lilydesignsystem/svelte-data-grid";

  type Overview = Awaited<ReturnType<typeof mentorshipOverview>>;
  let overview = $state<Overview | null>(null);
  let error = $state<string | null>(null);

  $effect(() => {
    void (async () => {
      try {
        overview = await mentorshipOverview();
      } catch (cause) {
        error = cause instanceof Error ? cause.message : String(cause);
      }
    })();
  });

  const loadColumns = $derived<DataGridColumn[]>([
    { id: "mentor", header: t("pages.mentorship.mentor"), sortable: true, rowHeader: true, accessor: (r) => r.mentor ?? r.mentor_pid },
    { id: "active_mentees", header: t("pages.mentorship.active_mentees"), sortable: true, align: "end" },
  ]);
  const staleColumns = $derived<DataGridColumn[]>([
    { id: "mentor", header: t("pages.mentorship.mentor"), sortable: true },
    { id: "mentee", header: t("pages.mentorship.mentee"), sortable: true },
    { id: "last_session", header: t("pages.mentorship.last_session"), sortable: true, accessor: (r) => r.last_session ?? "never" },
  ]);
</script>

<svelte:head><title>{tf("pages.mentorship.page_title", { mentorship: t("nav.mentorship") })}</title></svelte:head>

<h1>{t("nav.mentorship")}</h1>
{#if error}<p class="error" data-testid="error">{error}</p>{/if}

{#if overview}
  <section class="tiles" data-testid="mentorship-tiles">
    <div class="tile"><strong>{overview.active_pairings}</strong><span>{t("pages.mentorship.active_pairings")}</span></div>
    <div class="tile"><strong>{overview.unmatched_workers.length}</strong><span>{t("pages.mentorship.unmatched")}</span></div>
    <div class="tile">
      <strong>{overview.stale_mentorships.length}</strong>
      <span>{tf("pages.mentorship.stale_over_d", { stale_days: overview.stale_days })}</span>
    </div>
  </section>

  <h2>{t("pages.mentorship.mentor_load")}</h2>
  {#if overview.mentor_load.length === 0}
    <p class="muted" data-testid="mentor-load">{t("pages.mentorship.no_active_mentorships")}</p>
  {:else}
    <DataTable
      testid="mentor-load"
      label={t("pages.mentorship.mentor_load")}
      columns={loadColumns}
      rows={overview.mentor_load as unknown as DataGridRow[]}
      rowId={(row) => String(row.mentor_pid)}
      pageSize={25}
    />
  {/if}

  {#if overview.stale_mentorships.length > 0}
    <h2>{t("pages.mentorship.stale_mentorships")}</h2>
    <DataTable
      testid="stale-mentorships"
      label={t("pages.mentorship.stale_mentorships")}
      columns={staleColumns}
      rows={overview.stale_mentorships as unknown as DataGridRow[]}
      rowId={(row) => String(row.pid)}
      pageSize={25}
    />
  {/if}

  <h2>{t("pages.mentorship.unmatched_workers")}</h2>
  <ul data-testid="unmatched">
    {#each overview.unmatched_workers as worker (worker.pid)}
      <li>{worker.display_name} <span class="muted">({worker.department})</span></li>
    {:else}
      <li class="muted">{t("pages.mentorship.everyone_active_is_in_a_mentorship")}</li>
    {/each}
  </ul>
{/if}
