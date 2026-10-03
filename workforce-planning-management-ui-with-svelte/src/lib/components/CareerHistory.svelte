<!--
  Career history: the roles a person has held with when each started and
  stopped, how each skill level changed over time, a "what did they have on
  this date" view, and retrospective entries for the past. A role or level
  change closes the previous one automatically; entries made by someone other
  than the person are marked.
-->
<script lang="ts">
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
  <h2>Career history</h2>
  {#if error}<p class="error" data-testid="error">{error}</p>{/if}
  {#if notice}<p data-testid="notice">{notice}</p>{/if}

  <h3>Roles over time</h3>
  <table data-testid="role-history">
    <thead><tr><th>Framework</th><th>Role</th><th>From</th><th>To</th><th>Recorded by</th></tr></thead>
    <tbody>
      {#each roles as r (r.pid)}
        <tr>
          <td>{r.framework === "esco" ? "ESCO" : "UK GDAD PCF"}</td>
          <td>{r.role_label} {#if r.current}<span class="chip ok">current</span>{/if}</td>
          <td>{day(r.started_at)}</td><td>{day(r.ended_at)}</td>
          <td class="muted">{r.recorded_by ?? "—"}{#if r.on_behalf} <span class="chip">on their behalf</span>{/if}</td>
        </tr>
      {:else}
        <tr><td colspan="5" class="muted">No roles recorded yet.</td></tr>
      {/each}
    </tbody>
  </table>

  <h4>Add a past role</h4>
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
      Framework
      <select bind:value={roleFramework} onchange={() => (picked = null)}>
        <option value="uk-gdad-pcf">UK GDAD PCF</option><option value="esco">ESCO</option>
      </select>
    </label>
    <RolePicker framework={roleFramework} onpick={(p) => (picked = p)} />
    {#if picked}<strong data-testid="picked-role">{picked.label}</strong>{/if}
    <label>First day <input type="date" bind:value={roleFrom} required /></label>
    <label>Last day <input type="date" bind:value={roleTo} required /></label>
    <button type="submit" disabled={!picked}>Add</button>
  </form>

  <h3>Skill levels over time</h3>
  <table data-testid="skill-history">
    <thead><tr><th>Skill</th><th>Level</th><th>From</th><th>To</th><th>Recorded by</th></tr></thead>
    <tbody>
      {#each history as h (h.pid)}
        <tr>
          <td>{h.skill}</td>
          <td>{h.proficiency} {#if h.current}<span class="chip ok">current</span>{/if}</td>
          <td>{day(h.started_at)}</td><td>{day(h.ended_at)}</td>
          <td class="muted">{h.source}{#if h.on_behalf} <span class="chip">on their behalf</span>{/if}</td>
        </tr>
      {:else}
        <tr><td colspan="5" class="muted">No skill levels recorded yet.</td></tr>
      {/each}
    </tbody>
  </table>

  <h4>Add a past skill level</h4>
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
      Skill
      <select bind:value={skillPid} required>
        <option value="" disabled>Choose…</option>
        {#each catalogue as s (s.pid)}<option value={s.pid}>{s.name}</option>{/each}
      </select>
    </label>
    <label>Level <select bind:value={skillLevel}>{#each [1, 2, 3, 4, 5] as l (l)}<option value={l}>{l}</option>{/each}</select></label>
    <label>First day <input type="date" bind:value={skillFrom} required /></label>
    <label>Last day <input type="date" bind:value={skillTo} required /></label>
    <button type="submit">Add</button>
  </form>

  <h3>What did they have on…</h3>
  <p>
    <label>Date <input type="date" data-testid="as-of" bind:value={asOf} /></label>
    <button type="button" onclick={() => void lookup()}>Look up</button>
  </p>
  {#if snapshot}
    <p data-testid="as-of-result">
      Roles: {snapshot.roles.map((r) => r.role_label).join(", ") || "none"} ·
      Skills: {snapshot.skills.map((s) => `${s.skill} ${s.proficiency}`).join(", ") || "none"}
    </p>
  {/if}
</section>
