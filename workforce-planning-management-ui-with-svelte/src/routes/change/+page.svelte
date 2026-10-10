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
  import LilyKanban from "#lib/components/LilyKanban.svelte";
  import { t, tf, tv } from "#lib/i18n.svelte.js";

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

  const COLUMNS = ["draft", "active", "completed", "cancelled"].map((id) => ({ id, title: id }));
  const cards = $derived(
    initiatives.map((i) => ({
      id: i.pid,
      columnId: i.status,
      title: `${i.name} · ${i.roles_affected} role(s), ${i.skills_shifting} skill(s)`,
    })),
  );

  /** A card move is a lifecycle transition; the service refuses illegal ones
   * (422) and the reload puts the card back where the truth says it is. */
  async function moveInitiative(pid: string, to: string) {
    const from = initiatives.find((i) => i.pid === pid)?.status;
    if (from === to) return;
    selected = pid;
    await run(() => setChangeStatus(pid, to));
  }

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

<svelte:head><title>{tf("pages.change.page_title", { change: t("nav.change") })}</title></svelte:head>

<h1>{t("nav.change")}</h1>
{#if error}<p class="error" data-testid="error">{error}</p>{/if}
<p class="muted">
  {t("pages.change.track_an_automation_or_ai")}
</p>

<LilyKanban
  label={t("pages.change.change_initiatives_by_status")}
  columns={COLUMNS}
  {cards}
  onMove={(pid, to) => void moveInitiative(pid, to)}
/>

<p>
  <label>
    {t("pages.change.initiative")}
    <select
      data-testid="change-select"
      bind:value={selected}
      onchange={() => void loadDetail()}
    >
      {#each initiatives as i (i.pid)}
        <option value={i.pid}>{i.name} ({tv(i.status)})</option>
      {:else}
        <option value="">{t("pages.change.none_yet")}</option>
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
  <label>{t("pages.change.new_initiative")} <input data-testid="change-new-name" bind:value={newName} required /></label>
  <label>
    {t("pages.change.kind")}
    <select bind:value={newKind}>{#each CHANGE_KINDS as k (k)}<option value={k}>{tv(k)}</option>{/each}</select>
  </label>
  <button type="submit" data-testid="change-create">{t("pages.change.create")}</button>
</form>

{#if initiative}
  <h2>{initiative.name} <span class="chip">{tv(initiative.status)}</span></h2>
  {#each CHANGE_STATUSES_NEXT[initiative.status] ?? [] as next (next)}
    <button type="button" onclick={() => void run(() => setChangeStatus(initiative!.pid, next))}>
      {tf("pages.change.mark", { next: tv(next) })}
    </button>
  {/each}

  {#if readiness}
    <p class="muted">{readiness.derivation}</p>
    <p data-testid="change-summary">
      {tf("pages.change.affected_workers_with_an_active", { affected_workers: readiness.affected_workers, reskill_plan_coverage: percentWithWorkings(readiness.reskill_plan_coverage) })}
    </p>
    <h3>{t("pages.change.roles")}</h3>
    <table data-testid="change-roles">
      <thead><tr><th>{t("pages.change.role")}</th><th>{t("pages.change.impact")}</th><th>{t("pages.change.when")}</th><th>{t("pages.change.employed")}</th><th>{t("pages.change.reskill_plans")}</th></tr></thead>
      <tbody>
        {#each readiness.roles as row (row.job_title + row.impact)}
          <tr>
            <td>{row.job_title}</td><td>{tv(row.impact)}</td><td>{tv(row.timeframe)}</td>
            <td>{row.employed_workers}</td>
            <td>{percentWithWorkings(row.reskill_plan_coverage)}</td>
          </tr>
        {:else}
          <tr><td colspan="5" class="muted">{t("pages.change.no_roles_recorded_yet")}</td></tr>
        {/each}
      </tbody>
    </table>
    <h3>{t("pages.change.skills")}</h3>
    <table data-testid="change-skills">
      <thead><tr><th>{t("pages.change.rising_skill")}</th><th>{tf("pages.change.meeting", { bar: readiness.bar })}</th><th>{t("pages.change.below")}</th><th>{t("pages.change.undeclared")}</th></tr></thead>
      <tbody>
        {#each readiness.rising_skills as row (row.skill)}
          <tr><td>{row.skill}</td><td>{row.meeting}</td><td>{row.below}</td><td>{row.undeclared}</td></tr>
        {:else}
          <tr><td colspan="4" class="muted">{t("pages.change.no_rising_skills_recorded")}</td></tr>
        {/each}
      </tbody>
    </table>
    {#if readiness.declining_skills.length > 0}
      <p class="muted">{tf("pages.change.declining", { declining_skills: readiness.declining_skills.join(", ") })}</p>
    {/if}
  {/if}

  {#if initiative.status !== "completed" && initiative.status !== "cancelled"}
    <h3>{t("pages.change.record_a_role_impact")}</h3>
    <form
      onsubmit={(event) => {
        event.preventDefault();
        void run(() => setRoleImpact(initiative!.pid, { role_profile_pid: impactRole, impact, timeframe }));
      }}
    >
      <label>
        {t("pages.change.role")}
        <select bind:value={impactRole} required>
          <option value="" disabled>{t("pages.change.choose")}</option>
          {#each roles as r (r.pid)}<option value={r.pid}>{r.job_title}</option>{/each}
        </select>
      </label>
      <label>
        {t("pages.change.impact")}
        <select bind:value={impact}>{#each CHANGE_IMPACTS as i (i)}<option value={i}>{tv(i)}</option>{/each}</select>
      </label>
      <label>
        {t("pages.change.when")}
        <select bind:value={timeframe}>{#each CHANGE_TIMEFRAMES as f (f)}<option value={f}>{tv(f)}</option>{/each}</select>
      </label>
      <button type="submit">{t("pages.change.add")}</button>
    </form>
    <h3>{t("pages.change.record_a_skill_shift")}</h3>
    <form
      onsubmit={(event) => {
        event.preventDefault();
        void run(() => setSkillShift(initiative!.pid, { skill_pid: shiftSkill, direction }));
      }}
    >
      <label>
        {t("pages.change.skill")}
        <select bind:value={shiftSkill} required>
          <option value="" disabled>{t("pages.change.choose")}</option>
          {#each skills as s (s.pid)}<option value={s.pid}>{s.name}</option>{/each}
        </select>
      </label>
      <label>
        {t("pages.change.direction")}
        <select bind:value={direction}><option value="rising">{t("pages.change.rising")}</option><option value="declining">{t("pages.change.declining_2")}</option></select>
      </label>
      <button type="submit">{t("pages.change.add")}</button>
    </form>
  {/if}
{/if}
