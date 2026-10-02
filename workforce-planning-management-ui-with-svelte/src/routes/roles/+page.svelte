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
    listRoleProfiles,
    listSkills,
    removeRoleRequirement,
    roleGap,
    setRoleRequirement,
  } from "#lib/api/wpm.js";
  import { percentWithWorkings } from "#lib/format.js";
  import { t } from "#lib/i18n.svelte.js";

  type Profiles = Awaited<ReturnType<typeof listRoleProfiles>>;
  type Profile = Awaited<ReturnType<typeof getRoleProfile>>;
  type Skills = Awaited<ReturnType<typeof listSkills>>;
  type Coverage = Awaited<ReturnType<typeof roleGap>>;

  const IMPORTANCES = ["critical", "important", "useful"] as const;

  let profiles = $state<Profiles>([]);
  let skills = $state<Skills>([]);
  let selected = $state("");
  let coverage = $state<Coverage | null>(null);
  let profile = $state<Profile | null>(null);
  let error = $state<string | null>(null);

  let newTitle = $state("");
  let reqSkill = $state("");
  let reqLevel = $state(3);
  let reqImportance = $state<string>("important");

  const message = (cause: unknown) =>
    cause instanceof Error ? cause.message : String(cause);

  async function loadProfiles() {
    try {
      profiles = await listRoleProfiles();
      if (!selected && profiles.length > 0) selected = profiles[0]?.pid ?? "";
      if (selected) {
        profile = await getRoleProfile(selected);
        coverage = await roleGap(selected);
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
    if (pid) {
      try {
        profile = await getRoleProfile(pid);
        coverage = await roleGap(pid);
      } catch (cause) {
        error = message(cause);
      }
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

<h2>Profiles</h2>
<p>
  <label>
    Profile
    <select
      data-testid="role-select"
      value={selected}
      onchange={(event) => void select(event.currentTarget.value)}
    >
      {#each profiles as p (p.pid)}
        <option value={p.pid}>{p.job_title} ({p.requirement_count})</option>
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

{#if profile}
  <h2>{profile.job_title}</h2>
  {#if profile.source_ref}<p class="muted">Source: {profile.source_ref}</p>{/if}
  <table data-testid="role-requirements">
    <thead>
      <tr><th>Skill</th><th>Category</th><th>Minimum</th><th>Importance</th><th></th></tr>
    </thead>
    <tbody>
      {#each profile.requirements as req (req.skill_pid)}
        <tr>
          <td>{req.skill ?? req.skill_pid}</td>
          <td>{req.category ?? "—"}</td>
          <td>{req.min_proficiency} / 5</td>
          <td class:warn={req.importance === "critical"}>{req.importance}</td>
          <td>
            <button type="button" onclick={() => void dropRequirement(req.skill_pid)}>
              Remove
            </button>
          </td>
        </tr>
      {:else}
        <tr><td colspan="5" class="muted">No required skills yet.</td></tr>
      {/each}
    </tbody>
  </table>

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
