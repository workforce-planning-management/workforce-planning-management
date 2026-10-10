<!--
  Groups (`/groups`): communities of practice, communities of interest, and
  other self-organised sets. Pick one to see its members and what it knows —
  declared skills in aggregate, never naming anyone, and withheld for any skill
  fewer than the floor of people declare.
-->
<script lang="ts">
  import { createGroup, groupMembers, groupSkills, listGroups, GROUP_KINDS, type Group } from "#lib/api/wpm.js";
  import { page } from "$app/state";
  import { t, tf } from "#lib/i18n.svelte.js";

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

  const message = (cause: unknown) => (cause instanceof Error ? cause.message : String(cause));
  const kindLabel = (k: string) => (k === "practice" ? t("pages.groups.kind_practice") : k === "interest" ? t("pages.groups.kind_interest") : t("pages.groups.kind_group"));

  async function load() {
    try {
      groups = organization ? await listGroups(organization) : [];
    } catch (cause) {
      error = message(cause);
    }
  }
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

<h1>{t("nav.groups")}</h1>
{#if error}<p class="error" data-testid="error">{error}</p>{/if}
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

<table data-testid="group-list">
  <thead><tr><th>{t("pages.groups.group")}</th><th>{t("pages.groups.kind")}</th><th>{t("pages.groups.members")}</th></tr></thead>
  <tbody>
    {#each groups as g (g.pid)}
      <tr>
        <td><button type="button" onclick={() => void open(g.pid)}>{g.name}</button>{#if g.description}<br /><span class="muted">{g.description}</span>{/if}</td>
        <td>{kindLabel(g.kind)}{#if g.scope === "confederation"} <span class="chip">{t("pages.groups.spans_organizations")}</span>{/if}</td><td>{g.members}</td>
      </tr>
    {:else}
      <tr><td colspan="3" class="muted">{t("pages.groups.no_groups_yet")}</td></tr>
    {/each}
  </tbody>
</table>

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
    <table data-testid="group-skills">
      <thead><tr><th>{t("pages.groups.skill")}</th><th>{t("pages.groups.declared_by")}</th><th>{t("pages.groups.coverage")}</th><th>{t("pages.groups.level_1")}</th><th>2</th><th>3</th><th>4</th><th>5</th></tr></thead>
      <tbody>
        {#each know.skills as s (s.skill_pid)}
          <tr>
            <td>{s.skill}</td><td>{s.declared}</td>
            <td>{s.coverage === null ? "—" : `${Math.round(s.coverage * 100)}%`}</td>
            {#each ["1", "2", "3", "4", "5"] as l (l)}<td>{s.levels[l as "1"]}</td>{/each}
          </tr>
        {:else}
          <tr><td colspan="8" class="muted">{t("pages.groups.nothing_to_show_yet")}</td></tr>
        {/each}
      </tbody>
    </table>
    {#if know.withheld_below_floor > 0}<p class="muted">{tf("pages.groups.skill_s_withheld_too_few_people_to", { withheld_below_floor: know.withheld_below_floor })}</p>{/if}
  </section>
{/if}
{/if}
