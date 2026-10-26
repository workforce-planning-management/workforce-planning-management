<!--
  Team equipment: for a manager, how many of their direct reports still hold
  equipment. The service gives the count only from the floor; under it,
  nothing, since a small number would name people. Never an item, a kind or a
  name. Renders nothing for anyone with no direct reports in work.
-->
<script lang="ts">
  import { guardedLoad } from "#lib/loader.svelte.js";
  import { errorText } from "#lib/write.js";
  import ErrorNote from "#lib/components/ErrorNote.svelte";
  import { getTeamEquipmentCount } from "#lib/api/wpm.js";
  import type { TeamHolding } from "#lib/api/wpm.js";
  import { t, tf } from "#lib/i18n.svelte.js";

  let { workerPid }: { workerPid: string } = $props();

  let count = $state<TeamHolding | null>(null);
  let floor = $state(0);
  let error = $state<string | null>(null);

  const load = guardedLoad(
    () => getTeamEquipmentCount(workerPid),
    (found) => {
      count = found.count;
      floor = found.floor;
    },
    (cause) => {
      error = errorText(cause);
    },
  );

  $effect(() => {
    void workerPid;
    void load();
  });
</script>

{#if error}
  <ErrorNote testid="team-equipment-error" message={error} />
{:else if count && count.kind !== "no_one"}
  <section class="panel" data-testid="team-equipment">
    <h2>{t("teamEquipment.title")}</h2>
    <p class="muted">{tf("teamEquipment.hint", { floor })}</p>
    <p data-testid="team-equipment-count">
      {#if count.kind === "shown"}
        {tf("teamEquipment.shown", { holding: count.holding, none: count.none })}
      {:else}
        <span class="muted">{tf("teamEquipment.withheld", { floor })}</span>
      {/if}
    </p>
  </section>
{/if}
