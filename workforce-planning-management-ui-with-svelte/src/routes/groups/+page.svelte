<!--
  Groups (`/groups`): communities of practice, communities of interest, and
  other self-organised sets. Pick one to see its members and what it knows —
  declared skills in aggregate, never naming anyone, and withheld for any skill
  fewer than the floor of people declare.
-->
<script lang="ts">
  import { guardedLoad } from "#lib/loader.svelte.js";
  import { errorText } from "#lib/write.js";
  import ErrorNote from "#lib/components/ErrorNote.svelte";
  import { createGroup, groupMembers, groupSkills, listGroups, GROUP_KINDS, type Group } from "#lib/api/wpm.js";
  import { page } from "$app/state";
  import { t, tf } from "#lib/i18n.svelte.js";
  import DataTable from "#lib/components/DataTable.svelte";
  import type {
    DataGridColumn,
    DataGridRow,
  } from "@lilydesignsystem/svelte-data-grid";

  // The organizations the caller can read (confederation-expanded), as for the org chart.
  const organizations = $derived((page.data.scope ?? []) as string[]);
  let organization = $state<string>("");
  $effect(() => {
    if (!organization && organizations.length > 0) organization = organizations[0] ?? "";
  });

  let groups = $state<Group[]>([]);
  let selected = $state<string | null>(null);
  let members = $state<Awaited<ReturnType<typeof groupMembers>> | null>(null);
  let know = $state<Awaited<ReturnType<typeof groupSkills>> | null>(null);
  let error = $state<string | null>(null);
  let name = $state("");
  let kind = $state<string>("practice");
  let description = $state("");
  let spans = $state(false);

  const message = (cause: unknown) => (errorText(cause));
  const kindLabel = (k: string) => (k === "practice" ? t("pages.groups.kind_practice") : k === "interest" ? t("pages.groups.kind_interest") : t("pages.groups.kind_group"));

  const groupColumns = $derived<DataGridColumn[]>([
    { id: "name", header: t("pages.groups.group"), sortable: true, rowHeader: true, cell: groupCell },
    { id: "kind", header: t("pages.groups.kind"), sortable: true, accessor: (r) => kindLabel(String(r.kind)), cell: groupKindCell },
    { id: "members", header: t("pages.groups.members"), sortable: true, align: "end" },
  ]);
  const skillColumns = $derived<DataGridColumn[]>([
    { id: "skill", header: t("pages.groups.skill"), sortable: true, rowHeader: true },
    { id: "declared", header: t("pages.groups.declared_by"), sortable: true, align: "end" },
    {
      id: "coverage",
      header: t("pages.groups.coverage"),
      sortable: true,
      align: "end",
      format: (v) => (typeof v === "number" ? `${Math.round(v * 100)}%` : "—"),
    },
    ...(["1", "2", "3", "4", "5"] as const).map(
      (level): DataGridColumn => ({
        id: `level_${level}`,
        header: level === "1" ? t("pages.groups.level_1") : level,
        sortable: true,
        align: "end",
        accessor: (r) => (r as unknown as { levels: Record<string, number> }).levels[level],
      }),
    ),
  ]);

  const load = guardedLoad(
    async () => (organization ? await listGroups(organization) : []),
    (found) => {
      groups = found;
    },
    (cause) => {
      error = message(cause);
    },
  );
  $effect(() => {
    void organization;
    selected = null;
    void load();
  });

  async function open(pid: string) {
    selected = pid;
    error = null;
    try {
      [members, know] = await Promise.all([groupMembers(pid), groupSkills(pid)]);
    } catch (cause) {
      error = message(cause);
    }
  }

  async function create(event: SubmitEvent) {
    event.preventDefault();
    error = null;
    try {
      await createGroup({ organization_ref: organization, scope: spans ? "confederation" : "organization", name: name.trim(), kind, ...(description.trim() ? { description: description.trim() } : {}) });
      name = "";
      description = "";
      await load();
    } catch (cause) {
      error = message(cause);
    }
  }
</script>

<svelte:head><title>{t("pages.groups.page_title")}</title></svelte:head>

{#snippet groupCell({ row }: { row: DataGridRow })}
  <button type="button" onclick={() => void open(String(row.pid))}>{row.name}</button>{#if row.description}<br /><span class="muted">{row.description}</span>{/if}
{/snippet}

{#snippet groupKindCell({ row }: { row: DataGridRow })}
  {kindLabel(String(row.kind))}{#if row.scope === "confederation"} <span class="chip">{t("pages.groups.spans_organizations")}</span>{/if}
{/snippet}

<h1>{t("nav.groups")}</h1>
{#if error}<ErrorNote testid="error" message={error} />{/if}
{#if organizations.length === 0}
  <p class="muted">{t("org.noOrganizations")}</p>
{:else}
  <p>
    <label>
      {t("pages.groups.organization_shows_its_groups_and")}
      <select bind:value={organization} data-testid="group-organization">
        {#each organizations as o (o)}<option value={o}>{o}</option>{/each}
      </select>
    </label>
  </p>

{#if groups.length === 0}
  <p class="muted" data-testid="group-list">{t("pages.groups.no_groups_yet")}</p>
{:else}
  <DataTable
    testid="group-list"
    label={t("nav.groups")}
    columns={groupColumns}
    rows={groups as unknown as DataGridRow[]}
    rowId={(row) => String(row.pid)}
    pageSize={25}
  />
{/if}

<form onsubmit={create}>
  <label>{t("pages.groups.start_a_group")} <input bind:value={name} required maxlength="120" /></label>
  <label>{t("pages.groups.kind")} <select bind:value={kind}>{#each GROUP_KINDS as k (k)}<option value={k}>{kindLabel(k)}</option>{/each}</select></label>
  <label>{t("pages.groups.about")} <input bind:value={description} maxlength="1000" /></label>
  <label><input type="checkbox" bind:checked={spans} /> {t("pages.groups.spans_member_organizations_a")}</label>
  <button type="submit">{t("pages.groups.start")}</button>
</form>

{#if selected && members && know}
  <section class="panel" data-testid="group-detail">
    <h2>{members.group.name}</h2>
    <h3>{t("pages.groups.members")}</h3>
    <ul>
      {#each members.members as m (m.worker_pid)}
        <li>{m.display_name} {#if m.role === "lead"}<span class="chip ok">{t("pages.groups.lead")}</span>{/if}<span class="muted"> {tf("pages.groups.since", { joined_at: m.joined_at.slice(0, 10) })}</span></li>
      {/each}
    </ul>
    <h3>{t("pages.groups.what_the_group_knows")}</h3>
    <p class="muted">{tf("pages.groups.declared_skills_of_current_members", { floor: know.floor })}</p>
    {#if know.skills.length === 0}
      <p class="muted" data-testid="group-skills">{t("pages.groups.nothing_to_show_yet")}</p>
    {:else}
      <DataTable
        testid="group-skills"
        label={t("pages.groups.what_the_group_knows")}
        columns={skillColumns}
        rows={know.skills as unknown as DataGridRow[]}
        rowId={(row) => String(row.skill_pid)}
        pageSize={25}
      />
    {/if}
    {#if know.withheld_below_floor > 0}<p class="muted">{tf("pages.groups.skill_s_withheld_too_few_people_to", { withheld_below_floor: know.withheld_below_floor })}</p>{/if}
  </section>
{/if}
{/if}
