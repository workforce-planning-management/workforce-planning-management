<!--
  Start a role profile from an ESCO occupation (WPM-R84): search, look at which skills ESCO says the occupation
  needs, then create a draft profile with those skills at a starting level. The profile is a draft to edit, not a
  decision; nothing is created until "Create draft profile".
-->
<script lang="ts">
  import {
    getEscoOccupation,
    searchEscoOccupations,
    seedProfileFromEsco,
  } from "#lib/api/wpm.js";
  import { t, tf } from "#lib/i18n.svelte.js";
  import { errorText } from "#lib/write.js";

  type EscoHits = Awaited<ReturnType<typeof searchEscoOccupations>>;
  type EscoOccupation = Awaited<ReturnType<typeof getEscoOccupation>>;

  let {
    onseeded,
    onfail,
  }: {
    /** Called with the new profile's pid, once it exists. */
    onseeded: (pid: string) => Promise<void>;
    /** Called with the words for a failure, or `null` when an action starts. */
    onfail: (message: string | null) => void;
  } = $props();

  let escoQuery = $state("");
  let escoHits = $state<EscoHits>([]);
  let escoPicked = $state<EscoOccupation | null>(null);
  let escoLevel = $state(3);
  let escoOptional = $state(false);
  let escoTitle = $state("");

  async function searchEsco() {
    onfail(null);
    escoPicked = null;
    try {
      escoHits = escoQuery.trim().length >= 2 ? await searchEscoOccupations(escoQuery.trim()) : [];
    } catch (cause) {
      onfail(errorText(cause));
    }
  }

  async function pickEsco(uri: string) {
    onfail(null);
    try {
      escoPicked = await getEscoOccupation(uri);
      escoTitle = "";
    } catch (cause) {
      onfail(errorText(cause));
    }
  }

  async function seedEsco() {
    if (!escoPicked) return;
    onfail(null);
    try {
      const created = await seedProfileFromEsco({
        occupation_uri: escoPicked.uri,
        default_min_proficiency: escoLevel,
        include_optional: escoOptional,
        ...(escoTitle.trim() ? { job_title: escoTitle.trim() } : {}),
      });
      escoPicked = null;
      escoHits = [];
      escoQuery = "";
      await onseeded(created.pid);
    } catch (cause) {
      onfail(errorText(cause));
    }
  }
</script>

<h3>{t("pages.roles.start_from_an_esco_occupation")}</h3>
<p class="muted">
  {t("pages.roles.esco_says_which_skills_an")}
</p>
<form
  onsubmit={(event) => {
    event.preventDefault();
    void searchEsco();
  }}
>
  <label>{t("pages.roles.occupation")} <input data-testid="esco-query" bind:value={escoQuery} minlength="2" /></label>
  <button type="submit" data-testid="esco-search">{t("pages.roles.search")}</button>
</form>
{#if escoHits.length > 0 && !escoPicked}
  <ul data-testid="esco-hits">
    {#each escoHits as hit (hit.uri)}
      <li>
        <button type="button" onclick={() => void pickEsco(hit.uri)}>{hit.label}</button>
        <span class="muted">{tf("pages.roles.isco_essential_optional", { isco_code: hit.isco_code ?? "—", essential_skills: hit.essential_skills, optional_skills: hit.optional_skills })}</span>
      </li>
    {/each}
  </ul>
{/if}
{#if escoPicked}
  <p><strong>{escoPicked.label}</strong> <span class="muted">{tf("pages.roles.isco", { isco_code: escoPicked.isco_code ?? "—" })}</span></p>
  {#if escoPicked.description}<p class="muted">{escoPicked.description}</p>{/if}
  <ul>
    {#each escoPicked.skills.slice(0, 12) as s (s.uri)}
      <li>{s.label} <span class="muted">({s.relation}{s.catalogue_skill_pid ? ", in catalogue" : ""})</span></li>
    {/each}
  </ul>
  {#if escoPicked.skills.length > 12}<p class="muted">{tf("pages.roles.and_more", { skills: escoPicked.skills.length - 12 })}</p>{/if}
  <form
    onsubmit={(event) => {
      event.preventDefault();
      void seedEsco();
    }}
  >
    <label>
      {t("pages.roles.starting_minimum_level")}
      <select data-testid="esco-level" bind:value={escoLevel}>
        {#each [1, 2, 3, 4, 5] as level (level)}<option value={level}>{level}</option>{/each}
      </select>
    </label>
    <label><input type="checkbox" bind:checked={escoOptional} /> {t("pages.roles.include_optional_skills_as_useful")}</label>
    <label>{t("pages.roles.job_title_optional")} <input bind:value={escoTitle} placeholder={escoPicked.label} /></label>
    <button type="submit" data-testid="esco-create">{t("pages.roles.create_draft_profile")}</button>
    <button type="button" onclick={() => (escoPicked = null)}>{t("pages.roles.back")}</button>
  </form>
{/if}
