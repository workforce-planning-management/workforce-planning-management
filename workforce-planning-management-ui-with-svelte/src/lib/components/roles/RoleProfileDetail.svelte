<!--
  One role profile (WPM-R83): its framework attribution, its grade, the skills it requires, what the next level up
  asks for, and whether the workforce could staff it today (aggregate counts of declarations). A profile says what
  a job requires, never what a person has.
-->
<script lang="ts">
  import {
    removeRoleRequirement,
    setRoleRequirement,
    type getRoleProfile,
    type listSkills,
    type roleGap,
    type roleProgression,
  } from "#lib/api/wpm.js";
  import DataTable from "#lib/components/DataTable.svelte";
  import RoleGrade from "#lib/components/RoleGrade.svelte";
  import { percentWithWorkings } from "#lib/format.js";
  import { t, tf, tn, tv } from "#lib/i18n.svelte.js";
  import { errorText } from "#lib/write.js";
  import type { DataGridColumn, DataGridRow } from "@lilydesignsystem/svelte-data-grid";

  type Profile = Awaited<ReturnType<typeof getRoleProfile>>;
  type Skills = Awaited<ReturnType<typeof listSkills>>;
  type Coverage = Awaited<ReturnType<typeof roleGap>>;
  type Progression = Awaited<ReturnType<typeof roleProgression>>;

  const IMPORTANCES = ["critical", "important", "useful"] as const;

  let {
    profile,
    skills,
    coverage,
    progression,
    onchange,
    onfail,
  }: {
    profile: Profile;
    skills: Skills;
    coverage: Coverage | null;
    progression: Progression | null;
    /** Called after a change that the page should read again. */
    onchange: () => Promise<void>;
    /** Called with the words for a failure, or `null` when an action starts. */
    onfail: (message: string | null) => void;
  } = $props();

  let reqSkill = $state("");
  let reqLevel = $state(3);
  let reqImportance = $state<string>("important");

  async function addRequirement() {
    if (!reqSkill) return;
    onfail(null);
    try {
      await setRoleRequirement(profile.pid, {
        skill_pid: reqSkill,
        min_proficiency: reqLevel,
        importance: reqImportance,
      });
      await onchange();
    } catch (cause) {
      onfail(errorText(cause));
    }
  }

  async function dropRequirement(skillPid: string) {
    onfail(null);
    try {
      await removeRoleRequirement(profile.pid, skillPid);
      await onchange();
    } catch (cause) {
      onfail(errorText(cause));
    }
  }

  type Requirement = Profile["requirements"][number];

  const requirementColumns = $derived<DataGridColumn[]>([
    { id: "skill", header: t("pages.roles.skill"), sortable: true, rowHeader: true, cell: skillCell,
      accessor: (r) => r.skill ?? r.skill_pid },
    { id: "category", header: t("pages.roles.category"), sortable: true },
    {
      id: "min_proficiency",
      header: t("pages.roles.minimum"),
      sortable: true,
      align: "end",
      format: (v) => `${v} / 5`,
    },
    {
      id: "source_level",
      header: t("pages.roles.framework_level"),
      sortable: true,
      align: "end",
      format: (v, r) => {
        if (v === null || v === undefined) return "—";
        // Stated on a framework's own scale: say which, and how it is counted on WPM's 1–5.
        const counted = r.source_framework
          ? tf("pages.roles.native_level_counted", {
              level: v,
              scale: r.source_scale_max,
              framework: r.source_framework_name ?? r.source_framework,
              wpm: r.min_proficiency,
              mapping: r.mapping ?? "",
            })
          : `${v} of ${r.source_scale_max}`;
        return counted;
      },
    },
    {
      id: "importance",
      header: t("pages.roles.importance"),
      sortable: true,
      format: (v) => tv(String(v)),
      cellClass: ({ value }) => (value === "critical" ? "cx-warn" : ""),
    },
    { id: "remove", header: t("pages.roles.remove"), cell: removeCell },
  ]);

  const coverageColumns = $derived<DataGridColumn[]>([
    { id: "skill", header: t("pages.roles.skill"), sortable: true },
    { id: "min_proficiency", header: t("pages.roles.needs"), sortable: true, align: "end" },
    { id: "meeting", header: t("pages.roles.meeting"), sortable: true, align: "end" },
    { id: "below", header: t("pages.roles.below"), sortable: true, align: "end" },
    { id: "undeclared", header: t("pages.roles.undeclared"), sortable: true, align: "end" },
    {
      id: "coverage",
      header: t("pages.roles.coverage"),
      sortable: true,
      align: "end",
      accessor: (r) => (r.coverage as { value: number | null } | null)?.value ?? null,
      format: (_v, r) => percentWithWorkings(r.coverage as Parameters<typeof percentWithWorkings>[0]),
    },
  ]);
</script>

{#snippet skillCell({ row }: { row: DataGridRow })}
  {@const req = row as unknown as Requirement}
  {req.skill ?? req.skill_pid}
  {#if req.note}
    <details><summary class="muted">{t("pages.roles.what_the_framework_says")}</summary><p>{req.note}</p></details>
  {/if}
{/snippet}

{#snippet removeCell({ row }: { row: DataGridRow })}
  <button type="button" onclick={() => void dropRequirement(String(row.skill_pid))}>
    {t("pages.roles.remove")}
  </button>
{/snippet}

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
{#if profile.requirements.length === 0}
  <p class="muted" data-testid="role-requirements">{t("pages.roles.no_required_skills_yet")}</p>
{:else}
  <DataTable
    testid="role-requirements"
    label={t("pages.roles.profiles")}
    columns={requirementColumns}
    rows={profile.requirements as unknown as DataGridRow[]}
    rowId={(row) => String(row.skill_pid)}
  />
  {@const worded = profile.requirements.filter((r) => r.native_description)}
  {#if worded.length > 0}
    <h3>{t("pages.roles.framework_wording")}</h3>
    <ul data-testid="role-requirement-wording">
      {#each worded as r (r.skill_pid)}
        <li><strong>{r.skill ?? r.skill_pid}</strong> — {r.native_description}</li>
      {/each}
    </ul>
  {/if}
{/if}

{#if progression && typeof progression.profile !== "string"}
  <h3>{t("pages.roles.next_level_up")}</h3>
  {#each progression.next as step (step.pid)}
    <p data-testid="role-progression">
      <strong>{step.job_title}</strong>{tn("pages.roles.new_skill_s_raised_unchanged", step.added.length, { added: step.added.length, raised: step.raised.length, unchanged: step.unchanged })}
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
  <DataTable
    testid="role-coverage"
    label={t("pages.roles.can_we_staff_this_role_today")}
    columns={coverageColumns}
    rows={coverage.requirements as unknown as DataGridRow[]}
    rowId={(row) => String(row.skill)}
  />
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
