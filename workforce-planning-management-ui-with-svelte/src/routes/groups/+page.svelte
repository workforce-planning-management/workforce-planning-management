<!--
  Groups (`/groups`): communities of practice, communities of interest, and
  other self-organised sets. Pick one to see its members and what it knows —
  declared skills in aggregate, never naming anyone, and withheld for any skill
  fewer than the floor of people declare.
-->
<script lang="ts">
  import { createGroup, groupMembers, groupSkills, listGroups, GROUP_KINDS, type Group } from "#lib/api/wpm.js";
  import { t } from "#lib/i18n.svelte.js";

  let groups = $state<Group[]>([]);
  let selected = $state<string | null>(null);
  let members = $state<Awaited<ReturnType<typeof groupMembers>> | null>(null);
  let know = $state<Awaited<ReturnType<typeof groupSkills>> | null>(null);
  let error = $state<string | null>(null);
  let name = $state("");
  let kind = $state<string>("practice");
  let description = $state("");

  const message = (cause: unknown) => (cause instanceof Error ? cause.message : String(cause));
  const kindLabel = (k: string) => (k === "practice" ? "community of practice" : k === "interest" ? "community of interest" : "group");

  async function load() {
    try {
      groups = await listGroups();
    } catch (cause) {
      error = message(cause);
    }
  }
  $effect(() => {
    void load();
  });

  async function open(pid: string) {
    selected = pid;
    error = null;
    try {
      [members, know] = await Promise.all([groupMembers(pid), groupSkills(pid)]);
    } catch (cause) {
      error = message(cause);
    }
  }

  async function create(event: SubmitEvent) {
    event.preventDefault();
    error = null;
    try {
      await createGroup({ name: name.trim(), kind, ...(description.trim() ? { description: description.trim() } : {}) });
      name = "";
      description = "";
      await load();
    } catch (cause) {
      error = message(cause);
    }
  }
</script>

<svelte:head><title>Groups — WPM</title></svelte:head>

<h1>{t("nav.groups")}</h1>
{#if error}<p class="error" data-testid="error">{error}</p>{/if}

<table data-testid="group-list">
  <thead><tr><th>Group</th><th>Kind</th><th>Members</th></tr></thead>
  <tbody>
    {#each groups as g (g.pid)}
      <tr>
        <td><button type="button" onclick={() => void open(g.pid)}>{g.name}</button>{#if g.description}<br /><span class="muted">{g.description}</span>{/if}</td>
        <td>{kindLabel(g.kind)}</td><td>{g.members}</td>
      </tr>
    {:else}
      <tr><td colspan="3" class="muted">No groups yet.</td></tr>
    {/each}
  </tbody>
</table>

<form onsubmit={create}>
  <label>Start a group <input bind:value={name} required maxlength="120" /></label>
  <label>Kind <select bind:value={kind}>{#each GROUP_KINDS as k (k)}<option value={k}>{kindLabel(k)}</option>{/each}</select></label>
  <label>About <input bind:value={description} maxlength="1000" /></label>
  <button type="submit">Start</button>
</form>

{#if selected && members && know}
  <section class="panel" data-testid="group-detail">
    <h2>{members.group.name}</h2>
    <h3>Members</h3>
    <ul>
      {#each members.members as m (m.worker_pid)}
        <li>{m.display_name} {#if m.role === "lead"}<span class="chip ok">lead</span>{/if}<span class="muted"> since {m.joined_at.slice(0, 10)}</span></li>
      {/each}
    </ul>
    <h3>What the group knows</h3>
    <p class="muted">Declared skills of current members, in aggregate. Skills fewer than {know.floor} people declare are withheld.</p>
    <table data-testid="group-skills">
      <thead><tr><th>Skill</th><th>Declared by</th><th>Coverage</th><th>Level 1</th><th>2</th><th>3</th><th>4</th><th>5</th></tr></thead>
      <tbody>
        {#each know.skills as s (s.skill_pid)}
          <tr>
            <td>{s.skill}</td><td>{s.declared}</td>
            <td>{s.coverage === null ? "—" : `${Math.round(s.coverage * 100)}%`}</td>
            {#each ["1", "2", "3", "4", "5"] as l (l)}<td>{s.levels[l as "1"]}</td>{/each}
          </tr>
        {:else}
          <tr><td colspan="8" class="muted">Nothing to show yet.</td></tr>
        {/each}
      </tbody>
    </table>
    {#if know.withheld_below_floor > 0}<p class="muted">{know.withheld_below_floor} skill(s) withheld: too few people to show without pointing at someone.</p>{/if}
  </section>
{/if}
