<!--
  The exit criteria of the level an organization is at, and the evidence for each measure: how many
  consecutive months have met the threshold, and what stopped the run. A missing reading is never a
  pass. Shared by the oversight page and the frozen pack.
-->
<script lang="ts">
  import type { OversightStatus } from "#lib/api/wpm.js";
  import { t, tf, tv } from "#lib/i18n.svelte.js";
  import { stateLabel } from "#lib/stateLabel.js";

  let { criteria }: { criteria: OversightStatus["criteria"] } = $props();

  type Stop = { reason: string; month?: string };
  /** Why a run stopped, in words; `null` when it did not stop. */
  function stopped(by: Stop): string | null {
    const month = by.month ?? "";
    switch (by.reason) {
      case "breach":
        return tf("ovs.stopBreach", { month });
      case "no_reading":
        return tf("ovs.stopNoReading", { month });
      case "null_reading":
        return tf("ovs.stopNullReading", { month });
      case "no_measures":
        return t("ovs.noMeasures");
      default:
        return null;
    }
  }
</script>

{#each criteria as c (c.criterion_pid)}
  <section class="criterion" data-testid="criterion">
    <h3>
      {c.criterion}
      <span class="chip" class:ok={c.status === "met"} class:danger={c.status === "not_met"}>
        {stateLabel(c.status)}
      </span>
    </h3>
    {#if c.measures.length === 0}
      <p class="muted">{t("ovs.noMeasures")}</p>
    {:else}
      <ul>
        {#each c.measures as m (m.measure_pid)}
          <li data-testid="measure">
            <strong>{m.name}</strong>
            <span class="chip">{tv(m.origin)}</span>
            <span class="chip" class:ok={m.status === "met"} class:danger={m.status === "not_met"}>
              {stateLabel(m.status)}
            </span>
            <span class="muted">
              · {tf("ovs.run", { run: m.run, required: m.required })} ·
              {t("ovs.threshold")}: {tv(m.direction)}
              {m.threshold}
              {m.unit}
            </span>
            {#if stopped(m.stopped_by)}
              <span class="muted">· {stopped(m.stopped_by)}</span>
            {/if}
          </li>
        {/each}
      </ul>
    {/if}
  </section>
{/each}

<style>
  .criterion h3 {
    font-size: 1rem;
    margin-bottom: 0.25rem;
  }
  ul {
    margin: 0;
    padding-inline-start: 1.1rem;
  }
  li {
    margin-bottom: 0.3rem;
  }
</style>
