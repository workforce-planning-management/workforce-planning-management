<!--
  Skills catalogue (`/skills`): search, rename, and categorise the catalogue;
  accept keyword-rule category suggestions (suggestions only — nothing changes
  until accepted); and see each skill's references in external frameworks
  (UK GDAD PCF name, ESCO concept URI). A rename never breaks an import: the
  framework imports match by reference first.
-->
<script lang="ts">
  import { guardedLoad } from "#lib/loader.svelte.js";
  import { errorText } from "#lib/write.js";
  import ErrorNote from "#lib/components/ErrorNote.svelte";
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
  import { t, tf, tv } from "#lib/i18n.svelte.js";
  import DataTable from "#lib/components/DataTable.svelte";
  import type {
    DataGridColumn,
    DataGridRow,
  } from "@lilydesignsystem/svelte-data-grid";

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

  const message = (cause: unknown) => (errorText(cause));

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

  const suggestionColumns = $derived<DataGridColumn[]>([
    { id: "name", header: t("pages.skills.skill"), sortable: true, rowHeader: true },
    { id: "suggested", header: t("pages.skills.suggested"), sortable: true },
    { id: "keyword", header: t("pages.skills.because"), sortable: true, cell: keywordCell },
    { id: "accept", header: t("pages.skills.accept"), cell: acceptCell },
  ]);

  const load = guardedLoad(
    () => Promise.all([listSkills(), categorySuggestions()]),
    (answer) => {
      [skills, suggestions] = answer;
    },
    (cause) => {
      error = message(cause);
    },
  );

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

<svelte:head><title>{tf("pages.skills.page_title", { skills: t("nav.skills") })}</title></svelte:head>

{#snippet keywordCell({ row }: { row: DataGridRow })}
  <span class="muted">"{row.keyword}"</span>
{/snippet}

{#snippet acceptCell({ row }: { row: DataGridRow })}
  <button type="button" onclick={() => void run(() => applyCategorySuggestions([String(row.pid)]))}>{t("pages.skills.accept")}</button>
{/snippet}

<h1>{t("nav.skills")}</h1>
{#if error}<ErrorNote testid="error" message={error} />{/if}
{#if notice}<p data-testid="notice">{notice}</p>{/if}

<p class="muted" data-testid="skill-counts">
  {tf("pages.skills.skills", { skills: skills.length })}
  {#each SKILL_CATEGORIES as c, i (c)}{i > 0 ? " · " : ""}{c} {counts[c]}{/each}
</p>

{#if suggestions && suggestions.suggestions.length > 0}
  <h2>{t("pages.skills.suggested_categories")}</h2>
  <p class="muted">{tf("pages.skills.skill_s_have_no_suggestion", { derivation: suggestions.derivation, unsuggested: suggestions.unsuggested })}</p>
  <p>
    <button
      type="button"
      data-testid="apply-all"
      onclick={() => void run(() => applyCategorySuggestions(suggestions!.suggestions.map((s) => s.pid)))}
    >
      {tf("pages.skills.apply_all", { suggestions: suggestions.suggestions.length })}
    </button>
  </p>
  <DataTable
    testid="suggestions"
    label={t("pages.skills.suggested_categories")}
    columns={suggestionColumns}
    rows={suggestions.suggestions.slice(0, 50) as unknown as DataGridRow[]}
    rowId={(row) => String(row.pid)}
    pageSize={25}
  />
  {#if suggestions.suggestions.length > 50}<p class="muted">{tf("pages.skills.showing_50_of", { suggestions: suggestions.suggestions.length })}</p>{/if}
{/if}

<h2>{t("pages.skills.catalogue")}</h2>
<p>
  <label>{t("pages.skills.search")} <input data-testid="skill-search" bind:value={query} /></label>
  <label>
    {t("pages.skills.category")}
    <select bind:value={category}>
      <option value="">{t("pages.skills.all")}</option>
      {#each SKILL_CATEGORIES as c (c)}<option value={c}>{tv(c)}</option>{/each}
    </select>
  </label>
</p>
<!-- Left as a plain table: every cell holds live inline forms (rename, category select, ESCO search, merge) that must keep focus and state. -->
<table data-testid="skill-table">
  <thead><tr><th>{t("pages.skills.skill")}</th><th>{t("pages.skills.category")}</th><th>{t("pages.skills.external_references")}</th><th></th></tr></thead>
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
              <button type="submit">{t("pages.skills.save")}</button>
              <button type="button" onclick={() => (editing = null)}>{t("pages.skills.cancel")}</button>
            </form>
          {:else}
            {s.name}
            <button type="button" onclick={() => { editing = s.pid; draftName = s.name; }}>{t("pages.skills.rename")}</button>
          {/if}
        </td>
        <td>
          <select
            aria-label={`Category of ${s.name}`}
            value={s.category}
            onchange={(event) => void run(() => updateSkill(s.pid, { category: event.currentTarget.value }))}
          >
            {#each SKILL_CATEGORIES as c (c)}<option value={c}>{tv(c)}</option>{/each}
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
                <button type="submit">{t("pages.skills.search")}</button>
                <button type="button" onclick={() => { linking = null; linkHits = []; }}>{t("pages.skills.cancel")}</button>
              </form>
              <ul>
                {#each linkHits as hit (hit.uri)}
                  <li>
                    {hit.label}
                    {#if hit.catalogue_skill_pid}<span class="muted">{t("pages.skills.already_linked")}</span>{:else}
                      <button
                        type="button"
                        onclick={() =>
                          void run(async () => {
                            await addSkillRef(s.pid, { framework_slug: "esco", ref: hit.uri, label: hit.label });
                            linking = null;
                            linkHits = [];
                          })}
                      >
                        {t("pages.skills.link")}
                      </button>
                    {/if}
                  </li>
                {/each}
              </ul>
            {:else}
              <button type="button" onclick={() => { linking = s.pid; linkQuery = s.name; linkHits = []; }}>{t("pages.skills.link_esco")}</button>
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
                <option value="" disabled>{t("pages.skills.merge_into")}</option>
                {#each skills.filter((x) => x.pid !== s.pid) as other (other.pid)}<option value={other.pid}>{other.name}</option>{/each}
              </select>
              <button type="submit">{t("pages.skills.merge")}</button>
              <button type="button" onclick={() => (merging = null)}>{t("pages.skills.cancel")}</button>
            </form>
          {:else}
            <button type="button" onclick={() => { merging = s.pid; mergeInto = ""; notice = null; }}>{t("pages.skills.merge_2")}</button>
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
              {t("pages.skills.delete")}
            </button>
          {/if}
        </td>
      </tr>
    {:else}
      <tr><td colspan="4" class="muted">{t("pages.skills.no_skills_match")}</td></tr>
    {/each}
  </tbody>
</table>
{#if shown.length > 200}<p class="muted">{tf("pages.skills.showing_200_of_narrow_the_search", { shown: shown.length })}</p>{/if}
