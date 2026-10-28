<!--
  A worker's declared skills, and declaring one (WPM-R111). A level can be stated on WPM's 1–5 or on SFIA's seven levels:
  SFIA's level is kept as stated, WPM counts it on 1–5 by the stated rule, and the framework's own wording for that
  level is shown beside it (a deployer's licensed content, so only to a signed-in caller). Nothing is worked out beyond
  that mapping. The panel renders nothing for a caller who may not read the worker's skills.
-->
<script lang="ts">
  import { guardedLoad } from "#lib/loader.svelte.js";
  import { errorText } from "#lib/write.js";
  import ErrorNote from "#lib/components/ErrorNote.svelte";
  import { ApiError } from "#lib/api/client.js";
  import {
    declareSkill,
    listFrameworks,
    listSkills,
    listWorkerSkills,
  } from "#lib/api/wpm.js";
  import { t, tf } from "#lib/i18n.svelte.js";

  let { workerPid }: { workerPid: string } = $props();

  type Declared = Awaited<ReturnType<typeof listWorkerSkills>>;
  type Skills = Awaited<ReturnType<typeof listSkills>>;

  let declared = $state<Declared>([]);
  let skills = $state<Skills>([]);
  let sfia = $state(false);
  let allowed = $state(true);
  let error = $state<string | null>(null);
  let skill = $state("");
  let scale = $state<"wpm" | "sfia">("wpm");
  let level = $state(3);

  const nameOf = (pid: string) => skills.find((s) => s.pid === pid)?.name ?? pid;
  const max = $derived(scale === "sfia" ? 7 : 5);

  const load = guardedLoad(
    async () => {
      const [rows, catalogue, frameworks] = await Promise.all([
        listWorkerSkills(workerPid),
        listSkills(),
        listFrameworks(),
      ]);
      return { rows, catalogue, hasSfia: frameworks.some((f) => f.slug === "sfia") };
    },
    ({ rows, catalogue, hasSfia }) => {
      declared = rows;
      skills = catalogue;
      sfia = hasSfia;
      allowed = true;
    },
    (cause) => {
      // Not theirs to see: not an error.
      if (cause instanceof ApiError && (cause.status === 403 || cause.status === 401)) allowed = false;
      else error = errorText(cause);
    },
  );

  $effect(() => {
    void workerPid;
    void load();
  });

  async function save(event: SubmitEvent) {
    event.preventDefault();
    if (!skill) return;
    error = null;
    try {
      await declareSkill(
        workerPid,
        scale === "sfia"
          ? { skill_pid: skill, framework: "sfia", framework_level: level }
          : { skill_pid: skill, proficiency: level },
      );
      await load();
    } catch (cause) {
      error = errorText(cause);
    }
  }

  function chooseScale(next: "wpm" | "sfia") {
    scale = next;
    level = Math.min(level, next === "sfia" ? 7 : 5);
  }
</script>

{#if allowed}
  <section class="panel" data-testid="declared-skills">
    <h2>{t("declaredSkills.title")}</h2>
    <p class="muted">{t("declaredSkills.hint")}</p>
    {#if error}<ErrorNote testid="error" message={error} />{/if}
    {#if declared.length === 0}
      <p class="muted" data-testid="declared-none">{t("declaredSkills.none")}</p>
    {:else}
      <ul data-testid="declared-list">
        {#each declared as d (d.skill_pid)}
          <li data-testid={`declared-${d.skill_pid}`}>
            <strong>{nameOf(d.skill_pid)}</strong> — {d.proficiency} / 5
            {#if d.source_level !== null && d.source_framework}
              <span class="muted">
                · {tf("pages.roles.native_level_counted", {
                  level: d.source_level,
                  scale: d.source_scale_max ?? "",
                  framework: d.source_framework_name ?? d.source_framework,
                  wpm: d.proficiency,
                  mapping: d.mapping ?? "",
                })}{d.native_description ? `: ${d.native_description}` : ""}
              </span>
            {/if}
          </li>
        {/each}
      </ul>
    {/if}
    <form class="cx-form" onsubmit={save} data-testid="declared-form">
      <label>
        {t("pages.roles.skill")}
        <select data-testid="declared-skill" bind:value={skill}>
          <option value="">—</option>
          {#each skills as s (s.pid)}<option value={s.pid}>{s.name}</option>{/each}
        </select>
      </label>
      {#if sfia}
        <label>
          {t("declaredSkills.scale")}
          <select
            data-testid="declared-scale"
            value={scale}
            onchange={(event) => chooseScale(event.currentTarget.value === "sfia" ? "sfia" : "wpm")}
          >
            <option value="wpm">{t("declaredSkills.scaleWpm")}</option>
            <option value="sfia">{t("declaredSkills.scaleSfia")}</option>
          </select>
        </label>
      {/if}
      <label>
        {t("jobLevels.level")}
        <select data-testid="declared-level" bind:value={level}>
          {#each Array.from({ length: max }, (_, i) => i + 1) as n (n)}<option value={n}>{n}</option>{/each}
        </select>
      </label>
      <button type="submit" data-testid="declared-save">{t("grades.save")}</button>
    </form>
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
</style>
