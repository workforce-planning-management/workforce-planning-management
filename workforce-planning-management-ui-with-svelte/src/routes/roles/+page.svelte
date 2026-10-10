<!--
  Role profiles (`/roles`): what a role (keyed by job title) requires — a
  set of catalogue skills, each with a minimum proficiency (1–5) and an
  importance. A profile describes a role, never a person; it is the
  yardstick competency-gap analysis measures declared skills against.
-->
<script lang="ts">
  import {
    createRoleProfile,
    getRoleProfile,
    listFrameworks,
    listRoleProfiles,
    listSkills,
    mobilityInterestSummary,
    removeRoleRequirement,
    roleGap,
    roleProgression,
    searchEscoOccupations,
    getEscoOccupation,
    seedProfileFromEsco,
    setRoleRequirement,
  } from "#lib/api/wpm.js";
  import RoleGrade from "#lib/components/RoleGrade.svelte";
  import { percentWithWorkings } from "#lib/format.js";
  import { t, tf, tv } from "#lib/i18n.svelte.js";

  type Profiles = Awaited<ReturnType<typeof listRoleProfiles>>;
  type Profile = Awaited<ReturnType<typeof getRoleProfile>>;
  type Skills = Awaited<ReturnType<typeof listSkills>>;
  type Coverage = Awaited<ReturnType<typeof roleGap>>;
  type Interest = Awaited<ReturnType<typeof mobilityInterestSummary>>;
  type Frameworks = Awaited<ReturnType<typeof listFrameworks>>;
  type Progression = Awaited<ReturnType<typeof roleProgression>>;
  type EscoHits = Awaited<ReturnType<typeof searchEscoOccupations>>;
  type EscoOccupation = Awaited<ReturnType<typeof getEscoOccupation>>;

  const IMPORTANCES = ["critical", "important", "useful"] as const;

  let profiles = $state<Profiles>([]);
  let skills = $state<Skills>([]);
  let selected = $state("");
  let coverage = $state<Coverage | null>(null);
  let interest = $state<Interest | null>(null);
  let frameworks = $state<Frameworks>([]);
  let framework = $state("");
  let progression = $state<Progression | null>(null);
  let escoQuery = $state("");
  let escoHits = $state<EscoHits>([]);
  let escoPicked = $state<EscoOccupation | null>(null);
  let escoLevel = $state(3);
  let escoOptional = $state(false);
  let escoTitle = $state("");
  const escoLoaded = $derived(frameworks.some((f) => f.slug === "esco"));
  let profile = $state<Profile | null>(null);
  let error = $state<string | null>(null);

  let newTitle = $state("");
  let reqSkill = $state("");
  let reqLevel = $state(3);
  let reqImportance = $state<string>("important");

  /** The group key for profiles with no profession (shown translated, sorted last). */
  const OTHER_PROFILES = "\u0000other";

  /** Profiles grouped profession → role (levels in order); own profiles last. */
  const groups = $derived.by(() => {
    const byLabel = new Map<string, typeof profiles>();
    for (const p of profiles) {
      const label = p.profession && p.role_name ? `${p.profession} — ${p.role_name}` : OTHER_PROFILES;
      byLabel.set(label, [...(byLabel.get(label) ?? []), p]);
    }
    return [...byLabel.entries()]
      .sort(([a], [b]) => (a === OTHER_PROFILES ? 1 : b === OTHER_PROFILES ? -1 : a.localeCompare(b)))
      .map(([label, items]) => ({
        label: label === OTHER_PROFILES ? t("pages.roles.other_profiles") : label,
        items: items.sort(
          (x, y) => (x.level_order ?? 0) - (y.level_order ?? 0) || x.job_title.localeCompare(y.job_title),
        ),
      }));
  });

  const message = (cause: unknown) =>
    cause instanceof Error ? cause.message : String(cause);

  async function loadProfiles() {
    try {
      profiles = await listRoleProfiles(framework || undefined);
      frameworks = await listFrameworks();
      interest = await mobilityInterestSummary();
      if (!selected && profiles.length > 0) selected = profiles[0]?.pid ?? "";
      if (selected) {
        profile = await getRoleProfile(selected);
        coverage = await roleGap(selected);
        progression = await roleProgression(selected);
      }
    } catch (cause) {
      error = message(cause);
    }
  }

  $effect(() => {
    void (async () => {
      try {
        skills = await listSkills();
      } catch (cause) {
        error = message(cause);
      }
      await loadProfiles();
    })();
  });

  async function select(pid: string) {
    selected = pid;
    profile = null;
    coverage = null;
    progression = null;
    if (pid) {
      try {
        profile = await getRoleProfile(pid);
        coverage = await roleGap(pid);
        progression = await roleProgression(pid);
      } catch (cause) {
        error = message(cause);
      }
    }
  }

  async function searchEsco() {
    error = null;
    escoPicked = null;
    try {
      escoHits = escoQuery.trim().length >= 2 ? await searchEscoOccupations(escoQuery.trim()) : [];
    } catch (cause) {
      error = message(cause);
    }
  }

  async function pickEsco(uri: string) {
    error = null;
    try {
      escoPicked = await getEscoOccupation(uri);
      escoTitle = "";
    } catch (cause) {
      error = message(cause);
    }
  }

  async function seedEsco() {
    if (!escoPicked) return;
    error = null;
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
      selected = created.pid;
      await loadProfiles();
    } catch (cause) {
      error = message(cause);
    }
  }

  async function addProfile() {
    error = null;
    try {
      const created = await createRoleProfile({ job_title: newTitle });
      newTitle = "";
      selected = created.pid;
      await loadProfiles();
    } catch (cause) {
      error = message(cause);
    }
  }

  async function addRequirement() {
    if (!selected || !reqSkill) return;
    error = null;
    try {
      await setRoleRequirement(selected, {
        skill_pid: reqSkill,
        min_proficiency: reqLevel,
        importance: reqImportance,
      });
      await loadProfiles();
    } catch (cause) {
      error = message(cause);
    }
  }

  async function dropRequirement(skillPid: string) {
    error = null;
    try {
      await removeRoleRequirement(selected, skillPid);
      await loadProfiles();
    } catch (cause) {
      error = message(cause);
    }
  }
