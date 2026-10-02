<!--
  Learning area (`/learning`): the per-department skills matrix + the
  declared-gap list, per-department training analytics (completion
  ratios carry numerator/denominator; cert-expiry counts), and
  learning-path progress (honest — a step counts only against a
  completed training enrolment). All server-derived; derivations shown.
-->
<script lang="ts">
  import {
    capabilityAnalysis,
    listPaths,
    pathProgress,
    skillsMatrix,
    trainingAnalytics,
  } from "#lib/api/wpm.js";
  import { percentOf, percentWithWorkings } from "#lib/format.js";
  import { t } from "#lib/i18n.svelte.js";

  type Matrix = Awaited<ReturnType<typeof skillsMatrix>>;
  type Analytics = Awaited<ReturnType<typeof trainingAnalytics>>;
  type Capability = Awaited<ReturnType<typeof capabilityAnalysis>>;
  type Paths = Awaited<ReturnType<typeof listPaths>>;
  type Progress = Awaited<ReturnType<typeof pathProgress>>;

  let matrix = $state<Matrix | null>(null);
  let analytics = $state<Analytics | null>(null);
  let capability = $state<Capability | null>(null);
  let minProficiency = $state(3);
  let minDepth = $state(2);
  let paths = $state<Paths>([]);
  let selectedPath = $state("");
  let progress = $state<Progress | null>(null);
  let error = $state<string | null>(null);

  $effect(() => {
    void (async () => {
      try {
        matrix = await skillsMatrix();
        analytics = await trainingAnalytics();
        await loadCapability();
        paths = await listPaths();
        if (!selectedPath && paths.length > 0) {
          selectedPath = paths[0]?.pid ?? "";
        }
        if (selectedPath) progress = await pathProgress(selectedPath);
      } catch (cause) {
        error = cause instanceof Error ? cause.message : String(cause);
      }
    })();
  });

  async function loadCapability() {
    try {
      capability = await capabilityAnalysis({ minProficiency, minDepth });
    } catch (cause) {
      error = cause instanceof Error ? cause.message : String(cause);
    }
  }

  async function loadProgress(pid: string) {
    selectedPath = pid;
    progress = null;
    if (pid) {
      try {
        progress = await pathProgress(pid);
      } catch (cause) {
        error = cause instanceof Error ? cause.message : String(cause);
      }
    }
  }

</script>

<svelte:head><title>{t("nav.learning")} — WPM</title></svelte:head>

<h1>{t("nav.learning")}</h1>
{#if error}<p class="error" data-testid="error">{error}</p>{/if}

{#if matrix}
  <h2>Skills matrix</h2>
  <p class="muted">{matrix.note}</p>
  <table data-testid="skills-matrix">
    <thead>
      <tr><th>Department</th><th>Skill</th><th>Workers</th><th>Avg proficiency</th><th>Below target</th></tr>
    </thead>
    <tbody>
      {#each matrix.matrix as cell (cell.department + cell.skill)}
        <tr>
          <td>{cell.department}</td>
          <td>{cell.skill ?? "—"}</td>
          <td>{cell.workers}</td>
          <td>{cell.average_proficiency.toFixed(1)}</td>
          <td class:warn={cell.below_target > 0}>{cell.below_target}</td>
        </tr>
      {:else}
        <tr><td colspan="5" class="muted">No declared skills yet.</td></tr>
      {/each}
    </tbody>
  </table>
  {#if matrix.gaps.length > 0}
    <h3>Skill gaps</h3>
    <ul data-testid="skills-gaps">
      {#each matrix.gaps as gap, index (index)}
        <li>{gap.skill} in {gap.department}: {gap.proficiency} → target {gap.target}</li>
      {/each}
    </ul>
  {/if}
{/if}

{#if analytics}
  <h2>Training analytics</h2>
  <p class="muted">{analytics.note} · certs expiring by {analytics.horizon}</p>
  <table data-testid="training-analytics">
    <thead>
      <tr><th>Department</th><th>Completion</th><th>Certs expiring</th></tr>
    </thead>
    <tbody>
      {#each analytics.departments as dept (dept.department)}
        <tr>
          <td>{dept.department}</td>
          <td>{percentWithWorkings(dept.completion_rate)}</td>
          <td>{dept.certs_expiring}</td>
        </tr>
      {:else}
        <tr><td colspan="3" class="muted">No training enrolments yet.</td></tr>
      {/each}
    </tbody>
  </table>
{/if}

{#if capability}
  <h2>Capability analysis</h2>
  <p class="muted">{capability.derivation}</p>
  <p>
    <label>
      Proficiency bar
      <select
        data-testid="capability-min-proficiency"
        bind:value={minProficiency}
        onchange={() => void loadCapability()}
      >
        {#each [1, 2, 3, 4, 5] as level (level)}<option value={level}>{level}</option>{/each}
      </select>
    </label>
    <label>
      Workers needed
      <select
        data-testid="capability-min-depth"
        bind:value={minDepth}
        onchange={() => void loadCapability()}
      >
        {#each [1, 2, 3, 4, 5] as depth (depth)}<option value={depth}>{depth}</option>{/each}
      </select>
    </label>
  </p>
  <p data-testid="capability-summary">
    Adequately covered skills: {percentWithWorkings(capability.adequately_covered)}
  </p>
  <table data-testid="capability-analysis">
    <thead>
      <tr><th>Skill</th><th>Category</th><th>Declared</th><th>Proficient</th><th>Departments</th><th>Status</th></tr>
    </thead>
    <tbody>
      {#each capability.skills as row (row.skill + row.category)}
        <tr>
          <td>{row.skill}</td>
          <td>{row.category}</td>
          <td>{row.declared_by}</td>
          <td>{row.proficient}</td>
          <td>{row.proficient_departments}</td>
          <td class:warn={row.status !== "adequate"}>{row.status.replace("_", " ")}</td>
        </tr>
      {:else}
        <tr><td colspan="6" class="muted">No skills in the catalogue yet.</td></tr>
      {/each}
    </tbody>
  </table>
{/if}

<h2>Learning path progress</h2>
{#if paths.length > 0}
  <p>
    <label>
      <select
        data-testid="path-select"
        value={selectedPath}
        onchange={(event) => void loadProgress(event.currentTarget.value)}
      >
        {#each paths as path (path.pid)}
          <option value={path.pid}>{path.name} ({path.steps} steps)</option>
        {/each}
      </select>
    </label>
  </p>
{:else}
  <p class="muted">No learning paths defined.</p>
{/if}

{#if progress}
  <p class="muted">{progress.derivation}</p>
  <table data-testid="path-progress">
    <thead><tr><th>Worker</th><th>Completed</th><th>Progress</th></tr></thead>
    <tbody>
      {#each progress.members as member (member.worker_pid)}
        <tr>
          <td>{member.display_name ?? member.worker_pid}</td>
          <td>{member.completed_steps} / {member.total_steps}</td>
          <td>{percentOf(member.completed_steps, member.total_steps)}</td>
        </tr>
      {:else}
        <tr><td colspan="3" class="muted">No one enrolled.</td></tr>
      {/each}
    </tbody>
  </table>
{/if}

<style>
  td.warn { color: #b45309; font-weight: 600; }
</style>
