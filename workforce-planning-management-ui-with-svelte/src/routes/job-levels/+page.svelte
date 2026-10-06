<!--
  Job levels (`/job-levels`): a published ladder of levels — Google's technical
  levels L3–L11 — with what each level is, typical experience and its management
  equivalent. Reference data with no pay; a field the source does not state shows
  "—", never a guess.
-->
<script lang="ts">
  import { getJobLevelFramework, listJobLevelFrameworks } from "#lib/api/wpm.js";
  import { t } from "#lib/i18n.svelte.js";
  import type { JobLevelFramework } from "#lib/api/types.js";

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
        if (!stale) error = cause instanceof Error ? cause.message : String(cause);
      }
    })();
    return () => {
      stale = true;
    };
  });
</script>

<svelte:head><title>{t("nav.jobLevels")} — WPM</title></svelte:head>

<h1>{t("nav.jobLevels")}</h1>
{#if error}<p class="error" data-testid="error">{t("common.error")}: {error}</p>{/if}

{#if framework}
  <p data-testid="levels-source">
    <strong>{framework.name}</strong>. {t("jobLevels.source")}: {framework.source}
  </p>
  <p class="muted">{t("jobLevels.noPay")}</p>

  <table data-testid="levels-table">
    <thead>
      <tr>
        <th>{t("jobLevels.level")}</th>
        <th>{t("jobLevels.title")}</th>
        <th>{t("jobLevels.summary")}</th>
        <th>{t("jobLevels.experience")}</th>
        <th>{t("jobLevels.management")}</th>
      </tr>
    </thead>
    <tbody>
      {#each framework.levels as level (level.code)}
        <tr data-testid={`level-${level.code}`}>
          <th scope="row">{level.code}</th>
          <td>{level.title}</td>
          <td>{level.summary}</td>
          <td>{level.experience ?? "—"}</td>
          <td>{level.management_equivalent ?? "—"}</td>
        </tr>
      {/each}
    </tbody>
  </table>
{:else if !error}
  <p>{t("common.loading")}</p>
{/if}
