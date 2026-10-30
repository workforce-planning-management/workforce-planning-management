<!--
  A worker's job level on a published ladder (Google's L3–L11 today). Seniority is
  career-sensitive and tracks pay closely, so only the worker and HR can see or
  change it: the panel renders nothing for anyone else (the service refuses the
  read with 403). Nothing is derived: the level is what was recorded.
-->
<script lang="ts">
  import { formatDate } from "#lib/locale-format.js";
  import { guardedLoad } from "#lib/loader.svelte.js";
  import { errorText } from "#lib/write.js";
  import ErrorNote from "#lib/components/ErrorNote.svelte";
  import {
    clearWorkerJobLevel,
    getJobLevelFramework,
    getWorkerJobLevel,
    listJobLevelFrameworks,
    setWorkerJobLevel,
  } from "#lib/api/wpm.js";
  import { ApiError } from "#lib/api/client.js";
  import type { JobLevelFramework, JobLevelFrameworkSummary, WorkerJobLevel } from "#lib/api/types.js";
  import { t } from "#lib/i18n.svelte.js";

  let { workerPid }: { workerPid: string } = $props();

  let held = $state<WorkerJobLevel | null>(null);
  let ladder = $state<JobLevelFramework | null>(null);
  let ladders = $state<JobLevelFrameworkSummary[]>([]);
  let allowed = $state(true);
  let error = $state<string | null>(null);
  let choice = $state("");
  let since = $state("");

  const message = (cause: unknown) => (errorText(cause));

  const load = guardedLoad(
    async () => {
      const level = (await getWorkerJobLevel(workerPid)).job_level;
      const list = await listJobLevelFrameworks();
      // Keep the ladder the person chose; otherwise the one their level is on, otherwise the first.
      const wanted = ladder?.id ?? level?.framework ?? list[0]?.id;
      const framework = wanted ? await getJobLevelFramework(wanted) : null;
      return { level, framework, list };
    },
    ({ level, framework, list }) => {
      held = level;
      allowed = true;
      ladder = framework;
      ladders = list;
    },
    (cause) => {
      // Not the worker or HR: not an error, just not theirs to see.
      if (cause instanceof ApiError && (cause.status === 403 || cause.status === 401)) {
        allowed = false;
      } else {
        error = message(cause);
      }
    },
  );

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

  async function chooseLadder(event: Event) {
    const id = (event.currentTarget as HTMLSelectElement).value;
    choice = "";
    try {
      ladder = await getJobLevelFramework(id);
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
    {#if error}<ErrorNote testid="error" message={error} />{/if}
    {#if held?.level}
      <p data-testid="job-level-held">
        <strong>{held.level.code}</strong> — {held.level.title}
        <span class="muted">({held.framework_name}; {t("grades.since")} {formatDate(held.effective_on)})</span>
      </p>
      <button data-testid="job-level-clear" onclick={() => run(() => clearWorkerJobLevel(workerPid))}>
        {t("grades.clear")}
      </button>
    {:else}
      <p class="muted" data-testid="job-level-none">{t("grades.none")}</p>
    {/if}
    {#if ladder}
      <form class="cx-form" onsubmit={save} data-testid="job-level-form">
        {#if ladders.length > 1}
          <label>
            {t("jobLevels.framework")}
            <select data-testid="job-level-framework" value={ladder.id} onchange={chooseLadder}>
              {#each ladders as f (f.id)}<option value={f.id}>{f.name}</option>{/each}
            </select>
          </label>
        {/if}
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

<style>
  .cx-form {
    display: flex;
    flex-wrap: wrap;
    gap: 0.75rem 1rem;
    align-items: flex-end;
    margin-top: 0.75rem;
  }
  .cx-form label {
    display: flex;
    flex-direction: column;
    gap: 0.25rem;
    min-width: 0;
    max-width: 100%;
  }
  .cx-form select {
    max-width: 100%;
  }
</style>
