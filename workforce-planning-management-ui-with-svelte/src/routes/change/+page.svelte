<!--
  AI-driven change (`/change`): an automation or AI initiative, the roles it
  displaces, reshapes, or creates, the skills whose demand it moves, and how
  ready the affected workforce is. It tracks roles and skills, never people —
  readiness is aggregate and no individual is named.
-->
<script lang="ts">
  import {
    CHANGE_IMPACTS,
    CHANGE_KINDS,
    CHANGE_STATUSES_NEXT,
    CHANGE_TIMEFRAMES,
    changeReadiness,
    createChangeInitiative,
    getChangeInitiative,
    listChangeInitiatives,
    listRoleProfiles,
    listSkills,
    setChangeStatus,
    setRoleImpact,
    setSkillShift,
  } from "#lib/api/wpm.js";
  import { percentWithWorkings } from "#lib/format.js";
  import { t } from "#lib/i18n.svelte.js";

  type Initiatives = Awaited<ReturnType<typeof listChangeInitiatives>>;
  type Initiative = Awaited<ReturnType<typeof getChangeInitiative>>;
  type Readiness = Awaited<ReturnType<typeof changeReadiness>>;

  let initiatives = $state<Initiatives>([]);
  let selected = $state("");
  let initiative = $state<Initiative | null>(null);
  let readiness = $state<Readiness | null>(null);
  let roles = $state<Awaited<ReturnType<typeof listRoleProfiles>>>([]);
  let skills = $state<Awaited<ReturnType<typeof listSkills>>>([]);
  let error = $state<string | null>(null);

  let newName = $state("");
  let newKind = $state<string>("ai_assistance");
  let impactRole = $state("");
  let impact = $state<string>("reshaped");
  let timeframe = $state<string>("within_1y");
  let shiftSkill = $state("");
  let direction = $state<string>("rising");

  const message = (cause: unknown) =>
    cause instanceof Error ? cause.message : String(cause);

  async function loadDetail() {
    initiative = null;
    readiness = null;
    if (!selected) return;
    try {
      [initiative, readiness] = await Promise.all([
        getChangeInitiative(selected),
        changeReadiness(selected),
      ]);
    } catch (cause) {
      error = message(cause);
    }
  }

  async function loadAll() {
    try {
      initiatives = await listChangeInitiatives();
      if (!selected && initiatives.length > 0) selected = initiatives[0]?.pid ?? "";
      await loadDetail();
    } catch (cause) {
      error = message(cause);
    }
  }

  $effect(() => {
    void (async () => {
      try {
        [roles, skills] = await Promise.all([listRoleProfiles(), listSkills()]);
      } catch (cause) {
        error = message(cause);
      }
      await loadAll();
    })();
  });

  async function run(action: () => Promise<unknown>) {
    error = null;
    try {
      await action();
      await loadAll();
    } catch (cause) {
      error = message(cause);
    }
  }
</script>

<svelte:head><title>{t("nav.change")} — WPM</title></svelte:head>

