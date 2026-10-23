<!--
  Learning area (`/learning`): the per-department skills matrix + the
  declared-gap list, per-department training analytics (completion
  ratios carry numerator/denominator; cert-expiry counts), and
  learning-path progress (honest — a step counts only against a
  completed training enrolment). All server-derived; derivations shown.
-->
<script lang="ts">
  import { errorText } from "#lib/write.js";
  import ErrorNote from "#lib/components/ErrorNote.svelte";
  import {
    capabilityAnalysis,
    listPaths,
    pathProgress,
    skillsMatrix,
    trainingAnalytics,
  } from "#lib/api/wpm.js";
  import { percentOf, percentWithWorkings } from "#lib/format.js";
  import { t, tf, tv } from "#lib/i18n.svelte.js";
  import DataTable from "#lib/components/DataTable.svelte";
  import type { DataGridColumn, DataGridRow } from "@lilydesignsystem/svelte-data-grid";

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
        error = errorText(cause);
      }
    })();
  });

  async function loadCapability() {
    try {
      capability = await capabilityAnalysis({ minProficiency, minDepth });
    } catch (cause) {
      error = errorText(cause);
    }
  }

  async function loadProgress(pid: string) {
    selectedPath = pid;
    progress = null;
    if (pid) {
      try {
        progress = await pathProgress(pid);
      } catch (cause) {
        error = errorText(cause);
      }
    }
  }

  const matrixColumns = $derived<DataGridColumn[]>([
    { id: "department", pinned: "start", header: t("pages.learning.department"), sortable: true },
    { id: "skill", header: t("pages.learning.skill"), sortable: true },
    { id: "workers", header: t("pages.learning.workers"), sortable: true, align: "end" },
    {
      id: "average_proficiency",
      header: t("pages.learning.avg_proficiency"),
      sortable: true,
      align: "end",
      format: (v) => (typeof v === "number" ? v.toFixed(1) : "—"),
    },
    {
      id: "below_target",
      header: t("pages.learning.below_target"),
      sortable: true,
      align: "end",
      cellClass: ({ value }) => (typeof value === "number" && value > 0 ? "cx-warn" : ""),
    },
  ]);

  const analyticsColumns = $derived<DataGridColumn[]>([
    { id: "department", header: t("pages.learning.department"), sortable: true },
    {
      id: "completion_rate",
      header: t("pages.learning.completion"),
      sortable: true,
      align: "end",
      accessor: (r) => (r.completion_rate as { value: number | null } | null)?.value ?? null,
      format: (_v, r) => percentWithWorkings(r.completion_rate as Parameters<typeof percentWithWorkings>[0]),
      exportValue: (r) => percentWithWorkings(r.completion_rate as Parameters<typeof percentWithWorkings>[0]),
    },
    { id: "certs_expiring", header: t("pages.learning.certs_expiring"), sortable: true, align: "end" },
  ]);

  const capabilityColumns = $derived<DataGridColumn[]>([
    { id: "skill", pinned: "start", header: t("pages.learning.skill"), sortable: true },
    { id: "category", header: t("pages.learning.category"), sortable: true, format: (v) => tv(String(v)) },
    { id: "declared_by", header: t("pages.learning.declared"), sortable: true, align: "end" },
    { id: "proficient", header: t("pages.learning.proficient"), sortable: true, align: "end" },
    { id: "proficient_departments", header: t("pages.learning.departments"), sortable: true, align: "end" },
    {
      id: "status",
      header: t("pages.learning.status"),
      sortable: true,
      format: (v) => tv(String(v)),
      cellClass: ({ value }) => (value !== "adequate" ? "cx-warn" : ""),
    },
  ]);

  const progressColumns = $derived<DataGridColumn[]>([
    {
      id: "display_name",
      header: t("pages.learning.worker"),
      sortable: true,
      accessor: (r) => r.display_name ?? r.worker_pid,
    },
    {
      id: "completed_steps",
      header: t("pages.learning.completed"),
      sortable: true,
      align: "end",
      format: (_v, r) => `${r.completed_steps} / ${r.total_steps}`,
    },
    {
      id: "progress",
      header: t("pages.learning.progress"),
      sortable: true,
      align: "end",
      accessor: (r) => (r.total_steps === 0 ? null : (r.completed_steps as number) / (r.total_steps as number)),
      format: (_v, r) => percentOf(r.completed_steps as number, r.total_steps as number),
    },
  ]);
