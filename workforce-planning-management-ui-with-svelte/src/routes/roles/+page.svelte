<!--
  Role profiles (`/roles`): what a role (keyed by job title) requires — a
  set of catalogue skills, each with a minimum proficiency (1–5) and an
  importance. A profile describes a role, never a person; it is the
  yardstick competency-gap analysis measures declared skills against.
-->
<script lang="ts">
  import { guardedLoad } from "#lib/loader.svelte.js";
  import EscoOccupationSeed from "#lib/components/roles/EscoOccupationSeed.svelte";
  import SfiaSkillSeed from "#lib/components/roles/SfiaSkillSeed.svelte";
  import RoleProfileDetail from "#lib/components/roles/RoleProfileDetail.svelte";
  import { errorText } from "#lib/write.js";
  import ErrorNote from "#lib/components/ErrorNote.svelte";
  import {
    createRoleProfile,
    getRoleProfile,
    listFrameworks,
    listRoleProfiles,
    listSkills,
    mobilityInterestSummary,
    roleGap,
    roleProgression,
    searchEscoOccupations,
    getEscoOccupation,
  } from "#lib/api/wpm.js";
  import { t, tf, tv } from "#lib/i18n.svelte.js";
  import DataTable from "#lib/components/DataTable.svelte";
  import type { DataGridColumn, DataGridRow } from "@lilydesignsystem/svelte-data-grid";

  type Profiles = Awaited<ReturnType<typeof listRoleProfiles>>;
  type Profile = Awaited<ReturnType<typeof getRoleProfile>>;
  type Skills = Awaited<ReturnType<typeof listSkills>>;
  type Coverage = Awaited<ReturnType<typeof roleGap>>;
  type Interest = Awaited<ReturnType<typeof mobilityInterestSummary>>;
  type Frameworks = Awaited<ReturnType<typeof listFrameworks>>;
  type Progression = Awaited<ReturnType<typeof roleProgression>>;

  let profiles = $state<Profiles>([]);
  let skills = $state<Skills>([]);
  let selected = $state("");
  let coverage = $state<Coverage | null>(null);
  let interest = $state<Interest | null>(null);
  let frameworks = $state<Frameworks>([]);
  let framework = $state("");
  let progression = $state<Progression | null>(null);
  const escoLoaded = $derived(frameworks.some((f) => f.slug === "esco"));
  const sfiaLoaded = $derived(frameworks.some((f) => f.slug === "sfia"));
  let profile = $state<Profile | null>(null);
  let error = $state<string | null>(null);

  let newTitle = $state("");

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
    errorText(cause);

  const loadProfiles = guardedLoad(
    async () => {
      const found = await listRoleProfiles(framework || undefined);
      const known = await listFrameworks();
      const wanted = await mobilityInterestSummary();
      const pick = selected || found[0]?.pid || "";
      const detail = pick
        ? await Promise.all([getRoleProfile(pick), roleGap(pick), roleProgression(pick)])
        : null;
      return { found, known, wanted, pick, detail };
    },
    ({ found, known, wanted, pick, detail }) => {
      profiles = found;
      frameworks = known;
      interest = wanted;
      if (!selected) selected = pick;
      if (detail) [profile, coverage, progression] = detail;
    },
    (cause) => {
      error = message(cause);
    },
  );

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

  const interestColumns = $derived<DataGridColumn[]>([
    { id: "title", header: t("pages.roles.target"), sortable: true, accessor: (r) => r.title ?? r.target_pid },
    { id: "kind", header: t("pages.roles.kind"), sortable: true, format: (v) => tv(String(v)) },
    { id: "interested", header: t("pages.roles.interested"), sortable: true, align: "end" },
  ]);

</script>

<svelte:head><title>{tf("pages.roles.page_title", { roles: t("nav.roles") })}</title></svelte:head>

<h1>{t("nav.roles")}</h1>
{#if error}<ErrorNote testid="error" message={error} />{/if}

<p class="muted">
  {t("pages.roles.a_role_profile_says_what_a_job")}
</p>

{#if interest && interest.targets.length > 0}
  <h2>{t("pages.roles.employee_interest")}</h2>
  <p class="muted">{interest.derivation}</p>
  <DataTable
    testid="mobility-summary"
    label={t("pages.roles.employee_interest")}
    columns={interestColumns}
    rows={interest.targets as unknown as DataGridRow[]}
    rowId={(row) => String(row.target_pid)}
  />
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
    <!-- Re-made when the list arrives: a value set before its options exist is not applied again. -->
    {#key profiles}
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
    {/key}
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
  <EscoOccupationSeed
    onseeded={async (pid) => {
      selected = pid;
      await loadProfiles();
    }}
    onfail={(message) => (error = message)}
  />
{/if}

{#if sfiaLoaded}
  <SfiaSkillSeed
    onseeded={async (pid) => {
      selected = pid;
      await loadProfiles();
    }}
    onfail={(message) => (error = message)}
  />
{/if}

{#if profile}
  <RoleProfileDetail
    {profile}
    {skills}
    {coverage}
    {progression}
    onchange={loadProfiles}
    onfail={(message) => (error = message)}
  />
{/if}

<style>
  :global(td.cx-warn) { color: #b45309; font-weight: 600; }
</style>
