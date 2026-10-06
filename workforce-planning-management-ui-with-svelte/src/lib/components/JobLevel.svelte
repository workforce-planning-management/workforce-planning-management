<!--
  A worker's job level on a published ladder (Google's L3–L11 today). Seniority is
  career-sensitive and tracks pay closely, so only the worker and HR can see or
  change it: the panel renders nothing for anyone else (the service refuses the
  read with 403). Nothing is derived: the level is what was recorded.
-->
<script lang="ts">
  import {
    clearWorkerJobLevel,
    getJobLevelFramework,
    getWorkerJobLevel,
    listJobLevelFrameworks,
    setWorkerJobLevel,
  } from "#lib/api/wpm.js";
  import { ApiError } from "#lib/api/client.js";
  import type { JobLevelFramework, WorkerJobLevel } from "#lib/api/types.js";
  import { t } from "#lib/i18n.svelte.js";

  let { workerPid }: { workerPid: string } = $props();

  let held = $state<WorkerJobLevel | null>(null);
  let ladder = $state<JobLevelFramework | null>(null);
  let allowed = $state(true);
  let error = $state<string | null>(null);
  let choice = $state("");
  let since = $state("");

  const message = (cause: unknown) => (cause instanceof Error ? cause.message : String(cause));

  async function load() {
    try {
      held = (await getWorkerJobLevel(workerPid)).job_level;
      allowed = true;
      const [first] = await listJobLevelFrameworks();
      ladder = first ? await getJobLevelFramework(first.id) : null;
    } catch (cause) {
      // Not the worker or HR: not an error, just not theirs to see.
      if (cause instanceof ApiError && (cause.status === 403 || cause.status === 401)) {
        allowed = false;
      } else {
        error = message(cause);
      }
    }
  }

  $effect(() => {
    void workerPid;
    void load();
  });

  async function run(action: () => Promise<unknown>) {
    error = null;
    try {
      await action();
      await load();
    } catch (cause) {
      error = message(cause);
    }
  }

  function save(event: SubmitEvent) {
    event.preventDefault();
    if (!ladder || !choice) return;
    const framework = ladder.id;
    void run(() =>
      setWorkerJobLevel(workerPid, {
        framework,
        level: choice,
        ...(since && { effective_on: since }),
      }),
    );
  }
</script>

{#if allowed}
  <section class="panel" data-testid="job-level">
    <h2>{t("grades.workerTitle")}</h2>
    <p class="muted">{t("grades.workerHint")}</p>
    {#if error}<p class="error" data-testid="error">{error}</p>{/if}
    {#if held?.level}
      <p data-testid="job-level-held">
        <strong>{held.level.code}</strong> — {held.level.title}
        <span class="muted">({held.framework_name}; {t("grades.since")} {held.effective_on})</span>
      </p>
      <button data-testid="job-level-clear" onclick={() => run(() => clearWorkerJobLevel(workerPid))}>
        {t("grades.clear")}
      </button>
    {:else}
      <p class="muted" data-testid="job-level-none">{t("grades.none")}</p>
    {/if}
    {#if ladder}
      <form onsubmit={save} data-testid="job-level-form">
        <label>
          {t("jobLevels.level")}
          <select data-testid="job-level-choice" bind:value={choice}>
            <option value="">—</option>
            {#each ladder.levels as level (level.code)}
              <option value={level.code}>{level.code} — {level.title}</option>
            {/each}
          </select>
        </label>
        <label>
          {t("grades.since")}
          <input type="date" data-testid="job-level-since" bind:value={since} />
        </label>
        <button type="submit" data-testid="job-level-save">{t("grades.save")}</button>
      </form>
    {/if}
  </section>
{/if}
