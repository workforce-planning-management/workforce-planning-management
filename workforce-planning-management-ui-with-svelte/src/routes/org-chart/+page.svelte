<script lang="ts">
  import { page } from "$app/state";
  import { orgChart } from "#lib/api/wpm.js";
  import { t } from "#lib/i18n.svelte.js";
  import type { OrgNode } from "#lib/api/types.js";
  import OrgTree from "#lib/components/OrgTree.svelte";
  import { ORG_VIEWS, byDepartment, byLevel, type OrgView } from "#lib/orgViews.js";

  // No switcher: every organization this person can read gets its own
  // section below, fetched in parallel — not a single guessed org.
  // `scope` (not `organizations`) since it's expanded through org
  // confederation: a membership in a parent confederation reads as
  // every descendant too.
  const organizationRefs = $derived((page.data.scope ?? []) as string[]);

  let byOrganization = $state<Record<string, OrgNode[]> | null>(null);
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
      } catch (cause) {
        error = cause instanceof Error ? cause.message : String(cause);
      }
    })();
  });
</script>

<h1>{t("nav.orgChart")}</h1>

{#if error}
  <p class="error" data-testid="error">{t("common.error")}: {error}</p>
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
                <a href={`/workers/${member.pid}`}>{member.display_name}</a>
                <span class="muted">— {member.job_title}</span>
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
                <a href={`/workers/${member.pid}`}>{member.display_name}</a>
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
