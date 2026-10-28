<!--
  Start a role profile from SFIA skills (WPM-R111): search the deployer's imported SFIA catalogue, pick skills at the
  levels SFIA defines for them, then create a draft profile. The level stays on SFIA's scale; the service counts it on
  WPM's 1–5 by the stated rule. The wording is the deployer's licensed content, so it is shown only to a signed-in
  caller. Nothing is created until "Create draft profile".
-->
<script lang="ts">
  import { SKILL_CATEGORIES, searchSfiaSkills, seedProfileFromSfia } from "#lib/api/wpm.js";
  import { t, tv } from "#lib/i18n.svelte.js";
  import { errorText } from "#lib/write.js";

  type Hits = Awaited<ReturnType<typeof searchSfiaSkills>>;

  let {
    onseeded,
    onfail,
  }: {
    /** Called with the new profile's pid, once it exists. */
    onseeded: (pid: string) => Promise<void>;
    /** Called with the words for a failure, or `null` when an action starts. */
    onfail: (message: string | null) => void;
  } = $props();

  let query = $state("");
  let hits = $state<Hits>([]);
  let picked = $state<Array<{ code: string; name: string; level: number }>>([]);
  let levelFor = $state<Record<string, number>>({});
  let title = $state("");
  let category = $state<string>("technical");

  async function search() {
    onfail(null);
    try {
      hits = query.trim().length >= 2 ? await searchSfiaSkills(query.trim()) : [];
    } catch (cause) {
      onfail(errorText(cause));
    }
  }

  function add(hit: Hits[number]) {
    const level = levelFor[hit.code] ?? hit.levels[0];
    if (level === undefined) return;
    picked = [...picked.filter((p) => p.code !== hit.code), { code: hit.code, name: hit.name, level }];
  }

  async function create() {
    if (picked.length === 0 || !title.trim()) return;
    onfail(null);
    try {
      const created = await seedProfileFromSfia({
        job_title: title.trim(),
        skills: picked.map((p) => ({ code: p.code, level: p.level })),
        category,
      });
      picked = [];
      hits = [];
      query = "";
      title = "";
      await onseeded(created.pid);
    } catch (cause) {
      onfail(errorText(cause));
    }
  }
</script>

<h3>{t("pages.roles.start_from_sfia_skills")}</h3>
<p class="muted">{t("pages.roles.sfia_hint")}</p>
<form
  onsubmit={(event) => {
    event.preventDefault();
    void search();
  }}
>
  <label>{t("pages.roles.sfia_skill")} <input data-testid="sfia-query" bind:value={query} minlength="2" /></label>
  <button type="submit" data-testid="sfia-search">{t("pages.roles.search")}</button>
</form>
{#if hits.length > 0}
  <ul data-testid="sfia-hits">
    {#each hits.filter((h) => h.levels.length > 0) as hit (hit.code)}
      <li>
        <strong>{hit.code}</strong> {hit.name}
        <select
          aria-label={t("pages.roles.sfia_level")}
          data-testid={`sfia-level-${hit.code}`}
          value={levelFor[hit.code] ?? hit.levels[0]}
          onchange={(event) => (levelFor[hit.code] = Number(event.currentTarget.value))}
        >
          {#each hit.levels as level (level)}<option value={level}>{level}</option>{/each}
        </select>
        <button type="button" data-testid={`sfia-add-${hit.code}`} onclick={() => add(hit)}>
          {t("pages.roles.sfia_add")}
        </button>
      </li>
    {/each}
  </ul>
{/if}
{#if picked.length > 0}
  <ul data-testid="sfia-picked">
    {#each picked as p (p.code)}<li>{p.code} {p.name} — {t("pages.roles.sfia_level")} {p.level}</li>{/each}
  </ul>
  <form
    onsubmit={(event) => {
      event.preventDefault();
      void create();
    }}
  >
    <label>
      {t("pages.roles.new_profile_job_title")}
      <input data-testid="sfia-title" bind:value={title} required />
    </label>
    <label>
      {t("pages.roles.category")}
      <select data-testid="sfia-category" bind:value={category}>
        {#each SKILL_CATEGORIES as c (c)}<option value={c}>{tv(c)}</option>{/each}
      </select>
    </label>
    <button type="submit" data-testid="sfia-create">{t("pages.roles.create_draft_profile")}</button>
  </form>
{/if}
