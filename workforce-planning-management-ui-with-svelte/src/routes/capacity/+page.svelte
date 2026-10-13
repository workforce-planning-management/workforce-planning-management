<!--
  Capacity (`/capacity`, WPM-R56 to R60): the skill pools of an organization by month, supply against
  the demand of active programmes, with firm partner capacity, the constraint pool and how many
  programmes draw on it. Pools and months, never people. An unknown month is shown as unknown.
-->
<script lang="ts">
  import { ApiError } from "#lib/api/client.js";
  import { capacityView, type CapacityView } from "#lib/api/wpm.js";
  import CapacityHeatmap from "#lib/components/CapacityHeatmap.svelte";
  import OrgPicker from "#lib/components/OrgPicker.svelte";
  import { t, tf } from "#lib/i18n.svelte.js";

  let org = $state("");
  let view = $state<CapacityView | null>(null);
  let error = $state<string | null>(null);
  let loading = $state(false);

  $effect(() => {
    const chosen = org;
    if (!chosen) return;
    let current = true;
    loading = true;
    void (async () => {
      try {
        const loaded = await capacityView(chosen);
        if (!current) return;
        view = loaded;
        error = null;
      } catch (cause) {
        if (!current) return;
        view = null;
        error =
          cause instanceof ApiError || cause instanceof Error ? cause.message : String(cause);
      } finally {
        if (current) loading = false;
      }
    })();
    return () => {
      current = false;
    };
  });

  const constraintName = $derived(
    view?.pools.find((p) => p.pid === view?.constraint_pool)?.name ?? null,
  );
</script>

<svelte:head><title>{t("titles.capacity")}</title></svelte:head>

<h1>{t("cap.title")}</h1>
<p class="muted">{t("cap.hint")}</p>

<div class="form"><OrgPicker bind:value={org} /></div>

{#if error}
  <p class="error" data-testid="error">{t("common.error")}: {error}</p>
{:else if !org}
  <p class="muted">{t("common.noOrganization")}</p>
{:else if view === null || loading}
  <p>{t("common.loading")}</p>
{:else if view.pools.length === 0}
  <p class="muted" data-testid="capacity-none">{t("cap.none")}</p>
{:else}
  <section class="panel">
    {#if constraintName}
      <p data-testid="constraint">
        <strong>{tf("cap.constraint", { name: constraintName })}</strong>
        {#if view.wip}
          <span class="muted">
            ·
            {#if view.wip.limit === null}
              {tf("cap.wipNoLimit", { active: view.wip.active })}
            {:else}
              {tf("cap.wip", { active: view.wip.active, limit: view.wip.limit })}
            {/if}
          </span>
        {/if}
      </p>
    {/if}
    <CapacityHeatmap months={view.months} pools={view.pools} />
    <p class="muted small">{t("cap.cellHint")}</p>
    <details>
      <summary>{t("cap.basis")}</summary>
      <p class="muted">{view.derivation}</p>
    </details>
  </section>
{/if}

<style>
  .small {
    font-size: 0.82rem;
  }
</style>
