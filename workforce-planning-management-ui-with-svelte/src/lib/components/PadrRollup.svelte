<!--
  PADRs across the organization (WPM-R172): people by where they stand against the cycle, by department, with the floor. Counts only,
  and HR reads nothing inside a PADR (WPM-D176).
-->
<script lang="ts">
  import { page } from "$app/state";
  import { ApiError } from "#lib/api/client.js";
  import { guardedLoad } from "#lib/loader.svelte.js";
  import { errorText } from "#lib/write.js";
  import ErrorNote from "#lib/components/ErrorNote.svelte";
  import { padrRollup, type PadrRollupView } from "#lib/api/wpm.js";
  import { standingWord } from "#lib/padr.js";
  import { t } from "#lib/i18n.svelte.js";

  const organization = $derived(((page.data.scope ?? []) as string[])[0] ?? "");
  let view = $state<PadrRollupView | null>(null);
  let error = $state<string | null>(null);
  // The roll-up is for HR; for anyone else the panel is simply not there.
  let refused = $state(false);

  const load = guardedLoad(
    async () => (organization ? await padrRollup(organization) : null),
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
  <section class="panel" data-testid="padr-rollup">
    <h2>{t("padr.rollupTitle")}</h2>
    {#if error}<ErrorNote testid="padr-rollup-error" message={error} />{/if}
    {#if view}
      <ul>
        {#each view.departments as row (row.department)}
          <li data-testid="padr-rollup-line">
            <strong>{row.department}</strong>:
            {#if row.by_status}
              {Object.entries(row.by_status)
                .map(([standing, count]) => `${standingWord(standing)} ${count}`)
                .join(" · ")}
            {:else}
              <span class="muted">{t("goals.withheld")}</span>
            {/if}
          </li>
        {:else}
          <li class="muted">{t("padr.rollupNone")}</li>
        {/each}
      </ul>
    {/if}
  </section>
{/if}
