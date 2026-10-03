<!--
  Skills catalogue (`/skills`): search, rename, and categorise the catalogue;
  accept keyword-rule category suggestions (suggestions only — nothing changes
  until accepted); and see each skill's references in external frameworks
  (UK GDAD PCF name, ESCO concept URI). A rename never breaks an import: the
  framework imports match by reference first.
-->
<script lang="ts">
  import {
    SKILL_CATEGORIES,
    applyCategorySuggestions,
    categorySuggestions,
    listSkills,
    updateSkill,
  } from "#lib/api/wpm.js";
  import { t } from "#lib/i18n.svelte.js";

  type Skills = Awaited<ReturnType<typeof listSkills>>;
  type Suggestions = Awaited<ReturnType<typeof categorySuggestions>>;

  let skills = $state<Skills>([]);
  let suggestions = $state<Suggestions | null>(null);
  let query = $state("");
  let category = $state("");
  let editing = $state<string | null>(null);
  let draftName = $state("");
  let error = $state<string | null>(null);

  const message = (cause: unknown) => (cause instanceof Error ? cause.message : String(cause));

  const shown = $derived(
    skills.filter(
      (s) =>
        (!category || s.category === category) &&
        (!query || s.name.toLowerCase().includes(query.toLowerCase())),
    ),
  );
  const counts = $derived(
    Object.fromEntries(SKILL_CATEGORIES.map((c) => [c, skills.filter((s) => s.category === c).length])),
  );

  async function load() {
    try {
      [skills, suggestions] = await Promise.all([listSkills(), categorySuggestions()]);
    } catch (cause) {
      error = message(cause);
    }
  }

  $effect(() => {
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
</script>

<svelte:head><title>{t("nav.skills")} — WPM</title></svelte:head>

<h1>{t("nav.skills")}</h1>
{#if error}<p class="error" data-testid="error">{error}</p>{/if}

<p class="muted" data-testid="skill-counts">
  {skills.length} skills ·
  {#each SKILL_CATEGORIES as c, i (c)}{i > 0 ? " · " : ""}{c} {counts[c]}{/each}
</p>

{#if suggestions && suggestions.suggestions.length > 0}
  <h2>Suggested categories</h2>
  <p class="muted">{suggestions.derivation}. {suggestions.unsuggested} skill(s) have no suggestion.</p>
  <p>
    <button
      type="button"
      data-testid="apply-all"
      onclick={() => void run(() => applyCategorySuggestions(suggestions!.suggestions.map((s) => s.pid)))}
    >
      Apply all {suggestions.suggestions.length}
    </button>
  </p>
  <table data-testid="suggestions">
    <thead><tr><th>Skill</th><th>Suggested</th><th>Because</th><th></th></tr></thead>
    <tbody>
      {#each suggestions.suggestions.slice(0, 50) as s (s.pid)}
        <tr>
          <td>{s.name}</td><td>{s.suggested}</td><td class="muted">"{s.keyword}"</td>
          <td><button type="button" onclick={() => void run(() => applyCategorySuggestions([s.pid]))}>Accept</button></td>
        </tr>
      {/each}
    </tbody>
  </table>
  {#if suggestions.suggestions.length > 50}<p class="muted">Showing 50 of {suggestions.suggestions.length}.</p>{/if}
{/if}

<h2>Catalogue</h2>
<p>
  <label>Search <input data-testid="skill-search" bind:value={query} /></label>
  <label>
    Category
    <select bind:value={category}>
      <option value="">All</option>
      {#each SKILL_CATEGORIES as c (c)}<option value={c}>{c}</option>{/each}
    </select>
  </label>
</p>
<table data-testid="skill-table">
  <thead><tr><th>Skill</th><th>Category</th><th>External references</th></tr></thead>
  <tbody>
    {#each shown.slice(0, 200) as s (s.pid)}
      <tr>
        <td>
          {#if editing === s.pid}
            <form
              onsubmit={(event) => {
                event.preventDefault();
                void run(async () => {
                  await updateSkill(s.pid, { name: draftName });
                  editing = null;
                });
              }}
            >
              <input aria-label={`Rename ${s.name}`} bind:value={draftName} required />
              <button type="submit">Save</button>
              <button type="button" onclick={() => (editing = null)}>Cancel</button>
            </form>
          {:else}
            {s.name}
            <button type="button" onclick={() => { editing = s.pid; draftName = s.name; }}>Rename</button>
          {/if}
        </td>
        <td>
          <select
            aria-label={`Category of ${s.name}`}
            value={s.category}
            onchange={(event) => void run(() => updateSkill(s.pid, { category: event.currentTarget.value }))}
          >
            {#each SKILL_CATEGORIES as c (c)}<option value={c}>{c}</option>{/each}
          </select>
        </td>
        <td class="muted">
          {#each s.external_refs as r (r.framework)}
            <span class="chip" title={r.ref}>{r.framework}{r.label ? `: ${r.label}` : ""}</span>
          {:else}—{/each}
        </td>
      </tr>
    {:else}
      <tr><td colspan="3" class="muted">No skills match.</td></tr>
    {/each}
  </tbody>
</table>
{#if shown.length > 200}<p class="muted">Showing 200 of {shown.length}; narrow the search.</p>{/if}
