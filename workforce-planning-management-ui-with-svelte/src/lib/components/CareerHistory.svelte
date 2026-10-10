<!--
  Career history: the roles a person has held with when each started and
  stopped, how each skill level changed over time, a "what did they have on
  this date" view, and retrospective entries for the past. A role or level
  change closes the previous one automatically; entries made by someone other
  than the person are marked.
-->
<script lang="ts">
  import { t, tf, tv } from "#lib/i18n.svelte.js";
  import {
    addPastRole,
    addPastSkill,
    listSkills,
    roleHistory,
    skillHistory,
    skillsAsOf,
    type RoleHistoryRow,
    type SkillHistoryRow,
  } from "#lib/api/wpm.js";
  import RolePicker from "#lib/components/RolePicker.svelte";

  let { workerPid }: { workerPid: string } = $props();

  let roles = $state<RoleHistoryRow[]>([]);
  let history = $state<SkillHistoryRow[]>([]);
  let catalogue = $state<Awaited<ReturnType<typeof listSkills>>>([]);
  let error = $state<string | null>(null);
  let notice = $state<string | null>(null);

  // past role form
  let roleFramework = $state<"uk-gdad-pcf" | "esco">("uk-gdad-pcf");
  let picked = $state<{ role_profile_pid?: string; occupation_uri?: string; label: string } | null>(null);
  let roleFrom = $state("");
  let roleTo = $state("");
  // past skill form
  let skillPid = $state("");
  let skillLevel = $state(3);
  let skillFrom = $state("");
  let skillTo = $state("");
  // as of
  let asOf = $state("");
  let snapshot = $state<Awaited<ReturnType<typeof skillsAsOf>> | null>(null);

  const message = (cause: unknown) => (cause instanceof Error ? cause.message : String(cause));
  const day = (iso: string | null) => (iso ? iso.slice(0, 10) : "now");

  async function load() {
    try {
      [roles, history, catalogue] = await Promise.all([
        roleHistory(workerPid),
        skillHistory(workerPid),
        listSkills(),
      ]);
    } catch (cause) {
      error = message(cause);
    }
  }

  $effect(() => {
    void workerPid;
    void load();
  });

  async function run(action: () => Promise<unknown>, done: string) {
    error = null;
    notice = null;
    try {
      await action();
      notice = done;
      await load();
    } catch (cause) {
      error = message(cause);
    }
  }

  async function lookup() {
    error = null;
    try {
      snapshot = asOf ? await skillsAsOf(workerPid, asOf) : null;
    } catch (cause) {
      error = message(cause);
    }
  }
</script>

