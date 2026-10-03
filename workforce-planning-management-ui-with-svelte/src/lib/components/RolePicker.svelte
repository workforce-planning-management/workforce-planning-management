<!--
  Pick a role in a framework: a UK GDAD PCF role level from a grouped list, or
  an ESCO occupation by search. Reports the choice (and its label) to the caller.
-->
<script lang="ts">
  import { listRoleProfiles, searchEscoOccupations } from "#lib/api/wpm.js";

  type Pick = { role_profile_pid?: string; occupation_uri?: string; label: string };
  let { framework, onpick }: { framework: string; onpick: (pick: Pick) => void } = $props();

  type Profiles = Awaited<ReturnType<typeof listRoleProfiles>>;
  let profiles = $state<Profiles>([]);
  let query = $state("");
  let hits = $state<Awaited<ReturnType<typeof searchEscoOccupations>>>([]);
  let error = $state<string | null>(null);

  $effect(() => {
    if (framework === "uk-gdad-pcf" && profiles.length === 0) {
      void listRoleProfiles("uk-gdad-pcf")
        .then((p) => (profiles = p))
        .catch((cause) => (error = cause instanceof Error ? cause.message : String(cause)));
    }
  });

  const groups = $derived.by(() => {
    const byLabel = new Map<string, Profiles>();
    for (const p of profiles) {
      const label = `${p.profession ?? "Other"} — ${p.role_name ?? ""}`;
      byLabel.set(label, [...(byLabel.get(label) ?? []), p]);
    }
    return [...byLabel.entries()]
      .sort(([a], [b]) => a.localeCompare(b))
      .map(([label, items]) => ({ label, items: items.sort((x, y) => (x.level_order ?? 0) - (y.level_order ?? 0)) }));
  });

  async function search() {
    error = null;
    try {
      hits = query.trim().length >= 2 ? await searchEscoOccupations(query.trim()) : [];
    } catch (cause) {
      error = cause instanceof Error ? cause.message : String(cause);
    }
  }
</script>

{#if error}<p class="error">{error}</p>{/if}
{#if framework === "uk-gdad-pcf"}
  <select
    aria-label="Role level"
    onchange={(event) => {
      const p = profiles.find((x) => x.pid === event.currentTarget.value);
      if (p) onpick({ role_profile_pid: p.pid, label: p.job_title });
    }}
  >
    <option value="">Choose a role level…</option>
    {#each groups as g (g.label)}
      <optgroup label={g.label}>{#each g.items as p (p.pid)}<option value={p.pid}>{p.job_title}</option>{/each}</optgroup>
    {/each}
  </select>
{:else}
  <span>
    <input aria-label="Search occupations" bind:value={query} minlength="2" placeholder="Search ESCO occupations" />
    <button type="button" onclick={() => void search()}>Search</button>
  </span>
  <ul>
    {#each hits as hit (hit.uri)}
      <li><button type="button" onclick={() => { onpick({ occupation_uri: hit.uri, label: hit.label }); hits = []; }}>{hit.label}</button></li>
    {/each}
  </ul>
{/if}