</script>

<svelte:head><title>{tf("pages.roles.page_title", { roles: t("nav.roles") })}</title></svelte:head>

<h1>{t("nav.roles")}</h1>
{#if error}<p class="error" data-testid="error">{error}</p>{/if}

<p class="muted">
  {t("pages.roles.a_role_profile_says_what_a_job")}
</p>

{#if interest && interest.targets.length > 0}
  <h2>{t("pages.roles.employee_interest")}</h2>
  <p class="muted">{interest.derivation}</p>
  <table data-testid="mobility-summary">
    <thead><tr><th>{t("pages.roles.target")}</th><th>{t("pages.roles.kind")}</th><th>{t("pages.roles.interested")}</th></tr></thead>
    <tbody>
      {#each interest.targets as row (row.target_pid)}
        <tr><td>{row.title ?? row.target_pid}</td><td>{tv(row.kind)}</td><td>{row.interested}</td></tr>
      {/each}
    </tbody>
  </table>
{/if}

<h2>{t("pages.roles.profiles")}</h2>
{#if frameworks.length > 0}
  <p>
    <label>
      {t("pages.roles.framework")}
      <select
        data-testid="framework-filter"
        bind:value={framework}
        onchange={() => { selected = ""; void loadProfiles(); }}
      >
        <option value="">{t("pages.roles.all_profiles")}</option>
        {#each frameworks as f (f.slug)}
          <option value={f.slug}>{f.name} ({f.profiles})</option>
        {/each}
      </select>
    </label>
  </p>
{/if}
<p>
  <label>
    {t("pages.roles.profile")}
    <select
      data-testid="role-select"
      value={selected}
      onchange={(event) => void select(event.currentTarget.value)}
    >
      {#each groups as g (g.label)}
        <optgroup label={g.label}>
          {#each g.items as p (p.pid)}
            <option value={p.pid}>{p.job_title} ({p.requirement_count})</option>
          {/each}
        </optgroup>
      {:else}
        <option value="">{t("pages.roles.no_profiles_yet")}</option>
      {/each}
    </select>
  </label>
</p>
<form
  onsubmit={(event) => {
    event.preventDefault();
    void addProfile();
  }}
>
  <label>
    {t("pages.roles.new_profile_job_title")}
    <input data-testid="role-new-title" bind:value={newTitle} required />
  </label>
  <button type="submit" data-testid="role-create">{t("pages.roles.create")}</button>
</form>

{#if escoLoaded}
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
{/if}

{#if profile}
  <h2>{profile.job_title}</h2>
  {#if profile.framework}
    <p class="muted" data-testid="framework-attribution">
      {tf("pages.roles.level", { profession: profile.profession, role_name: profile.role_name, level_order: profile.level_order, name: profile.framework.name, licence: profile.framework.licence })}
    </p>
    <p class="muted">{profile.framework.attribution}</p>
    <p class="muted">
      {tf("pages.roles.required_levels_come_from_the", { scale_labels: profile.framework.scale_labels, note: profile.framework.note })}
    </p>
  {:else if profile.source_ref}<p class="muted">{tf("pages.roles.source", { source_ref: profile.source_ref })}</p>{/if}
  {#if profile.description}<p>{profile.description}</p>{/if}
  <RoleGrade profilePid={profile.pid} />
  <table data-testid="role-requirements">
    <thead>
      <tr><th>{t("pages.roles.skill")}</th><th>{t("pages.roles.category")}</th><th>{t("pages.roles.minimum")}</th><th>{t("pages.roles.framework_level")}</th><th>{t("pages.roles.importance")}</th><th></th></tr>
    </thead>
    <tbody>
      {#each profile.requirements as req (req.skill_pid)}
        <tr>
          <td>
            {req.skill ?? req.skill_pid}
            {#if req.note}
              <details><summary class="muted">{t("pages.roles.what_the_framework_says")}</summary><p>{req.note}</p></details>
            {/if}
          </td>
          <td>{req.category ?? "—"}</td>
          <td>{req.min_proficiency} / 5</td>
          <td>{req.source_level !== null ? `${req.source_level} of ${req.source_scale_max}` : "—"}</td>
          <td class:warn={req.importance === "critical"}>{tv(req.importance)}</td>
          <td>
            <button type="button" onclick={() => void dropRequirement(req.skill_pid)}>
              {t("pages.roles.remove")}
            </button>
          </td>
        </tr>
      {:else}
        <tr><td colspan="6" class="muted">{t("pages.roles.no_required_skills_yet")}</td></tr>
      {/each}
    </tbody>
  </table>

  {#if progression && typeof progression.profile !== "string"}
    <h3>{t("pages.roles.next_level_up")}</h3>
    {#each progression.next as step (step.pid)}
      <p data-testid="role-progression">
        <strong>{step.job_title}</strong>{tf("pages.roles.new_skill_s_raised_unchanged", { added: step.added.length, raised: step.raised.length, unchanged: step.unchanged })}
      </p>
      <ul>
        {#each step.added as a (a.skill)}<li>{tf("pages.roles.new_needs", { skill: a.skill, min_proficiency: a.min_proficiency })}</li>{/each}
        {#each step.raised as r (r.skill)}<li>{tf("pages.roles.raised", { skill: r.skill, from: r.from, to: r.to })}</li>{/each}
      </ul>
    {:else}
      <p class="muted">{t("pages.roles.this_is_the_top_level_of_the_role")}</p>
    {/each}
  {/if}

  {#if coverage}
    <h3>{t("pages.roles.can_we_staff_this_role_today")}</h3>
    <p class="muted">{coverage.derivation}</p>
    <table data-testid="role-coverage">
      <thead>
        <tr><th>{t("pages.roles.skill")}</th><th>{t("pages.roles.needs")}</th><th>{t("pages.roles.meeting")}</th><th>{t("pages.roles.below")}</th><th>{t("pages.roles.undeclared")}</th><th>{t("pages.roles.coverage")}</th></tr>
      </thead>
      <tbody>
        {#each coverage.requirements as row (row.skill)}
          <tr>
            <td>{row.skill}</td>
            <td>{row.min_proficiency}</td>
            <td>{row.meeting}</td>
            <td>{row.below}</td>
            <td>{row.undeclared}</td>
            <td>{percentWithWorkings(row.coverage)}</td>
          </tr>
        {/each}
      </tbody>
    </table>
  {/if}

  <h3>{t("pages.roles.require_a_skill")}</h3>
  <form
    onsubmit={(event) => {
      event.preventDefault();
      void addRequirement();
    }}
  >
    <label>
      {t("pages.roles.skill")}
      <select data-testid="role-req-skill" bind:value={reqSkill} required>
        <option value="" disabled>{t("pages.roles.choose")}</option>
        {#each skills as skill (skill.pid)}
          <option value={skill.pid}>{skill.name}</option>
        {/each}
      </select>
    </label>
    <label>
      {t("pages.roles.minimum_proficiency")}
      <select data-testid="role-req-level" bind:value={reqLevel}>
        {#each [1, 2, 3, 4, 5] as level (level)}<option value={level}>{level}</option>{/each}
      </select>
    </label>
    <label>
      {t("pages.roles.importance")}
      <select data-testid="role-req-importance" bind:value={reqImportance}>
        {#each IMPORTANCES as importance (importance)}
          <option value={importance}>{tv(importance)}</option>
        {/each}
      </select>
    </label>
    <button type="submit" data-testid="role-req-add">{t("pages.roles.add")}</button>
  </form>
{/if}

<style>
  td.warn { color: #b45309; font-weight: 600; }
</style>