<section class="panel" data-testid="career-history">
  <h2>{t("comp.careerHistory.career_history")}</h2>
  {#if error}<p class="error" data-testid="error">{error}</p>{/if}
  {#if notice}<p data-testid="notice">{notice}</p>{/if}

  <h3>{t("comp.careerHistory.roles_over_time")}</h3>
  <table data-testid="role-history">
    <thead><tr><th>{t("comp.careerHistory.framework")}</th><th>{t("comp.careerHistory.role")}</th><th>{t("comp.careerHistory.from")}</th><th>{t("comp.careerHistory.to")}</th><th>{t("comp.careerHistory.recorded_by")}</th></tr></thead>
    <tbody>
      {#each roles as r (r.pid)}
        <tr>
          <td>{r.framework === "esco" ? "ESCO" : "UK GDAD PCF"}</td>
          <td>{r.role_label} {#if r.current}<span class="chip ok">{t("comp.careerHistory.current")}</span>{/if}</td>
          <td>{day(r.started_at)}</td><td>{day(r.ended_at)}</td>
          <td class="muted">{r.recorded_by ?? "—"}{#if r.on_behalf} <span class="chip">{t("comp.careerHistory.on_their_behalf")}</span>{/if}</td>
        </tr>
      {:else}
        <tr><td colspan="5" class="muted">{t("comp.careerHistory.no_roles_recorded_yet")}</td></tr>
      {/each}
    </tbody>
  </table>

  <h4>{t("comp.careerHistory.add_a_past_role")}</h4>
  <form
    onsubmit={(event) => {
      event.preventDefault();
      if (!picked) return;
      const p = picked;
      void run(
        () => addPastRole(workerPid, roleFramework, {
          ...(p.role_profile_pid ? { role_profile_pid: p.role_profile_pid } : {}),
          ...(p.occupation_uri ? { occupation_uri: p.occupation_uri } : {}),
          started_on: roleFrom,
          ended_on: roleTo,
        }),
        `Added ${p.label}.`,
      );
    }}
  >
    <label>
      {t("comp.careerHistory.framework")}
      <select bind:value={roleFramework} onchange={() => (picked = null)}>
        <option value="uk-gdad-pcf">{t("comp.careerHistory.uk_gdad_pcf")}</option><option value="esco">{t("comp.careerHistory.esco")}</option>
      </select>
    </label>
    <RolePicker framework={roleFramework} onpick={(p) => (picked = p)} />
    {#if picked}<strong data-testid="picked-role">{picked.label}</strong>{/if}
    <label>{t("comp.careerHistory.first_day")} <input type="date" bind:value={roleFrom} required /></label>
    <label>{t("comp.careerHistory.last_day")} <input type="date" bind:value={roleTo} required /></label>
    <button type="submit" disabled={!picked}>{t("comp.careerHistory.add")}</button>
  </form>

  <h3>{t("comp.careerHistory.skill_levels_over_time")}</h3>
  <table data-testid="skill-history">
    <thead><tr><th>{t("comp.careerHistory.skill")}</th><th>{t("comp.careerHistory.level")}</th><th>{t("comp.careerHistory.from")}</th><th>{t("comp.careerHistory.to")}</th><th>{t("comp.careerHistory.recorded_by")}</th></tr></thead>
    <tbody>
      {#each history as h (h.pid)}
        <tr>
          <td>{h.skill}</td>
          <td>{h.proficiency} {#if h.current}<span class="chip ok">{t("comp.careerHistory.current")}</span>{/if}</td>
          <td>{day(h.started_at)}</td><td>{day(h.ended_at)}</td>
          <td class="muted">{tv(h.source)}{#if h.on_behalf} <span class="chip">{t("comp.careerHistory.on_their_behalf")}</span>{/if}</td>
        </tr>
      {:else}
        <tr><td colspan="5" class="muted">{t("comp.careerHistory.no_skill_levels_recorded_yet")}</td></tr>
      {/each}
    </tbody>
  </table>

  <h4>{t("comp.careerHistory.add_a_past_skill_level")}</h4>
  <form
    onsubmit={(event) => {
      event.preventDefault();
      void run(
        () => addPastSkill(workerPid, { skill_pid: skillPid, proficiency: skillLevel, started_on: skillFrom, ended_on: skillTo }),
        "Added the past level.",
      );
    }}
  >
    <label>
      {t("comp.careerHistory.skill")}
      <select bind:value={skillPid} required>
        <option value="" disabled>{t("comp.careerHistory.choose")}</option>
        {#each catalogue as s (s.pid)}<option value={s.pid}>{s.name}</option>{/each}
      </select>
    </label>
    <label>{t("comp.careerHistory.level")} <select bind:value={skillLevel}>{#each [1, 2, 3, 4, 5] as l (l)}<option value={l}>{l}</option>{/each}</select></label>
    <label>{t("comp.careerHistory.first_day")} <input type="date" bind:value={skillFrom} required /></label>
    <label>{t("comp.careerHistory.last_day")} <input type="date" bind:value={skillTo} required /></label>
    <button type="submit">{t("comp.careerHistory.add")}</button>
  </form>

  <h3>{t("comp.careerHistory.what_did_they_have_on")}</h3>
  <p>
    <label>{t("comp.careerHistory.date")} <input type="date" data-testid="as-of" bind:value={asOf} /></label>
    <button type="button" onclick={() => void lookup()}>{t("comp.careerHistory.look_up")}</button>
  </p>
  {#if snapshot}
    <p data-testid="as-of-result">
      {tf("comp.careerHistory.roles_skills", { role_label: snapshot.roles.map((r) => r.role_label).join(", ") || t("comp.careerHistory.none"), proficiency: snapshot.skills.map((s) => `${s.skill} ${s.proficiency}`).join(", ") || t("comp.careerHistory.none") })}
    </p>
  {/if}
</section>
