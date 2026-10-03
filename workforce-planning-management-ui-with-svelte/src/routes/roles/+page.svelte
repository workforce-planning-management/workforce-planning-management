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
  import { percentWithWorkings } from "#lib/format.js";
  import { t } from "#lib/i18n.svelte.js";

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

  /** Profiles grouped profession → role (levels in order); own profiles last. */
  const groups = $derived.by(() => {
    const byLabel = new Map<string, typeof profiles>();
    for (const p of profiles) {
      const label = p.profession && p.role_name ? `${p.profession} — ${p.role_name}` : "Other profiles";
      byLabel.set(label, [...(byLabel.get(label) ?? []), p]);
    }
    return [...byLabel.entries()]
      .sort(([a], [b]) => (a === "Other profiles" ? 1 : b === "Other profiles" ? -1 : a.localeCompare(b)))
      .map(([label, items]) => ({
        label,
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

<svelte:head><title>{t("nav.roles")} — WPM</title></svelte:head>

<h1>{t("nav.roles")}</h1>
{#if error}<p class="error" data-testid="error">{error}</p>{/if}

<p class="muted">
  A role profile says what a job title requires — not what a person has. Declared
  skills are compared against it for gap analysis.
</p>

{#if interest && interest.targets.length > 0}
  <h2>Employee interest</h2>
  <p class="muted">{interest.derivation}</p>
  <table data-testid="mobility-summary">
    <thead><tr><th>Target</th><th>Kind</th><th>Interested</th></tr></thead>
    <tbody>
      {#each interest.targets as row (row.target_pid)}
        <tr><td>{row.title ?? row.target_pid}</td><td>{row.kind}</td><td>{row.interested}</td></tr>
      {/each}
    </tbody>
  </table>
{/if}

<h2>Profiles</h2>
{#if frameworks.length > 0}
  <p>
    <label>
      Framework
      <select
        data-testid="framework-filter"
        bind:value={framework}
        onchange={() => { selected = ""; void loadProfiles(); }}
      >
        <option value="">All profiles</option>
        {#each frameworks as f (f.slug)}
          <option value={f.slug}>{f.name} ({f.profiles})</option>
        {/each}
      </select>
    </label>
  </p>
{/if}
<p>
  <label>
    Profile
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
        <option value="">No profiles yet</option>
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
    New profile (job title)
    <input data-testid="role-new-title" bind:value={newTitle} required />
  </label>
  <button type="submit" data-testid="role-create">Create</button>
</form>

{#if escoLoaded}
  <h3>Start from an ESCO occupation</h3>
  <p class="muted">
    ESCO says which skills an occupation needs, not how well — so you choose the starting level
    below. The result is a draft for you to edit.
  </p>
  <form
    onsubmit={(event) => {
      event.preventDefault();
      void searchEsco();
    }}
  >
    <label>Occupation <input data-testid="esco-query" bind:value={escoQuery} minlength="2" /></label>
    <button type="submit" data-testid="esco-search">Search</button>
  </form>
  {#if escoHits.length > 0 && !escoPicked}
    <ul data-testid="esco-hits">
      {#each escoHits as hit (hit.uri)}
        <li>
          <button type="button" onclick={() => void pickEsco(hit.uri)}>{hit.label}</button>
          <span class="muted">ISCO {hit.isco_code ?? "—"} · {hit.essential_skills} essential, {hit.optional_skills} optional</span>
        </li>
      {/each}
    </ul>
  {/if}
  {#if escoPicked}
    <p><strong>{escoPicked.label}</strong> <span class="muted">(ISCO {escoPicked.isco_code ?? "—"})</span></p>
    {#if escoPicked.description}<p class="muted">{escoPicked.description}</p>{/if}
    <ul>
      {#each escoPicked.skills.slice(0, 12) as s (s.uri)}
        <li>{s.label} <span class="muted">({s.relation}{s.catalogue_skill_pid ? ", in catalogue" : ""})</span></li>
      {/each}
    </ul>
    {#if escoPicked.skills.length > 12}<p class="muted">… and {escoPicked.skills.length - 12} more.</p>{/if}
    <form
      onsubmit={(event) => {
        event.preventDefault();
        void seedEsco();
      }}
    >
      <label>
        Starting minimum level
        <select data-testid="esco-level" bind:value={escoLevel}>
          {#each [1, 2, 3, 4, 5] as level (level)}<option value={level}>{level}</option>{/each}
        </select>
      </label>
      <label><input type="checkbox" bind:checked={escoOptional} /> Include optional skills (as "useful")</label>
      <label>Job title (optional) <input bind:value={escoTitle} placeholder={escoPicked.label} /></label>
      <button type="submit" data-testid="esco-create">Create draft profile</button>
      <button type="button" onclick={() => (escoPicked = null)}>Back</button>
    </form>
  {/if}
{/if}

{#if profile}
  <h2>{profile.job_title}</h2>
  {#if profile.framework}
    <p class="muted" data-testid="framework-attribution">
      {profile.profession} › {profile.role_name} › level {profile.level_order}
      · {profile.framework.name} · {profile.framework.licence}
    </p>
    <p class="muted">{profile.framework.attribution}</p>
    <p class="muted">
      Required levels come from the framework's own scale ({profile.framework.scale_labels}); WPM
      shows its 1–5 minimum alongside. {profile.framework.note}
    </p>
  {:else if profile.source_ref}<p class="muted">Source: {profile.source_ref}</p>{/if}
  {#if profile.description}<p>{profile.description}</p>{/if}
  <table data-testid="role-requirements">
    <thead>
      <tr><th>Skill</th><th>Category</th><th>Minimum</th><th>Framework level</th><th>Importance</th><th></th></tr>
    </thead>
    <tbody>
      {#each profile.requirements as req (req.skill_pid)}
        <tr>
          <td>
            {req.skill ?? req.skill_pid}
            {#if req.note}
              <details><summary class="muted">What the framework says</summary><p>{req.note}</p></details>
            {/if}
          </td>
          <td>{req.category ?? "—"}</td>
          <td>{req.min_proficiency} / 5</td>
          <td>{req.source_level !== null ? `${req.source_level} of ${req.source_scale_max}` : "—"}</td>
          <td class:warn={req.importance === "critical"}>{req.importance}</td>
          <td>
            <button type="button" onclick={() => void dropRequirement(req.skill_pid)}>
              Remove
            </button>
          </td>
        </tr>
      {:else}
        <tr><td colspan="6" class="muted">No required skills yet.</td></tr>
      {/each}
    </tbody>
  </table>

  {#if progression && typeof progression.profile !== "string"}
    <h3>Next level up</h3>
    {#each progression.next as step (step.pid)}
      <p data-testid="role-progression">
        <strong>{step.job_title}</strong>:
        {step.added.length} new skill(s), {step.raised.length} raised, {step.unchanged} unchanged
      </p>
      <ul>
        {#each step.added as a (a.skill)}<li>New: {a.skill} (needs {a.min_proficiency})</li>{/each}
        {#each step.raised as r (r.skill)}<li>Raised: {r.skill} {r.from} → {r.to}</li>{/each}
      </ul>
    {:else}
      <p class="muted">This is the top level of the role.</p>
    {/each}
  {/if}

  {#if coverage}
    <h3>Can we staff this role today?</h3>
    <p class="muted">{coverage.derivation}</p>
    <table data-testid="role-coverage">
      <thead>
        <tr><th>Skill</th><th>Needs</th><th>Meeting</th><th>Below</th><th>Undeclared</th><th>Coverage</th></tr>
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

  <h3>Require a skill</h3>
  <form
    onsubmit={(event) => {
      event.preventDefault();
      void addRequirement();
    }}
  >
    <label>
      Skill
      <select data-testid="role-req-skill" bind:value={reqSkill} required>
        <option value="" disabled>Choose…</option>
        {#each skills as skill (skill.pid)}
          <option value={skill.pid}>{skill.name}</option>
        {/each}
      </select>
    </label>
    <label>
      Minimum proficiency
      <select data-testid="role-req-level" bind:value={reqLevel}>
        {#each [1, 2, 3, 4, 5] as level (level)}<option value={level}>{level}</option>{/each}
      </select>
    </label>
    <label>
      Importance
      <select data-testid="role-req-importance" bind:value={reqImportance}>
        {#each IMPORTANCES as importance (importance)}
          <option value={importance}>{importance}</option>
        {/each}
      </select>
    </label>
    <button type="submit" data-testid="role-req-add">Add</button>
  </form>
{/if}

<style>
  td.warn { color: #b45309; font-weight: 600; }
</style>