</script>

<svelte:head><title>{tf("pages.learning.page_title", { learning: t("nav.learning") })}</title></svelte:head>

<h1>{t("nav.learning")}</h1>
{#if error}<ErrorNote testid="error" message={error} />{/if}

{#if matrix}
  <h2>{t("pages.learning.skills_matrix")}</h2>
  <p class="muted">{matrix.note}</p>
  {#if matrix.matrix.length === 0}
    <p class="muted">{t("pages.learning.no_declared_skills_yet")}</p>
  {:else}
    <DataTable
      testid="skills-matrix"
      label={t("pages.learning.skills_matrix")}
      columns={matrixColumns}
      rows={matrix.matrix as unknown as DataGridRow[]}
      rowId={(row) => String(row.department) + String(row.skill)}
      pageSize={25}
    />
  {/if}
  {#if matrix.gaps.length > 0}
    <h3>{t("pages.learning.skill_gaps")}</h3>
    <ul data-testid="skills-gaps">
      {#each matrix.gaps as gap, index (index)}
        <li>{tf("pages.learning.in_target", { skill: gap.skill, department: gap.department, proficiency: gap.proficiency, target: gap.target })}</li>
      {/each}
    </ul>
  {/if}
{/if}

{#if analytics}
  <h2>{t("pages.learning.training_analytics")}</h2>
  <p class="muted">{tf("pages.learning.certs_expiring_by", { note: analytics.note, horizon: analytics.horizon })}</p>
  {#if analytics.departments.length === 0}
    <p class="muted">{t("pages.learning.no_training_enrolments_yet")}</p>
  {:else}
    <DataTable
      testid="training-analytics"
      exportName="training-analytics"
      label={t("pages.learning.training_analytics")}
      columns={analyticsColumns}
      rows={analytics.departments as unknown as DataGridRow[]}
      rowId={(row) => String(row.department)}
    />
  {/if}
{/if}

{#if capability}
  <h2>{t("pages.learning.capability_analysis")}</h2>
  <p class="muted">{capability.derivation}</p>
  <p>
    <label>
      {t("pages.learning.proficiency_bar")}
      <select
        data-testid="capability-min-proficiency"
        bind:value={minProficiency}
        onchange={() => void loadCapability()}
      >
        {#each [1, 2, 3, 4, 5] as level (level)}<option value={level}>{level}</option>{/each}
      </select>
    </label>
    <label>
      {t("pages.learning.workers_needed")}
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
    {tf("pages.learning.adequately_covered_skills", { adequately_covered: percentWithWorkings(capability.adequately_covered) })}
  </p>
  {#if capability.skills.length === 0}
    <p class="muted">{t("pages.learning.no_skills_in_the_catalogue_yet")}</p>
  {:else}
    <DataTable
      testid="capability-analysis"
      label={t("pages.learning.capability_analysis")}
      columns={capabilityColumns}
      rows={capability.skills as unknown as DataGridRow[]}
      rowId={(row) => String(row.skill) + String(row.category)}
      pageSize={25}
    />
  {/if}
{/if}

<h2>{t("pages.learning.learning_path_progress")}</h2>
{#if paths.length > 0}
  <p>
    <label>
      <select
        data-testid="path-select"
        value={selectedPath}
        onchange={(event) => void loadProgress(event.currentTarget.value)}
      >
        {#each paths as path (path.pid)}
          <option value={path.pid}>{tf("pages.learning.steps", { name: path.name, steps: path.steps })}</option>
        {/each}
      </select>
    </label>
  </p>
{:else}
  <p class="muted">{t("pages.learning.no_learning_paths_defined")}</p>
{/if}

{#if progress}
  <p class="muted">{progress.derivation}</p>
  {#if progress.members.length === 0}
    <p class="muted">{t("pages.learning.no_one_enrolled")}</p>
  {:else}
    <DataTable
      testid="path-progress"
      label={t("pages.learning.learning_path_progress")}
      columns={progressColumns}
      rows={progress.members as unknown as DataGridRow[]}
      rowId={(row) => String(row.worker_pid)}
      pageSize={25}
    />
  {/if}
{/if}

<style>
  :global(td.cx-warn) { color: #b45309; font-weight: 600; }
</style>
