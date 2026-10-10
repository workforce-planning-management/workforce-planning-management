<!--
  Mentorship area (`/mentorship`): the coaching overview — active
  pairings, mentor load (active mentees per mentor), unmatched active
  workers, and stale mentorships (no session within the window).
  All server-derived.
-->
<script lang="ts">
  import { mentorshipOverview } from "#lib/api/wpm.js";
  import { t, tf } from "#lib/i18n.svelte.js";

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
  <table data-testid="mentor-load">
    <thead><tr><th>{t("pages.mentorship.mentor")}</th><th>{t("pages.mentorship.active_mentees")}</th></tr></thead>
    <tbody>
      {#each overview.mentor_load as row (row.mentor_pid)}
        <tr><td>{row.mentor ?? row.mentor_pid}</td><td>{row.active_mentees}</td></tr>
      {:else}
        <tr><td colspan="2" class="muted">{t("pages.mentorship.no_active_mentorships")}</td></tr>
      {/each}
    </tbody>
  </table>

  {#if overview.stale_mentorships.length > 0}
    <h2>{t("pages.mentorship.stale_mentorships")}</h2>
    <table data-testid="stale-mentorships">
      <thead><tr><th>{t("pages.mentorship.mentor")}</th><th>{t("pages.mentorship.mentee")}</th><th>{t("pages.mentorship.last_session")}</th></tr></thead>
      <tbody>
        {#each overview.stale_mentorships as row (row.pid)}
          <tr>
            <td>{row.mentor ?? "—"}</td>
            <td>{row.mentee ?? "—"}</td>
            <td>{row.last_session ?? "never"}</td>
          </tr>
        {/each}
      </tbody>
    </table>
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