<h1>{t("nav.change")}</h1>
{#if error}<p class="error" data-testid="error">{error}</p>{/if}
<p class="muted">
  Track an automation or AI initiative's effect on roles and skills. Readiness is
  aggregate; no individual is named or ranked.
</p>

<p>
  <label>
    Initiative
    <select
      data-testid="change-select"
      bind:value={selected}
      onchange={() => void loadDetail()}
    >
      {#each initiatives as i (i.pid)}
        <option value={i.pid}>{i.name} ({i.status})</option>
      {:else}
        <option value="">None yet</option>
      {/each}
    </select>
  </label>
</p>
<form
  onsubmit={(event) => {
    event.preventDefault();
    void run(async () => {
      const created = await createChangeInitiative({ name: newName, kind: newKind });
      newName = "";
      selected = created.pid;
    });
  }}
>
  <label>New initiative <input data-testid="change-new-name" bind:value={newName} required /></label>
  <label>
    Kind
    <select bind:value={newKind}>{#each CHANGE_KINDS as k (k)}<option value={k}>{k}</option>{/each}</select>
  </label>
  <button type="submit" data-testid="change-create">Create</button>
</form>

{#if initiative}
  <h2>{initiative.name} <span class="chip">{initiative.status}</span></h2>
  {#each CHANGE_STATUSES_NEXT[initiative.status] ?? [] as next (next)}
    <button type="button" onclick={() => void run(() => setChangeStatus(initiative!.pid, next))}>
      Mark {next}
    </button>
  {/each}

  {#if readiness}
    <p class="muted">{readiness.derivation}</p>
    <p data-testid="change-summary">
      Affected workers: {readiness.affected_workers} · with an active reskill plan:
      {percentWithWorkings(readiness.reskill_plan_coverage)}
    </p>
    <h3>Roles</h3>
    <table data-testid="change-roles">
      <thead><tr><th>Role</th><th>Impact</th><th>When</th><th>Employed</th><th>Reskill plans</th></tr></thead>
      <tbody>
        {#each readiness.roles as row (row.job_title + row.impact)}
          <tr>
            <td>{row.job_title}</td><td>{row.impact}</td><td>{row.timeframe}</td>
            <td>{row.employed_workers}</td>
            <td>{percentWithWorkings(row.reskill_plan_coverage)}</td>
          </tr>
        {:else}
          <tr><td colspan="5" class="muted">No roles recorded yet.</td></tr>
        {/each}
      </tbody>
    </table>
    <h3>Skills</h3>
    <table data-testid="change-skills">
      <thead><tr><th>Rising skill</th><th>Meeting {readiness.bar}+</th><th>Below</th><th>Undeclared</th></tr></thead>
      <tbody>
        {#each readiness.rising_skills as row (row.skill)}
          <tr><td>{row.skill}</td><td>{row.meeting}</td><td>{row.below}</td><td>{row.undeclared}</td></tr>
        {:else}
          <tr><td colspan="4" class="muted">No rising skills recorded.</td></tr>
        {/each}
      </tbody>
    </table>
    {#if readiness.declining_skills.length > 0}
      <p class="muted">Declining: {readiness.declining_skills.join(", ")}</p>
    {/if}
  {/if}

  {#if initiative.status !== "completed" && initiative.status !== "cancelled"}
    <h3>Record a role impact</h3>
    <form
      onsubmit={(event) => {
        event.preventDefault();
        void run(() => setRoleImpact(initiative!.pid, { role_profile_pid: impactRole, impact, timeframe }));
      }}
    >
      <label>
        Role
        <select bind:value={impactRole} required>
          <option value="" disabled>Choose…</option>
          {#each roles as r (r.pid)}<option value={r.pid}>{r.job_title}</option>{/each}
        </select>
      </label>
      <label>
        Impact
        <select bind:value={impact}>{#each CHANGE_IMPACTS as i (i)}<option value={i}>{i}</option>{/each}</select>
      </label>
      <label>
        When
        <select bind:value={timeframe}>{#each CHANGE_TIMEFRAMES as f (f)}<option value={f}>{f}</option>{/each}</select>
      </label>
      <button type="submit">Add</button>
    </form>
    <h3>Record a skill shift</h3>
    <form
      onsubmit={(event) => {
        event.preventDefault();
        void run(() => setSkillShift(initiative!.pid, { skill_pid: shiftSkill, direction }));
      }}
    >
      <label>
        Skill
        <select bind:value={shiftSkill} required>
          <option value="" disabled>Choose…</option>
          {#each skills as s (s.pid)}<option value={s.pid}>{s.name}</option>{/each}
        </select>
      </label>
      <label>
        Direction
        <select bind:value={direction}><option value="rising">rising</option><option value="declining">declining</option></select>
      </label>
      <button type="submit">Add</button>
    </form>
  {/if}
{/if}
