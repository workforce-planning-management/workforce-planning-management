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
    addSkillRef,
    applyCategorySuggestions,
    categorySuggestions,
    deleteSkill,
    listSkills,
    mergeSkill,
    removeSkillRef,
    searchEscoSkills,
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
  let merging = $state<string | null>(null);
  let mergeInto = $state("");
  let notice = $state<string | null>(null);
  let linking = $state<string | null>(null);
  let linkQuery = $state("");
  let linkHits = $state<Awaited<ReturnType<typeof searchEscoSkills>>>([]);

  async function searchLink() {
    error = null;
    try {
      linkHits = linkQuery.trim().length >= 2 ? await searchEscoSkills(linkQuery.trim()) : [];
    } catch (cause) {
      error = message(cause);
    }
  }

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
{#if notice}<p data-testid="notice">{notice}</p>{/if}

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
  <thead><tr><th>Skill</th><th>Category</th><th>External references</th><th></th></tr></thead>
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
            {#if r.framework === "esco"}
              <button type="button" aria-label={`Unlink ESCO from ${s.name}`} onclick={() => void run(() => removeSkillRef(s.pid, "esco"))}>×</button>
            {/if}
          {:else}—{/each}
          {#if !s.external_refs.some((r) => r.framework === "esco")}
            {#if linking === s.pid}
              <form
                onsubmit={(event) => {
                  event.preventDefault();
                  void searchLink();
                }}
              >
                <input aria-label={`Search ESCO for ${s.name}`} bind:value={linkQuery} minlength="2" />
                <button type="submit">Search</button>
                <button type="button" onclick={() => { linking = null; linkHits = []; }}>Cancel</button>
              </form>
              <ul>
                {#each linkHits as hit (hit.uri)}
                  <li>
                    {hit.label}
                    {#if hit.catalogue_skill_pid}<span class="muted">(already linked)</span>{:else}
                      <button
                        type="button"
                        onclick={() =>
                          void run(async () => {
                            await addSkillRef(s.pid, { framework_slug: "esco", ref: hit.uri, label: hit.label });
                            linking = null;
                            linkHits = [];
                          })}
                      >
                        Link
                      </button>
                    {/if}
                  </li>
                {/each}
              </ul>
            {:else}
              <button type="button" onclick={() => { linking = s.pid; linkQuery = s.name; linkHits = []; }}>Link ESCO</button>
            {/if}
          {/if}
        </td>
        <td>
          {#if merging === s.pid}
            <form
              onsubmit={(event) => {
                event.preventDefault();
                const target = skills.find((x) => x.pid === mergeInto);
                if (!target || !confirm(`Merge "${s.name}" into "${target.name}"? "${s.name}" is retired.`)) return;
                void run(async () => {
                  const r = await mergeSkill(s.pid, mergeInto);
                  notice = `Merged "${s.name}" into "${target.name}": ${r.records_moved} moved, ${r.records_merged} combined.`;
                  merging = null;
                });
              }}
            >
              <select aria-label={`Merge ${s.name} into`} bind:value={mergeInto} required>
                <option value="" disabled>Merge into…</option>
                {#each skills.filter((x) => x.pid !== s.pid) as other (other.pid)}<option value={other.pid}>{other.name}</option>{/each}
              </select>
              <button type="submit">Merge</button>
              <button type="button" onclick={() => (merging = null)}>Cancel</button>
            </form>
          {:else}
            <button type="button" onclick={() => { merging = s.pid; mergeInto = ""; notice = null; }}>Merge…</button>
            <button
              type="button"
              onclick={() => {
                if (confirm(`Delete "${s.name}"? Only possible when nothing uses it.`)) {
                  void run(async () => {
                    await deleteSkill(s.pid);
                    notice = `Deleted "${s.name}".`;
                  });
                }
              }}
            >
              Delete
            </button>
          {/if}
        </td>
      </tr>
    {:else}
      <tr><td colspan="4" class="muted">No skills match.</td></tr>
    {/each}
  </tbody>
</table>
{#if shown.length > 200}<p class="muted">Showing 200 of {shown.length}; narrow the search.</p>{/if}
