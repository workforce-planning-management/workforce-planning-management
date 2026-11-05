<!--
  Goals across the organization (WPM-R226): counts by status and department, with the floor. No rate, no score, and nobody named.
-->
<script lang="ts">
  import { page } from "$app/state";
  import { ApiError } from "#lib/api/client.js";
  import { guardedLoad } from "#lib/loader.svelte.js";
  import { errorText } from "#lib/write.js";
  import ErrorNote from "#lib/components/ErrorNote.svelte";
  import { goalsRollup, type GoalsRollupView } from "#lib/api/wpm.js";
  import { t, tv } from "#lib/i18n.svelte.js";

  const organization = $derived(((page.data.scope ?? []) as string[])[0] ?? "");
  let view = $state<GoalsRollupView | null>(null);
  let error = $state<string | null>(null);
  // The roll-up is for HR; for anyone else the panel is simply not there.
  let refused = $state(false);

  const load = guardedLoad(
    async () => (organization ? await goalsRollup(organization) : null),
    (found) => {
      view = found;
    },
    (cause) => {
      if (cause instanceof ApiError && cause.status === 403) refused = true;
      else error = errorText(cause);
    },
  );

  $effect(() => {
    void organization;
    void load();
  });
</script>

{#if organization && !refused}
  <section class="panel" data-testid="goals-rollup">
    <h2>{t("goals.rollupTitle")}</h2>
    {#if error}<ErrorNote testid="rollup-error" message={error} />{/if}
    {#if view}
      <ul>
        {#each view.departments as row (row.department)}
          <li data-testid="rollup-line">
            <strong>{row.department}</strong>:
            {#if row.by_status}
              {Object.entries(row.by_status)
                .map(([status, count]) => `${tv(status)} ${count}`)
                .join(" · ")}
            {:else}
              <span class="muted">{t("goals.withheld")}</span>
            {/if}
          </li>
        {:else}
          <li class="muted">{t("goals.rollupNone")}</li>
        {/each}
      </ul>
    {/if}
  </section>
{/if}
