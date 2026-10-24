<script lang="ts">
  import { errorText } from "#lib/write.js";
  import ErrorNote from "#lib/components/ErrorNote.svelte";
  import { page } from "$app/state";
  import { orgChart, orgChartSize } from "#lib/api/wpm.js";
  import { t, tf, l } from "#lib/i18n.svelte.js";
  import type { OrgNode } from "#lib/api/types.js";
  import OrgTree from "#lib/components/OrgTree.svelte";
  import {
    ORG_VIEWS,
    byDepartment,
    byLevel,
    byLocation,
    byTenure,
    tenureLabel,
    type OrgView,
  } from "#lib/orgViews.js";

  // No switcher: every organization this person can read gets its own
  // section below, fetched in parallel — not a single guessed org.
  // `scope` (not `organizations`) since it's expanded through org
  // confederation: a membership in a parent confederation reads as
  // every descendant too.
  const organizationRefs = $derived((page.data.scope ?? []) as string[]);

  let byOrganization = $state<Record<string, OrgNode[]> | null>(null);
  /** How many people each organization has in all; the chart draws at most a few thousand. */
  let sizes = $state<Record<string, number>>({});
  const drawn = (nodes: OrgNode[]): number =>
    nodes.reduce((sum, n) => sum + 1 + drawn(n.reports), 0);
  let error = $state<string | null>(null);
  let view = $state<OrgView>("manager");

  $effect(() => {
    const refs = organizationRefs;
    void (async () => {
      try {
        const entries = await Promise.all(
          refs.map(
            async (ref) => [ref, await orgChart(ref)] as const,
          ),
        );
        byOrganization = Object.fromEntries(entries);
        // Only to say when the chart is part of the organization; a failure here hides nothing else.
        sizes = Object.fromEntries(
          await Promise.all(
            refs.map(async (ref) => {
              try {
                return [ref, (await orgChartSize(ref)).people] as const;
              } catch {
                return [ref, 0] as const;
              }
            }),
          ),
        );
      } catch (cause) {
        error = errorText(cause);
      }
    })();
  });
</script>

<svelte:head><title>{page.data.title}</title></svelte:head>

<h1>{t("nav.orgChart")}</h1>

{#if error}
  <ErrorNote testid="error" prefixed message={error} />
{:else if byOrganization === null}
  <p>{t("common.loading")}</p>
{:else if organizationRefs.length === 0}
  <p class="muted">{t("org.noOrganizations")}</p>
{:else}
  <p>
    <label>
      {t("org.view.label")}
      <select data-testid="org-view" bind:value={view}>
        {#each ORG_VIEWS as mode (mode)}
          <option value={mode}>{t(`org.view.${mode}`)}</option>
        {/each}
      </select>
    </label>
  </p>
  {#each organizationRefs as ref (ref)}
    <section class="panel org-chart-section" data-testid="org-chart">
      <h2><code>{ref}</code></h2>
      {#if (sizes[ref] ?? 0) > drawn(byOrganization[ref] ?? [])}
        <p class="muted" data-testid="org-chart-truncated">
          {tf("org.truncated", { shown: drawn(byOrganization[ref] ?? []), total: sizes[ref] ?? 0 })}
        </p>
      {/if}
      {#if view === "manager"}
        {#each byOrganization[ref] ?? [] as root (root.pid)}
          <OrgTree node={root} />
        {/each}
      {:else if view === "department"}
        {#each byDepartment(byOrganization[ref] ?? []) as group (group.department)}
          <h3>{group.department} <span class="muted">({group.members.length})</span></h3>
          <ul>
            {#each group.members as member (member.pid)}
              <li>
                <a href={l(`/workers/${member.pid}`)}>{member.display_name}</a>
                <span class="muted">— {member.job_title}</span>
              </li>
            {/each}
          </ul>
        {/each}
      {:else if view === "tenure"}
        {#each byTenure(byOrganization[ref] ?? []) as group (group.band)}
          <h3>
            {tenureLabel(group.band)} {t("org.tenureYears")}
            <span class="muted">({group.members.length})</span>
          </h3>
          <ul>
            {#each group.members as member (member.pid)}
              <li>
                <a href={l(`/workers/${member.pid}`)}>{member.display_name}</a>
                <span class="muted">— {member.job_title} · {member.department}</span>
              </li>
            {/each}
          </ul>
        {/each}
      {:else if view === "location"}
        {#each byLocation(byOrganization[ref] ?? []) as group (group.location ?? "")}
          <h3>
            {group.location ?? t("org.noLocation")}
            <span class="muted">({group.members.length})</span>
          </h3>
          <ul>
            {#each group.members as member (member.pid)}
              <li>
                <a href={l(`/workers/${member.pid}`)}>{member.display_name}</a>
                <span class="muted">— {member.job_title} · {member.department}</span>
              </li>
            {/each}
          </ul>
        {/each}
      {:else}
        {#each byLevel(byOrganization[ref] ?? []) as group (group.level)}
          <h3>{t("org.level")} {group.level} <span class="muted">({group.members.length})</span></h3>
          <ul>
            {#each group.members as member (member.pid)}
              <li>
                <a href={l(`/workers/${member.pid}`)}>{member.display_name}</a>
                <span class="muted">— {member.job_title} · {member.department}</span>
              </li>
            {/each}
          </ul>
        {/each}
      {/if}
    </section>
  {/each}
{/if}

<style>
  .org-chart-section + .org-chart-section {
    margin-top: 1rem;
  }
  .org-chart-section h2 {
    font-size: 0.95rem;
    margin: 0 0 0.75rem;
  }
  .org-chart-section h2 code {
    color: var(--muted);
    font-weight: 400;
  }
</style>
