<!--
  Aspirations, learning goals, and growth ideas: a future role or a skill level
  to aim for, with a horizon, a status, and a note in the person's own words.
  Private unless the person shares it. Progress is measured against the skills
  they have declared today.
-->
<script lang="ts">
  import {
    ASPIRATION_HORIZONS,
    ASPIRATION_STATUSES,
    addAspiration,
    deleteAspiration,
    listAspirations,
    listSkills,
    updateAspiration,
    VISIBILITIES,
    type Visibility,
    type Aspiration,
  } from "#lib/api/wpm.js";
  import RolePicker from "#lib/components/RolePicker.svelte";

  let { workerPid }: { workerPid: string } = $props();

  let items = $state<Aspiration[]>([]);
  let isPerson = $state(true);
  let catalogue = $state<Awaited<ReturnType<typeof listSkills>>>([]);
  let error = $state<string | null>(null);

  let kind = $state<"skill" | "role">("skill");
  let skillPid = $state("");
  let target = $state(4);
  let framework = $state<"uk-gdad-pcf" | "esco">("uk-gdad-pcf");
  let picked = $state<{ role_profile_pid?: string; occupation_uri?: string; label: string } | null>(null);
  let horizon = $state<string>("within_1y");
  let note = $state("");
  let visibility = $state<Visibility>("private");

  const message = (cause: unknown) => (cause instanceof Error ? cause.message : String(cause));

  async function load() {
    try {
      const [list, skills] = await Promise.all([listAspirations(workerPid), listSkills()]);
      items = list.aspirations;
      isPerson = list.viewer_is_the_person;
      catalogue = skills;
    } catch (cause) {
      error = message(cause);
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

  function describe(a: Aspiration): string {
    const p = a.progress as Record<string, number | boolean | null> | null;
    if (!p) return "";
    if (a.kind === "skill") {
      return p.achieved
        ? "reached"
        : p.current_level === null ? "not declared yet" : `at ${p.current_level}, ${p.gap} to go`;
    }
    if ("requirements" in p) return `${p.met} of ${p.requirements} requirements met`;
    if ("essential_skills" in p) return `${p.have} of ${p.essential_skills} essential skills`;
    return "";
  }
</script>

<section class="panel" data-testid="aspirations">
  <h2>Aspirations and growth ideas</h2>
  {#if error}<p class="error" data-testid="error">{error}</p>{/if}
  {#if !isPerson}
    <p class="muted">You see only what this person has chosen to share.</p>
  {/if}
  <table data-testid="aspiration-list">
    <thead><tr><th>Goal</th><th>When</th><th>Status</th><th>Progress</th><th>Who can see</th><th></th></tr></thead>
    <tbody>
      {#each items as a (a.pid)}
        <tr>
          <td>
            {a.kind === "skill" ? `${a.skill} → level ${a.target_level}` : `Role: ${a.role_label} (${a.framework === "esco" ? "ESCO" : "PCF"})`}
            {#if a.note}<br /><span class="muted">{a.note}</span>{/if}
            {#if a.on_behalf}<span class="chip">added on their behalf</span>{/if}
          </td>
          <td>{a.horizon.replaceAll("_", " ")}</td>
          <td>
            <select
              aria-label="Status"
              value={a.status}
              onchange={(event) => void run(() => updateAspiration(a.pid, { status: event.currentTarget.value }))}
            >
              {#each ASPIRATION_STATUSES as s (s)}<option value={s}>{s.replace("_", " ")}</option>{/each}
            </select>
          </td>
          <td class="muted">{describe(a)}</td>
          <td>
            {#if isPerson}
              <select
                aria-label="Who can see this"
                value={a.visibility}
                onchange={(event) => void run(() => updateAspiration(a.pid, { visibility: event.currentTarget.value as Visibility }))}
              >
                {#each VISIBILITIES as v (v)}<option value={v}>{v === "manager" ? "my managers" : v}</option>{/each}
              </select>
            {:else}
              {a.visibility}
            {/if}
          </td>
          <td><button type="button" onclick={() => void run(() => deleteAspiration(a.pid))}>Remove</button></td>
        </tr>
      {:else}
        <tr><td colspan="6" class="muted">Nothing here yet.</td></tr>
      {/each}
    </tbody>
  </table>

  <h3>Add one</h3>
  <form
    onsubmit={(event) => {
      event.preventDefault();
      const body =
        kind === "skill"
          ? { kind, skill_pid: skillPid, target_level: target }
          : picked
            ? { kind, framework_slug: framework, ...(picked.role_profile_pid ? { role_profile_pid: picked.role_profile_pid } : {}), ...(picked.occupation_uri ? { occupation_uri: picked.occupation_uri } : {}) }
            : null;
      if (!body) return;
      void run(async () => {
        await addAspiration(workerPid, { ...body, horizon, visibility, ...(note.trim() ? { note: note.trim() } : {}) });
        note = "";
        picked = null;
      });
    }}
  >
    <label>
      I want to
      <select bind:value={kind}><option value="skill">grow a skill</option><option value="role">move into a role</option></select>
    </label>
    {#if kind === "skill"}
      <label>
        Skill
        <select bind:value={skillPid} required>
          <option value="" disabled>Choose…</option>
          {#each catalogue as s (s.pid)}<option value={s.pid}>{s.name}</option>{/each}
        </select>
      </label>
      <label>Level <select bind:value={target}>{#each [1, 2, 3, 4, 5] as l (l)}<option value={l}>{l}</option>{/each}</select></label>
    {:else}
      <label>Framework <select bind:value={framework} onchange={() => (picked = null)}><option value="uk-gdad-pcf">UK GDAD PCF</option><option value="esco">ESCO</option></select></label>
      <RolePicker {framework} onpick={(p) => (picked = p)} />
      {#if picked}<strong>{picked.label}</strong>{/if}
    {/if}
    <label>When <select bind:value={horizon}>{#each ASPIRATION_HORIZONS as h (h)}<option value={h}>{h.replaceAll("_", " ")}</option>{/each}</select></label>
    <label>In my own words <input bind:value={note} maxlength="1000" placeholder="a growth idea, a learning goal…" /></label>
    <label>Who can see it <select bind:value={visibility}>{#each VISIBILITIES as v (v)}<option value={v}>{v === "manager" ? "my managers (everyone above me)" : v === "everyone" ? "everyone who can view my record" : "only me"}</option>{/each}</select></label>
    <button type="submit" data-testid="aspiration-add">Add</button>
  </form>
</section>
